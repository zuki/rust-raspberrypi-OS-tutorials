# チュートリアル 04 - 安全なグローバル

## tl;dr

- 疑似ロックを導入します。
- 擬似ロックはOSの同期プリミティブの最初のショーケースであり、グローバルなデータ構造への安全なアクセスを可能にします。

## Rustにおけるミュータブルなグローバル

[チュートリアル 03]でグローバルに使用可能な`print!`マクロを導入した際、少しずるを
しました。`core::fmt`の`write_fmt()`関数は`&mut self`を受け取りますが、これが
動作したのは、呼び出すごとに`QEMUOutput`の新しいインスタンスを生成していたから
です。

書き込んだ文字数の統計など、何らかの状態を保持したい場合、`QEMUOutput`の
グローバルインスタンスを一つ作成する必要があります（Rustでは`static`キーワードを
使用します）。

しかし、`static QEMU_OUTPUT`は`&mut self`を取る関数を呼び出すことができません。
そのためには`static mut`が必要です。しかし、`static mut`で状態を変更する関数を
呼び出すことは安全ではありません。Rustコンパイラにおけるその理由は、複数の
コアやスレッドがデータを同時に変異することを防ぐことができないからです
（グローバルなので誰でもどこからでも参照できます。ここでは借用チェッカーは
役に立ちません）。

この問題を解決するにはグローバルを同期プリミティブでラップすればよいです。
ここでは、*MUTual EXclusion*プリミティブの一つを使います。`Mutex`は
`synchronization.rs`でトレイトとして導入され、同じファイルで`NullLock`に
よって実装されています。教育用に簡潔なコードにするため、同時アクセスを保護
するためのアーキテクチャ固有の実際的なロジックは省いています。カーネルが
シングルコアで割り込みを無効にして実行している限り、これは必要ないからです。

`NullLock`はRustの中心コンセプトである内部可変性を紹介することに重点を
置いています。[この文書]を読んでみてください。また、Rustの参照型の正確な
メンタルモデルについては、[こちらの文書]を読むことをおすすめします。

`NullLock`とその他の実在するmutexの実装を比較したい場合は、[spin]クレイトや
[parking lot]クレイトの実装をチェックしてください。

[チュートリアル 03]: ../03_hacky_hello_world
[この文書]: https://doc.rust-lang.org/std/cell/index.html
[こちらの文書]: https://docs.rs/dtolnay/0.0.6/dtolnay/macro._02__reference_types.html
[spin]: https://github.com/mvdnes/spin-rs
[parking lot]: https://github.com/Amanieu/parking_lot

## テスト

```console
$ make qemu
[...]

[0] Hello from Rust!
[1] Chars written: 22
[2] Stopping here.
```

