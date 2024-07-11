// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>

//! PL011 UARTドライバ
//!
//! # 参考資料
//!
//! - <https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf>
//! - <https://developer.arm.com/documentation/ddi0183/latest>

use crate::{
    bsp::device_driver::common::MMIODerefWrapper, console, cpu, driver, synchronization,
    synchronization::NullLock,
};
use core::fmt;
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

//--------------------------------------------------------------------------------------------------
// プライベート定義
//--------------------------------------------------------------------------------------------------

// PL011 UARTレジスタ
//
// 記述は"PrimeCell UART (PL011) Technical Reference Manual" r1p5から採った
register_bitfields! {
    u32,

    /// フラグレジスタ
    FR [
        /// 送信FIFOが空。このビットの意味は、ラインコントロールレジスタ
        /// (LCR_H）のFENビットの状態に依存する。
        ///
        /// - FIFOが無効(FEN=0)の場合、送信ホールディングレジスタが空の時にこのビットがセットされる。
        /// - FIFOが有効(FEN=10な場合、送信FIFOが空の時にこのビットがセットされる。
        /// - このビットは、送信シフトレジスタにデータがあるか否かは示さない。
        TXFE OFFSET(7) NUMBITS(1) [],

        /// 送信FIFOが満杯。このビットの意味は、ラインコントロールレジスタ
        /// (LCR_H）のFENビットの状態に依存する。
        ///
        /// - FIFOが無効(FEN=0)の場合、送信ホールディングレジスタが満杯の時にこのビットがセットされる。
        /// - FIFOが有効(FEN=10な場合、送信FIFOが満杯の時にこのビットがセットされる。
        TXFF OFFSET(5) NUMBITS(1) [],

        /// 受信FIFOが空。このビットの意味は、ラインコントロールレジスタ
        /// (LCR_H）のFENビットの状態に依存する。
        ///
        /// - FIFOが無効(FEN=0)の場合、受信ホールディングレジスタが空の時にこのビットがセットされる。
        /// - FIFOが有効(FEN=10な場合、受信FIFOが空の時にこのビットがセットされる。
        RXFE OFFSET(4) NUMBITS(1) [],
        /// 受信FIFOが満杯。このビットの意味は、ラインコントロールレジスタ
        /// (LCR_H）のFENビットの状態に依存する。
        ///
        /// - FIFOが無効(FEN=0)の場合、受信ホールディングレジスタが満杯の時にこのビットがセットされる。
        /// - FIFOが有効(FEN=10な場合、受信FIFOが満杯の時にこのビットがセットされる。
        RXFF OFFSET(6) NUMBITS(1) [],

        /// UARTがビジー。このビットが1にセットされている場合、UARTはデータの
        /// 送信中でビジーである。バイト送信が完了する（ストップビットを
        /// すべてのビットがシフトレジスタから送信される）までこのビットは
        /// セットされ続ける。
        ///
        /// 送信FIFOが空でなくなると、UARTが有効か否かにかかわらず、
        /// 直ちにこのビットはセットされる。
        BUSY OFFSET(3) NUMBITS(1) []
    ],

    /// 通信速度整数除数レジスタ
    IBRD [
        /// 通信速度除数の整数部分
        BAUD_DIVINT OFFSET(0) NUMBITS(16) []
    ],

    /// 通信速度小数除数レジスタ
    FBRD [
        ///  通信速度除数の小数部分
        BAUD_DIVFRAC OFFSET(0) NUMBITS(6) []
    ],

    /// ラインコントロールレジスタ
    LCR_H [
        /// ワード長。このビットは送信または受信する1フレームのデータビット数を
        /// 示す。
        #[allow(clippy::enum_variant_names)]
        WLEN OFFSET(5) NUMBITS(2) [
            FiveBit = 0b00,
            SixBit = 0b01,
            SevenBit = 0b10,
            EightBit = 0b11
        ],

        /// FIFOを有効にする
        ///
        /// 0 = FIFOは無効（キャラクタモード）。FIFOは1バイトのホールディング
        /// レジスタになる。
        ///
        /// 1 = 送信/受信FIFOバッファは有効（FIFOモード）。
        FEN  OFFSET(4) NUMBITS(1) [
            FifosDisabled = 0,
            FifosEnabled = 1
        ]
    ],

    /// コントロールレジスタ
    CR [
        /// 受信は有効。このビットが1にセットされている場合、UARTの受信
        /// セクションは有効である。SIRENビットの設定に応じて、UART信号
        /// またはSIR信号のいずれかでデータの受信が行われる。受信の途中で
        /// UARTが無効になった場合は、現在のキャラクタの受信を完了して
        /// から停止する。
        RXE OFFSET(9) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],

        /// 送信は有効。このビットが1にセットされている場合、UARTの送信
        /// セクションは有効である。SIRENビットの設定に応じて、UART信号
        /// またはSIR信号のいずれかでデータの送信が行われる。送信の途中で
        /// UARTが無効になった場合は、現在のキャラクタの送信を完了して
        /// から停止する。
        TXE OFFSET(8) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],

        /// UARTを有効にする
        ///
        /// 0 = UARTは無効。 送信または受信の途中でUARTが無効になった場合は、
        /// 現在のキャラクタの送信または受信を完了してから停止する。
        ///
        /// 1 = UARTは有効。SIRENビットの設定に応じて、UART信号またはSIR信号の
        /// いずれかでデータの送信または受信が行われる。
        UARTEN OFFSET(0) NUMBITS(1) [
            /// If the UART is disabled in the middle of transmission or reception, it completes the
            /// current character before stopping.
            Disabled = 0,
            Enabled = 1
        ]
    ],

    /// 割り込みクリアレジスタ
    ICR [
        /// すべての保留中割り込みを示すメタフィールド
        ALL OFFSET(0) NUMBITS(11) []
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    pub RegisterBlock {
        (0x00 => DR: ReadWrite<u32>),
        (0x04 => _reserved1),
        (0x18 => FR: ReadOnly<u32, FR::Register>),
        (0x1c => _reserved2),
        (0x24 => IBRD: WriteOnly<u32, IBRD::Register>),
        (0x28 => FBRD: WriteOnly<u32, FBRD::Register>),
        (0x2c => LCR_H: WriteOnly<u32, LCR_H::Register>),
        (0x30 => CR: WriteOnly<u32, CR::Register>),
        (0x34 => _reserved3),
        (0x44 => ICR: WriteOnly<u32, ICR::Register>),
        (0x48 => @END),
    }
}

