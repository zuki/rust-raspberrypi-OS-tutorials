// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2023 Andre Richter <andre.o.richter@gmail.com>

//! 同期プリミティブ
//!
//! # 参考
//!
//!   - <https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html>
//!   - <https://stackoverflow.com/questions/59428096/understanding-the-send-trait>
//!   - <https://doc.rust-lang.org/std/cell/index.html>

use core::cell::UnsafeCell;

//--------------------------------------------------------------------------------------------------
// パブリック定義
//--------------------------------------------------------------------------------------------------

/// 同期インタフェース
pub mod interface {

    /// このトレイトを実装しているオブジェクトは、与えられたクロージャにおいて
    /// Mutexでラップされたデータへの排他的アクセスを保証する
    pub trait Mutex {
        /// このmutexでラップされるデータの型
        type Data;

        /// mutexをロックし、ラップされたデータへの一時的可変アクセスをクロージャに保証する.
        fn lock<'a, R>(&'a self, f: impl FnOnce(&'a mut Self::Data) -> R) -> R;
    }
}

/// 教育目的の疑似ロック
///
/// 実際のMutexの実装とは異なり、保持するデータに対する他のコアからの同時アクセス
/// からは保護されない。この部分は後のレッスン用に残される。
///
/// ロックは、そうすることが安全な場合に限り、すなわち、カーネルがシングルスレッド、
/// つまり割り込み無効でシングルコアで実行されている場合に限り、使用される。
pub struct NullLock<T>
where
    T: ?Sized,
{
    data: UnsafeCell<T>,
}

//--------------------------------------------------------------------------------------------------
// パブリックコード
//--------------------------------------------------------------------------------------------------

unsafe impl<T> Send for NullLock<T> where T: ?Sized + Send {}
unsafe impl<T> Sync for NullLock<T> where T: ?Sized + Send {}

impl<T> NullLock<T> {
    /// インスタンスを作成する
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }
}

//------------------------------------------------------------------------------
// OSインタフェースコード
//------------------------------------------------------------------------------

impl<T> interface::Mutex for NullLock<T> {
    type Data = T;

    fn lock<'a, R>(&'a self, f: impl FnOnce(&'a mut Self::Data) -> R) -> R {
        // 実際のロックでは、この行をカプセル化するコードがあり、
        // この可変参照が一度に一つしか渡されないことを保証している
        let data = unsafe { &mut *self.data.get() };

        f(data)
    }
}
