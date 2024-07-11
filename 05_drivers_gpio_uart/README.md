# Tutorial 05 - Drivers: GPIO and UART

## tl;dr

- Drivers for the real `UART` and the `GPIO` controller are added.
- **For the first time, we will be able to run the code on the real hardware** (scroll down for
  instructions).

## Introduction

Now that we enabled safe globals in the previous tutorial, the infrastructure is laid for adding the
first real device drivers. We throw out the magic QEMU console and introduce a `driver manager`,
which allows the `BSP` to register device drivers with the `kernel`.

## Driver Manager

The first step consists of adding a `driver subsystem` to the kernel. The corresponding code will
live in `src/driver.rs`. The subsystem introduces `interface::DeviceDriver`, a common trait that
every device driver will need to implement and that is known to the kernel. A global
`DRIVER_MANAGER` instance (of type `DriverManager`) that is instantiated in the same file serves as
the central entity that can be called to manage all things device drivers in the kernel. For
example, by using the globally accessible `crate::driver::driver_manager().register_driver(...)`,
any code can can register an object with static lifetime that implements the
`interface::DeviceDriver` trait.

During kernel init, a call to `crate::driver::driver_manager().init_drivers(...)` will let the
driver manager loop over all registered drivers and kick off their initialization, and also execute
an optional `post-init callback` that can be registered alongside the driver. For example, this
mechanism is used to switch over to the `UART` driver as the main system console after the `UART`
driver has been initialized.

## BSP Driver Implementation

In `src/bsp/raspberrypi/driver.rs`, the function `init()` takes care of registering the `UART` and
`GPIO` drivers. It is therefore important that during kernel init, the correct order of (i) first
initializing the BSP driver subsystem, and only then (ii) calling the `driver_manager()` is
followed, like the following excerpt from `main.rs` shows:

```rust
unsafe fn kernel_init() -> ! {
    // Initialize the BSP driver subsystem.
    if let Err(x) = bsp::driver::init() {
        panic!("Error initializing BSP driver subsystem: {}", x);
    }

    // Initialize all device drivers.
    driver::driver_manager().init_drivers();
    // println! is usable from here on.
```



The drivers themselves are stored in `src/bsp/device_driver`, and can be reused between `BSP`s. The
first driver added in these tutorials is the `PL011Uart` driver: It implements the
`console::interface::*` traits and is from now on used as the main system console. The second driver
is the `GPIO` driver, which pinmuxes (that is, routing signals from inside the `SoC` to actual HW
pins) the RPi's PL011 UART accordingly. Note how the `GPIO` driver differentiates between **RPi 3**
and **RPi 4**. Their HW is different, so we have to account for it in SW.

The `BSP`s now also contain a memory map in `src/bsp/raspberrypi/memory.rs`. It provides the
Raspberry's `MMIO` addresses which are used by the `BSP` to instantiate the respective device
drivers, so that the driver code knows where to find the device's registers in memory.

## SDカードからのブート

Since we have real `UART` output now, we can run the code on the real hardware. Building is
differentiated between the **RPi 3** and the **RPi 4** due to before mentioned differences in the
`GPIO` driver. By default, all `Makefile` targets will build for the **RPi 3**. In order to build
for the the **RPi 4**, prepend `BSP=rpi4` to each target. For example:

```console
$ BSP=rpi4 make
$ BSP=rpi4 make doc
```

Unfortunately, QEMU does not yet support the **RPi 4**, so `BSP=rpi4 make qemu` won't work.

**Some steps for preparing the SD card differ between RPi 3 and RPi 4, so be careful in the
following.**

### 両者に共通

1. `boot`という名前の単一の`FAT32`パーティションを作成します。
2. カードに次の内容の`config.txt`という名前のファイルを作成します。

```txt
arm_64bit=1
init_uart_clock=48000000
```
### RPi 3

