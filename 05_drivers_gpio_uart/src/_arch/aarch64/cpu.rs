// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>

//! アーキテクチャ固有のブートコード。
//!
//! # オリエンテーション
//!
//! archモジュールはpath属性を使って汎用モジュールにインポートされるので
//! このファイルのパスは次の通り:
//!
//! crate::cpu::arch_cpu

use aarch64_cpu::asm;

//--------------------------------------------------------------------------------------------------
// パブリックコード
//--------------------------------------------------------------------------------------------------

pub use asm::nop;

/// `n`サイクルスピンする
#[cfg(feature = "bsp_rpi3")]
#[inline(always)]
pub fn spin_for_cycles(n: usize) {
    for _ in 0..n {
        asm::nop();
    }
}

/// コア上での実行を休止する
#[inline(always)]
pub fn wait_forever() -> ! {
    loop {
        asm::wfe()
    }
}
