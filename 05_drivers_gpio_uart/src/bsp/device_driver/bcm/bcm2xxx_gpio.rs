// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! GPIOドライバ

use crate::{
    bsp::device_driver::common::MMIODerefWrapper, driver, synchronization,
    synchronization::NullLock,
};
use register::{mmio::*, register_bitfields, register_structs};

//--------------------------------------------------------------------------------------------------
// プライベート定義
//--------------------------------------------------------------------------------------------------

// GPIOレジスタ
//
// 記述は以下から採った
// - https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf
// - https://datasheets.raspberrypi.org/bcm2711/bcm2711-peripherals.pdf
register_bitfields! {
    u32,

    /// GPIO機能選択 1
    GPFSEL1 [
        /// ピン15
        FSEL15 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AltFunc0 = 0b100  // PL011 UART RX

        ],

        /// ピン 14
        FSEL14 OFFSET(12) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AltFunc0 = 0b100  // PL011 UART TX
        ]
    ],

    /// GPIOプルアップ/プルダウンレジスタ
    ///
    /// BCM2837のみ
    GPPUD [
        /// すべてのGPIOピンの内部プルアップ/プルダウンコントロールラインの動作を制御する
        PUD OFFSET(0) NUMBITS(2) [
            Off = 0b00,
            PullDown = 0b01,
            PullUp = 0b10
        ]
    ],

    /// GPIOプルアップ/プルダウンクロックレジスタ
    ///
    /// BCM2837のみ
    GPPUDCLK0 [
        /// ピン 15
        PUDCLK15 OFFSET(15) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// ピン 14
        PUDCLK14 OFFSET(14) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ]
    ],

    /// GPIOプルアップ/プルダウンレジスタ 0
    ///
    /// BCM2711のみ
    GPIO_PUP_PDN_CNTRL_REG0 [
        /// ピン 15
        GPIO_PUP_PDN_CNTRL15 OFFSET(30) NUMBITS(2) [
            NoResistor = 0b00,
            PullUp = 0b01
        ],

        /// ピン 14
        GPIO_PUP_PDN_CNTRL14 OFFSET(28) NUMBITS(2) [
            NoResistor = 0b00,
            PullUp = 0b01
        ]
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    RegisterBlock {
        (0x00 => _reserved1),
        (0x04 => GPFSEL1: ReadWrite<u32, GPFSEL1::Register>),
        (0x08 => _reserved2),
        (0x94 => GPPUD: ReadWrite<u32, GPPUD::Register>),
        (0x98 => GPPUDCLK0: ReadWrite<u32, GPPUDCLK0::Register>),
        (0x9C => _reserved3),
        (0xE4 => GPIO_PUP_PDN_CNTRL_REG0: ReadWrite<u32, GPIO_PUP_PDN_CNTRL_REG0::Register>),
        (0xE8 => @END),
    }
}

/// 対応するMMIOレジスタのための抽象化
type Registers = MMIODerefWrapper<RegisterBlock>;

//--------------------------------------------------------------------------------------------------
// パブリック定義
//--------------------------------------------------------------------------------------------------

pub struct GPIOInner {
    registers: Registers,
}

// BSPがpanicハンドラで使用できるように内部構造体をエクスポートする
pub use GPIOInner as PanicGPIO;

/// GPIO HWを表す構造体
pub struct GPIO {
    inner: NullLock<GPIOInner>,
}

//--------------------------------------------------------------------------------------------------
// パブリックコード
//--------------------------------------------------------------------------------------------------

impl GPIOInner {
    /// インスタンスを作成する
    ///
    /// # 安全性
    ///
    /// - ユーザは正しいMMIO開始アドレスを提供する必要がある
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            registers: Registers::new(mmio_start_addr),
        }
    }

    /// ピン14と15のプルアップ/プルダウンを無効にする
    #[cfg(feature = "bsp_rpi3")]
    fn disable_pud_14_15_bcm2837(&mut self) {
        use crate::cpu;

        // （BCM2837ペリフェラルのPDFに記載されているシーケンスの）適切な遅延値を
        // 経験的に推測する。
        //   - Wikipediaによると、最速のPi3のクロックは1.4GHz程度
        //   - Linuxの2837 GPIOドライバは、ステップ間で1μs待つ
        //
        // 安全側にふって、デフォルトを2000サイクルとする。CPUのクロックが2GHzの場合、
        // この値は1μsに相当する。
        const DELAY: usize = 2000;

        self.registers.GPPUD.write(GPPUD::PUD::Off);
        cpu::spin_for_cycles(DELAY);

        self.registers
            .GPPUDCLK0
            .write(GPPUDCLK0::PUDCLK15::AssertClock + GPPUDCLK0::PUDCLK14::AssertClock);
        cpu::spin_for_cycles(DELAY);

        self.registers.GPPUD.write(GPPUD::PUD::Off);
        self.registers.GPPUDCLK0.set(0);
    }

    /// ピン14と15のプルアップ/プルダウンを無効にする
    #[cfg(feature = "bsp_rpi4")]
    fn disable_pud_14_15_bcm2711(&mut self) {
        self.registers.GPIO_PUP_PDN_CNTRL_REG0.write(
            GPIO_PUP_PDN_CNTRL_REG0::GPIO_PUP_PDN_CNTRL15::PullUp
                + GPIO_PUP_PDN_CNTRL_REG0::GPIO_PUP_PDN_CNTRL14::PullUp,
        );
    }

    /// PL011 UARTを標準アウトプットにマップする
    ///
    /// TXをピン14に
    /// RXをピン15に
    pub fn map_pl011_uart(&mut self) {
        // ピン14と15のUARTを選択する
        self.registers
            .GPFSEL1
            .modify(GPFSEL1::FSEL15::AltFunc0 + GPFSEL1::FSEL14::AltFunc0);

        // ピン14と15のプルアップ/プルダウンを無効にする
        #[cfg(feature = "bsp_rpi3")]
        self.disable_pud_14_15_bcm2837();

        #[cfg(feature = "bsp_rpi4")]
        self.disable_pud_14_15_bcm2711();
    }
}

impl GPIO {
    /// インスタンスを作成する
    ///
    /// # 安全性
    ///
    /// - ユーザは正しいMMIO開始アドレスを提供する必要がある
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            inner: NullLock::new(GPIOInner::new(mmio_start_addr)),
        }
    }

    /// `GPIOInner.map_pl011_uart()`の並行処理性安全バージョン
    pub fn map_pl011_uart(&self) {
        self.inner.lock(|inner| inner.map_pl011_uart())
    }
}

//------------------------------------------------------------------------------
// OSインタフェースコード
//------------------------------------------------------------------------------
use synchronization::interface::Mutex;

impl driver::interface::DeviceDriver for GPIO {
    fn compatible(&self) -> &'static str {
        "BCM GPIO"
    }
}
