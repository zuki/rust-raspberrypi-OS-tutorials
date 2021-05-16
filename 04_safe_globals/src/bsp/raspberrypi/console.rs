// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! BSPコンソール装置

use crate::{console, synchronization, synchronization::NullLock};
use core::fmt;

//--------------------------------------------------------------------------------------------------
// プライベート定義
//--------------------------------------------------------------------------------------------------

/// QEMUの出力を無から生成する神秘的で魔法のような装置
///
/// mutexで保護される部分.
struct QEMUOutputInner {
    chars_written: usize,
}

//--------------------------------------------------------------------------------------------------
// パブリックコード
//--------------------------------------------------------------------------------------------------

/// メイン構造体
pub struct QEMUOutput {
    inner: NullLock<QEMUOutputInner>,
}

//--------------------------------------------------------------------------------------------------
// グローバルインスタンス
//--------------------------------------------------------------------------------------------------

static QEMU_OUTPUT: QEMUOutput = QEMUOutput::new();

//--------------------------------------------------------------------------------------------------
// プライベートコード
//--------------------------------------------------------------------------------------------------

impl QEMUOutputInner {
    const fn new() -> QEMUOutputInner {
        QEMUOutputInner { chars_written: 0 }
    }

    /// 1文字送信
    fn write_char(&mut self, c: char) {
        unsafe {
            core::ptr::write_volatile(0x3F20_1000 as *mut u8, c as u8);
        }

        self.chars_written += 1;
    }
}

/// `core::fmt::Write`を実装すると`format_args!`マクロが利用可能になる。これはひいては
/// `カーネル`の`print!`と`println!`マクロを実装することになる。`write_str()`を実装する
/// ことにより自動的に`write_fmt()`を手にすることができる。
///
/// この関数は `&mut self` を取るので、内部構造体を実装する必要がある
///
/// [`src/print.rs`]を参照
///
/// [`src/print.rs`]: ../../print/index.html
impl fmt::Write for QEMUOutputInner {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            // 改行を復帰+改行に変換する
            if c == '\n' {
                self.write_char('\r')
            }

            self.write_char(c);
        }

        Ok(())
    }
}

//--------------------------------------------------------------------------------------------------
// パブリックコード
//--------------------------------------------------------------------------------------------------

impl QEMUOutput {
    /// 新しいインスタンスを作成する
    pub const fn new() -> QEMUOutput {
        QEMUOutput {
            inner: NullLock::new(QEMUOutputInner::new()),
        }
    }
}

/// コンソールへの参照を返す
pub fn console() -> &'static impl console::interface::All {
    &QEMU_OUTPUT
}

//------------------------------------------------------------------------------
// OSインタフェースコード
//------------------------------------------------------------------------------
use synchronization::interface::Mutex;

/// `core::fmt::Write`の実装に`args`をそのまま渡すが、ミューテックスで
/// ガードしてアクセスをシリアライズしている
impl console::interface::Write for QEMUOutput {
    fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
        // 可読性を高めるために`core::fmt::Write::write:fmt()`の
        // 呼び出しに完全修飾構文を採用
        self.inner.lock(|inner| fmt::Write::write_fmt(inner, args))
    }
}

impl console::interface::Statistics for QEMUOutput {
    fn chars_written(&self) -> usize {
        self.inner.lock(|inner| inner.chars_written)
    }
}
