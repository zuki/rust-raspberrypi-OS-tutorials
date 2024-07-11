# タイマー関係レジストまとめ

| レジスタ名 | 説明 | レジスタ英語名 |
|:-----------|:-----|:---------------|
| CNTPCT_EL0 | 64ビットの物理的カウント値を保持 | Counter-timer Physical Count Register |
| CNTFRQ_EL0 | システムカウンタの周波数を保持。システム初期化時に設定。  | Counter-timer Freqeu |
| CNTP_TVAL_EL0 | EL1物理タイマ用のタイマ値を保持 | Counter-teimer Physical Timer |
| CNTP_CTL_EL0 | EL1物理タイマ用の制御レジスタ | Counter-timer Physical Timer Control register |

# トレイト `time::interface::TimeManager`

```
pub trait TimeManager {
    fn resolution(&self) -> Duration;           // タイマーの分解能
    fn uptime(&self) -> Duration;               // デバイスの電源オンからの経過時間
    fn spin_for(&self, duration: Duration);     // 指定期間スピン
}
```

# ディレクトリ構成

```
$ tree ./src
./src
├── _arch
│   └── aarch64
│       ├── cpu
│       │   ├── boot.rs
│       │   └── boot.s                      # 変更
│       ├── cpu.rs                          # 変更
│       └── time.rs                         # 変更
├── bsp
│   ├── device_driver
│   │   ├── bcm
│   │   │   ├── bcm2xxx_gpio.rs             # 変更
│   │   │   └── bcm2xxx_pl011_uart.rs       # 変更
│   │   ├── bcm.rs
│   │   └── common.rs
│   ├── device_driver.rs
│   ├── raspberrypi
│   │   ├── console.rs
│   │   ├── cpu.rs
│   │   ├── driver.rs
│   │   ├── link.ld                         # 変更
│   │   └── memory.rs                       # 変更
│   └── raspberrypi.rs
├── bsp.rs
├── console.rs
├── cpu
│   └── boot.rs
├── cpu.rs                                  # 変更
├── driver.rs
├── main.rs                                 # 変更
├── memory.rs
├── panic_wait.rs
├── print.rs                                # 変更
├── runtime_init.rs
├── synchronization.rs
└── time.rs                                 # 変更
```

# 実機による実行結果

- メッセージに経過時間（タイムスタンプ）が表示
- 遅延関数がより正確になった

```
$ make

Compiling kernel - rpi3
   Compiling tock-registers v0.6.0
   Compiling mingo v0.7.0 (/Users/dspace/raspi_os/rust_raspi_os/07_timestamps)
   Compiling register v1.0.2
   Compiling cortex-a v5.1.6
    Finished release [optimized] target(s) in 3.32s

$ make chainboot

Compiling kernel - rpi3
    Finished release [optimized] target(s) in 0.00s

Minipush 1.0

[MP] ✅ Serial connected
[MP] 🔌 Please power the target now

 __  __ _      _ _                 _
|  \/  (_)_ _ (_) |   ___  __ _ __| |
| |\/| | | ' \| | |__/ _ \/ _` / _` |
|_|  |_|_|_||_|_|____\___/\__,_\__,_|

           Raspberry Pi 3

[ML] Requesting binary
[MP] ⏩ Pushing 0 KiB 🦀                                             0% 0 KiB/s
[MP] ⏩ Pushing 3 KiB ============🦀                                29% 0 KiB/s
[MP] ⏩ Pushing 5 KiB ==================🦀                          42% 0 KiB/s
[MP] ⏩ Pushing 6 KiB =======================🦀                     55% 0 KiB/s
[MP] ⏩ Pushing 8 KiB ==============================🦀              71% 0 KiB/s
[MP] ⏩ Pushing 10 KiB ===================================🦀        84% 0 KiB/s
[MP] ⏩ Pushing 11 KiB =========================================🦀 100% 0 KiB/s Time: 00:00:00
[ML] Loaded! Executing the payload now

[    0.167500] mingo version 0.7.0
[    0.167699] Booting on: Raspberry Pi 3
[    0.168154] Architectural timer resolution: 52 ns
[    0.168728] Drivers loaded:
[    0.169064]       1. BCM GPIO
[    0.169421]       2. BCM PL011 UART
[W   0.169845] Spin duration smaller than architecturally supported, skipping
[    0.170690] Spinning for 1 second
[    1.171092] Spinning for 1 second
[    2.171315] Spinning for 1 second

[MP] Bye 👋
```
