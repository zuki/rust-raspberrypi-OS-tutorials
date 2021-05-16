// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! メモリ管理

use core::ops::RangeInclusive;

//--------------------------------------------------------------------------------------------------
// パブリックコード
//--------------------------------------------------------------------------------------------------

/// メモリ範囲をゼロ詰めする
///
/// # 安全性
///
/// - `range.start` と `range.end` はvalidでなければならない
/// - `range.start` と `range.end` は`T`アラインされていなければならない
pub unsafe fn zero_volatile<T>(range: RangeInclusive<*mut T>)
where
    T: From<u8>,
{
    let mut ptr = *range.start();
    let end_inclusive = *range.end();

    while ptr <= end_inclusive {
        core::ptr::write_volatile(ptr, T::from(0));
        ptr = ptr.offset(1);
    }
}
