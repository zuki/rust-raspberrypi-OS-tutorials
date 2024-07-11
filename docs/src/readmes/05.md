# Tutorial 05 - Drivers: GPIO and UART

## tl;dr

- 先のチュートリアルで安全なグローバルを有効にしたので、最初の本物のデバイスドライバを追加するための基盤が整いました。
- 魔法のQEMUコンソールは捨てて、本物の`UART`を使います。本格的な組み込みハッカーがするように!

## 特筆すべき追加事項

- 初めて、実際のハードウェア上でコードを実行できるようになります。
  - そのため、**RPi 3**と**RPi4**でビルドが区別されます。
  - デフォルトでは`Makefile`のすべてのターゲットは**RPi 3**用にビルドします。
  - **RPi 4**用にビルドするには、各ターゲットの前に`BSP=rpi4`を付けます。たとえば
    - `BSP=rpi4 make`
    - `BSP=rpi4 make doc`
  - 残念ながら、QEMUはまだ**RPi 4**をサポートしていないので、`BSP=rpi4 make qemu`は動作しません。
- カーネルコードで`BSP`ドライバの実装を抽象化するために`driver::interface::DeviceDriver`トレイトが追加されました。
- ドライバは`src/bsp/device_driver`に格納されており、`BSP`で再利用できます。
  - RPiのPL011 UARTをPinmux（`SoC`の内部から実際のHWピンに信号をルーティングすること）する`GPIO`ドライバを導入します。
    - このドライバがどのように**RPi 3**と**RPi 4*を区別するのかに注意してください。両者はHWが異なるので、SWでそれを考慮する必要があるからです。
  - 最も重要なのものは`PL011Uart`ドライバです。これは`console::interface::*`トレイトを実装しており、今後、メインシステムのコンソール出力として使用されます。
- BSPは`src/bsp/raspberrypi/memory.rs`にメモリマップを含むようになりました。具体的には、RasPiのMMIOアドレスを含んでおり、各デバイスドライバのインスタンス化に使用されます。
- `panic!`ハンドラを変更し、`println!`に依存しないようにしました。これはエラーが発生した際にロックされる可能性のあるグローバルに共有される`UART`のインストを使用しているからです（今のところ、`NullLock`のためロックは発生しませんが、本物のロックでは問題になります）。
  - 代わりに、新しいUARTドライバインスタンスを作成し、デバイスを再初期化し、そのインスタンスをprintに使用します。これにより、システムが自身をサスペンドする前に最後の重要なメッセージをprintできる可能性が高まります。

## SDカードからのブート

SDカードを用意する方法はRPi3とRPi4で異なるので注意が必要です。

### 両者に共通

1. `boot`という名前の単一の`FAT32`パーティションを作成します。
2. カードに次の内容の`config.txt`という名前のファイルを作成します。

```txt
arm_64bit=1
init_uart_clock=48000000
```
### Pi 3

