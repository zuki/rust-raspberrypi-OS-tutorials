// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>

//! システムコンソール

mod null_console;

use crate::synchronization::{self, NullLock};

//--------------------------------------------------------------------------------------------------
// パブリック定義
//--------------------------------------------------------------------------------------------------

/// コンソールインタフェース
pub mod interface {
    use core::fmt;

    /// コンソールwrite関数
    pub trait Write {
        /// 1文字Write
        fn write_char(&self, c: char);

        /// Rust形式の文字列をWrite
        fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;

        /// バッファされた最後の文字が物理的にTXワイヤに置かれるまでブロックする
        fn flush(&self);
    }

    /// コンソールread関数
    pub trait Read {
        /// 1文字Read
        fn read_char(&self) -> char {
            ' '
        }

        /// もしあれば、RXバッファをクリアする
        fn clear_rx(&self);
    }

    /// コンソール統計
    pub trait Statistics {
        /// 書き出した文字数を返す
        fn chars_written(&self) -> usize {
            0
        }

        /// 読み込んだ文字数を返す
        fn chars_read(&self) -> usize {
            0
        }
    }

<<<<<<< HEAD
    /// 本格的コンソール用のトレイトエイリアス
    pub trait All = Write + Read + Statistics;
=======
    /// Trait alias for a full-fledged console.
    pub trait All: Write + Read + Statistics {}
}

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

static CUR_CONSOLE: NullLock<&'static (dyn interface::All + Sync)> =
    NullLock::new(&null_console::NULL_CONSOLE);

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------
use synchronization::interface::Mutex;

/// Register a new console.
pub fn register_console(new_console: &'static (dyn interface::All + Sync)) {
    CUR_CONSOLE.lock(|con| *con = new_console);
}

/// Return a reference to the currently registered console.
///
/// This is the global console used by all printing macros.
pub fn console() -> &'static dyn interface::All {
    CUR_CONSOLE.lock(|con| *con)
>>>>>>> master
}
