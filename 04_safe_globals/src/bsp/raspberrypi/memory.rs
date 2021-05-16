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
// パブリックコード
//--------------------------------------------------------------------------------------------------

/// .bssセクションに含まれる範囲を返す
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
