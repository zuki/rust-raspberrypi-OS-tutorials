# チュートリアル 06 - UARTチェインローダ

## tl;dr

- SDカードからの起動は良い経験でしたが、新しいバイナリのたびに行うのは非常に面倒です。
  そこで、[チェインローダ]を書いてみます。
- 今回がSDカードに書き込む必要のある最後のバイナリになります。今後のチュートリアルで
  は、`Makefile`に`chainboot`ターゲットを用意することで`UART`経由でカーネルを便利に
  ロードできるようにします。

[チェインローダ]: https://en.wikipedia.org/wiki/Chain_loading


## 注意

今回のチュートリアルでは、ソースコードの変更点を見ただけでは理解するのが非常に
難しいことがある点に注意してください。

それは`boot.s`にあります。そこには[位置独立なコード]が書かれています。それは
ファームウェアがバイナリをロードする場所（`0x8_0000`）とバイナリがリンクされる場所（`0x200_0000`、`link.ld`を参照）を自動的に決定します。バイナリは自分自身をロード
アドレスからリンクアドレスにコピーし（つまり、自身を「再配置（リロケート）」し）、
再配置されたバージョンの`_start_rust()`にジャンプします。

チェインローダは自分自身を「邪魔にならない」場所に置くので、`UART`から別のカーネル
バイナリを受信し、それをRPiファームウェアの標準ロードアドレスである`0x8_0000`に
コピーすることができます。最後に、`0x8_0000`にジャンプすると、新しくロードされた
バイナリは、あたかも初めからSDカードからロードされたかのように透過的に実行されます。

すべてを詳しく説明する時間ができるまで、どうかご容赦ください。当面、今回のチュート
リアルは、今後のチュートリアルを素早く起動できるようにするための便利な機能を実現
するためのものと考えてください。

[位置独立なコード]: https://en.wikipedia.org/wiki/Position-independent_code

## インストールとテスト

我々のチェインローダは`MiniLoad`という名前であり、[raspbootin]の影響を受けています。

