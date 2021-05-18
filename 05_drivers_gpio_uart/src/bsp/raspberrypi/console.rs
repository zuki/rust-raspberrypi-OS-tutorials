// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! BSPコンソール装置

use super::memory;
use crate::{bsp::device_driver, console};
use core::fmt;

//--------------------------------------------------------------------------------------------------
// パブリックコード
//--------------------------------------------------------------------------------------------------

/// パニックが発生した場合、パニックハンドラはこの関数を使用してシステムが停止する前に
/// 何かをプリントするという最後の手段をとる。
///
/// GPIOとUARTのパニックバージョンの初期化を試みる。パニックバージョンは同期プリミティブで
/// 保護されていないため、パニック発生時にカーネルデフォルトのGPIOやUARTインスタンスが
/// たまたまロックされていても何かをプリントできる可能性は大きい。
///
/// # 安全性
///
/// - パニック時のプリントにのみ使用する
pub unsafe fn panic_console_out() -> impl fmt::Write {
    let mut panic_gpio = device_driver::PanicGPIO::new(memory::map::mmio::GPIO_START);
    let mut panic_uart = device_driver::PanicUart::new(memory::map::mmio::PL011_UART_START);

    panic_gpio.map_pl011_uart();
    panic_uart.init();
    panic_uart
}

/// コンソールへの参照を返す
pub fn console() -> &'static impl console::interface::All {
    &super::PL011_UART
}