3. [Raspberry Pi firmware repo](https://github.com/raspberrypi/firmware/tree/master/boot) から次のファイルをSDカードにコピーします。
    - [bootcode.bin](https://github.com/raspberrypi/firmware/raw/master/boot/bootcode.bin)
    - [fixup.dat](https://github.com/raspberrypi/firmware/raw/master/boot/fixup.dat)
    - [start.elf](https://github.com/raspberrypi/firmware/raw/master/boot/start.elf)
4. `make`を実行します。

### RPi 4

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
    - **注**: TX (transmit) は RX (receive) ピンに接続してください.
    - USBシリアルの電源ピンは接続**しない**でください。RX/TXとGNDだけを接続します。
8. RPiを(USB)電源ケーブルに接続し、出力を観察します。

```console
Miniterm 1.0

[MT] ⏳ Waiting for /dev/ttyUSB0
[MT] ✅ Serial connected
[0] mingo version 0.5.0
[1] Booting on: Raspberry Pi 3
[2] Drivers loaded:
      1. BCM PL011 UART
      2. BCM GPIO
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
 edition = "2021"

@@ -9,8 +9,8 @@

 [features]
 default = []
-bsp_rpi3 = []
-bsp_rpi4 = []
+bsp_rpi3 = ["tock-registers"]
+bsp_rpi4 = ["tock-registers"]

 [[bin]]
 name = "kernel"
@@ -22,6 +22,9 @@

 [dependencies]

+# Optional dependencies
+tock-registers = { version = "0.8.x", default-features = false, features = ["register_types"], optional = true }
+
 # Platform specific dependencies
 [target.'cfg(target_arch = "aarch64")'.dependencies]
 aarch64-cpu = { version = "9.x.x" }

diff -uNr 04_safe_globals/Makefile 05_drivers_gpio_uart/Makefile
--- 04_safe_globals/Makefile
+++ 05_drivers_gpio_uart/Makefile
@@ -13,6 +13,9 @@
 # Default to the RPi3.
 BSP ?= rpi3

+# Default to a serial device name that is common in Linux.
+DEV_SERIAL ?= /dev/ttyUSB0
+


 ##--------------------------------------------------------------------------------------------------
@@ -88,6 +91,7 @@

 EXEC_QEMU          = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
 EXEC_TEST_DISPATCH = ruby ../common/tests/dispatch.rb
+EXEC_MINITERM      = ruby ../common/serial/miniterm.rb

 ##------------------------------------------------------------------------------
 ## Dockerization
@@ -95,18 +99,26 @@
 DOCKER_CMD            = docker run -t --rm -v $(shell pwd):/work/tutorial -w /work/tutorial
 DOCKER_CMD_INTERACT   = $(DOCKER_CMD) -i
 DOCKER_ARG_DIR_COMMON = -v $(shell pwd)/../common:/work/common
+DOCKER_ARG_DEV        = --privileged -v /dev:/dev

 # DOCKER_IMAGE defined in include file (see top of this file).
 DOCKER_QEMU  = $(DOCKER_CMD_INTERACT) $(DOCKER_IMAGE)
 DOCKER_TOOLS = $(DOCKER_CMD) $(DOCKER_IMAGE)
 DOCKER_TEST  = $(DOCKER_CMD) $(DOCKER_ARG_DIR_COMMON) $(DOCKER_IMAGE)

+# Dockerize commands, which require USB device passthrough, only on Linux.
+ifeq ($(shell uname -s),Linux)
+    DOCKER_CMD_DEV = $(DOCKER_CMD_INTERACT) $(DOCKER_ARG_DEV)
+
+    DOCKER_MINITERM = $(DOCKER_CMD_DEV) $(DOCKER_ARG_DIR_COMMON) $(DOCKER_IMAGE)
+endif
+


 ##--------------------------------------------------------------------------------------------------
 ## Targets
 ##--------------------------------------------------------------------------------------------------
-.PHONY: all doc qemu clippy clean readelf objdump nm check
+.PHONY: all doc qemu miniterm clippy clean readelf objdump nm check

 all: $(KERNEL_BIN)

@@ -156,9 +168,16 @@
 qemu: $(KERNEL_BIN)
 	$(call color_header, "Launching QEMU")
 	@$(DOCKER_QEMU) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN)
+
 endif

 ##------------------------------------------------------------------------------
+## Connect to the target's serial
+##------------------------------------------------------------------------------
+miniterm:
+	@$(DOCKER_MINITERM) $(EXEC_MINITERM) $(DEV_SERIAL)
+
+##------------------------------------------------------------------------------
 ## Run clippy
 ##------------------------------------------------------------------------------
 clippy:

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
@@ -0,0 +1,228 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
+
+//! GPIOドライバ
+
+use crate::{
+    bsp::device_driver::common::MMIODerefWrapper, driver, synchronization,
+    synchronization::NullLock,
+};
+use tock_registers::{
+    interfaces::{ReadWriteable, Writeable},
+    register_bitfields, register_structs,
+    registers::ReadWrite,
+};
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
+struct GPIOInner {
+    registers: Registers,
+}
+
+//--------------------------------------------------------------------------------------------------
+// パブリック定義
+//--------------------------------------------------------------------------------------------------
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
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+impl GPIO {
+    pub const COMPATIBLE: &'static str = "BCM GPIO";
+
+    /// インスタンスを作成する.
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
+        Self::COMPATIBLE
+    }
+}

diff -uNr 04_safe_globals/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs 05_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
--- 04_safe_globals/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
+++ 05_drivers_gpio_uart/src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
@@ -0,0 +1,407 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
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
+use tock_registers::{
+    interfaces::{Readable, Writeable},
+    register_bitfields, register_structs,
+    registers::{ReadOnly, ReadWrite, WriteOnly},
+};
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
+        #[allow(clippy::enum_variant_names)]
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
+struct PL011UartInner {
+    registers: Registers,
+    chars_written: usize,
+    chars_read: usize,
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// UARTを表す構造体
+pub struct PL011Uart {
+    inner: NullLock<PL011UartInner>,
+}
+
+//--------------------------------------------------------------------------------------------------
+// Private Code
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
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+impl PL011Uart {
+    pub const COMPATIBLE: &'static str = "BCM PL011 UART";
+
+    /// インスタンスを作成する.
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
+        Self::COMPATIBLE
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
+
+impl console::interface::All for PL011Uart {}

diff -uNr 04_safe_globals/src/bsp/device_driver/bcm.rs 05_drivers_gpio_uart/src/bsp/device_driver/bcm.rs
--- 04_safe_globals/src/bsp/device_driver/bcm.rs
+++ 05_drivers_gpio_uart/src/bsp/device_driver/bcm.rs
@@ -0,0 +1,11 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
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
+// Copyright (c) 2020-2023 Andre Richter <andre.o.richter@gmail.com>
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
+// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
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
@@ -4,115 +4,13 @@

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
+use crate::console;

 //--------------------------------------------------------------------------------------------------
 // パブリックコード
 //--------------------------------------------------------------------------------------------------

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
+    &super::driver::PL011_UART
 }
-
-impl console::interface::All for QEMUOutput {}

diff -uNr 04_safe_globals/src/bsp/raspberrypi/driver.rs 05_drivers_gpio_uart/src/bsp/raspberrypi/driver.rs
--- 04_safe_globals/src/bsp/raspberrypi/driver.rs
+++ 05_drivers_gpio_uart/src/bsp/raspberrypi/driver.rs
@@ -0,0 +1,71 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
+
+//! BSPドライバサポート
+
+use super::memory::map::mmio;
+use crate::{bsp::device_driver, console, driver as generic_driver};
+use core::sync::atomic::{AtomicBool, Ordering};
+
+//--------------------------------------------------------------------------------------------------
+// グローバルインスンタンス
+//--------------------------------------------------------------------------------------------------
+
+static PL011_UART: device_driver::PL011Uart =
+    unsafe { device_driver::PL011Uart::new(mmio::PL011_UART_START) };
+static GPIO: device_driver::GPIO = unsafe { device_driver::GPIO::new(mmio::GPIO_START) };
+
+//--------------------------------------------------------------------------------------------------
+// Private Code
+//--------------------------------------------------------------------------------------------------
+
+/// This must be called only after successful init of the UART driver.
+fn post_init_uart() -> Result<(), &'static str> {
+    console::register_console(&PL011_UART);
+
+    Ok(())
+}
+
+/// This must be called only after successful init of the GPIO driver.
+fn post_init_gpio() -> Result<(), &'static str> {
+    GPIO.map_pl011_uart();
+    Ok(())
+}
+
+fn driver_uart() -> Result<(), &'static str> {
+    let uart_descriptor =
+        generic_driver::DeviceDriverDescriptor::new(&PL011_UART, Some(post_init_uart));
+    generic_driver::driver_manager().register_driver(uart_descriptor);
+
+    Ok(())
+}
+
+fn driver_gpio() -> Result<(), &'static str> {
+    let gpio_descriptor = generic_driver::DeviceDriverDescriptor::new(&GPIO, Some(post_init_gpio));
+    generic_driver::driver_manager().register_driver(gpio_descriptor);
+
+    Ok(())
+}
+
+//--------------------------------------------------------------------------------------------------
+// パブリックコード
+//--------------------------------------------------------------------------------------------------
+
+/// Initialize the driver subsystem.
+///
+/// # Safety
+///
+/// See child function calls.
+pub unsafe fn init() -> Result<(), &'static str> {
+    static INIT_DONE: AtomicBool = AtomicBool::new(false);
+    if INIT_DONE.load(Ordering::Relaxed) {
+        return Err("Init already done");
+    }
+
+    driver_uart()?;
+    driver_gpio()?;
+
+    INIT_DONE.store(true, Ordering::Relaxed);
+    Ok(())
+}

diff -uNr 04_safe_globals/src/bsp/raspberrypi/memory.rs 05_drivers_gpio_uart/src/bsp/raspberrypi/memory.rs
--- 04_safe_globals/src/bsp/raspberrypi/memory.rs
+++ 05_drivers_gpio_uart/src/bsp/raspberrypi/memory.rs
@@ -0,0 +1,37 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
+
+//! BSP Memory Management.
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
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

diff -uNr 04_safe_globals/src/bsp/raspberrypi.rs 05_drivers_gpio_uart/src/bsp/raspberrypi.rs
--- 04_safe_globals/src/bsp/raspberrypi.rs
+++ 05_drivers_gpio_uart/src/bsp/raspberrypi.rs
@@ -4,5 +4,23 @@

 //! Top-level BSP file for the Raspberry Pi 3 and 4.

-pub mod console;
 pub mod cpu;
+pub mod driver;
+pub mod memory;
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


diff -uNr 04_safe_globals/src/console/null_console.rs 05_drivers_gpio_uart/src/console/null_console.rs
--- 04_safe_globals/src/console/null_console.rs
+++ 05_drivers_gpio_uart/src/console/null_console.rs
@@ -0,0 +1,41 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2022-2023 Andre Richter <andre.o.richter@gmail.com>
+
+//! Null console.
+
+use super::interface;
+use core::fmt;
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+pub struct NullConsole;
+
+//--------------------------------------------------------------------------------------------------
+// Global instances
+//--------------------------------------------------------------------------------------------------
+
+pub static NULL_CONSOLE: NullConsole = NullConsole {};
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+impl interface::Write for NullConsole {
+    fn write_char(&self, _c: char) {}
+
+    fn write_fmt(&self, _args: fmt::Arguments) -> fmt::Result {
+        fmt::Result::Ok(())
+    }
+
+    fn flush(&self) {}
+}
+
+impl interface::Read for NullConsole {
+    fn clear_rx(&self) {}
+}
+
+impl interface::Statistics for NullConsole {}
+impl interface::All for NullConsole {}

diff -uNr 04_safe_globals/src/console.rs 05_drivers_gpio_uart/src/console.rs
--- 04_safe_globals/src/console.rs
+++ 05_drivers_gpio_uart/src/console.rs
@@ -4,7 +4,9 @@

 //! System console.

-use crate::bsp;
+mod null_console;
+
+use crate::synchronization::{self, NullLock};

 //--------------------------------------------------------------------------------------------------
 // Public Definitions
@@ -16,8 +18,25 @@

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

     /// Console statistics.
@@ -26,19 +45,37 @@
         fn chars_written(&self) -> usize {
             0
         }
+
+        /// 読み込んだ文字数を返す
+        fn chars_read(&self) -> usize {
+            0
+        }
     }

     /// Trait alias for a full-fledged console.
-    pub trait All: Write + Statistics {}
+    pub trait All: Write + Read + Statistics {}
 }

 //--------------------------------------------------------------------------------------------------
+// Global instances
+//--------------------------------------------------------------------------------------------------
+
+static CUR_CONSOLE: NullLock<&'static (dyn interface::All + Sync)> =
+    NullLock::new(&null_console::NULL_CONSOLE);
+
+//--------------------------------------------------------------------------------------------------
 // Public Code
 //--------------------------------------------------------------------------------------------------
+use synchronization::interface::Mutex;
+
+/// Register a new console.
+pub fn register_console(new_console: &'static (dyn interface::All + Sync)) {
+    CUR_CONSOLE.lock(|con| *con = new_console);
+}

-/// Return a reference to the console.
+/// Return a reference to the currently registered console.
 ///
 /// This is the global console used by all printing macros.
 pub fn console() -> &'static dyn interface::All {
-    bsp::console::console()
+    CUR_CONSOLE.lock(|con| *con)
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
@@ -0,0 +1,167 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
+
+//! ドライバサポート
+
+use crate::{
+    println,
+    synchronization::{interface::Mutex, NullLock},
+};
+
+//--------------------------------------------------------------------------------------------------
+// Private Definitions
+//--------------------------------------------------------------------------------------------------
+
+const NUM_DRIVERS: usize = 5;
+
+struct DriverManagerInner {
+    next_index: usize,
+    descriptors: [Option<DeviceDriverDescriptor>; NUM_DRIVERS],
+}
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
+}
+
+/// Tpye to be used as an optional callback after a driver's init() has run.
+pub type DeviceDriverPostInitCallback = unsafe fn() -> Result<(), &'static str>;
+
+/// A descriptor for device drivers.
+#[derive(Copy, Clone)]
+pub struct DeviceDriverDescriptor {
+    device_driver: &'static (dyn interface::DeviceDriver + Sync),
+    post_init_callback: Option<DeviceDriverPostInitCallback>,
+}
+
+/// Provides device driver management functions.
+pub struct DriverManager {
+    inner: NullLock<DriverManagerInner>,
+}
+
+//--------------------------------------------------------------------------------------------------
+// Global instances
+//--------------------------------------------------------------------------------------------------
+
+static DRIVER_MANAGER: DriverManager = DriverManager::new();
+
+//--------------------------------------------------------------------------------------------------
+// Private Code
+//--------------------------------------------------------------------------------------------------
+
+impl DriverManagerInner {
+    /// Create an instance.
+    pub const fn new() -> Self {
+        Self {
+            next_index: 0,
+            descriptors: [None; NUM_DRIVERS],
+        }
+    }
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+impl DeviceDriverDescriptor {
+    /// Create an instance.
+    pub fn new(
+        device_driver: &'static (dyn interface::DeviceDriver + Sync),
+        post_init_callback: Option<DeviceDriverPostInitCallback>,
+    ) -> Self {
+        Self {
+            device_driver,
+            post_init_callback,
+        }
+    }
+}
+
+/// Return a reference to the global DriverManager.
+pub fn driver_manager() -> &'static DriverManager {
+    &DRIVER_MANAGER
+}
+
+impl DriverManager {
+    /// Create an instance.
+    pub const fn new() -> Self {
+        Self {
+            inner: NullLock::new(DriverManagerInner::new()),
+        }
+    }
+
+    /// Register a device driver with the kernel.
+    pub fn register_driver(&self, descriptor: DeviceDriverDescriptor) {
+        self.inner.lock(|inner| {
+            inner.descriptors[inner.next_index] = Some(descriptor);
+            inner.next_index += 1;
+        })
+    }
+
+    /// Helper for iterating over registered drivers.
+    fn for_each_descriptor<'a>(&'a self, f: impl FnMut(&'a DeviceDriverDescriptor)) {
+        self.inner.lock(|inner| {
+            inner
+                .descriptors
+                .iter()
+                .filter_map(|x| x.as_ref())
+                .for_each(f)
+        })
+    }
+
+    /// Fully initialize all drivers.
+    ///
+    /// # Safety
+    ///
+    /// - During init, drivers might do stuff with system-wide impact.
+    pub unsafe fn init_drivers(&self) {
+        self.for_each_descriptor(|descriptor| {
+            // 1. Initialize driver.
+            if let Err(x) = descriptor.device_driver.init() {
+                panic!(
+                    "Error initializing driver: {}: {}",
+                    descriptor.device_driver.compatible(),
+                    x
+                );
+            }
+
+            // 2. Call corresponding post init callback.
+            if let Some(callback) = &descriptor.post_init_callback {
+                if let Err(x) = callback() {
+                    panic!(
+                        "Error during driver post-init callback: {}: {}",
+                        descriptor.device_driver.compatible(),
+                        x
+                    );
+                }
+            }
+        });
+    }
+
+    /// Enumerate all registered device drivers.
+    pub fn enumerate(&self) {
+        let mut i: usize = 1;
+        self.for_each_descriptor(|descriptor| {
+            println!("      {}. {}", i, descriptor.device_driver.compatible());
+
+            i += 1;
+        });
+    }
+}

diff -uNr 04_safe_globals/src/main.rs 05_drivers_gpio_uart/src/main.rs
--- 04_safe_globals/src/main.rs
+++ 05_drivers_gpio_uart/src/main.rs
@@ -106,6 +106,7 @@
 //!     - It is implemented in `src/_arch/__arch_name__/cpu/boot.s`.
 //! 2. Once finished with architectural setup, the arch code calls `kernel_init()`.

+#![allow(clippy::upper_case_acronyms)]
 #![feature(asm_const)]
 #![feature(format_args_nl)]
 #![feature(panic_info_message)]
@@ -116,6 +117,7 @@
 mod bsp;
 mod console;
 mod cpu;
+mod driver;
 mod panic_wait;
 mod print;
 mod synchronization;
@@ -125,13 +127,42 @@
 /// # Safety
 ///
 /// - アクティブなコアはこの関数を実行しているコアだけでなければならない
+/// - この関数内のinitコールは正しい順番でなければならない
 unsafe fn kernel_init() -> ! {
-    use console::console;
+    // Initialize the BSP driver subsystem.
+    if let Err(x) = bsp::driver::init() {
+        panic!("Error initializing BSP driver subsystem: {}", x);
+    }
+
+    // Initialize all device drivers.
+    driver::driver_manager().init_drivers();
+    // println! is usable from here on.

-    println!("[0] Hello from Rust!");
+    // Transition from unsafe to safe.
+    kernel_main()
+}

-    println!("[1] Chars written: {}", console().chars_written());
+/// The main function running after the early init.
+fn kernel_main() -> ! {
+    use console::console;

-    println!("[2] Stopping here.");
-    cpu::wait_forever()
+    println!(
+        "[0] {} version {}",
+        env!("CARGO_PKG_NAME"),
+        env!("CARGO_PKG_VERSION")
+    );
+    println!("[1] Booting on: {}", bsp::board_name());
+
+    println!("[2] Drivers loaded:");
+    driver::driver_manager().enumerate();
+
+    println!("[3] Chars written: {}", console().chars_written());
+    println!("[4] Echoing input now");
+
+    // Discard any spurious received characters before going into echo mode.
+    console().clear_rx();
+    loop {
+        let c = console().read_char();
+        console().write_char(c);
+    }
 }

diff -uNr 04_safe_globals/tests/boot_test_string.rb 05_drivers_gpio_uart/tests/boot_test_string.rb
--- 04_safe_globals/tests/boot_test_string.rb
+++ 05_drivers_gpio_uart/tests/boot_test_string.rb
@@ -1,3 +1,3 @@
 # frozen_string_literal: true

-EXPECTED_PRINT = 'Stopping here'
+EXPECTED_PRINT = 'Echoing input now'

```