すでに、今回のチュートリアルで試すことができます。
1. ターゲットハードウェアに応じて、`make`または`BSP=rpi4 make`を実行します。
2. `kernel8.img`をSDカードにコピーして、SDカードをRPiに差し戻します。
3. `make chainboot`または`BSP=rpi4 make chainboot`を実行します。
4. USBシリアルをホストPCに接続します。
     - 配線図は[トップレベルのREADME](../README.md#-usb-serial-output)にあります。
     - USBシリアルの電源ピンは接続**しない**でください。RX/TXとGNDのみ接続します。
5. RPiを(USB)電源ケーブルに接続します。
6. ローダが`UART`経由でカーネルを取得するのを確認します。

> ! **注意**: `make chainboot`はデフォルトのシリアルデバイス名を`/dev/ttyUSB0`と
> 仮定しています。ホストOSによっては、デバイス名が異なる場合があります。たとえば、
> `macOS`では、`/dev/tty.usbserial-0001`のような名前になります。この場合は、
> 明示的に名前を指定してください。

```console
$ DEV_SERIAL=/dev/tty.usbserial-0001 make chainboot
```

[raspbootin]: https://github.com/mrvn/raspbootin

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
[MP] ⏩ Pushing 6 KiB ==========================================🦀 100% 0 KiB/s Time: 00:00:00
[ML] Loaded! Executing the payload now

[0] mingo version 0.5.0
[1] Booting on: Raspberry Pi 3
[2] Drivers loaded:
      1. BCM GPIO
      2. BCM PL011 UART
[3] Chars written: 117
[4] Echoing input now
```

今回のチュートリアルでは、前回のチュートリアルで作成したバージョンのカーネルを
デモ用にロードします。以降のチュートリアルでは、作業ディレクトリのカーネルを
使用します。

## テスト

今回のチュートリアルの`Makefile`には`qemuasm`というターゲットが追加されており、
カーネルが自分自身を再配置した後、ロードアドレス領域(0x80_XXX)から(`0x0200_0XXX`)に
再配置されたコードにジャンプする様子をよく観察することができます。

```console
$ make qemuasm
[...]
N:
0x00080030:  58000140  ldr      x0, #0x80058
0x00080034:  9100001f  mov      sp, x0
0x00080038:  58000141  ldr      x1, #0x80060
0x0008003c:  d61f0020  br       x1

----------------
IN:
0x02000070:  9400044c  bl       #0x20011a0

----------------
IN:
0x020011a0:  90000008  adrp     x8, #0x2001000
0x020011a4:  90000009  adrp     x9, #0x2001000
0x020011a8:  f9446508  ldr      x8, [x8, #0x8c8]
0x020011ac:  f9446929  ldr      x9, [x9, #0x8d0]
0x020011b0:  eb08013f  cmp      x9, x8
0x020011b4:  54000109  b.ls     #0x20011d4
[...]
```

## 前チュートリアルとのdiff
```diff

diff -uNr 05_drivers_gpio_uart/Cargo.toml 06_uart_chainloader/Cargo.toml
--- 05_drivers_gpio_uart/Cargo.toml
+++ 06_uart_chainloader/Cargo.toml
@@ -1,6 +1,6 @@
 [package]
 name = "mingo"
-version = "0.5.0"
+version = "0.6.0"
 authors = ["Andre Richter <andre.o.richter@gmail.com>"]
 edition = "2018"

Binary files 05_drivers_gpio_uart/demo_payload_rpi3.img and 06_uart_chainloader/demo_payload_rpi3.img differ
Binary files 05_drivers_gpio_uart/demo_payload_rpi4.img and 06_uart_chainloader/demo_payload_rpi4.img differ

diff -uNr 05_drivers_gpio_uart/Makefile 06_uart_chainloader/Makefile
--- 05_drivers_gpio_uart/Makefile
+++ 06_uart_chainloader/Makefile
@@ -25,6 +25,7 @@
     READELF_BINARY    = aarch64-none-elf-readelf
     LINKER_FILE       = src/bsp/raspberrypi/link.ld
     RUSTC_MISC_ARGS   = -C target-cpu=cortex-a53
+    CHAINBOOT_DEMO_PAYLOAD = demo_payload_rpi3.img
 else ifeq ($(BSP),rpi4)
     TARGET            = aarch64-unknown-none-softfloat
     KERNEL_BIN        = kernel8.img
@@ -36,6 +37,7 @@
     READELF_BINARY    = aarch64-none-elf-readelf
     LINKER_FILE       = src/bsp/raspberrypi/link.ld
     RUSTC_MISC_ARGS   = -C target-cpu=cortex-a72
+    CHAINBOOT_DEMO_PAYLOAD = demo_payload_rpi4.img
 endif

 # Export for build.rs
@@ -68,19 +70,22 @@
 DOCKER_ARG_DEV       = --privileged -v /dev:/dev

 DOCKER_QEMU  = $(DOCKER_CMD_INTERACT) $(DOCKER_IMAGE)
+DOCKER_TEST  = $(DOCKER_CMD) -t $(DOCKER_ARG_DIR_UTILS) $(DOCKER_IMAGE)
 DOCKER_TOOLS = $(DOCKER_CMD) $(DOCKER_IMAGE)

 # Dockerize commands that require USB device passthrough only on Linux
 ifeq ($(UNAME_S),Linux)
     DOCKER_CMD_DEV = $(DOCKER_CMD_INTERACT) $(DOCKER_ARG_DEV)

-    DOCKER_MINITERM = $(DOCKER_CMD_DEV) $(DOCKER_ARG_DIR_UTILS) $(DOCKER_IMAGE)
+    DOCKER_CHAINBOOT = $(DOCKER_CMD_DEV) $(DOCKER_ARG_DIR_UTILS) $(DOCKER_IMAGE)
 endif

-EXEC_QEMU     = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
-EXEC_MINITERM = ruby ../utils/miniterm.rb
+EXEC_QEMU          = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
+EXEC_MINIPUSH      = ruby ../utils/minipush.rb
+EXEC_QEMU_MINIPUSH = ruby tests/qemu_minipush.rb

-.PHONY: all $(KERNEL_ELF) $(KERNEL_BIN) doc qemu miniterm clippy clean readelf objdump nm check
+.PHONY: all $(KERNEL_ELF) $(KERNEL_BIN) doc qemu qemuasm chainboot clippy clean readelf objdump nm \
+    check

 all: $(KERNEL_BIN)

@@ -96,16 +101,26 @@
 	@$(DOC_CMD) --document-private-items --open

 ifeq ($(QEMU_MACHINE_TYPE),)
-qemu:
+qemu test:
 	$(call colorecho, "\n$(QEMU_MISSING_STRING)")
 else
 qemu: $(KERNEL_BIN)
 	$(call colorecho, "\nLaunching QEMU")
 	@$(DOCKER_QEMU) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN)
+
+qemuasm: $(KERNEL_BIN)
+	$(call colorecho, "\nLaunching QEMU with ASM output")
+	@$(DOCKER_QEMU) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN) -d in_asm
+
+test: $(KERNEL_BIN)
+	$(call colorecho, "\nTesting chainloading - $(BSP)")
+	@$(DOCKER_TEST) $(EXEC_QEMU_MINIPUSH) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) \
+                -kernel $(KERNEL_BIN) $(CHAINBOOT_DEMO_PAYLOAD)
+
 endif