## 前チュートリアルとのdiff
```diff

diff -uNr 03_hacky_hello_world/Cargo.toml 04_safe_globals/Cargo.toml
--- 03_hacky_hello_world/Cargo.toml
+++ 04_safe_globals/Cargo.toml
@@ -1,6 +1,6 @@
 [package]
 name = "mingo"
-version = "0.3.0"
+version = "0.4.0"
 authors = ["Andre Richter <andre.o.richter@gmail.com>"]
 edition = "2018"


diff -uNr 03_hacky_hello_world/src/bsp/raspberrypi/console.rs 04_safe_globals/src/bsp/raspberrypi/console.rs
--- 03_hacky_hello_world/src/bsp/raspberrypi/console.rs
+++ 04_safe_globals/src/bsp/raspberrypi/console.rs
@@ -4,7 +4,7 @@

 //! BSPコンソール装置

-use crate::console;
+use crate::{console, synchronization, synchronization::NullLock};
 use core::fmt;

 //--------------------------------------------------------------------------------------------------
@@ -12,25 +12,64 @@
 //--------------------------------------------------------------------------------------------------

 /// QEMUの出力を無から生成する神秘的で魔法のような装置
-struct QEMUOutput;
+///
+/// mutexで保護される部分.
+struct QEMUOutputInner {
+    chars_written: usize,
+}

 //--------------------------------------------------------------------------------------------------
-// プライベート定義
+// パブリックコード
+//--------------------------------------------------------------------------------------------------
+
+/// メイン構造体
+pub struct QEMUOutput {
+    inner: NullLock<QEMUOutputInner>,
+}
+
 //--------------------------------------------------------------------------------------------------
+// グローバルインスタンス
+//--------------------------------------------------------------------------------------------------
+
+static QEMU_OUTPUT: QEMUOutput = QEMUOutput::new();
+
+//--------------------------------------------------------------------------------------------------
+// プライベートコード
+//--------------------------------------------------------------------------------------------------
+
+impl QEMUOutputInner {
+    const fn new() -> QEMUOutputInner {
+        QEMUOutputInner { chars_written: 0 }
+    }
+
+    /// 1文字送信
+    fn write_char(&mut self, c: char) {
+        unsafe {
+            core::ptr::write_volatile(0x3F20_1000 as *mut u8, c as u8);
+        }
+
+        self.chars_written += 1;
+    }
+}

 /// `core::fmt::Write`を実装すると`format_args!`マクロが利用可能になる。これはひいては
 /// `カーネル`の`print!`と`println!`マクロを実装することになる。`write_str()`を実装する
 /// ことにより自動的に`write_fmt()`を手にすることができる。
 ///
+/// この関数は `&mut self` を取るので、内部構造体を実装する必要がある
+///
 /// [`src/print.rs`]を参照
 ///
 /// [`src/print.rs`]: ../../print/index.html
-impl fmt::Write for QEMUOutput {
+impl fmt::Write for QEMUOutputInner {
     fn write_str(&mut self, s: &str) -> fmt::Result {
         for c in s.chars() {
-            unsafe {
-                core::ptr::write_volatile(0x3F20_1000 as *mut u8, c as u8);
+            // 改行を復帰+改行に変換する
+            if c == '\n' {
+                self.write_char('\r')
             }
+
+            self.write_char(c);
         }

         Ok(())
@@ -41,7 +80,37 @@
 // パブリックコード
 //--------------------------------------------------------------------------------------------------

+impl QEMUOutput {
+    /// 新しいインスタンスを作成する
+    pub const fn new() -> QEMUOutput {
+        QEMUOutput {
+            inner: NullLock::new(QEMUOutputInner::new()),
+        }
+    }
+}
+
 /// コンソールへの参照を返す
-pub fn console() -> impl console::interface::Write {
-    QEMUOutput {}
+pub fn console() -> &'static impl console::interface::All {
+    &QEMU_OUTPUT
+}
+
+//------------------------------------------------------------------------------
+// OSインタフェースコード
+//------------------------------------------------------------------------------
+use synchronization::interface::Mutex;
+
+/// `core::fmt::Write`の実装に`args`をそのまま渡すが、ミューテックスで
+/// ガードしてアクセスをシリアライズしている
+impl console::interface::Write for QEMUOutput {
+    fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
+        // 可読性を高めるために`core::fmt::Write::write:fmt()`の
+        // 呼び出しに完全修飾構文を採用
+        self.inner.lock(|inner| fmt::Write::write_fmt(inner, args))
+    }
+}
+
+impl console::interface::Statistics for QEMUOutput {
+    fn chars_written(&self) -> usize {
+        self.inner.lock(|inner| inner.chars_written)
+    }
 }

diff -uNr 03_hacky_hello_world/src/console.rs 04_safe_globals/src/console.rs
--- 03_hacky_hello_world/src/console.rs
+++ 04_safe_globals/src/console.rs
@@ -10,9 +10,22 @@

 /// コンソールインタフェース
 pub mod interface {
+    use core::fmt;
+
     /// コンソール write関数
-    ///
-    /// `core::fmt::Write` は今まさに必要なもの。console::Write`の実装が
-    /// 読者に意図を伝える良いヒントになるので、ここで再エクスポートする。
-    pub use core::fmt::Write;
+    pub trait Write {
+        /// Rust形式の文字列をWrite
+        fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;
+    }
+
+    /// コンソール統計
+    pub trait Statistics {
+        /// 書き込んだ文字数を返す
+        fn chars_written(&self) -> usize {
+            0
+        }
+    }
+
+    /// 本格的コンソール用のトレイトエイリアス
+    pub trait All = Write + Statistics;
 }

