// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>

//! システムコンソール

use crate::bsp;

//--------------------------------------------------------------------------------------------------
// パブリック定義
//--------------------------------------------------------------------------------------------------

/// コンソールインタフェース
pub mod interface {
    /// コンソール write関数
    ///
    /// `core::fmt::Write` は今まさに必要なもの。console::Write`の実装が
    /// 読者に意図を伝える良いヒントになるので、ここで再エクスポートする。
    pub use core::fmt::Write;
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Return a reference to the console.
///
/// This is the global console used by all printing macros.
pub fn console() -> impl interface::Write {
    bsp::console::console()
}
