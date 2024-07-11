// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>

//! Raspberry Pi 3/4用のトップレベルのBSPファイル

pub mod cpu;
pub mod driver;
pub mod memory;

//--------------------------------------------------------------------------------------------------
<<<<<<< HEAD
// グローバルインスタンス
//--------------------------------------------------------------------------------------------------
use super::device_driver;

static GPIO: device_driver::GPIO =
    unsafe { device_driver::GPIO::new(memory::map::mmio::GPIO_START) };

static PL011_UART: device_driver::PL011Uart =
    unsafe { device_driver::PL011Uart::new(memory::map::mmio::PL011_UART_START) };

//--------------------------------------------------------------------------------------------------
// パブリックコード
=======
// Public Code
>>>>>>> master
//--------------------------------------------------------------------------------------------------

/// ボード識別
pub fn board_name() -> &'static str {
    #[cfg(feature = "bsp_rpi3")]
    {
        "Raspberry Pi 3"
    }

    #[cfg(feature = "bsp_rpi4")]
    {
        "Raspberry Pi 4"
    }
}
