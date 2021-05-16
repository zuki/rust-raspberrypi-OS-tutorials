# チュートリアル 03 - ハック版 Hello World

## tl;dr

- グローバルな`print!()`マクロを導入し、いち早く「printfデバッグ」を可能にします。
- チュートリアルの長さを適度に保つため、プリント機能は現時点ではRaspberryの`UART`を正しく設定しなくても使用できるようにQEMUのプロパティを「悪用」します。
- 本物のハードウェア`UART`の使用については後のチュートリアルで順に説明します。

## 特筆すべき追加事項

- `src/console.rs`でコンソールコマンドのためのインタフェース`Traits`を導入します。
- `src/bsp/raspberrypi/console.rs`でQEMUのエミュレートUART用のインタフェースを実装します。
- パニックハンドラはこの新しい`print!()`を使ってエラーメッセージを表示します。

## テスト

このチュートリアルからはQEMUをアセンブリモードで実行しません。ここからは`console`出力を表示します。

```console
$ make qemu
[...]
Hello from Rust!

Kernel panic: Stopping here.
```

## 前チュートリアルとのdiff
```diff

diff -uNr 02_runtime_init/Cargo.toml 03_hacky_hello_world/Cargo.toml
--- 02_runtime_init/Cargo.toml
+++ 03_hacky_hello_world/Cargo.toml
@@ -1,6 +1,6 @@
 [package]
 name = "mingo"
-version = "0.2.0"
+version = "0.3.0"
 authors = ["Andre Richter <andre.o.richter@gmail.com>"]
 edition = "2018"


diff -uNr 02_runtime_init/Makefile 03_hacky_hello_world/Makefile
--- 02_runtime_init/Makefile
+++ 03_hacky_hello_world/Makefile
@@ -13,7 +13,7 @@
     KERNEL_BIN        = kernel8.img
     QEMU_BINARY       = qemu-system-aarch64
     QEMU_MACHINE_TYPE = raspi3
-    QEMU_RELEASE_ARGS = -d in_asm -display none
+    QEMU_RELEASE_ARGS = -serial stdio -display none
     OBJDUMP_BINARY    = aarch64-none-elf-objdump
     NM_BINARY         = aarch64-none-elf-nm
     READELF_BINARY    = aarch64-none-elf-readelf
@@ -24,7 +24,7 @@
     KERNEL_BIN        = kernel8.img
     QEMU_BINARY       = qemu-system-aarch64
     QEMU_MACHINE_TYPE =
-    QEMU_RELEASE_ARGS = -d in_asm -display none
+    QEMU_RELEASE_ARGS = -serial stdio -display none
     OBJDUMP_BINARY    = aarch64-none-elf-objdump
     NM_BINARY         = aarch64-none-elf-nm
     READELF_BINARY    = aarch64-none-elf-readelf

diff -uNr 02_runtime_init/src/bsp/raspberrypi/console.rs 03_hacky_hello_world/src/bsp/raspberrypi/console.rs
--- 02_runtime_init/src/bsp/raspberrypi/console.rs
+++ 03_hacky_hello_world/src/bsp/raspberrypi/console.rs
@@ -0,0 +1,47 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! BSPコンソール装置
+
+use crate::console;
+use core::fmt;
+
+//--------------------------------------------------------------------------------------------------
+// プライベート定義
+//--------------------------------------------------------------------------------------------------
+
+/// QEMUの出力を無から生成する神秘的で魔法のような装置
+struct QEMUOutput;
+
+//--------------------------------------------------------------------------------------------------
+// プライベート定義
+//--------------------------------------------------------------------------------------------------
+
+/// `core::fmt::Write`を実装すると`format_args!`マクロが利用可能になる。これはひいては
+/// `カーネル`の`print!`と`println!`マクロを実装することになる。`write_str()`を実装する
+/// ことにより自動的に`write_fmt()`を手にすることができる。
+///
+/// [`src/print.rs`]を参照
+///
+/// [`src/print.rs`]: ../../print/index.html
+impl fmt::Write for QEMUOutput {
+    fn write_str(&mut self, s: &str) -> fmt::Result {
+        for c in s.chars() {
+            unsafe {
+                core::ptr::write_volatile(0x3F20_1000 as *mut u8, c as u8);
+            }
+        }
+
+        Ok(())
+    }
+}
+
+//--------------------------------------------------------------------------------------------------
+// パブリックコード
+//--------------------------------------------------------------------------------------------------
+
+/// コンソールへの参照を返す
+pub fn console() -> impl console::interface::Write {
+    QEMUOutput {}
+}

diff -uNr 02_runtime_init/src/bsp/raspberrypi.rs 03_hacky_hello_world/src/bsp/raspberrypi.rs
--- 02_runtime_init/src/bsp/raspberrypi.rs
+++ 03_hacky_hello_world/src/bsp/raspberrypi.rs
@@@ -4,5 +4,6 @@

 //! Raspberry Pi 3/4用のトップレベルのBSPファイル

+pub mod console;
 pub mod cpu;
 pub mod memory;


diff -uNr 02_runtime_init/src/console.rs 03_hacky_hello_world/src/console.rs
--- 02_runtime_init/src/console.rs
+++ 03_hacky_hello_world/src/console.rs
@@ -0,0 +1,18 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! システムコンソール
+
+//--------------------------------------------------------------------------------------------------
+// パブリック定義
+//--------------------------------------------------------------------------------------------------
+
+/// コンソールインタフェース
+pub mod interface {
+    /// コンソール write関数
+    ///
+    /// `core::fmt::Write` は今まさに必要なもの。console::Write`の実装が
+    /// 読者に意図を伝える良いヒントになるので、ここで再エクスポートする。
+    pub use core::fmt::Write;
+}