-miniterm:
-	@$(DOCKER_MINITERM) $(EXEC_MINITERM) $(DEV_SERIAL)
+chainboot:
+	@$(DOCKER_CHAINBOOT) $(EXEC_MINIPUSH) $(DEV_SERIAL) $(CHAINBOOT_DEMO_PAYLOAD)

 clippy:
 	@RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(CLIPPY_CMD)

diff -uNr 05_drivers_gpio_uart/src/_arch/aarch64/cpu/boot.s 06_uart_chainloader/src/_arch/aarch64/cpu/boot.s
--- 05_drivers_gpio_uart/src/_arch/aarch64/cpu/boot.s
+++ 06_uart_chainloader/src/_arch/aarch64/cpu/boot.s
@@ -6,11 +6,11 @@
 // 定義
 //--------------------------------------------------------------------------------------------------

-// シンボルのアドレスをレジスタにロードする（PC-相対）。
+// シンボルのアドレス（PC-相対アドレス）をレジスタにロードする。
 //
 // シンボルはプログラムカウンタの +/- 4GiB以内になければならない。
 //
-// # リソース
+// # 参考資料
 //
 // - https://sourceware.org/binutils/docs-2.36/as/AArch64_002dRelocations.html
 .macro ADR_REL register, symbol
@@ -18,6 +18,17 @@
        add     \register, \register, #:lo12:\symbol
 .endm

+// シンボルのアドレス（絶対アドレス）をレジスタにロードする
+//
+// # Resources
+//
+// - https://sourceware.org/binutils/docs-2.36/as/AArch64_002dRelocations.html
+.macro ADR_ABS register, symbol
+       movz    \register, #:abs_g2:\symbol
+       movk    \register, #:abs_g1_nc:\symbol
+       movk    \register, #:abs_g0_nc:\symbol
+.endm
+
 .equ _core_id_mask, 0b11

 //--------------------------------------------------------------------------------------------------
@@ -34,20 +45,31 @@
        and     x1, x1, _core_id_mask // _code_id_mask = 0b11; このファイルの先頭で定義
        ldr     x2, BOOT_CORE_ID      // BOOT_CORE_ID=0: bsp/__board_name__/cpu.rs で定義
        cmp     x1, x2
-       b.ne    1f                    // core0以外は1へジャンプ
+       b.ne    2f                    // core0以外は2へジャンプ
+
+       // 処理がここに来たらそれはブートコア。

-       // 処理がここに来たらそれはブートコア。Rustコードにジャンプするための準備をする。
+       // 次に、バイナリを再配置する
+       ADR_REL x0, __binary_nonzero_start         // バイナリのロードアドレス
+       ADR_ABS x1, __binary_nonzero_start         // バイナリのリンクアドレス
+       ADR_ABS x2, __binary_nonzero_end_exclusive
+
+1:     ldr     x3, [x0], #8    // x3 <- [x0]; x0+=8
+       str     x3, [x1], #8    // x3 -> [x1]; x1+=8
+       cmp     x1, x2          // x1 - x2
+       b.lo    1b              // goto 1b if x1 < x2

        // スタックポインタを設定する。
