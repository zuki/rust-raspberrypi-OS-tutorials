// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! BSPメモリ管理

use core::{cell::UnsafeCell, ops::RangeInclusive};

//--------------------------------------------------------------------------------------------------
// プライベート定義
//--------------------------------------------------------------------------------------------------

// リンカスクリプトで定義されているシンボル
extern "Rust" {
    static __bss_start: UnsafeCell<u64>;
    static __bss_end_inclusive: UnsafeCell<u64>;
}

//--------------------------------------------------------------------------------------------------
// パブリック定義
//--------------------------------------------------------------------------------------------------

/// ボードの物理メモリアドレス
#[rustfmt::skip]
pub(super) mod map {
    pub const BOARD_DEFAULT_LOAD_ADDRESS: usize =        0x8_0000;

    pub const GPIO_OFFSET:                usize =        0x0020_0000;
    pub const UART_OFFSET:                usize =        0x0020_1000;

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

//--------------------------------------------------------------------------------------------------
// パブリックコード
//--------------------------------------------------------------------------------------------------

/// Raspberryのファームウェアがデフォルトですべてのバイナリをロードするアドレス
#[inline(always)]
pub fn board_default_load_addr() -> *const u64 {
    map::BOARD_DEFAULT_LOAD_ADDRESS as _
}

/// 再配置されたbssセクションに含まれる範囲を返す
///
/// # 安全性
///
/// - 値はリンカスクリプトが提供するものであり、そのまま信用する必要がある
/// - リンカスクリプトが提供するアドレスはu64にアラインされている必要がある
pub fn bss_range_inclusive() -> RangeInclusive<*mut u64> {
    let range;
    unsafe {
        range = RangeInclusive::new(__bss_start.get(), __bss_end_inclusive.get());
    }
    assert!(!range.is_empty());

    range
}
