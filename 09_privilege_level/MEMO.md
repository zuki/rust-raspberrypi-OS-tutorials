# EL2関連レジスタまとめ

| レジスタ名 | 説明 | レジスタ英語名 |
|:-----------|:-----|:---------------|
| CNTHCTL_EL2 | EL1カウンタレジスタを有効化 | Counter-timer Hypervisor Control register |
| CNTVOFF_EL2 | カウンタ値を読み込む際のオフセット値を設定 | Counter-timer Virtual Offset register |
| HCR_EL2 | EL1の実行モードを設定 | Hypervisor Configuration Register |
| SPSR_EL2 | 割り込みの無効化とSP_EL1をSPに指定 | Saved Program Status Register (EL2) |
| ELR_EL2 | 例外発生時の復帰アドレスを保持 | Exception Link Register (EL2) |
| SP_EL1 | EL1のSPを保持 | Stack Pointer (EL1) |
| CurrentEL | 現在の例外レベルを保持 | Current Exception Level |

# ディレクトリ構成

```
$ tree ./src
./src
├── _arch
│   └── aarch64
│       ├── cpu
│       │   ├── boot.rs                     # 変更
│       │   └── boot.s                      # 変更
│       ├── cpu.rs
│       ├── exception
│       │   └── asynchronous.rs             # 変更
│       ├── exception.rs                    # 変更
│       └── time.rs
├── bsp
│   ├── device_driver
│   │   ├── bcm
│   │   │   ├── bcm2xxx_gpio.rs
│   │   │   └── bcm2xxx_pl011_uart.rs
│   │   ├── bcm.rs
│   │   └── common.rs
│   ├── device_driver.rs
│   ├── raspberrypi
│   │   ├── console.rs
│   │   ├── cpu.rs
│   │   ├── driver.rs
│   │   ├── link.ld
│   │   └── memory.rs
│   └── raspberrypi.rs
├── bsp.rs
├── console.rs
├── cpu
│   └── boot.rs
├── cpu.rs
├── driver.rs
├── exception
│   └── asynchronous.rs                     # 変更
├── exception.rs                            # 変更
├── main.rs                                 # 変更
├── memory.rs
├── panic_wait.rs
├── print.rs
├── runtime_init.rs
├── synchronization.rs
└── time.rs
```

# 実機での実行

```
$ make

Compiling kernel - rpi3
   Compiling tock-registers v0.6.0
   Compiling mingo v0.9.0 (/Users/dspace/raspi_os/rust_raspi_os/09_privilege_level)
   Compiling register v1.0.2
   Compiling cortex-a v5.1.6
    Finished release [optimized] target(s) in 4.16s

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
[MP] ⏩ Pushing 3 KiB ==========🦀                                  25% 0 KiB/s
[MP] ⏩ Pushing 5 KiB ===============🦀                             36% 0 KiB/s
[MP] ⏩ Pushing 7 KiB =====================🦀                       50% 0 KiB/s
[MP] ⏩ Pushing 8 KiB ==========================🦀                  61% 0 KiB/s
[MP] ⏩ Pushing 10 KiB ===============================🦀            75% 0 KiB/s
[MP] ⏩ Pushing 12 KiB ====================================🦀       86% 0 KiB/s
[MP] ⏩ Pushing 13 KiB =========================================🦀 100% 0 KiB/s Time: 00:00:00
[ML] Loaded! Executing the payload now

[    0.191967] mingo version 0.9.0
[    0.192166] Booting on: Raspberry Pi 3
[    0.192622] Current privilege level: EL1
[    0.193098] Exception handling state:
[    0.193542]       Debug:  Masked
[    0.193932]       SError: Masked
[    0.194322]       IRQ:    Masked
[    0.194712]       FIQ:    Masked
[    0.195102] Architectural timer resolution: 52 ns
[    0.195676] Drivers loaded:
[    0.196012]       1. BCM GPIO
[    0.196370]       2. BCM PL011 UART
[    0.196792] Timer test, spinning for 1 second
[    1.197324] Echoing input now
ab
[MP] Bye 👋
```
