# コードリーディング

## トレイとの定義と実装

- src/driver.rs
    - interface::DeviceDriverトレイトの定義
    - interface::DriverManagerトレイの定義
- src/bsp/device_driver.rs
  - pub mod bcm, pub mod commonのモジュール定義
  - bcm::*の再エクスポート
- src/bps/device_driver/common.rs
  - pub struct MMIODerefWrapper<T>の定義と実装
- src/bsp/device_driver/bcm.rs
  - mod bcm2xxx_gpio, mod bcm2xxx_pl011_uartのモジュール定義
  - bcm2xxx_gpio::*, bcm2xxx_pl011_uart::*の再エクスポート
- src/bsp/device_driver/bcm/bcm2xxx_gpio.rs
  - pub struct GPIOInner, pub struct GPIOの定義と実装
  - pub use GPIOInner as PanicGPIOの最エクスポート
  - interface::DeviceDriverの実装
- src/bsp/device_driver/bcm/bcm2xxx_pl011_uart.rs
  - pub struct PL011UartInner, pub struct PL011Uartの定義と実装
  - pub use PL011UartInner as PanicUartの最エクスポート
  - interface::DeviceDriverの実装
- src/bsp/raspberrypi.rs
  - pub mod driverのモジュール定義
  - static GPIO: device_driver::GPIO, static PL011_UART: device_driver::PL011Uartインスタンスの定義
- src/bsp/raspberrypi/driver.rs
  - struct BSPDriverManagerの定義と実装
  - interface::DriverManagerの実装

# ラズパイのGPIO関係レジスタ

- レジスタ一覧
  - RPi3 (BCM2837): 6.1 Register View (p.90)
  - RPi4 (BCM2711): 5.2. Register View (p.65)
- 各ピンの機能選択
  - RPi3: 6.2 Alternative Function Assignments  (p.102)
  - RPi4: 5.3. Alternative Function Assignments (p.76)
- GPFSEL[0-5]: GPIOピン0-53の機能を選択する。各ピンの設定はFSELnnを使用する
- プルアップ/プルダウンの設定(RPi3)
  1. GPPUDに必要な制御信号を書き込む
  2. 150サイクル待機する
  3. GPPUDCLK0/1に変更が必要なピンに該当する制御信号を書き込む（不要なピンは触らない）
  4. 150サイクル待機する
  5. GPPUD に制御信号を削除するよう書き込む
  6. Write to GPPUDCLK0/1にクロックを削除するよう書き込む

## UART関係レジスタ

- PL011: Table 3-1 lists the UART registers (p.3-3)

# ディレクトリ構成

```bash
├── _arch
│   └── aarch64
│       ├── cpu
│       │   ├── boot.rs
│       │   └── boot.s
│       └── cpu.rs                          # 変更
├── bsp
│   ├── device_driver
│   │   ├── bcm
│   │   │   ├── bcm2xxx_gpio.rs             # 新規追加
│   │   │   └── bcm2xxx_pl011_uart.rs       # 新規追加
│   │   ├── bcm.rs                          # 新規追加
│   │   └── common.rs                       # 新規追加
│   ├── device_driver.rs                    # 新規追加
│   ├── raspberrypi
│   │   ├── console.rs                      # 変更
│   │   ├── cpu.rs
│   │   ├── driver.rs                       # 新規追加
│   │   ├── link.ld
│   │   └── memory.rs                       # 変更
│   └── raspberrypi.rs                      # 変更
├── bsp.rs                                  # 変更
├── console.rs                              # 変更
├── cpu
│   └── boot.rs
├── cpu.rs                                  # 変更
├── driver.rs                               # 新規追加
├── main.rs                                 # 変更
├── memory.rs
├── panic_wait.rs                           # 変更
├── print.rs
├── runtime_init.rs
└── synchronization.rs
```

# 実行

```bash
$ make qemu

Compiling kernel - rpi3
    Finished release [optimized] target(s) in 0.00s

Launching QEMU
[0] mingo version 0.5.0
[1] Booting on: Raspberry Pi 3
[2] Drivers loaded:
      1. BCM GPIO
      2. BCM PL011 UART
[3] Chars written: 117
[4] Echoing input now
hello raspi
qemu-system-aarch64: terminating on signal 2
```