3. [Raspberry Pi firmware repo](https://github.com/raspberrypi/firmware/tree/master/boot) から次のファイルをSDカードにコピーします。
    - [bootcode.bin](https://github.com/raspberrypi/firmware/raw/master/boot/bootcode.bin)
    - [fixup.dat](https://github.com/raspberrypi/firmware/raw/master/boot/fixup.dat)
    - [start.elf](https://github.com/raspberrypi/firmware/raw/master/boot/start.elf)
4. `make`を実行します。

### Pi 4

3. [Raspberry Pi firmware repo](https://github.com/raspberrypi/firmware/tree/master/boot)から次のファイルをSDカードにコピーします。
    - [fixup4.dat](https://github.com/raspberrypi/firmware/raw/master/boot/fixup4.dat)
    - [start4.elf](https://github.com/raspberrypi/firmware/raw/master/boot/start4.elf)
    - [bcm2711-rpi-4-b.dtb](https://github.com/raspberrypi/firmware/raw/master/boot/bcm2711-rpi-4-b.dtb)
4. `BSP=rpi4 make`を実行します。


_**注意**: RPi4で動かなかった場合は、カード上の`start4.elf`を`start.elf`と改名（4を取る）してみてください。_

### 再度、両者共通

5. SDカードに`kernel8.img`をコピーして、RPiに再度挿入します。
6. ホスト上のUARTデバイスを開く、`miniterm`ターゲットを実行します。

```console
$ make miniterm
```

> ❗ **注意**: `Miniterm`はデフォルトのシリアルデバイス名を`/dev/ttyUSB0`としています。使用する. Depending on your
> ホストOSにより、デバイス名は異なる場合があります。たとえば、`macOS`では`/dev/tty.usbserial-0001`の
> ような名前になります。この場合は、名前を明示的に与えてください。

```console
$ DEV_SERIAL=/dev/tty.usbserial-0001 make miniterm
```

7. USBシリアルをホストPCに接続します。
    - 接続方は[トップページのREADME](../README.md#-usb-serial-output)にあります。
    - USBシリアルの電源ピンは接続**しない**でください。RX/TXとGNDだけを接続します。
8. RPiを(USB)電源ケーブルに接続し、出力を観察します。

```console
Miniterm 1.0

[MT] ⏳ Waiting for /dev/ttyUSB0
[MT] ✅ Serial connected
[0] mingo version 0.5.0
[1] Booting on: Raspberry Pi 3
[2] Drivers loaded:
      1. BCM GPIO
      2. BCM PL011 UART
[3] Chars written: 117
[4] Echoing input now
```

8. 終了するには<kbd>ctrl-c</kbd>を押下します。

## 前チュートリアルとのdiff
```diff

diff -uNr 04_safe_globals/Cargo.toml 05_drivers_gpio_uart/Cargo.toml
--- 04_safe_globals/Cargo.toml
+++ 05_drivers_gpio_uart/Cargo.toml
@@ -1,6 +1,6 @@
 [package]
 name = "mingo"
-version = "0.4.0"
+version = "0.5.0"
 authors = ["Andre Richter <andre.o.richter@gmail.com>"]
 edition = "2018"

@@ -9,8 +9,8 @@

 [features]
 default = []
-bsp_rpi3 = []
-bsp_rpi4 = []
+bsp_rpi3 = ["register"]
+bsp_rpi4 = ["register"]

 [[bin]]
 name = "kernel"
@@ -22,6 +22,9 @@

 [dependencies]

+# Optional dependencies
+register = { version = "1.x.x", optional = true }
+
 # Platform specific dependencies
 [target.'cfg(target_arch = "aarch64")'.dependencies]
 cortex-a = { version = "5.x.x" }

diff -uNr 04_safe_globals/Makefile 05_drivers_gpio_uart/Makefile
--- 04_safe_globals/Makefile
+++ 05_drivers_gpio_uart/Makefile
@@ -7,6 +7,12 @@
 # Default to the RPi3
 BSP ?= rpi3

+# Default to a serial device name that is common in Linux.
+DEV_SERIAL ?= /dev/ttyUSB0
+
+# Query the host system's kernel name
+UNAME_S = $(shell uname -s)
+
 # BSP-specific arguments
 ifeq ($(BSP),rpi3)
     TARGET            = aarch64-unknown-none-softfloat
@@ -58,13 +64,23 @@
 DOCKER_IMAGE         = rustembedded/osdev-utils
 DOCKER_CMD           = docker run --rm -v $(shell pwd):/work/tutorial -w /work/tutorial
 DOCKER_CMD_INTERACT  = $(DOCKER_CMD) -i -t
+DOCKER_ARG_DIR_UTILS = -v $(shell pwd)/../utils:/work/utils
+DOCKER_ARG_DEV       = --privileged -v /dev:/dev

 DOCKER_QEMU  = $(DOCKER_CMD_INTERACT) $(DOCKER_IMAGE)
 DOCKER_TOOLS = $(DOCKER_CMD) $(DOCKER_IMAGE)

-EXEC_QEMU = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
+# Dockerize commands that require USB device passthrough only on Linux
+ifeq ($(UNAME_S),Linux)
+    DOCKER_CMD_DEV = $(DOCKER_CMD_INTERACT) $(DOCKER_ARG_DEV)
+
+    DOCKER_MINITERM = $(DOCKER_CMD_DEV) $(DOCKER_ARG_DIR_UTILS) $(DOCKER_IMAGE)
+endif
+
+EXEC_QEMU     = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
+EXEC_MINITERM = ruby ../utils/miniterm.rb

-.PHONY: all $(KERNEL_ELF) $(KERNEL_BIN) doc qemu clippy clean readelf objdump nm check
+.PHONY: all $(KERNEL_ELF) $(KERNEL_BIN) doc qemu miniterm clippy clean readelf objdump nm check

 all: $(KERNEL_BIN)

@@ -88,6 +104,9 @@
 	@$(DOCKER_QEMU) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN)
 endif

+miniterm:
+	@$(DOCKER_MINITERM) $(EXEC_MINITERM) $(DEV_SERIAL)
+
 clippy:
 	@RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(CLIPPY_CMD)


diff -uNr 04_safe_globals/src/_arch/aarch64/cpu.rs 05_drivers_gpio_uart/src/_arch/aarch64/cpu.rs
--- 04_safe_globals/src/_arch/aarch64/cpu.rs
+++ 05_drivers_gpio_uart/src/_arch/aarch64/cpu.rs
@@ -17,6 +17,17 @@
 // パブリックコード
 //--------------------------------------------------------------------------------------------------

+pub use asm::nop;
+
+/// `n`サイクルスピンする
+#[cfg(feature = "bsp_rpi3")]
+#[inline(always)]
+pub fn spin_for_cycles(n: usize) {
+    for _ in 0..n {
+        asm::nop();
+    }
+}
+
 /// コア上での実行を休止する
 #[inline(always)]
 pub fn wait_forever() -> ! {

diff -uNr 04_safe_globals/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs 05_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
--- 04_safe_globals/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
+++ 05_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
@ -0,0 +1,220 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! GPIOドライバ
+
+use crate::{
+    bsp::device_driver::common::MMIODerefWrapper, driver, synchronization,
+    synchronization::NullLock,
+};
+use register::{mmio::*, register_bitfields, register_structs};
+
+//--------------------------------------------------------------------------------------------------
+// プライベート定義
+//--------------------------------------------------------------------------------------------------
+
+// GPIOレジスタ
+//
+// 記述は以下から採った
+// - https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf
+// - https://datasheets.raspberrypi.org/bcm2711/bcm2711-peripherals.pdf
+register_bitfields! {
+    u32,
+
+    /// GPIO機能選択 1
+    GPFSEL1 [
+        /// ピン15
+        FSEL15 OFFSET(15) NUMBITS(3) [
+            Input = 0b000,
+            Output = 0b001,
+            AltFunc0 = 0b100  // PL011 UART RX
+
+        ],
+
+        /// ピン 14
+        FSEL14 OFFSET(12) NUMBITS(3) [
+            Input = 0b000,
+            Output = 0b001,
+            AltFunc0 = 0b100  // PL011 UART TX
+        ]
+    ],
+
+    /// GPIOプルアップ/プルダウンレジスタ
+    ///
+    /// BCM2837のみ
+    GPPUD [
+        /// すべてのGPIOピンの内部プルアップ/プルダウンコントロールラインの動作を制御する
+        PUD OFFSET(0) NUMBITS(2) [
+            Off = 0b00,
+            PullDown = 0b01,
+            PullUp = 0b10
+        ]
+    ],
+
+    /// GPIOプルアップ/プルダウンクロックレジスタ
+    ///
+    /// BCM2837のみ
+    GPPUDCLK0 [
+        /// ピン 15
+        PUDCLK15 OFFSET(15) NUMBITS(1) [
+            NoEffect = 0,
+            AssertClock = 1
+        ],
+
+        /// ピン 14
+        PUDCLK14 OFFSET(14) NUMBITS(1) [
+            NoEffect = 0,
+            AssertClock = 1
+        ]
+    ],
+
+    /// GPIOプルアップ/プルダウンレジスタ 0
+    ///
+    /// BCM2711のみ
+    GPIO_PUP_PDN_CNTRL_REG0 [
+        /// ピン 15
+        GPIO_PUP_PDN_CNTRL15 OFFSET(30) NUMBITS(2) [
+            NoResistor = 0b00,
+            PullUp = 0b01
+        ],
+
+        /// ピン 14
+        GPIO_PUP_PDN_CNTRL14 OFFSET(28) NUMBITS(2) [
+            NoResistor = 0b00,
+            PullUp = 0b01
+        ]
+    ]
+}
+
+register_structs! {
+    #[allow(non_snake_case)]
+    RegisterBlock {
+        (0x00 => _reserved1),
+        (0x04 => GPFSEL1: ReadWrite<u32, GPFSEL1::Register>),
+        (0x08 => _reserved2),
+        (0x94 => GPPUD: ReadWrite<u32, GPPUD::Register>),
+        (0x98 => GPPUDCLK0: ReadWrite<u32, GPPUDCLK0::Register>),
+        (0x9C => _reserved3),
+        (0xE4 => GPIO_PUP_PDN_CNTRL_REG0: ReadWrite<u32, GPIO_PUP_PDN_CNTRL_REG0::Register>),
+        (0xE8 => @END),
+    }
+}
+
+/// 対応するMMIOレジスタのための抽象化
+type Registers = MMIODerefWrapper<RegisterBlock>;
+
+//--------------------------------------------------------------------------------------------------
+// パブリック定義
+//--------------------------------------------------------------------------------------------------
+
+pub struct GPIOInner {
+    registers: Registers,
+}
+
+// BSPがpanicハンドラで使用できるように内部構造体をエクスポートする
+pub use GPIOInner as PanicGPIO;
+
+/// GPIO HWを表す構造体
+pub struct GPIO {
+    inner: NullLock<GPIOInner>,
+}
+
+//--------------------------------------------------------------------------------------------------
+// パブリックコード
+//--------------------------------------------------------------------------------------------------
+
+impl GPIOInner {
+    /// インスタンスを作成する
+    ///
+    /// # 安全性
+    ///
+    /// - ユーザは正しいMMIO開始アドレスを提供する必要がある
+    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
+        Self {
+            registers: Registers::new(mmio_start_addr),
+        }
+    }
+
+    /// ピン14と15のプルアップ/プルダウンを無効にする
+    #[cfg(feature = "bsp_rpi3")]
+    fn disable_pud_14_15_bcm2837(&mut self) {
+        use crate::cpu;
+
+        // （BCM2837ペリフェラルのPDFに記載されているシーケンスの）適切な遅延値を
+        // 経験的に推測する。
+        //   - Wikipediaによると、最速のPi3のクロックは1.4GHz程度
+        //   - Linuxの2837 GPIOドライバは、ステップ間で1μs待つ
+        //
+        // 安全側にふって、デフォルトを2000サイクルとする。CPUのクロックが2GHzの場合、
+        // この値は1μsに相当する。
+        const DELAY: usize = 2000;
+
+        self.registers.GPPUD.write(GPPUD::PUD::Off);
+        cpu::spin_for_cycles(DELAY);
+
+        self.registers
+            .GPPUDCLK0
+            .write(GPPUDCLK0::PUDCLK15::AssertClock + GPPUDCLK0::PUDCLK14::AssertClock);
+        cpu::spin_for_cycles(DELAY);
+
+        self.registers.GPPUD.write(GPPUD::PUD::Off);
+        self.registers.GPPUDCLK0.set(0);
+    }
+
+    /// ピン14と15のプルアップ/プルダウンを無効にする
+    #[cfg(feature = "bsp_rpi4")]
+    fn disable_pud_14_15_bcm2711(&mut self) {
+        self.registers.GPIO_PUP_PDN_CNTRL_REG0.write(
+            GPIO_PUP_PDN_CNTRL_REG0::GPIO_PUP_PDN_CNTRL15::PullUp
+                + GPIO_PUP_PDN_CNTRL_REG0::GPIO_PUP_PDN_CNTRL14::PullUp,
+        );
+    }
+
+    /// PL011 UARTを標準アウトプットにマップする
+    ///
+    /// TXをピン14に
+    /// RXをピン15に
+    pub fn map_pl011_uart(&mut self) {
+        // ピン14と15のUARTを選択する
+        self.registers
+            .GPFSEL1
+            .modify(GPFSEL1::FSEL15::AltFunc0 + GPFSEL1::FSEL14::AltFunc0);
+
+        // ピン14と15のプルアップ/プルダウンを無効にする
+        #[cfg(feature = "bsp_rpi3")]
+        self.disable_pud_14_15_bcm2837();
+
+        #[cfg(feature = "bsp_rpi4")]
+        self.disable_pud_14_15_bcm2711();
+    }
+}
+
+impl GPIO {
+    /// インスタンスを作成する
+    ///
+    /// # 安全性
+    ///
+    /// - ユーザは正しいMMIO開始アドレスを提供する必要がある
+    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
+        Self {
+            inner: NullLock::new(GPIOInner::new(mmio_start_addr)),
+        }
+    }
+
+    /// `GPIOInner.map_pl011_uart()`の並行処理性安全バージョン
+    pub fn map_pl011_uart(&self) {
+        self.inner.lock(|inner| inner.map_pl011_uart())
+    }
+}
+
+//------------------------------------------------------------------------------
+// OSインタフェースコード
+//------------------------------------------------------------------------------
+use synchronization::interface::Mutex;
+
+impl driver::interface::DeviceDriver for GPIO {
+    fn compatible(&self) -> &'static str {
+        "BCM GPIO"
+    }
+}

diff -uNr 04_safe_globals/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs 05_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
--- 04_safe_globals/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
+++ 05_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
@@ -0,0 +1,409 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! PL011 UARTドライバ
+//!
+//! # 参考資料
+//!
+//! - <https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf>
+//! - <https://developer.arm.com/documentation/ddi0183/latest>
+
+use crate::{
+    bsp::device_driver::common::MMIODerefWrapper, console, cpu, driver, synchronization,
+    synchronization::NullLock,
+};
+use core::fmt;
+use register::{mmio::*, register_bitfields, register_structs};
+
+//--------------------------------------------------------------------------------------------------
+// プライベート定義
+//--------------------------------------------------------------------------------------------------
+
+// PL011 UARTレジスタ
+//
+// 記述は"PrimeCell UART (PL011) Technical Reference Manual" r1p5から採った
+register_bitfields! {
+    u32,
+
+    /// フラグレジスタ
+    FR [
+        /// 送信FIFOが空。このビットの意味は、ラインコントロールレジスタ
+        /// (LCR_H）のFENビットの状態に依存する。
+        ///
+        /// - FIFOが無効(FEN=0)の場合、送信ホールディングレジスタが空の時にこのビットがセットされる。
+        /// - FIFOが有効(FEN=10な場合、送信FIFOが空の時にこのビットがセットされる。
+        /// - このビットは、送信シフトレジスタにデータがあるか否かは示さない。
+        TXFE OFFSET(7) NUMBITS(1) [],
+
+        /// 送信FIFOが満杯。このビットの意味は、ラインコントロールレジスタ
+        /// (LCR_H）のFENビットの状態に依存する。
+        ///
+        /// - FIFOが無効(FEN=0)の場合、送信ホールディングレジスタが満杯の時にこのビットがセットされる。
+        /// - FIFOが有効(FEN=10な場合、送信FIFOが満杯の時にこのビットがセットされる。
+        TXFF OFFSET(5) NUMBITS(1) [],
+
+        /// 受信FIFOが空。このビットの意味は、ラインコントロールレジスタ
+        /// (LCR_H）のFENビットの状態に依存する。
+        ///
+        /// - FIFOが無効(FEN=0)の場合、受信ホールディングレジスタが空の時にこのビットがセットされる。
+        /// - FIFOが有効(FEN=10な場合、受信FIFOが空の時にこのビットがセットされる。
+        RXFE OFFSET(4) NUMBITS(1) [],
+        /// 受信FIFOが満杯。このビットの意味は、ラインコントロールレジスタ
+        /// (LCR_H）のFENビットの状態に依存する。
+        ///
+        /// - FIFOが無効(FEN=0)の場合、受信ホールディングレジスタが満杯の時にこのビットがセットされる。
+        /// - FIFOが有効(FEN=10な場合、受信FIFOが満杯の時にこのビットがセットされる。
+        RXFF OFFSET(6) NUMBITS(1) [],
+
+        /// UARTがビジー。このビットが1にセットされている場合、UARTはデータの
+        /// 送信中でビジーである。バイト送信が完了する（ストップビットを
+        /// すべてのビットがシフトレジスタから送信される）までこのビットは
+        /// セットされ続ける。
+        ///
+        /// 送信FIFOが空でなくなると、UARTが有効か否かにかかわらず、
+        /// 直ちにこのビットはセットされる。
+        BUSY OFFSET(3) NUMBITS(1) []
+    ],
+
+    /// 通信速度整数除数レジスタ
+    IBRD [
+        /// 通信速度除数の整数部分
+        BAUD_DIVINT OFFSET(0) NUMBITS(16) []
+    ],
+
+    /// 通信速度小数除数レジスタ
+    FBRD [
+        ///  通信速度除数の小数部分
+        BAUD_DIVFRAC OFFSET(0) NUMBITS(6) []
+    ],
+
+    /// ラインコントロールレジスタ
+    LCR_H [
+        /// ワード長。このビットは送信または受信する1フレームのデータビット数を
+        /// 示す。
+        WLEN OFFSET(5) NUMBITS(2) [
+            FiveBit = 0b00,
+            SixBit = 0b01,
+            SevenBit = 0b10,
+            EightBit = 0b11
+        ],
+
+        /// FIFOを有効にする
+        ///
+        /// 0 = FIFOは無効（キャラクタモード）。FIFOは1バイトのホールディング
+        /// レジスタになる。
+        ///
+        /// 1 = 送信/受信FIFOバッファは有効（FIFOモード）。
+        FEN  OFFSET(4) NUMBITS(1) [
+            FifosDisabled = 0,
+            FifosEnabled = 1
+        ]
+    ],
+
+    /// コントロールレジスタ
+    CR [
+        /// 受信は有効。このビットが1にセットされている場合、UARTの受信
+        /// セクションは有効である。SIRENビットの設定に応じて、UART信号
+        /// またはSIR信号のいずれかでデータの受信が行われる。受信の途中で
+        /// UARTが無効になった場合は、現在のキャラクタの受信を完了して
+        /// から停止する。
+        RXE OFFSET(9) NUMBITS(1) [
+            Disabled = 0,
+            Enabled = 1
+        ],
+
+        /// 送信は有効。このビットが1にセットされている場合、UARTの送信
+        /// セクションは有効である。SIRENビットの設定に応じて、UART信号
+        /// またはSIR信号のいずれかでデータの送信が行われる。送信の途中で
+        /// UARTが無効になった場合は、現在のキャラクタの送信を完了して
+        /// から停止する。
+        TXE OFFSET(8) NUMBITS(1) [
+            Disabled = 0,
+            Enabled = 1
+        ],
+
+        /// UARTを有効にする
+        ///
+        /// 0 = UARTは無効。 送信または受信の途中でUARTが無効になった場合は、
+        /// 現在のキャラクタの送信または受信を完了してから停止する。
+        ///
+        /// 1 = UARTは有効。SIRENビットの設定に応じて、UART信号またはSIR信号の
+        /// いずれかでデータの送信または受信が行われる。
+        UARTEN OFFSET(0) NUMBITS(1) [
+            /// If the UART is disabled in the middle of transmission or reception, it completes the
+            /// current character before stopping.
+            Disabled = 0,
+            Enabled = 1
+        ]
+    ],
+
+    /// 割り込みクリアレジスタ
+    ICR [
+        /// すべての保留中割り込みを示すメタフィールド
+        ALL OFFSET(0) NUMBITS(11) []
+    ]
+}
+
+register_structs! {
+    #[allow(non_snake_case)]
+    pub RegisterBlock {
+        (0x00 => DR: ReadWrite<u32>),
+        (0x04 => _reserved1),
+        (0x18 => FR: ReadOnly<u32, FR::Register>),
+        (0x1c => _reserved2),
+        (0x24 => IBRD: WriteOnly<u32, IBRD::Register>),
+        (0x28 => FBRD: WriteOnly<u32, FBRD::Register>),
+        (0x2c => LCR_H: WriteOnly<u32, LCR_H::Register>),
+        (0x30 => CR: WriteOnly<u32, CR::Register>),
+        (0x34 => _reserved3),
+        (0x44 => ICR: WriteOnly<u32, ICR::Register>),
+        (0x48 => @END),
+    }
+}
+
+/// 対応するMMIOレジスタのための抽象化
+type Registers = MMIODerefWrapper<RegisterBlock>;
+
+#[derive(PartialEq)]
+enum BlockingMode {
+    Blocking,
+    NonBlocking,
+}
+
+//--------------------------------------------------------------------------------------------------
+// パブリック定義
+//--------------------------------------------------------------------------------------------------
+
+pub struct PL011UartInner {
+    registers: Registers,
+    chars_written: usize,
+    chars_read: usize,
+}
+
+// BSPがpanicハンドラで使用できるように内部構造体をエクスポートする
+pub use PL011UartInner as PanicUart;
+
+/// UARTを表す構造体
+pub struct PL011Uart {
+    inner: NullLock<PL011UartInner>,
+}
+
+//--------------------------------------------------------------------------------------------------
+// パブリックコード
+//--------------------------------------------------------------------------------------------------
+
+impl PL011UartInner {
+    /// インスタンスを作成する
+    ///
+    /// # 安全性
+    ///
+    /// - ユーザは正しいMMIO開始アドレスを提供する必要がある
+    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
+        Self {
+            registers: Registers::new(mmio_start_addr),
+            chars_written: 0,
+            chars_read: 0,
+        }
+    }
+
+    /// 通信速度と特性を設定するSet up baud rate and characteristics.
+    ///
+    /// 8N1で921_600ボーと設定される。
+    ///
+    /// BRDの計算は（config.txtでクロックを48MHzと設定したので）次のようになる。
+    /// `(48_000_000 / 16) / 921_600 = 3.2552083`.
+    ///
+    /// これにより、整数部分は`3`になり、`IBRD`に設定し、
+    /// 小数部分は`0.2552083`となる。
+    ///
+    /// `FBRD`はPL011テクニカルリファレンスマニュアルにしたがって計算すると
+    /// 次のようになる。
+    /// `INTEGER((0.2552083 * 64) + 0.5) = 16`.
+    ///
+    /// したがって、生成される通信速度除数は`3 + 16/64 = 3.25`である。
+    /// これにより生成される通信速は`48_000_000 / (16 * 3.25) = 923_077`となる。
+    ///
+    /// エラー = `((923_077 - 921_600) / 921_600) * 100 = 0.16%`である。
+    pub fn init(&mut self) {
+        // TX FIFOにまだ文字がキューイングされており、UARTハードウェアが
+        // アクティブに送信している時に実行がここに到着する可能性がある。
+        // この場合にUARTがオフになると、キューに入っていた文字が失われる。
+        //
+        // たとえば、実行中にpanic!()が呼び出された時にこのような事態が
+        // 発生する可能性がある。panic!()が自身のUARTインスタンスを初期化して
+        // init()を呼び出すからである。
+        //
+        // そのため、保留中の文字がすべて送信されるように最初にフラッシュする。
+        self.flush();
+
+        // 一時的にUARTを無効にする
+        self.registers.CR.set(0);
+
+        // すべての保留中の割り込みをクリアする
+        self.registers.ICR.write(ICR::ALL::CLEAR);
+
+        // PL011テクニカルリファレンスマニュアルから:
+        //
+        // LCR_H、IBRD、FBRDの各レジスタは、LCR_Hの書き込みにより生成される
+        // 1回の書き込みストローブで更新される30ビット幅のLCRレジスタを形成
+        // する。そのため、IBRDやFBRDの内容を内部的に更新するには、常に
+        // LCR_Hの書き込みを最後に行う必要がある。
+        //
+        // 通信速度と8N1を設定し、FIFOを有効にする。
+        self.registers.IBRD.write(IBRD::BAUD_DIVINT.val(3));
+        self.registers.FBRD.write(FBRD::BAUD_DIVFRAC.val(16));
+        self.registers
+            .LCR_H
+            .write(LCR_H::WLEN::EightBit + LCR_H::FEN::FifosEnabled);
+
+        // UARTを有効にする。
+        self.registers
+            .CR
+            .write(CR::UARTEN::Enabled + CR::TXE::Enabled + CR::RXE::Enabled);
+    }
+
+    /// 1文字送信する
+    fn write_char(&mut self, c: char) {
+        // スロットが開くのを待って、TX FIFOフルが設定されている間、スピンする。
+        while self.registers.FR.matches_all(FR::TXFF::SET) {
+            cpu::nop();
+        }
+
+        // 文字をバッファに書き込む
+        self.registers.DR.set(c as u32);
+
+        self.chars_written += 1;
+    }
+
+    /// バッファされた最後の文字が物理的にTXワイヤに置かれるまで実行をブロックする
+    fn flush(&self) {
+        // ビジービットがクリアされるまでスピンする
+        while self.registers.FR.matches_all(FR::BUSY::SET) {
+            cpu::nop();
+        }
+    }
+
+    /// 1文字受信する
+    fn read_char_converting(&mut self, blocking_mode: BlockingMode) -> Option<char> {
+        // RX FIFOがからの場合
+        if self.registers.FR.matches_all(FR::RXFE::SET) {
+            // ノンブロッキングモードの場合はすぐにリターンする
+            if blocking_mode == BlockingMode::NonBlocking {
+                return None;
+            }
+
+            // そうでなければ、1文字受信されるまで待つ
+            while self.registers.FR.matches_all(FR::RXFE::SET) {
+                cpu::nop();
+            }
+        }
+
+        // 1文字読み込む
+        let mut ret = self.registers.DR.get() as u8 as char;
+
+        // 復帰を改行に変換する
+        if ret == '\r' {
+            ret = '\n'
+        }
+
+        // 統計を更新する
+        self.chars_read += 1;
+
+        Some(ret)
+    }
+}
+
+/// `core::fmt::Write`を実装すると`format_args!`マクロが利用可能になる。これはひいては
+/// `カーネル`の`print!`と`println!`マクロを実装することになる。`write_str()`を実装する
+/// ことにより自動的に`write_fmt()`を手にすることができる。
+///
+/// この関数は `&mut self` を取るので、内部構造体を実装する必要がある
+///
+/// [`src/print.rs`]を参照
+///
+/// [`src/print.rs`]: ../../print/index.html
+impl fmt::Write for PL011UartInner {
+    fn write_str(&mut self, s: &str) -> fmt::Result {
+        for c in s.chars() {
+            self.write_char(c);
+        }
+
+        Ok(())
+    }
+}
+
+impl PL011Uart {
+    /// インスタンスを作成する
+    ///
+    /// # Safety
+    ///
+    /// - ユーザは正しいMMIO開始アドレスを提供する必要がある
+    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
+        Self {
+            inner: NullLock::new(PL011UartInner::new(mmio_start_addr)),
+        }
+    }
+}
+
+//------------------------------------------------------------------------------
+// OSインタフェースコード
+//------------------------------------------------------------------------------
+use synchronization::interface::Mutex;
+
+impl driver::interface::DeviceDriver for PL011Uart {
+    fn compatible(&self) -> &'static str {
+        "BCM PL011 UART"
+    }
+
+    unsafe fn init(&self) -> Result<(), &'static str> {
+        self.inner.lock(|inner| inner.init());
+
+        Ok(())
+    }
+}
+
+impl console::interface::Write for PL011Uart {
+    /// `core::fmt::Write`の実装に`args`をそのまま渡すが、ミューテックスで
+    /// ガードしてアクセスをシリアライズしている
+    fn write_char(&self, c: char) {
+        self.inner.lock(|inner| inner.write_char(c));
+    }
+
+    fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
+        // 可読性を高めるために`core::fmt::Write::write:fmt()`の
+        // 呼び出しに完全修飾構文を採用
+        self.inner.lock(|inner| fmt::Write::write_fmt(inner, args))
+    }
+
+    fn flush(&self) {
+        // TX FIFOが空になるまでスピンする
+        self.inner.lock(|inner| inner.flush());
+    }
+}
+
+impl console::interface::Read for PL011Uart {
+    fn read_char(&self) -> char {
+        self.inner
+            .lock(|inner| inner.read_char_converting(BlockingMode::Blocking).unwrap())
+    }
+
+    fn clear_rx(&self) {
+        // 空になるまでRX FIFOを読み込む
+        while self
+            .inner
+            .lock(|inner| inner.read_char_converting(BlockingMode::NonBlocking))
+            .is_some()
+        {}
+    }
+}
+
+impl console::interface::Statistics for PL011Uart {
+    fn chars_written(&self) -> usize {
+        self.inner.lock(|inner| inner.chars_written)
+    }
+
+    fn chars_read(&self) -> usize {
+        self.inner.lock(|inner| inner.chars_read)
+    }
+}

diff -uNr 04_safe_globals/src/bsp/device_driver/bcm.rs 05_drivers_gpio_uart/src/bsp/device_driver/bcm.rs
--- 04_safe_globals/src/bsp/device_driver/bcm.rs
+++ 05_drivers_gpio_uart/src/bsp/device_driver/bcm.rs
@@ -0,0 +1,11 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! BCM ドライバのトップレベル
+
+mod bcm2xxx_gpio;
+mod bcm2xxx_pl011_uart;
+
+pub use bcm2xxx_gpio::*;
+pub use bcm2xxx_pl011_uart::*;

diff -uNr 04_safe_globals/src/bsp/device_driver/common.rs 05_drivers_gpio_uart/src/bsp/device_driver/common.rs
--- 04_safe_globals/src/bsp/device_driver/common.rs
+++ 05_drivers_gpio_uart/src/bsp/device_driver/common.rs
@@ -0,0 +1,38 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! 共通デバイスドライバコード
+
+use core::{marker::PhantomData, ops};
+
+//--------------------------------------------------------------------------------------------------
+// パブリック定義
+//--------------------------------------------------------------------------------------------------
+
+pub struct MMIODerefWrapper<T> {
+    start_addr: usize,
+    phantom: PhantomData<fn() -> T>,
+}
+
+//--------------------------------------------------------------------------------------------------
+// パブリックコード
+//--------------------------------------------------------------------------------------------------
+
+impl<T> MMIODerefWrapper<T> {
+    /// インスタンスを作成する
+    pub const unsafe fn new(start_addr: usize) -> Self {
+        Self {
+            start_addr,
+            phantom: PhantomData,
+        }
+    }
+}
+
+impl<T> ops::Deref for MMIODerefWrapper<T> {
+    type Target = T;
+
+    fn deref(&self) -> &Self::Target {
+        unsafe { &*(self.start_addr as *const _) }
+    }
+}

diff -uNr 04_safe_globals/src/bsp/device_driver.rs 05_drivers_gpio_uart/src/bsp/device_driver.rs
--- 04_safe_globals/src/bsp/device_driver.rs
+++ 05_drivers_gpio_uart/src/bsp/device_driver.rs
@@ -0,0 +1,12 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! デバイスドライバ
+
+#[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi4"))]
+mod bcm;
+mod common;
+
+#[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi4"))]
+pub use bcm::*;

diff -uNr 04_safe_globals/src/bsp/raspberrypi/console.rs 05_drivers_gpio_uart/src/bsp/raspberrypi/console.rs
--- 04_safe_globals/src/bsp/raspberrypi/console.rs
+++ 05_drivers_gpio_uart/src/bsp/raspberrypi/console.rs
@@ -4,113 +4,34 @@

 //! BSPコンソール装置

-use crate::{console, synchronization, synchronization::NullLock};
+use super::memory;
+use crate::{bsp::device_driver, console};
 use core::fmt;

 //--------------------------------------------------------------------------------------------------
-// プライベート定義
-//--------------------------------------------------------------------------------------------------
-
-/// QEMUの出力を無から生成する神秘的で魔法のような装置
-///
-/// mutexで保護される部分.
-struct QEMUOutputInner {
-    chars_written: usize,
-}
-
-//--------------------------------------------------------------------------------------------------
 // パブリックコード
 //--------------------------------------------------------------------------------------------------

-/// メイン構造体
-pub struct QEMUOutput {
-    inner: NullLock<QEMUOutputInner>,
-}
-
-//--------------------------------------------------------------------------------------------------
-// グローバルインスタンス
-//--------------------------------------------------------------------------------------------------
-
-static QEMU_OUTPUT: QEMUOutput = QEMUOutput::new();
-
-//--------------------------------------------------------------------------------------------------
-// プライベートコード
-//--------------------------------------------------------------------------------------------------
-
-impl QEMUOutputInner {
-    const fn new() -> QEMUOutputInner {
-        QEMUOutputInner { chars_written: 0 }
-    }
-
-    /// 1文字送信
-    fn write_char(&mut self, c: char) {
-        unsafe {
-            core::ptr::write_volatile(0x3F20_1000 as *mut u8, c as u8);
-        }
-
-        self.chars_written += 1;
-    }
-}
-
-/// `core::fmt::Write`を実装すると`format_args!`マクロが利用可能になる。これはひいては
-/// `カーネル`の`print!`と`println!`マクロを実装することになる。`write_str()`を実装する
-/// ことにより自動的に`write_fmt()`を手にすることができる。
+/// パニックが発生した場合、パニックハンドラはこの関数を使用してシステムが停止する前に
+/// 何かをプリントするという最後の手段をとる。
 ///
-/// この関数は `&mut self` を取るので、内部構造体を実装する必要がある
+/// GPIOとUARTのパニックバージョンの初期化を試みる。パニックバージョンは同期プリミティブで
+/// 保護されていないため、パニック発生時にカーネルデフォルトのGPIOやUARTインスタンスが
+/// たまたまロックされていても何かをプリントできる可能性は大きい。
 ///
-/// [`src/print.rs`]を参照
+/// # 安全性
 ///
-/// [`src/print.rs`]: ../../print/index.html
-impl fmt::Write for QEMUOutputInner {
-    fn write_str(&mut self, s: &str) -> fmt::Result {
-        for c in s.chars() {
-            // 改行を復帰+改行に変換する
-            if c == '\n' {
-                self.write_char('\r')
-            }
-
-            self.write_char(c);
-        }
-
-        Ok(())
-    }
-}
-
-//--------------------------------------------------------------------------------------------------
-// パブリックコード
-//--------------------------------------------------------------------------------------------------
-
-impl QEMUOutput {
-    /// 新しいインスタンスを作成する
-    pub const fn new() -> QEMUOutput {
-        QEMUOutput {
-            inner: NullLock::new(QEMUOutputInner::new()),
-        }
-    }
+/// - パニック時のプリントにのみ使用する
+pub unsafe fn panic_console_out() -> impl fmt::Write {
+    let mut panic_gpio = device_driver::PanicGPIO::new(memory::map::mmio::GPIO_START);
+    let mut panic_uart = device_driver::PanicUart::new(memory::map::mmio::PL011_UART_START);
+
+    panic_gpio.map_pl011_uart();
+    panic_uart.init();
+    panic_uart
 }

 /// コンソールへの参照を返す
 pub fn console() -> &'static impl console::interface::All {
-    &QEMU_OUTPUT
-}
-
-//------------------------------------------------------------------------------
-// OSインタフェースコード
-//------------------------------------------------------------------------------
-use synchronization::interface::Mutex;
-
-/// `core::fmt::Write`の実装に`args`をそのまま渡すが、ミューテックスで
-/// ガードしてアクセスをシリアライズしている
-impl console::interface::Write for QEMUOutput {
-    fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
-        // 可読性を高めるために`core::fmt::Write::write:fmt()`の
-        // 呼び出しに完全修飾構文を採用
-        self.inner.lock(|inner| fmt::Write::write_fmt(inner, args))
-    }
-}
-
-impl console::interface::Statistics for QEMUOutput {
-    fn chars_written(&self) -> usize {
-        self.inner.lock(|inner| inner.chars_written)
-    }
+    &super::PL011_UART
 }

diff -uNr 04_safe_globals/src/bsp/raspberrypi/driver.rs 05_drivers_gpio_uart/src/bsp/raspberrypi/driver.rs
--- 04_safe_globals/src/bsp/raspberrypi/driver.rs
+++ 05_drivers_gpio_uart/src/bsp/raspberrypi/driver.rs
@@ -0,0 +1,49 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! BSPドライバサポート
+
+use crate::driver;
+
+//--------------------------------------------------------------------------------------------------
+// プライベート定義
+//--------------------------------------------------------------------------------------------------
+
+/// デバイスドライバマネージャ型
+struct BSPDriverManager {
+    device_drivers: [&'static (dyn DeviceDriver + Sync); 2],
+}
+
+//--------------------------------------------------------------------------------------------------
+// グローバルインスンタンス
+//--------------------------------------------------------------------------------------------------
+
+static BSP_DRIVER_MANAGER: BSPDriverManager = BSPDriverManager {
+    device_drivers: [&super::GPIO, &super::PL011_UART],
+};
+
+//--------------------------------------------------------------------------------------------------
+// パブリックコード
+//--------------------------------------------------------------------------------------------------
+
+/// ドライバマネージャへの参照を返す
+pub fn driver_manager() -> &'static impl driver::interface::DriverManager {
+    &BSP_DRIVER_MANAGER
+}
+
+//------------------------------------------------------------------------------
+// OSインタフェースコード
+//------------------------------------------------------------------------------
+use driver::interface::DeviceDriver;
+
+impl driver::interface::DriverManager for BSPDriverManager {
+    fn all_device_drivers(&self) -> &[&'static (dyn DeviceDriver + Sync)] {
+        &self.device_drivers[..]
+    }
+
+    fn post_device_driver_init(&self) {
+        // PL011Uartの出力ピンを構成する
+        super::GPIO.map_pl011_uart();
+    }
+}

diff -uNr 04_safe_globals/src/bsp/raspberrypi/memory.rs 05_drivers_gpio_uart/src/bsp/raspberrypi/memory.rs
--- 04_safe_globals/src/bsp/raspberrypi/memory.rs
+++ 05_drivers_gpio_uart/src/bsp/raspberrypi/memory.rs
@@ -17,6 +17,38 @@
 }

 //--------------------------------------------------------------------------------------------------
+// パブリック定義
+//--------------------------------------------------------------------------------------------------
+
+/// ボードの物理メモリアドレス
+#[rustfmt::skip]
+pub(super) mod map {
+
+    pub const GPIO_OFFSET:         usize = 0x0020_0000;
+    pub const UART_OFFSET:         usize = 0x0020_1000;
+
+    /// 物理デバイス
+    #[cfg(feature = "bsp_rpi3")]
+    pub mod mmio {
+        use super::*;
+
+        pub const START:            usize =         0x3F00_0000;
+        pub const GPIO_START:       usize = START + GPIO_OFFSET;
+        pub const PL011_UART_START: usize = START + UART_OFFSET;
+    }
+
+    /// 物理デバイス
+    #[cfg(feature = "bsp_rpi4")]
+    pub mod mmio {
+        use super::*;
+
+        pub const START:            usize =         0xFE00_0000;
+        pub const GPIO_START:       usize = START + GPIO_OFFSET;
+        pub const PL011_UART_START: usize = START + UART_OFFSET;
+    }
+}
+
+//--------------------------------------------------------------------------------------------------
 // パブリックコード
 //--------------------------------------------------------------------------------------------------


diff -uNr 04_safe_globals/src/bsp/raspberrypi.rs 05_drivers_gpio_uart/src/bsp/raspberrypi.rs
--- 04_safe_globals/src/bsp/raspberrypi.rs
+++ 05_drivers_gpio_uart/src/bsp/raspberrypi.rs
@@ -6,4 +6,33 @@

 pub mod console;
 pub mod cpu;
+pub mod driver;
 pub mod memory;
+
+//--------------------------------------------------------------------------------------------------
+// グローバルインスタンス
+//--------------------------------------------------------------------------------------------------
+use super::device_driver;
+
+static GPIO: device_driver::GPIO =
+    unsafe { device_driver::GPIO::new(memory::map::mmio::GPIO_START) };
+
+static PL011_UART: device_driver::PL011Uart =
+    unsafe { device_driver::PL011Uart::new(memory::map::mmio::PL011_UART_START) };
+
+//--------------------------------------------------------------------------------------------------
+// パブリックコード
+//--------------------------------------------------------------------------------------------------
+
+/// ボード識別
+pub fn board_name() -> &'static str {
+    #[cfg(feature = "bsp_rpi3")]
+    {
+        "Raspberry Pi 3"
+    }
+
+    #[cfg(feature = "bsp_rpi4")]
+    {
+        "Raspberry Pi 4"
+    }
+}

diff -uNr 04_safe_globals/src/bsp.rs 05_drivers_gpio_uart/src/bsp.rs
--- 04_safe_globals/src/bsp.rs
+++ 05_drivers_gpio_uart/src/bsp.rs
@@ -4,6 +4,8 @@

 //! ボードサポートパッケージの条件再エクスポート

+mod device_driver;
+
 #[cfg(any(feature = "bsp_rpi3", feature = "bsp_rpi4"))]
 mod raspberrypi;


diff -uNr 04_safe_globals/src/console.rs 05_drivers_gpio_uart/src/console.rs
--- 04_safe_globals/src/console.rs
+++ 05_drivers_gpio_uart/src/console.rs
@@ -12,20 +12,42 @@
 pub mod interface {
     use core::fmt;

-    /// コンソール write関数
+    /// コンソールwrite関数
     pub trait Write {
+        /// 1文字Write
+        fn write_char(&self, c: char);
+
         /// Rust形式の文字列をWrite
         fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;
+
+        /// バッファされた最後の文字が物理的にTXワイヤに置かれるまでブロックする
+        fn flush(&self);
+    }
+
+    /// コンソールread関数
+    pub trait Read {
+        /// 1文字Read
+        fn read_char(&self) -> char {
+            ' '
+        }
+
+        /// もしあれば、RXバッファをクリアする
+        fn clear_rx(&self);
     }

     /// コンソール統計
     pub trait Statistics {
-        /// 書き込んだ文字数を返す
+        /// 書き出した文字数を返す
         fn chars_written(&self) -> usize {
             0
         }
+
+        /// 読み込んだ文字数を返す
+        fn chars_read(&self) -> usize {
+            0
+        }
     }

     /// 本格的コンソール用のトレイトエイリアス
-    pub trait All = Write + Statistics;
+    pub trait All = Write + Read + Statistics;
 }

diff -uNr 04_safe_globals/src/cpu.rs 05_drivers_gpio_uart/src/cpu.rs
--- 04_safe_globals/src/cpu.rs
+++ 05_drivers_gpio_uart/src/cpu.rs
@@ -13,4 +13,7 @@
 //--------------------------------------------------------------------------------------------------
 // アーキテクチャのパブリック再エクスポート
 //--------------------------------------------------------------------------------------------------
-pub use arch_cpu::wait_forever;
+pub use arch_cpu::{nop, wait_forever};
+
+#[cfg(feature = "bsp_rpi3")]
+pub use arch_cpu::spin_for_cycles;

diff -uNr 04_safe_globals/src/driver.rs 05_drivers_gpio_uart/src/driver.rs
--- 04_safe_globals/src/driver.rs
+++ 05_drivers_gpio_uart/src/driver.rs
@@ -0,0 +1,44 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
+
+//! ドライバサポート
+
+//--------------------------------------------------------------------------------------------------
+// パブリック定義
+//--------------------------------------------------------------------------------------------------
+
+/// ドライバインタフェース.
+pub mod interface {
+    /// デバイスドライバ関数
+    pub trait DeviceDriver {
+        /// ドライバを識別するための互換性文字列を返す
+        fn compatible(&self) -> &'static str;
+
+        /// デバイスを起動するためにカーネルから呼び出される
+        ///
+        /// # 安全性
+        ///
+        /// - initの間にドライバがシステム全体に影響を与えることをする可能性がある
+        unsafe fn init(&self) -> Result<(), &'static str> {
+            Ok(())
+        }
+    }
+
+    /// デバイスドライバ管理関数
+    ///
+    /// `BSP`はグローバルインスタンスを一つ提供することが想定されている.
+    pub trait DriverManager {
+        /// `BSP`がインスタンス化したすべてのドライバへの参照のスライスを返す
+        ///
+        /// # 安全性
+        ///
+        /// - デバイスの順番はその`DeviceDriver::init()`が呼び出された順番
+        fn all_device_drivers(&self) -> &[&'static (dyn DeviceDriver + Sync)];
+
+        /// ドライバのinit後に実行される初期化コード
+        ///
+        /// たとえば、すでにオンラインになっている他のドライバに依存するデバイスドライバのコード.
+        fn post_device_driver_init(&self);
+    }
+}

diff -uNr 04_safe_globals/src/main.rs 05_drivers_gpio_uart/src/main.rs
--- 04_safe_globals/src/main.rs
+++ 05_drivers_gpio_uart/src/main.rs
@ -106,6 +106,8 @@
 //!
 //! [`runtime_init::runtime_init()`]: runtime_init/fn.runtime_init.html

+#![allow(clippy::upper_case_acronyms)]
+#![feature(const_fn_fn_ptr_basics)]
 #![feature(format_args_nl)]
 #![feature(global_asm)]
 #![feature(panic_info_message)]
@@ -116,6 +118,7 @@
 mod bsp;
 mod console;
 mod cpu;
+mod driver;
 mod memory;
 mod panic_wait;
 mod print;
@@ -127,16 +130,54 @@
 /// # 安全性
 ///
 /// - アクティブなコアはこの関数を実行しているコアだけでなければならない
+/// - この関数内のinitコールは正しい順番でなければならない
 unsafe fn kernel_init() -> ! {
-    use console::interface::Statistics;
+    use driver::interface::DriverManager;

-    println!("[0] Hello from Rust!");
+    for i in bsp::driver::driver_manager().all_device_drivers().iter() {
+        if let Err(x) = i.init() {
+            panic!("Error loading driver: {}: {}", i.compatible(), x);
+        }
+    }
+    bsp::driver::driver_manager().post_device_driver_init();
+    // println!はここから利用可能
+
+    // unsafeからsafeに移行
+    kernel_main()
+}
+
+/// 最初の初期化後に実行するメイン関数
+fn kernel_main() -> ! {
+    use bsp::console::console;
+    use console::interface::All;
+    use driver::interface::DriverManager;
+
+    println!(
+        "[0] {} version {}",
+        env!("CARGO_PKG_NAME"),
+        env!("CARGO_PKG_VERSION")
+    );
+    println!("[1] Booting on: {}", bsp::board_name());
+
+    println!("[2] Drivers loaded:");
+    for (i, driver) in bsp::driver::driver_manager()
+        .all_device_drivers()
+        .iter()
+        .enumerate()
+    {
+        println!("      {}. {}", i + 1, driver.compatible());
+    }

     println!(
-        "[1] Chars written: {}",
+        "[3] Chars written: {}",
         bsp::console::console().chars_written()
     );
+    println!("[4] Echoing input now");

-    println!("[2] Stopping here.");
-    cpu::wait_forever()
+    // エコーモードに移行する前に受信したスプリアス文字を破棄する
+    console().clear_rx();
+    loop {
+        let c = bsp::console::console().read_char();
+        bsp::console::console().write_char(c);
+    }
 }

diff -uNr 04_safe_globals/src/panic_wait.rs 05_drivers_gpio_uart/src/panic_wait.rs
--- 04_safe_globals/src/panic_wait.rs
+++ 05_drivers_gpio_uart/src/panic_wait.rs
@@ -2,17 +2,37 @@
 //
 // Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

-//! 永久に待ち続けるパニックハンドラ
+//! A panic handler that infinitely waits.

-use crate::{cpu, println};
-use core::panic::PanicInfo;
+use crate::{bsp, cpu};
+use core::{fmt, panic::PanicInfo};
+
+//--------------------------------------------------------------------------------------------------
+// Private Code
+//--------------------------------------------------------------------------------------------------
+
+fn _panic_print(args: fmt::Arguments) {
+    use fmt::Write;
+
+    unsafe { bsp::console::panic_console_out().write_fmt(args).unwrap() };
+}
+
+/// Prints with a newline - only use from the panic handler.
+///
+/// Carbon copy from <https://doc.rust-lang.org/src/std/macros.rs.html>
+#[macro_export]
+macro_rules! panic_println {
+    ($($arg:tt)*) => ({
+        _panic_print(format_args_nl!($($arg)*));
+    })
+}

 #[panic_handler]
 fn panic(info: &PanicInfo) -> ! {
     if let Some(args) = info.message() {
-        println!("\nKernel panic: {}", args);
+        panic_println!("\nKernel panic: {}", args);
     } else {
-        println!("\nKernel panic!");
+        panic_println!("\nKernel panic!");
     }

     cpu::wait_forever()
```