-       ADR_REL x0, __boot_core_stack_end_exclusive     // link.ldで定義 = 0x80000 .textの下に伸びる
+       ADR_ABS x0, __boot_core_stack_end_exclusive
        mov     sp, x0

-       // Rustコードにジャンプする。
-       b       _start_rust
+       // 再配置されたRustコードにジャンプする
+       ADR_ABS x1, _start_rust
+       br      x1

        // イベントを無限に待つ（別名 "park the core"）
-1:     wfe
-       b       1b
+2:     wfe
+       b       2b

 .size  _start, . - _start
 .type  _start, function

diff -uNr 05_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs 06_uart_chainloader/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
--- 05_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
+++ 06_uart_chainloader/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
@@ -143,7 +143,7 @@

         // （BCM2837ペリフェラルのPDFに記載されているシーケンスの）適切な遅延値を
         // 経験的に推測する。
-        //   - Wikipediaによると、最速のPi3のクロックは1.4GHz程度
+        //   - Wikipediaによると、最速のRPi4のクロックは1.5GHz程度
         //   - Linuxの2837 GPIOドライバは、ステップ間で1μs待つ
         //
         // 安全側にふって、デフォルトを2000サイクルとする。CPUのクロックが2GHzの場合、

diff -uNr 05_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs 06_uart_chainloader/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
--- 05_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
+++ 06_uart_chainloader/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
@ -285,8 +285,8 @@
     }

     /// 1文字受信する