diff -uNr 02_runtime_init/src/main.rs 03_hacky_hello_world/src/main.rs
--- 02_runtime_init/src/main.rs
+++ 03_hacky_hello_world/src/main.rs
@@ -106,14 +106,18 @@
 //!
 //! [`runtime_init::runtime_init()`]: runtime_init/fn.runtime_init.html

+#![feature(format_args_nl)]
 #![feature(global_asm)]
+#![feature(panic_info_message)]
 #![no_main]
 #![no_std]

 mod bsp;
+mod console;
 mod cpu;
 mod memory;
 mod panic_wait;
+mod print;
 mod runtime_init;

 /// 最初の初期化コード
@@ -122,5 +126,7 @@
 ///
 /// - アクティブなコアはこの関数を実行しているコアだけでなければならない
 unsafe fn kernel_init() -> ! {
-    panic!()
+    println!("[0] Hello from Rust!");
+
+    panic!("Stopping here.")
 }

diff -uNr 02_runtime_init/src/panic_wait.rs 03_hacky_hello_world/src/panic_wait.rs
--- 02_runtime_init/src/panic_wait.rs
+++ 03_hacky_hello_world/src/panic_wait.rs
@@ -4,10 +4,16 @@

 //! 永久に待ち続けるパニックハンドラ

-use crate::cpu;
+use crate::{cpu, println};
 use core::panic::PanicInfo;

 #[panic_handler]
-fn panic(_info: &PanicInfo) -> ! {
+fn panic(info: &PanicInfo) -> ! {
+    if let Some(args) = info.message() {
+        println!("\nKernel panic: {}", args);
+    } else {
+        println!("\nKernel panic!");
+    }
+
     cpu::wait_forever()
 }

diff -uNr 02_runtime_init/src/print.rs 03_hacky_hello_world/src/print.rs
--- 02_runtime_init/src/print.rs
+++ 03_hacky_hello_world/src/print.rs
@@ -0,0 +1,38 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! プリント
+
+use crate::{bsp, console};
+use core::fmt;
+
+//--------------------------------------------------------------------------------------------------
+// パブリックコード
+//--------------------------------------------------------------------------------------------------
+
+#[doc(hidden)]
+pub fn _print(args: fmt::Arguments) {
+    use console::interface::Write;
+
+    bsp::console::console().write_fmt(args).unwrap();
+}
+
+/// 改行なしのプリント
+///
+/// <https://doc.rust-lang.org/src/std/macros.rs.html>からそのままコピー
+#[macro_export]
+macro_rules! print {
+    ($($arg:tt)*) => ($crate::print::_print(format_args!($($arg)*)));
+}
+
+/// 改行付きのプリント
+///
+/// <https://doc.rust-lang.org/src/std/macros.rs.html>からそのままコピー
+#[macro_export]
+macro_rules! println {
+    () => ($crate::print!("\n"));
+    ($($arg:tt)*) => ({
+        $crate::print::_print(format_args_nl!($($arg)*));
+    })
+}

```
