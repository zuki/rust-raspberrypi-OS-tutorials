# チュートリアル 02 - Runtime Init

## tl;dr

- `boot.s`を拡張して初めてのRustコードを呼び出します。そこでは、[bss]セクションをゼロクリアしてから`panic()`を呼び出して実行を停止します。
- `make qemu`を再度実行して、追加コードの実行を確認してください。

## 特筆すべき追加事項

- リンカスクリプトへの追加:
     - 新しいセクション: `.rodata`, `.got`, `.data`, `.bss`.
     - `_start()`で読み込む必要のあるブートタイム引数をリンクするための場所
- `_arch/__arch_name__/cpu/boot.s`の`_start()`:
     1. core != core0であればコアを停止します。
     2. `stack pointer`を設定します。
     3. `arch/__arch_name__/cpu/boot.rs`で定義されている`_start_rust()`関数にジャンプします。
- `runtime_init.rs`の`runtime_init()`:
     - `.bss`セクションをゼロクリアします。
     - `kernel_init()`を呼び出します。これは`panic!()`を呼び出し、最終的にcore0も停止します。
- このライブラリは現在、[cortex-a]クレイトを使用しています。このクレイトはゼロコスト抽象化を提供し、CPUのリソースを処理する際の`unsafe`な部分をラップします。
    - 動作は `_arch/__arch_name__/cpu.rs` を参照してください。

[bss]: https://en.wikipedia.org/wiki/.bss
[aarch64-cpu]: https://github.com/rust-embedded/aarch64-cpu