diff -uNr 03_hacky_hello_world/src/main.rs 04_safe_globals/src/main.rs
--- 03_hacky_hello_world/src/main.rs
+++ 04_safe_globals/src/main.rs
@@ -109,6 +109,7 @@
 #![feature(format_args_nl)]
 #![feature(global_asm)]
 #![feature(panic_info_message)]
+#![feature(trait_alias)]
 #![no_main]
 #![no_std]

@@ -119,6 +120,7 @@
 mod panic_wait;
 mod print;
 mod runtime_init;
+mod synchronization;

 /// 最初の初期化コード
 ///
@@ -126,7 +128,15 @@
 ///
 /// - アクティブなコアはこの関数を実行しているコアだけでなければならない
 unsafe fn kernel_init() -> ! {
+    use console::interface::Statistics;
+
     println!("[0] Hello from Rust!");

-    panic!("Stopping here.")
+    println!(
+        "[1] Chars written: {}",
+        bsp::console::console().chars_written()
+    );
+
+    println!("[2] Stopping here.");
+    cpu::wait_forever()
 }

diff -uNr 03_hacky_hello_world/src/synchronization.rs 04_safe_globals/src/synchronization.rs
--- 03_hacky_hello_world/src/synchronization.rs
+++ 04_safe_globals/src/synchronization.rs
@@ -0,0 +1,77 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! 同期プリミティブ
+//!
+//! # 参考
+//!
+//!   - <https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html>
+//!   - <https://stackoverflow.com/questions/59428096/understanding-the-send-trait>
+//!   - <https://doc.rust-lang.org/std/cell/index.html>
+
+use core::cell::UnsafeCell;
+
+//--------------------------------------------------------------------------------------------------
+// パブリック定義
+//--------------------------------------------------------------------------------------------------
+
+/// 同期インタフェース
+pub mod interface {
+
+    /// このトレイトを実装しているオブジェクトは、与えられたクロージャにおいて
+    /// Mutexでラップされたデータへの排他的アクセスを保証する
+    pub trait Mutex {
+        /// このmutexでラップされるデータの型
+        type Data;
+
+        /// mutexをロックし、ラップされたデータへの一時的可変アクセスをクロージャに保証する
+        fn lock<R>(&self, f: impl FnOnce(&mut Self::Data) -> R) -> R;
+    }
+}
+
+/// 教育目的の疑似ロック
+///
+/// 実際のMutexの実装とは異なり、保持するデータに対する他のコアからの同時アクセス
+/// からは保護されない。この部分は後のレッスン用に残される。
+///
+/// ロックは、そうすることが安全な場合に限り、すなわち、カーネルがシングルスレッド、
+/// つまり割り込み無効でシングルコアで実行されている場合に限り、使用される。
+pub struct NullLock<T>
+where
+    T: ?Sized,
+{
+    data: UnsafeCell<T>,
+}
+
+//--------------------------------------------------------------------------------------------------
+// パブリックコード
+//--------------------------------------------------------------------------------------------------
+
+unsafe impl<T> Send for NullLock<T> where T: ?Sized + Send {}
+unsafe impl<T> Sync for NullLock<T> where T: ?Sized + Send {}
+
+impl<T> NullLock<T> {
+    /// インスタンスを作成する
+    pub const fn new(data: T) -> Self {
+        Self {
+            data: UnsafeCell::new(data),
+        }
+    }
+}
+
+//------------------------------------------------------------------------------
+// OSインタフェースコード
+//------------------------------------------------------------------------------
+
+impl<T> interface::Mutex for NullLock<T> {
+    type Data = T;
+
+    fn lock<R>(&self, f: impl FnOnce(&mut Self::Data) -> R) -> R {
+        // 実際のロックでは、この行をカプセル化するコードがあり、
+        // この可変参照が一度に一つしか渡されないことを保証している
+        let data = unsafe { &mut *self.data.get() };
+
+        f(data)
+    }
+}

```