/// 対応するMMIOレジスタのための抽象化
type Registers = MMIODerefWrapper<RegisterBlock>;

#[derive(PartialEq)]
enum BlockingMode {
    Blocking,
    NonBlocking,
}

struct PL011UartInner {
    registers: Registers,
    chars_written: usize,
    chars_read: usize,
}

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// UARTを表す構造体
pub struct PL011Uart {
    inner: NullLock<PL011UartInner>,
}

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

impl PL011UartInner {
    /// インスタンスを作成する
    ///
    /// # 安全性
    ///
    /// - ユーザは正しいMMIO開始アドレスを提供する必要がある
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            registers: Registers::new(mmio_start_addr),
            chars_written: 0,
            chars_read: 0,
        }
    }

    /// 通信速度と特性を設定するSet up baud rate and characteristics.
    ///
    /// 8N1で921_600ボーと設定される。
    ///
    /// BRDの計算は（config.txtでクロックを48MHzと設定したので）次のようになる。
    /// `(48_000_000 / 16) / 921_600 = 3.2552083`.
    ///
    /// これにより、整数部分は`3`になり、`IBRD`に設定し、
    /// 小数部分は`0.2552083`となる。
    ///
    /// `FBRD`はPL011テクニカルリファレンスマニュアルにしたがって計算すると
    /// 次のようになる。
    /// `INTEGER((0.2552083 * 64) + 0.5) = 16`.
    ///
    /// したがって、生成される通信速度除数は`3 + 16/64 = 3.25`である。
    /// これにより生成される通信速は`48_000_000 / (16 * 3.25) = 923_077`となる。
    ///
    /// エラー = `((923_077 - 921_600) / 921_600) * 100 = 0.16%`である。
    pub fn init(&mut self) {
        // TX FIFOにまだ文字がキューイングされており、UARTハードウェアが
        // アクティブに送信している時に実行がここに到着する可能性がある。
        // この場合にUARTがオフになると、キューに入っていた文字が失われる。
        //
        // たとえば、実行中にpanic!()が呼び出された時にこのような事態が
        // 発生する可能性がある。panic!()が自身のUARTインスタンスを初期化して
        // init()を呼び出すからである。
        //
        // そのため、保留中の文字がすべて送信されるように最初にフラッシュする。
        self.flush();

        // 一時的にUARTを無効にする
        self.registers.CR.set(0);

        // すべての保留中の割り込みをクリアする
        self.registers.ICR.write(ICR::ALL::CLEAR);

        // PL011テクニカルリファレンスマニュアルから:
        //
        // LCR_H、IBRD、FBRDの各レジスタは、LCR_Hの書き込みにより生成される
        // 1回の書き込みストローブで更新される30ビット幅のLCRレジスタを形成
        // する。そのため、IBRDやFBRDの内容を内部的に更新するには、常に
        // LCR_Hの書き込みを最後に行う必要がある。
        //
        // 通信速度と8N1を設定し、FIFOを有効にする。
        self.registers.IBRD.write(IBRD::BAUD_DIVINT.val(3));
        self.registers.FBRD.write(FBRD::BAUD_DIVFRAC.val(16));
        self.registers
            .LCR_H
            .write(LCR_H::WLEN::EightBit + LCR_H::FEN::FifosEnabled);

        // UARTを有効にする。
        self.registers
            .CR
            .write(CR::UARTEN::Enabled + CR::TXE::Enabled + CR::RXE::Enabled);
    }

    /// 1文字送信する
    fn write_char(&mut self, c: char) {
        // スロットが開くのを待って、TX FIFOフルが設定されている間、スピンする。
        while self.registers.FR.matches_all(FR::TXFF::SET) {
            cpu::nop();
        }

        // 文字をバッファに書き込む
        self.registers.DR.set(c as u32);

        self.chars_written += 1;
    }

    /// バッファされた最後の文字が物理的にTXワイヤに置かれるまで実行をブロックする
    fn flush(&self) {
        // ビジービットがクリアされるまでスピンする
        while self.registers.FR.matches_all(FR::BUSY::SET) {
            cpu::nop();
        }
    }

    /// 1文字受信する
    fn read_char_converting(&mut self, blocking_mode: BlockingMode) -> Option<char> {
        // RX FIFOがからの場合
        if self.registers.FR.matches_all(FR::RXFE::SET) {
            // ノンブロッキングモードの場合はすぐにリターンする
            if blocking_mode == BlockingMode::NonBlocking {
                return None;
            }

            // そうでなければ、1文字受信されるまで待つ
            while self.registers.FR.matches_all(FR::RXFE::SET) {
                cpu::nop();
            }
        }

        // 1文字読み込む
        let mut ret = self.registers.DR.get() as u8 as char;

        // 復帰を改行に変換する
        if ret == '\r' {
            ret = '\n'
        }

        // 統計を更新する
        self.chars_read += 1;

        Some(ret)
    }
}