## 前チュートリアルとのdiff
```diff

diff -uNr 01_wait_forever/Cargo.toml 02_runtime_init/Cargo.toml
--- 01_wait_forever/Cargo.toml
+++ 02_runtime_init/Cargo.toml
@@ -1,6 +1,6 @@
 [package]
 name = "mingo"
-version = "0.1.0"
+version = "0.2.0"
 authors = ["Andre Richter <andre.o.richter@gmail.com>"]
 edition = "2021"

@@ -21,3 +21,7 @@
 ##--------------------------------------------------------------------------------------------------

 [dependencies]
+
+# Platform specific dependencies
+[target.'cfg(target_arch = "aarch64")'.dependencies]
+aarch64-cpu = { version = "9.x.x" }

diff -uNr 01_wait_forever/Makefile 02_runtime_init/Makefile
--- 01_wait_forever/Makefile
+++ 02_runtime_init/Makefile
@@ -181,6 +181,7 @@
 	$(call color_header, "Launching objdump")
 	@$(DOCKER_TOOLS) $(OBJDUMP_BINARY) --disassemble --demangle \
                 --section .text   \
+                --section .rodata \
                 $(KERNEL_ELF) | rustfilt
 ##------------------------------------------------------------------------------

diff -uNr 01_wait_forever/src/_arch/aarch64/cpu/boot.rs 02_runtime_init/src/_arch/aarch64/cpu/boot.rs
--- 01_wait_forever/src/_arch/aarch64/cpu/boot.rs
+++ 02_runtime_init/src/_arch/aarch64/cpu/boot.rs
@@ -14,4 +14,19 @@
 use core::arch::global_asm;

+use crate::runtime_init;
+
 // このファイルに対応するアセンブリファイル。
 global_asm!(include_str!("boot.s"));
+
+//--------------------------------------------------------------------------------------------------
+// パブリックコード
+//--------------------------------------------------------------------------------------------------
+
+/// `kernel`バイナリのRust側エントリ。
+///
+/// この関数はアセンブリファイルの`_start`関数から呼び出される。
+///
+/// # 安全性
+///
+/// - `bss`セクションはまだ初期化されていない。コードはbssをいかなる方法であれ、使用または参照してはならない。
+#[no_mangle]
+pub unsafe fn _start_rust() -> ! {
+    crate::kernel_init()
+}

diff -uNr 01_wait_forever/src/_arch/aarch64/cpu/boot.s 02_runtime_init/src/_arch/aarch64/cpu/boot.s
--- 01_wait_forever/src/_arch/aarch64/cpu/boot.s
+++ 02_runtime_init/src/_arch/aarch64/cpu/boot.s
@@ -3,6 +3,22 @@
 // Copyright (c) 2021-2023 Andre Richter <andre.o.richter@gmail.com>

 //--------------------------------------------------------------------------------------------------
+// 定義
+//--------------------------------------------------------------------------------------------------
+
+// シンボルのアドレスをレジスタにロードする（PC-相対）。
+//
+// シンボルはプログラムカウンタの +/- 4GiB以内になければならない。
+//
+// # リソース
+//
+// - https://sourceware.org/binutils/docs-2.36/as/AArch64_002dRelocations.html
+.macro ADR_REL register, symbol
+       adrp    \register, \symbol
+       add     \register, \register, #:lo12:\symbol
+.endm
+
+//--------------------------------------------------------------------------------------------------
 // パブリックコード
 //--------------------------------------------------------------------------------------------------
 .section .text._start
@@ -11,6 +27,34 @@
 // fn _start()
 //------------------------------------------------------------------------------
 _start:
+       // ブートコア上でのみ実行する。他のコアは止める。
+       mrs     x1, MPIDR_EL1         // MARの[7:0]がコア番号（raspi3/4はcoreを4つ搭載: 0x00-0x03）
+       and     x1, x1, _core_id_mask // _code_id_mask = 0b11; このファイルの先頭で定義
+       ldr     x2, BOOT_CORE_ID      // BOOT_CORE_ID=0: bsp/__board_name__/cpu.rs で定義
+       cmp     x1, x2
+       b.ne    1f                    // core0以外は1へジャンプ
+
+       // 処理がここに来たらそれはブートコア。Rustコードにジャンプするための準備をする。
+
+       // スタックポインタを設定する。
+       ADR_REL x0, __boot_core_stack_end_exclusive     // link.ldで定義 = 0x80000 .textの下に伸びる
+       mov     sp, x0
+
+       // Rustコードにジャンプする。
+       b       _start_rust
+
        // イベントを無限に待つ（別名 "park the core"）
 1:     wfe
        b       1b

diff -uNr 01_wait_forever/src/_arch/aarch64/cpu.rs 02_runtime_init/src/_arch/aarch64/cpu.rs
--- 01_wait_forever/src/_arch/aarch64/cpu.rs
+++ 02_runtime_init/src/_arch/aarch64/cpu.rs
@@ -0,0 +1,26 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
+
+//! アーキテクチャ固有のブートコード。
+//!
+//! # オリエンテーション
+//!
+//! archモジュールはpath属性を使って汎用モジュールにインポートされるので
+//! このファイルのパスは次の通り:
+//!
+//! crate::cpu::arch_cpu
+
+use aarch64_cpu::asm;
+
+//--------------------------------------------------------------------------------------------------
+// パブリックコード
+//--------------------------------------------------------------------------------------------------
+
+/// コア上での実行を休止する
+#[inline(always)]
+pub fn wait_forever() -> ! {
+    loop {
+        asm::wfe()
+    }
+}


diff -uNr 01_wait_forever/src/bsp/raspberrypi/cpu.rs 02_runtime_init/src/bsp/raspberrypi/cpu.rs
--- 01_wait_forever/src/bsp/raspberrypi/cpu.rs
+++ 02_runtime_init/src/bsp/raspberrypi/cpu.rs
@@ -0,0 +1,14 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
+
+//! BSPプロセッサコード
+
+//--------------------------------------------------------------------------------------------------
+// パブリック定義
+//--------------------------------------------------------------------------------------------------
+
+/// 初期ブートコアを探すために`arch`コードにより使用される
+#[no_mangle]
+#[link_section = ".text._start_arguments"]
+pub static BOOT_CORE_ID: u64 = 0;

diff -uNr 01_wait_forever/src/bsp/raspberrypi/kernel.ld 02_runtime_init/src/bsp/raspberrypi/kernel.ld
--- 01_wait_forever/src/bsp/raspberrypi/kernel.ld
+++ 02_runtime_init/src/bsp/raspberrypi/kernel.ld
@@ -3,6 +3,8 @@
  * Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
  */

+__rpi_phys_dram_start_addr = 0;
+
 /* The physical address at which the the kernel binary will be loaded by the Raspberry's firmware */
 __rpi_phys_binary_load_addr = 0x80000;

@@ -13,21 +15,65 @@
  *     4 == R
  *     5 == RX
  *     6 == RW
+ *
+ * Segments are marked PT_LOAD below so that the ELF file provides virtual and physical addresses.
+ * It doesn't mean all of them need actually be loaded.
  */
 PHDRS
 {
-    segment_code PT_LOAD FLAGS(5);
+    segment_boot_core_stack PT_LOAD FLAGS(6);
+    segment_code            PT_LOAD FLAGS(5);
+    segment_data            PT_LOAD FLAGS(6);
 }

 SECTIONS
 {
-    . =  __rpi_phys_binary_load_addr;
+    . =  __rpi_phys_dram_start_addr;
+
+    /***********************************************************************************************
+    * Boot Core Stack
+    ***********************************************************************************************/
+    .boot_core_stack (NOLOAD) :
+    {
+                                             /*   ^             */
+                                             /*   | stack       */
+        . += __rpi_phys_binary_load_addr;    /*   | growth      */
+                                             /*   | direction   */
+        __boot_core_stack_end_exclusive = .; /*   |             */
+    } :segment_boot_core_stack

     /***********************************************************************************************
-    * Code
+    * Code + RO Data + Global Offset Table
     ***********************************************************************************************/
     .text :
     {
         KEEP(*(.text._start))
+        *(.text._start_arguments) /* _start()により読み込まれる定数（Rustで言うsttics） */
+        *(.text._start_rust)      /* Rustのエントリポイント */
+        *(.text*)                 /* その他のすべて */
     } :segment_rx
+
+    .rodata : ALIGN(8) { *(.rodata*) } :segment_code
+
+    /***********************************************************************************************
+    * Data + BSS
+    ***********************************************************************************************/
+    .data : { *(.data*) } :segment_data
+
+    /* セクションはu64のチャンクでゼロ詰めされる。start/endアドレスは8バイトアライン */
+    .bss : ALIGN(8)
+    {
+        __bss_start = .;
+        *(.bss*);
+        . = ALIGN(16);
+        __bss_end_exclusive = .;
+    } :segment_data
+
+        . += 8; /* bss == 0の場合にも __bss_start <= __bss_end_inclusive になるように詰める */
+        __bss_end_inclusive = . - 8;
+    } :NONE
 }

diff -uNr 01_wait_forever/src/bsp/raspberrypi/memory.rs 02_runtime_init/src/bsp/raspberrypi/memory.rs
--- 01_wait_forever/src/bsp/raspberrypi/memory.rs
+++ 02_runtime_init/src/bsp/raspberrypi/memory.rs
@@ -0,0 +1,37 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! BSPメモリ管理
+
+use core::{cell::UnsafeCell, ops::RangeInclusive};
+
+//--------------------------------------------------------------------------------------------------
+// プライベート定義
+//--------------------------------------------------------------------------------------------------
+
+// リンカスクリプトで定義されているシンボル
+extern "Rust" {
+    static __bss_start: UnsafeCell<u64>;
+    static __bss_end_inclusive: UnsafeCell<u64>;
+}
+
+//--------------------------------------------------------------------------------------------------
+// パブリックコード
+//--------------------------------------------------------------------------------------------------
+
+/// .bssセクションに含まれる範囲を返す
+///
+/// # 安全性
+///
+/// - 値はリンカスクリプトが提供するものであり、そのまま信用する必要がある
+/// - リンカスクリプトが提供するアドレスはu64にアラインされている必要がある
+pub fn bss_range_inclusive() -> RangeInclusive<*mut u64> {
+    let range;
+    unsafe {
+        range = RangeInclusive::new(__bss_start.get(), __bss_end_inclusive.get());
+    }
+    assert!(!range.is_empty());
+
+    range
+}

diff -uNr 01_wait_forever/src/bsp/raspberrypi.rs 02_runtime_init/src/bsp/raspberrypi.rs
--- 01_wait_forever/src/bsp/raspberrypi.rs
+++ 02_runtime_init/src/bsp/raspberrypi.rs
@@ -4,4 +4,4 @@

 //! Raspberry Pi 3/4用のトップレベルのBSPファイル

-// Coming soon.
+pub mod cpu;

diff -uNr 01_wait_forever/src/cpu.rs 02_runtime_init/src/cpu.rs
--- 01_wait_forever/src/cpu.rs
+++ 02_runtime_init/src/cpu.rs
@@ -4,4 +4,13 @@

 //! プロセッサコード

+#[cfg(target_arch = "aarch64")]
+#[path = "_arch/aarch64/cpu.rs"]
+mod arch_cpu;
+
 mod boot;
+
+//--------------------------------------------------------------------------------------------------
+// アーキテクチャのパブリック再エクスポート
+//--------------------------------------------------------------------------------------------------
+pub use arch_cpu::wait_forever;

diff -uNr 01_wait_forever/src/main.rs 02_runtime_init/src/main.rs
--- 01_wait_forever/src/main.rs
+++ 02_runtime_init/src/main.rs
@@ -104,7 +104,9 @@
 //!
 //! 1. カーネルのエントリポイントは関数 `cpu::boot::arch_boot::_start()`
 //!     - 実装は `src/_arch/__arch_name__/cpu/boot.s` にある
+//! 2. アーキテクチャのセットアップが終わったら、アーキテクチャのコードは[`runtime_init::runtime_init()`]を呼び出す
+//!
+//! [`runtime_init::runtime_init()`]: runtime_init/fn.runtime_init.html

+#![feature(asm_const)]
 #![no_main]
 #![no_std]

@@ -112,4 +114,11 @@
 mod cpu;
 mod panic_wait;

-// カーネルコードは次のチュートリアルで登場
+/// 最初の初期化コード
+///
+/// # 安全性
+///
+/// - アクティブなコアはこの関数を実行しているコアだけでなければならない
+unsafe fn kernel_init() -> ! {
+    panic!()
+}

diff -uNr 01_wait_forever/src/memory.rs 02_runtime_init/src/memory.rs
--- 01_wait_forever/src/memory.rs
+++ 02_runtime_init/src/memory.rs
@@ -0,0 +1,30 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! メモリ管理
+
+use core::ops::RangeInclusive;
+
+//--------------------------------------------------------------------------------------------------
+// パブリックコード
+//--------------------------------------------------------------------------------------------------
+
+/// メモリ範囲をゼロ詰めする
+///
+/// # 安全性
+///
+/// - `range.start` と `range.end` はvalidでなければならない
+/// - `range.start` と `range.end` は`T`アラインされていなければならない
+pub unsafe fn zero_volatile<T>(range: RangeInclusive<*mut T>)
+where
+    T: From<u8>,
+{
+    let mut ptr = *range.start();
+    let end_inclusive = *range.end();
+
+    while ptr <= end_inclusive {
+        core::ptr::write_volatile(ptr, T::from(0));
+        ptr = ptr.offset(1);
+    }
+}

diff -uNr 01_wait_forever/src/panic_wait.rs 02_runtime_init/src/panic_wait.rs
--- 01_wait_forever/src/panic_wait.rs
+++ 02_runtime_init/src/panic_wait.rs
@@ -4,6 +4,7 @@

 //! 永久に待ち続けるパニックハンドラ

+use crate::cpu;
 use core::panic::PanicInfo;

 //--------------------------------------------------------------------------------------------------
@@ -12,5 +13,5 @@

 #[panic_handler]
 fn panic(_info: &PanicInfo) -> ! {
-    unimplemented!()
+    cpu::wait_forever()
 }

diff -uNr 01_wait_forever/src/runtime_init.rs 02_runtime_init/src/runtime_init.rs
--- 01_wait_forever/src/runtime_init.rs
+++ 02_runtime_init/src/runtime_init.rs
@@ -0,0 +1,37 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! Rustランタイム初期化コード
+
+use crate::{bsp, memory};
+
+//--------------------------------------------------------------------------------------------------
+// プライベートコード
+//--------------------------------------------------------------------------------------------------
+
+/// .bssセクションをゼロ詰め
+///
+/// # 安全性
+///
+/// - `kernel_init()`の前に呼び出されなければならない
+#[inline(always)]
+unsafe fn zero_bss() {
+    memory::zero_volatile(bsp::memory::bss_range_inclusive());
+}
+
+//--------------------------------------------------------------------------------------------------
+// 公開コード
+//--------------------------------------------------------------------------------------------------
+
+/// C/C++における`crt0`や`c0`に相当する。`bss`セクションをクリアして
+/// カーネル初期化コードにジャンプする。
+///
+/// # 安全性
+///
+/// - 1つのコアだけがアクティブで、この関数を実行しなければならない。
+pub unsafe fn runtime_init() -> ! {
+    zero_bss();
+
+    crate::kernel_init()
+}

```
