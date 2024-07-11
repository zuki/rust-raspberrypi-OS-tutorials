# チュートリアル 09 - 権限レベル

## tl;dr

- 初期ブートコードで、`Hypervisor`権限レベル（AArch64では`EL2`）から
  `Kernel`（`EL1`）権限レベルに移行します。

## 目次

- [チュートリアル 09 - 権限レベル](#チュートリアル-09---権限レベル)
  - [tl;dr](#tldr)
  - [目次](#目次)
  - [はじめに](#はじめに)
  - [このチュートリアルの範囲](#このチュートリアルの範囲)
  - [エントリポイントでの`EL2`のチェック](#エントリポイントでのel2のチェック)
  - [移行準備](#移行準備)
  - [決して発生しない例外から復帰する](#決して発生しない例外から復帰する)
  - [テストする](#テストする)
  - [前回とのDiff](#前回とのdiff)

## はじめに

アプリケーショングレードのCPUには、それぞれ目的が異なる「特権レベル」と
呼ばれるものがあります。

| 通常の用途 | AArch64 | RISC-V | x86 |
| ------------- | ------------- | ------------- | ------------- |
| ユーザ空間アプリケーション | EL0 | U/VU | Ring 3 |
| OSカーネル | EL1 | S/VS | Ring 0 |
| ハイパーバイザ | EL2 | HS | Ring -1 |
| 低レベルファームウェア | EL3 | M | |

AArch64の`EL`は`Exception Level`（特権レベル）の略です。その他のアーキテクチャに
関する詳しい情報は、次のリンクをご覧ください。

- [x86の権限リング](https://en.wikipedia.org/wiki/Protection_ring).
- [RISC-Vの権限モード](https://content.riscv.org/wp-content/uploads/2017/12/Tue0942-riscv-hypervisor-waterman.pdf).

先に進む前に、[Programmer’s Guide forARMv8-A]の「第3章」に目を通すことを
強く勧めます。そこには、このトピックに関する簡潔な概要が書かれています。

[Programmer’s Guide for ARMv8-A]: http://infocenter.arm.com/help/topic/com.arm.doc.den0024a/DEN0024A_v8_architecture_PG.pdf

## このチュートリアルの範囲

デフォルトでは、Raspberryは常に`EL2`で実行を開始します。私たちは伝統的な
「カーネル」を書いているので、より適切な`EL1`に移行しなければなりません。

## エントリポイントでの`EL2`のチェック

まず最初に、`EL1`に移行するためのコードを呼び出す前に、実際に`EL2`で実行
されていることを確認する必要があります。そこで、`boot.s`の先頭に新しい
チェックコードを追加し、`EL2`でない場合はCPUコアをパークするようにします。

```
// コアがEL2で実行している場合のみ処理を継続する。そうでなければパークさせる。
mrs	x0, CurrentEL
cmp	x0, {CONST_CURRENTEL_EL2}
b.ne	.L_parking_loop
```

その後、`boot.rs`の`prepare_el2_to_el1_transition()`を呼び出して、`EL2→EL1`の
移行準備を続けます。

```rust
#[no_mangle]
pub unsafe extern "C" fn _start_rust(phys_boot_core_stack_end_exclusive_addr: u64) -> ! {
    prepare_el2_to_el1_transition(phys_boot_core_stack_end_exclusive_addr);

    // EL1に「復帰する」ために`eret`を使用する。これによりruntime_init()はEL1で実行される。
    asm::eret()
}
```

## 移行準備

`EL2`は`EL1`よりも高い権限を持っているため、様々なプロセッサの機能を制御
しており、`EL1`のコードにそれらの使用の許可・不許可を与えることができます。
たとえば、タイマレジスタやカウンタレジスタへのアクセスがその例です。それらは
[チュートリアル07](../07_timestamps/)からすでに使用しているので、もちろん
そのまま使用したいと思います。そこで、[Counter-timer Hypervisor Control register]
にそれぞれのフラグを設定し、さらに仮想オフセットを0に設定して、常に実際の
物理的な値を得るようにします。

[Counter-timer Hypervisor Control register]:  https://docs.rs/aarch64-cpu/9.0.0/src/aarch64_cpu/registers/cnthctl_el2.rs.html

```rust
// EL1のタイマカウンタレジスタを有効にする
CNTHCTL_EL2.write(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);

// カウンタを読み込むためのオフセットはなし
CNTVOFF_EL2.set(0);
```

次に、`EL1`が`AArch64`モードで実行し、（これも可能な）`AArch32`では実行
しないように[Hypervisor Configuration Register]を設定します。

[Hypervisor Configuration Register]: https://docs.rs/aarch64-cpu/9.0.0/src/aarch64_cpu/registers/hcr_el2.rs.html

```rust
// EL1の実行モードをAArch64に設定する
HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);
```

## 決して発生しない例外から復帰する

上位のELから下位のELに移行する方法は、実は1つしかなく、それは{ERET}命令を
実行することです。

[ERET]: https://docs.rs/aarch64-cpu/9.0.0/src/aarch64_cpu/asm.rs.html#92-101

この命令は、[Saved Program Status Register - EL2]の内容を
`Current Program Status Register - EL1`にコピーし、[Exception Link Register - EL2]
に格納されている命令アドレスにジャンプします。

これは基本的に例外が発生した時に行われることとは逆のことです。これに
ついては、次回のチュートリアルで学びます。

[Saved Program Status Register - EL2]: https://docs.rs/aarch64-cpu/9.0.0/src/aarch64_cpu/registers/spsr_el2.rs.html
[Exception Link Register - EL2]: https://docs.rs/aarch64-cpu/9.0.0/src/aarch64_cpu/registers/elr_el2.rs.html

```rust
// 模擬例外復帰を設定する
//
// まず、すべての割り込みがマスクし、SP_EL1をスタックポインタとして使用する
// ように保存プログラム状態を偽装する
SPSR_EL2.write(
    SPSR_EL2::D::Masked
        + SPSR_EL2::A::Masked
        + SPSR_EL2::I::Masked
        + SPSR_EL2::F::Masked
        + SPSR_EL2::M::EL1h,
);

// 次に、リンクレジスタが runtime_init()を指すようにする
ELR_EL2.set(crate::kernel_init as *const () as u64);

// SP_EL1 (スタックポインタ)を設定する。これはEL1に「復帰した」した際に
// EL1で使用されことになる。EL2に戻ることは全く想定していないので
// 同じスタックを再利用するだけである。
SP_EL1.set(phys_boot_core_stack_end_exclusive_addr);
```

ご覧のとおり、`ELR_EL2`にはこれまでエントリポイントから直接呼び出すために
使用していた[runtime_init()] 関数のアドレスを設定しています。最後に、
`SP_EL1`用のスタックポインタを設定します

スタックのアドレスが関数の引数として与えられていることにお気づきでしょうか。
覚えているかもしれませんが、`boot.s`の`_start()`で`EL2`用のスタックをすでに
設定しています。`EL2`に戻る予定はないので、`EL1`用のスタックとして再利用
することができます。それでそのアドレスを関数の引数として渡しています。

最後に、`_start_rust()`に戻って、`ERET`の呼び出しが行われます。

```rust
#[no_mangle]
pub unsafe extern "C" fn _start_rust(phys_boot_core_stack_end_exclusive_addr: u64) -> ! {
    prepare_el2_to_el1_transition(phys_boot_core_stack_end_exclusive_addr);

    // EL1に「復帰する」ために`eret`を使用する。これによりruntime_init()はEL1で実行される。
    asm::eret()
}
```

## テストする

`main.rs`では「現在の特権レベル」を表示し、さらに、`SPSR_EL2`のマスクビットが
`EL1`になっているかを検査しています。

```console
$ make chainboot
[...]
Minipush 1.0

[MP] ⏳ Waiting for /dev/ttyUSB0
[MP] ✅ Serial connected
[MP] 🔌 Please power the target now

 __  __ _      _ _                 _
|  \/  (_)_ _ (_) |   ___  __ _ __| |
| |\/| | | ' \| | |__/ _ \/ _` / _` |
|_|  |_|_|_||_|_|____\___/\__,_\__,_|

           Raspberry Pi 3

[ML] Requesting binary
[MP] ⏩ Pushing 14 KiB =========================================🦀 100% 0 KiB/s Time: 00:00:00
[ML] Loaded! Executing the payload now

[    0.162546] mingo version 0.9.0
[    0.162745] Booting on: Raspberry Pi 3
[    0.163201] Current privilege level: EL1
[    0.163677] Exception handling state:
[    0.164122]       Debug:  Masked
[    0.164511]       SError: Masked
[    0.164901]       IRQ:    Masked
[    0.165291]       FIQ:    Masked
[    0.165681] Architectural timer resolution: 52 ns
[    0.166255] Drivers loaded:
[    0.166592]       1. BCM PL011 UART
[    0.167014]       2. BCM GPIO
[    0.167371] Timer test, spinning for 1 second
[    1.167904] Echoing input now
```

## 前回とのDiff
```diff

diff -uNr 08_hw_debug_JTAG/Cargo.toml 09_privilege_level/Cargo.toml
--- 08_hw_debug_JTAG/Cargo.toml
+++ 09_privilege_level/Cargo.toml
@@ -1,6 +1,6 @@
 [package]
 name = "mingo"
-version = "0.8.0"
+version = "0.9.0"
 authors = ["Andre Richter <andre.o.richter@gmail.com>"]
 edition = "2021"


diff -uNr 08_hw_debug_JTAG/src/_arch/aarch64/cpu/boot.rs 09_privilege_level/src/_arch/aarch64/cpu/boot.rs
--- 08_hw_debug_JTAG/src/_arch/aarch64/cpu/boot.rs
+++ 09_privilege_level/src/_arch/aarch64/cpu/boot.rs
@@ -11,22 +11,73 @@
 //!
 //! crate::cpu::boot::arch_boot

+use aarch64_cpu::{asm, registers::*};
 use core::arch::global_asm;
+use tock_registers::interfaces::Writeable;

 // Assembly counterpart to this file.
 global_asm!(
     include_str!("boot.s"),
+    CONST_CURRENTEL_EL2 = const 0x8,
     CONST_CORE_ID_MASK = const 0b11
 );

 //--------------------------------------------------------------------------------------------------
+// Private Code
+//--------------------------------------------------------------------------------------------------
+
+/// Prepares the transition from EL2 to EL1.
+///
+/// # Safety
+///
+/// - The `bss` section is not initialized yet. The code must not use or reference it in any way.
+/// - The HW state of EL1 must be prepared in a sound way.
+#[inline(always)]
+unsafe fn prepare_el2_to_el1_transition(phys_boot_core_stack_end_exclusive_addr: u64) {
+    // Enable timer counter registers for EL1.
+    CNTHCTL_EL2.write(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);
+
+    // No offset for reading the counters.
+    CNTVOFF_EL2.set(0);
+
+    // Set EL1 execution state to AArch64.
+    HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);
+
+    // Set up a simulated exception return.
+    //
+    // First, fake a saved program status where all interrupts were masked and SP_EL1 was used as a
+    // stack pointer.
+    SPSR_EL2.write(
+        SPSR_EL2::D::Masked
+            + SPSR_EL2::A::Masked
+            + SPSR_EL2::I::Masked
+            + SPSR_EL2::F::Masked
+            + SPSR_EL2::M::EL1h,
+    );
+
+    // Second, let the link register point to kernel_init().
+    ELR_EL2.set(crate::kernel_init as *const () as u64);
+
+    // Set up SP_EL1 (stack pointer), which will be used by EL1 once we "return" to it. Since there
+    // are no plans to ever return to EL2, just re-use the same stack.
+    SP_EL1.set(phys_boot_core_stack_end_exclusive_addr);
+}
+
+//--------------------------------------------------------------------------------------------------
 // Public Code
 //--------------------------------------------------------------------------------------------------

 /// The Rust entry of the `kernel` binary.
 ///
 /// The function is called from the assembly `_start` function.
+///
+/// # Safety
+///
+/// - Exception return from EL2 must must continue execution in EL1 with `kernel_init()`.
 #[no_mangle]
-pub unsafe fn _start_rust() -> ! {
-    crate::kernel_init()
+pub unsafe extern "C" fn _start_rust(phys_boot_core_stack_end_exclusive_addr: u64) -> ! {
+    prepare_el2_to_el1_transition(phys_boot_core_stack_end_exclusive_addr);
+
+    // Use `eret` to "return" to EL1. This results in execution of kernel_init() in EL1.
+    asm::eret()
 }

diff -uNr 08_hw_debug_JTAG/src/_arch/aarch64/cpu/boot.s 09_privilege_level/src/_arch/aarch64/cpu/boot.s
--- 08_hw_debug_JTAG/src/_arch/aarch64/cpu/boot.s
+++ 09_privilege_level/src/_arch/aarch64/cpu/boot.s
@@ -27,11 +27,16 @@
 // fn _start()
 //------------------------------------------------------------------------------
 _start:
+	// Only proceed if the core executes in EL2. Park it otherwise.
+	mrs	x0, CurrentEL
+	cmp	x0, {CONST_CURRENTEL_EL2}
+	b.ne	.L_parking_loop
+
 	// Only proceed on the boot core. Park it otherwise.
-	mrs	x0, MPIDR_EL1
-	and	x0, x0, {CONST_CORE_ID_MASK}
-	ldr	x1, BOOT_CORE_ID      // provided by bsp/__board_name__/cpu.rs
-	cmp	x0, x1
+	mrs	x1, MPIDR_EL1
+	and	x1, x1, {CONST_CORE_ID_MASK}
+	ldr	x2, BOOT_CORE_ID      // provided by bsp/__board_name__/cpu.rs
+	cmp	x1, x2
 	b.ne	.L_parking_loop

 	// If execution reaches here, it is the boot core.
@@ -48,7 +53,7 @@

 	// Prepare the jump to Rust code.
 .L_prepare_rust:
-	// Set the stack pointer.
+	// Set the stack pointer. This ensures that any code in EL2 that needs the stack will work.
 	ADR_REL	x0, __boot_core_stack_end_exclusive
 	mov	sp, x0

@@ -60,7 +65,7 @@
 	b.eq	.L_parking_loop
 	str	w2, [x1]

-	// Jump to Rust code.
+	// Jump to Rust code. x0 holds the function argument provided to _start_rust().
 	b	_start_rust

 	// Infinitely wait for events (aka "park the core").

diff -uNr 08_hw_debug_JTAG/src/_arch/aarch64/exception/asynchronous.rs 09_privilege_level/src/_arch/aarch64/exception/asynchronous.rs
--- 08_hw_debug_JTAG/src/_arch/aarch64/exception/asynchronous.rs
+++ 09_privilege_level/src/_arch/aarch64/exception/asynchronous.rs
@@ -0,0 +1,82 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
+
+//! Architectural asynchronous exception handling.
+//!
+//! # Orientation
+//!
+//! Since arch modules are imported into generic modules using the path attribute, the path of this
+//! file is:
+//!
+//! crate::exception::asynchronous::arch_asynchronous
+
+use aarch64_cpu::registers::*;
+use tock_registers::interfaces::Readable;
+
+//--------------------------------------------------------------------------------------------------
+// Private Definitions
+//--------------------------------------------------------------------------------------------------
+
+trait DaifField {
+    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register>;
+}
+
+struct Debug;
+struct SError;
+struct IRQ;
+struct FIQ;
+
+//--------------------------------------------------------------------------------------------------
+// Private Code
+//--------------------------------------------------------------------------------------------------
+
+impl DaifField for Debug {
+    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
+        DAIF::D
+    }
+}
+
+impl DaifField for SError {
+    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
+        DAIF::A
+    }
+}
+
+impl DaifField for IRQ {
+    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
+        DAIF::I
+    }
+}
+
+impl DaifField for FIQ {
+    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
+        DAIF::F
+    }
+}
+
+fn is_masked<T>() -> bool
+where
+    T: DaifField,
+{
+    DAIF.is_set(T::daif_field())
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+/// Print the AArch64 exceptions status.
+#[rustfmt::skip]
+pub fn print_state() {
+    use crate::info;
+
+    let to_mask_str = |x| -> _ {
+        if x { "Masked" } else { "Unmasked" }
+    };
+
+    info!("      Debug:  {}", to_mask_str(is_masked::<Debug>()));
+    info!("      SError: {}", to_mask_str(is_masked::<SError>()));
+    info!("      IRQ:    {}", to_mask_str(is_masked::<IRQ>()));
+    info!("      FIQ:    {}", to_mask_str(is_masked::<FIQ>()));
+}

diff -uNr 08_hw_debug_JTAG/src/_arch/aarch64/exception.rs 09_privilege_level/src/_arch/aarch64/exception.rs
--- 08_hw_debug_JTAG/src/_arch/aarch64/exception.rs
+++ 09_privilege_level/src/_arch/aarch64/exception.rs
@@ -0,0 +1,31 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
+
+//! Architectural synchronous and asynchronous exception handling.
+//!
+//! # Orientation
+//!
+//! Since arch modules are imported into generic modules using the path attribute, the path of this
+//! file is:
+//!
+//! crate::exception::arch_exception
+
+use aarch64_cpu::registers::*;
+use tock_registers::interfaces::Readable;
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+use crate::exception::PrivilegeLevel;
+
+/// The processing element's current privilege level.
+pub fn current_privilege_level() -> (PrivilegeLevel, &'static str) {
+    let el = CurrentEL.read_as_enum(CurrentEL::EL);
+    match el {
+        Some(CurrentEL::EL::Value::EL2) => (PrivilegeLevel::Hypervisor, "EL2"),
+        Some(CurrentEL::EL::Value::EL1) => (PrivilegeLevel::Kernel, "EL1"),
+        Some(CurrentEL::EL::Value::EL0) => (PrivilegeLevel::User, "EL0"),
+        _ => (PrivilegeLevel::Unknown, "Unknown"),
+    }
+}

diff -uNr 08_hw_debug_JTAG/src/exception/asynchronous.rs 09_privilege_level/src/exception/asynchronous.rs
--- 08_hw_debug_JTAG/src/exception/asynchronous.rs
+++ 09_privilege_level/src/exception/asynchronous.rs
@@ -0,0 +1,14 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2020-2023 Andre Richter <andre.o.richter@gmail.com>
+
+//! Asynchronous exception handling.
+
+#[cfg(target_arch = "aarch64")]
+#[path = "../_arch/aarch64/exception/asynchronous.rs"]
+mod arch_asynchronous;
+
+//--------------------------------------------------------------------------------------------------
+// Architectural Public Reexports
+//--------------------------------------------------------------------------------------------------
+pub use arch_asynchronous::print_state;

diff -uNr 08_hw_debug_JTAG/src/exception.rs 09_privilege_level/src/exception.rs
--- 08_hw_debug_JTAG/src/exception.rs
+++ 09_privilege_level/src/exception.rs
@@ -0,0 +1,30 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2020-2023 Andre Richter <andre.o.richter@gmail.com>
+
+//! Synchronous and asynchronous exception handling.
+
+#[cfg(target_arch = "aarch64")]
+#[path = "_arch/aarch64/exception.rs"]
+mod arch_exception;
+
+pub mod asynchronous;
+
+//--------------------------------------------------------------------------------------------------
+// Architectural Public Reexports
+//--------------------------------------------------------------------------------------------------
+pub use arch_exception::current_privilege_level;
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// Kernel privilege levels.
+#[allow(missing_docs)]
+#[derive(Eq, PartialEq)]
+pub enum PrivilegeLevel {
+    User,
+    Kernel,
+    Hypervisor,
+    Unknown,
+}

diff -uNr 08_hw_debug_JTAG/src/main.rs 09_privilege_level/src/main.rs
--- 08_hw_debug_JTAG/src/main.rs
+++ 09_privilege_level/src/main.rs
@@ -121,6 +121,7 @@
 mod console;
 mod cpu;
 mod driver;
+mod exception;
 mod panic_wait;
 mod print;
 mod synchronization;
@@ -148,6 +149,7 @@

 /// The main function running after the early init.
 fn kernel_main() -> ! {
+    use console::console;
     use core::time::Duration;

     info!(
@@ -157,6 +159,12 @@
     );
     info!("Booting on: {}", bsp::board_name());

+    let (_, privilege_level) = exception::current_privilege_level();
+    info!("Current privilege level: {}", privilege_level);
+
+    info!("Exception handling state:");
+    exception::asynchronous::print_state();
+
     info!(
         "Architectural timer resolution: {} ns",
         time::time_manager().resolution().as_nanos()
@@ -165,11 +173,15 @@
     info!("Drivers loaded:");
     driver::driver_manager().enumerate();

-    // Test a failing timer case.
-    time::time_manager().spin_for(Duration::from_nanos(1));
+    info!("Timer test, spinning for 1 second");
+    time::time_manager().spin_for(Duration::from_secs(1));
+
+    info!("Echoing input now");

+    // Discard any spurious received characters before going into echo mode.
+    console().clear_rx();
     loop {
-        info!("Spinning for 1 second");
-        time::time_manager().spin_for(Duration::from_secs(1));
+        let c = console().read_char();
+        console().write_char(c);
     }
 }

diff -uNr 08_hw_debug_JTAG/tests/boot_test_string.rb 09_privilege_level/tests/boot_test_string.rb
--- 08_hw_debug_JTAG/tests/boot_test_string.rb
+++ 09_privilege_level/tests/boot_test_string.rb
@@ -1,3 +1,3 @@
 # frozen_string_literal: true

-EXPECTED_PRINT = 'Spinning for 1 second'
+EXPECTED_PRINT = 'Echoing input now'

```