-    fn read_char_converting(&mut self, blocking_mode: BlockingMode) -> Option<char> {
-        // RX FIFOがからの場合
+    fn read_char(&mut self, blocking_mode: BlockingMode) -> Option<char> {
+        // RX FIFOが空の場合
         if self.registers.FR.matches_all(FR::RXFE::SET) {
             // ノンブロッキングモードの場合はすぐにリターンする
             if blocking_mode == BlockingMode::NonBlocking {
@@ -300,12 +300,7 @@
         }

         // 1文字読み込む
-        let mut ret = self.registers.DR.get() as u8 as char;
-
-        // 復帰を改行に変換する
-        if ret == '\r' {
-            ret = '\n'
-        }
+        let ret = self.registers.DR.get() as u8 as char;

         // 統計を更新する
         self.chars_read += 1;
@@ -320,7 +315,7 @@
 ///
 /// この関数は `&mut self` を取るので、内部構造体を実装する必要がある
 ///
-/// [`src/print.rs`]を参照
+/// See [`src/print.rs`].
 ///
 /// [`src/print.rs`]: ../../print/index.html
 impl fmt::Write for PL011UartInner {
@@ -385,14 +380,14 @@
 impl console::interface::Read for PL011Uart {
     fn read_char(&self) -> char {
         self.inner
-            .lock(|inner| inner.read_char_converting(BlockingMode::Blocking).unwrap())
+            .lock(|inner| inner.read_char(BlockingMode::Blocking).unwrap())
     }

     fn clear_rx(&self) {
         // 空になるまでRX FIFOを読み込む
         while self
             .inner
-            .lock(|inner| inner.read_char_converting(BlockingMode::NonBlocking))
+            .lock(|inner| inner.read_char(BlockingMode::NonBlocking))
             .is_some()
         {}
     }

diff -uNr 05_drivers_gpio_uart/src/bsp/raspberrypi/link.ld 06_uart_chainloader/src/bsp/raspberrypi/link.ld
--- 05_drivers_gpio_uart/src/bsp/raspberrypi/link.ld
+++ 06_uart_chainloader/src/bsp/raspberrypi/link.ld
@@ -16,7 +16,8 @@

 SECTIONS
 {
-    . =  __rpi_load_addr;
+    /* Set the link address to 32 MiB */
+    . = 0x2000000;
                                         /*   ^             */
                                         /*   | stack       */
                                         /*   | growth      */
@@ -26,6 +27,7 @@
     /***********************************************************************************************
     * Code + RO Data + Global Offset Table
     ***********************************************************************************************/
+    __binary_nonzero_start = .;
     .text :
     {
         KEEP(*(.text._start))
@@ -42,8 +44,12 @@
     ***********************************************************************************************/
     .data : { *(.data*) } :segment_rw

+    /* Fill up to 8 byte, b/c relocating the binary is done in u64 chunks */
+    . = ALIGN(8);
+    __binary_nonzero_end_exclusive = .;
+
     /* Section is zeroed in u64 chunks, align start and end to 8 bytes */
-    .bss : ALIGN(8)
+    .bss :
     {
         __bss_start = .;
         *(.bss*);

diff -uNr 05_drivers_gpio_uart/src/bsp/raspberrypi/memory.rs 06_uart_chainloader/src/bsp/raspberrypi/memory.rs
--- 05_drivers_gpio_uart/src/bsp/raspberrypi/memory.rs
+++ 06_uart_chainloader/src/bsp/raspberrypi/memory.rs
@@ -23,9 +23,10 @@
 /// ボードの物理メモリアドレス
 #[rustfmt::skip]
 pub(super) mod map {
+    pub const BOARD_DEFAULT_LOAD_ADDRESS: usize =        0x8_0000;

-    pub const GPIO_OFFSET:         usize = 0x0020_0000;
-    pub const UART_OFFSET:         usize = 0x0020_1000;
+    pub const GPIO_OFFSET:                usize =        0x0020_0000;
+    pub const UART_OFFSET:                usize =        0x0020_1000;

     /// 物理デバイス
     #[cfg(feature = "bsp_rpi3")]
@@ -52,7 +53,13 @@
 // パブリックコード
 //--------------------------------------------------------------------------------------------------

-/// .bssセクションに含まれる範囲を返す
+/// Raspberryのファームウェアがデフォルトですべてのバイナリをロードするアドレス
+#[inline(always)]
+pub fn board_default_load_addr() -> *const u64 {
+    map::BOARD_DEFAULT_LOAD_ADDRESS as _
+}
+
+/// 再配置されたbssセクションに含まれる範囲を返す
 ///
 /// # 安全性
 ///

diff -uNr 05_drivers_gpio_uart/src/main.rs 06_uart_chainloader/src/main.rs
--- 05_drivers_gpio_uart/src/main.rs
+++ 06_uart_chainloader/src/main.rs
@@ -107,6 +107,7 @@
 //! [`runtime_init::runtime_init()`]: runtime_init/fn.runtime_init.html

 #![allow(clippy::upper_case_acronyms)]
+#![feature(asm)]
 #![feature(const_fn_fn_ptr_basics)]
 #![feature(format_args_nl)]
 #![feature(global_asm)]
@@ -146,38 +147,56 @@
     kernel_main()
 }

+const MINILOAD_LOGO: &str = r#"
+ __  __ _      _ _                 _
+|  \/  (_)_ _ (_) |   ___  __ _ __| |
+| |\/| | | ' \| | |__/ _ \/ _` / _` |
+|_|  |_|_|_||_|_|____\___/\__,_\__,_|
+"#;
+
 /// 最初の初期化後に実行するメイン関数
 fn kernel_main() -> ! {
     use bsp::console::console;
     use console::interface::All;
-    use driver::interface::DriverManager;
-
-    println!(
-        "[0] {} version {}",
-        env!("CARGO_PKG_NAME"),
-        env!("CARGO_PKG_VERSION")
-    );
-    println!("[1] Booting on: {}", bsp::board_name());
-
-    println!("[2] Drivers loaded:");
-    for (i, driver) in bsp::driver::driver_manager()
-        .all_device_drivers()
-        .iter()
-        .enumerate()
-    {
-        println!("      {}. {}", i + 1, driver.compatible());
-    }

-    println!(
-        "[3] Chars written: {}",
-        bsp::console::console().chars_written()
-    );
-    println!("[4] Echoing input now");
+    println!("{}", MINILOAD_LOGO);
+    println!("{:^37}", bsp::board_name());
+    println!();
+    println!("[ML] Requesting binary");
+    console().flush();

     // エコーモードに移行する前に受信したスプリアス文字を破棄する
     console().clear_rx();
-    loop {
-        let c = bsp::console::console().read_char();
-        bsp::console::console().write_char(c);
+
+    // `Minipush`にバイナリを送信するよう通知する
+    for _ in 0..3 {
+        console().write_char(3 as char);
     }
+
+    // バイナリサイズを読み込む
+    let mut size: u32 = u32::from(console().read_char() as u8);
+    size |= u32::from(console().read_char() as u8) << 8;
+    size |= u32::from(console().read_char() as u8) << 16;
+    size |= u32::from(console().read_char() as u8) << 24;
+
+    // サイズが巨大でないことを信じる
+    console().write_char('O');
+    console().write_char('K');
+
+    let kernel_addr: *mut u8 = bsp::memory::board_default_load_addr() as *mut u8;
+    unsafe {
+        // カーネルをバイトごとに読み込む
+        for i in 0..size {
+            core::ptr::write_volatile(kernel_addr.offset(i as isize), console().read_char() as u8)
+        }
+    }
+
+    println!("[ML] Loaded! Executing the payload now\n");
+    console().flush();
+
+    // 関数ポインタを作成するために頃魔術を使用する
+    let kernel: fn() -> ! = unsafe { core::mem::transmute(kernel_addr) };
+
+    // ロードしたカーネルにジャンプする!
+    kernel()
 }

diff -uNr 05_drivers_gpio_uart/tests/qemu_minipush.rb 06_uart_chainloader/tests/qemu_minipush.rb
--- 05_drivers_gpio_uart/tests/qemu_minipush.rb
+++ 06_uart_chainloader/tests/qemu_minipush.rb
@@ -0,0 +1,80 @@
+# frozen_string_literal: true
+
+# SPDX-License-Identifier: MIT OR Apache-2.0
+#
+# Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>
+
+require_relative '../../utils/minipush'
+require 'expect'
+require 'timeout'
+
+# Match for the last print that 'demo_payload_rpiX.img' produces.
+EXPECTED_PRINT = 'Echoing input now'
+
+# The main class
+class QEMUMiniPush < MiniPush
+    TIMEOUT_SECS = 3
+
+    # override
+    def initialize(qemu_cmd, binary_image_path)
+        super(nil, binary_image_path)
+
+        @qemu_cmd = qemu_cmd
+    end
+
+    private
+
+    def quit_qemu_graceful
+        Timeout.timeout(5) do
+            pid = @target_serial.pid
+            Process.kill('TERM', pid)
+            Process.wait(pid)
+        end
+    end
+
+    # override
+    def open_serial
+        @target_serial = IO.popen(@qemu_cmd, 'r+', err: '/dev/null')
+
+        # Ensure all output is immediately flushed to the device.
+        @target_serial.sync = true
+
+        puts "[#{@name_short}] ✅ Serial connected"
+    end
+
+    # override
+    def terminal
+        result = @target_serial.expect(EXPECTED_PRINT, TIMEOUT_SECS)
+        exit(1) if result.nil?
+
+        puts result
+
+        quit_qemu_graceful
+    end
+
+    # override
+    def connetion_reset; end
+
+    # override
+    def handle_reconnect(error)
+        handle_unexpected(error)
+    end
+end
+
+##--------------------------------------------------------------------------------------------------
+## Execution starts here
+##--------------------------------------------------------------------------------------------------
+puts
+puts 'QEMUMiniPush 1.0'.cyan
+puts
+
+# CTRL + C handler. Only here to suppress Ruby's default exception print.
+trap('INT') do
+    # The `ensure` block from `QEMUMiniPush::run` will run after exit, restoring console state.
+    exit
+end
+
+binary_image_path = ARGV.pop
+qemu_cmd = ARGV.join(' ')
+
+QEMUMiniPush.new(qemu_cmd, binary_image_path).run

diff -uNr 05_drivers_gpio_uart/update.sh 06_uart_chainloader/update.sh
--- 05_drivers_gpio_uart/update.sh
+++ 06_uart_chainloader/update.sh
@@ -0,0 +1,8 @@
+#!/usr/bin/env bash
+
+cd ../05_drivers_gpio_uart
+BSP=rpi4 make
+cp kernel8.img ../06_uart_chainloader/demo_payload_rpi4.img
+make
+cp kernel8.img ../06_uart_chainloader/demo_payload_rpi3.img
+rm kernel8.img

```
