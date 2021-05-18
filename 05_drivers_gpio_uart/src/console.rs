// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! システムコンソール

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

    /// 本格的コンソール用のトレイトエイリアス
    pub trait All = Write + Read + Statistics;
}
