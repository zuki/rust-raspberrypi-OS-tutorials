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
    use core::fmt;

    /// コンソール write関数
    pub trait Write {
        /// Rust形式の文字列をWrite
        fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;
    }

    /// コンソール統計
    pub trait Statistics {
        /// 書き込んだ文字数を返す
        fn chars_written(&self) -> usize {
            0
        }
    }

    /// 本格的コンソール用のトレイトエイリアス
    pub trait All = Write + Statistics;
}