/// `core::fmt::Write`を実装すると`format_args!`マクロが利用可能になる。これはひいては
/// `カーネル`の`print!`と`println!`マクロを実装することになる。`write_str()`を実装する
/// ことにより自動的に`write_fmt()`を手にすることができる。
///
/// この関数は `&mut self` を取るので、内部構造体を実装する必要がある
///
/// [`src/print.rs`]を参照
///
/// [`src/print.rs`]: ../../print/index.html
impl fmt::Write for PL011UartInner {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }

        Ok(())
    }
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl PL011Uart {
    pub const COMPATIBLE: &'static str = "BCM PL011 UART";

    /// インスタンスを作成する
    ///
    /// # Safety
    ///
    /// - ユーザは正しいMMIO開始アドレスを提供する必要がある
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            inner: NullLock::new(PL011UartInner::new(mmio_start_addr)),
        }
    }
}

//------------------------------------------------------------------------------
// OSインタフェースコード
//------------------------------------------------------------------------------
use synchronization::interface::Mutex;

impl driver::interface::DeviceDriver for PL011Uart {
    fn compatible(&self) -> &'static str {
        Self::COMPATIBLE
    }

    unsafe fn init(&self) -> Result<(), &'static str> {
        self.inner.lock(|inner| inner.init());

        Ok(())
    }
}

impl console::interface::Write for PL011Uart {
    /// `core::fmt::Write`の実装に`args`をそのまま渡すが、ミューテックスで
    /// ガードしてアクセスをシリアライズしている
    fn write_char(&self, c: char) {
        self.inner.lock(|inner| inner.write_char(c));
    }

    fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
        // 可読性を高めるために`core::fmt::Write::write:fmt()`の
        // 呼び出しに完全修飾構文を採用
        self.inner.lock(|inner| fmt::Write::write_fmt(inner, args))
    }

    fn flush(&self) {
        // TX FIFOが空になるまでスピンする
        self.inner.lock(|inner| inner.flush());
    }
}

impl console::interface::Read for PL011Uart {
    fn read_char(&self) -> char {
        self.inner
            .lock(|inner| inner.read_char_converting(BlockingMode::Blocking).unwrap())
    }

    fn clear_rx(&self) {
        // 空になるまでRX FIFOを読み込む
        while self
            .inner
            .lock(|inner| inner.read_char_converting(BlockingMode::NonBlocking))
            .is_some()
        {}
    }
}

impl console::interface::Statistics for PL011Uart {
    fn chars_written(&self) -> usize {
        self.inner.lock(|inner| inner.chars_written)
    }

    fn chars_read(&self) -> usize {
        self.inner.lock(|inner| inner.chars_read)
    }
}

impl console::interface::All for PL011Uart {}
