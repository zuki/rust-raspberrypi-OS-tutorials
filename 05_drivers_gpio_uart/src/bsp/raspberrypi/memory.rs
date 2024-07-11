// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>

//! BSPメモリ管理

//--------------------------------------------------------------------------------------------------
// パブリック定義
//--------------------------------------------------------------------------------------------------

/// ボードの物理メモリアドレス
#[rustfmt::skip]
pub(super) mod map {

    pub const GPIO_OFFSET:         usize = 0x0020_0000;
    pub const UART_OFFSET:         usize = 0x0020_1000;

    /// 物理デバイス
    #[cfg(feature = "bsp_rpi3")]
    pub mod mmio {
        use super::*;

        pub const START:            usize =         0x3F00_0000;
        pub const GPIO_START:       usize = START + GPIO_OFFSET;
        pub const PL011_UART_START: usize = START + UART_OFFSET;
    }

    /// 物理デバイス
    #[cfg(feature = "bsp_rpi4")]
    pub mod mmio {
        use super::*;

        pub const START:            usize =         0xFE00_0000;
        pub const GPIO_START:       usize = START + GPIO_OFFSET;
        pub const PL011_UART_START: usize = START + UART_OFFSET;
    }
}
