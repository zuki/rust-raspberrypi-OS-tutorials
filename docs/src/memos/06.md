# コードリーディング

## カーネルを読み込む

`main.rs`と`utils/minipush.rb`が通信してkernel8.imgを`kernel_addr=0x80000`に読み込む

| `main.rs` |  方向 | minipush.rb |
|:----------|:-----:|:------------|
| 0x3, 0x3, 0x3 | → |             |
| size      | ←    | カーネルサイズ(uint32_t)   |
| "OK"      | →    |              |
| data      | ←    | [0..size]1バイトずつ送信 |
| [kernel_addr+i] = data | |

```
let kernel: fn() -> ! = unsafe { core::mem::transmute(kernel_addr) }; // 関数ポインタに変換
kernel()                                                              // 実行
```

## レジスタにアドレスを設定する

### PC相対アドレス: 2段階設定

```nasm
adrp    x1, address1                // bits 12-32
add     x1, x1, #:lo12:address1     // bits 0-11
```

### 絶対アドレス: 3段階設定

```nasm
movz	x1, #:abs_g2:address2       // bits 32-47, overflow check
movk	x1, #:abs_g1_nc:address2    // bits 16-31, no overflow check
movk	x1, #:abs_g0_nc:address2    // bits  0-15, no overflow check
```

# ディレクトリ構成

```bash
./src
├── _arch
│   └── aarch64
│       ├── cpu
│       │   ├── boot.rs
│       │   └── boot.s                      # 変更
│       └── cpu.rs
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
│   │   ├── link.ld
│   │   └── memory.rs                       # 変更
│   └── raspberrypi.rs
├── bsp.rs
├── console.rs
├── cpu
│   └── boot.rs
├── cpu.rs
├── driver.rs
├── main.rs                                 # 変更
├── memory.rs
├── panic_wait.rs
├── print.rs
├── runtime_init.rs
└── synchronization.rs
./tests/
└── qemu_minipush.rb                        # 新規作成
./update.sh                                 # 新規作成
```

# make qemuasm

ロードアレスからリンクアドレスへの再配置の様子

```bash
$ make qemuasm
[...]
IN:
0x00080000:  d53800a1  mrs      x1, mpidr_el1
0x00080004:  92400421  and      x1, x1, #3
0x00080008:  58000342  ldr      x2, #0x80070
0x0008000c:  eb02003f  cmp      x1, x2
0x00080010:  54000121  b.ne     #0x80064
----------------
IN:
0x00080014:  90000000  adrp     x0, #0x80000
0x00080018:  91000000  add      x0, x0, #0              ; x0 = 0x80000
0x0008001c:  d2c00001  movz     x1, #0, lsl #32
0x00080020:  f2a04001  movk     x1, #0x200, lsl #16
0x00080024:  f2800001  movk     x1, #0                  ; x1 = 0x2000000
0x00080028:  d2c00002  movz     x2, #0, lsl #32
0x0008002c:  f2a04002  movk     x2, #0x200, lsl #16
0x00080030:  f282ff02  movk     x2, #0x17f8             ; x2 = 0x20017f8
0x00080034:  f8408403  ldr      x3, [x0], #8
0x00080038:  f8008423  str      x3, [x1], #8
0x0008003c:  eb02003f  cmp      x1, x2
0x00080040:  54ffffa3  b.lo     #0x80034
----------------
IN:
0x00080044:  d2c00000  movz     x0, #0, lsl #32
0x00080048:  f2a04000  movk     x0, #0x200, lsl #16
0x0008004c:  f2800000  movk     x0, #0
0x00080050:  9100001f  mov      sp, x0                  ; sp = 0x2000000
0x00080054:  d2c00001  movz     x1, #0, lsl #32
0x00080058:  f2a04001  movk     x1, #0x200, lsl #16
0x0008005c:  f2800f01  movk     x1, #0x78               ; x1 = 0x2000078
0x00080060:  d61f0020  br       x1

----------------
IN:
0x02000078:  94000417  bl       #0x20010d4

----------------
IN:
0x020010d4:  90000008  adrp     x8, #0x2001000
0x020010d8:  90000009  adrp     x9, #0x2001000
0x020010dc:  f943e508  ldr      x8, [x8, #0x7c8]
0x020010e0:  f943e929  ldr      x9, [x9, #0x7d0]
0x020010e4:  eb08013f  cmp      x9, x8
0x020010e8:  54000109  b.ls     #0x2001108
[...]
```

## objdump

```bash
$ aarch64-none-elf-objdump -d -C kernel
Disassembly of section .text:

0000000002000000 <_start>:
 2000000:   d53800a1    mrs x1, mpidr_el1
 2000004:   92400421    and x1, x1, #0x3
 2000008:   58000342    ldr x2, 2000070 <BOOT_CORE_ID>
 200000c:   eb02003f    cmp x1, x2
 2000010:   54000121    b.ne    2000064 <_start+0x64>  // b.any
 2000014:   90000000    adrp    x0, 2000000 <_start>
 2000018:   91000000    add x0, x0, #0x0
 200001c:   d2c00001    movz    x1, #0x0, lsl #32
 2000020:   f2a04001    movk    x1, #0x200, lsl #16
 2000024:   f2800001    movk    x1, #0x0
 2000028:   d2c00002    movz    x2, #0x0, lsl #32
 200002c:   f2a04002    movk    x2, #0x200, lsl #16
 2000030:   f282ff02    movk    x2, #0x17f8
 2000034:   f8408403    ldr x3, [x0], #8
 2000038:   f8008423    str x3, [x1], #8
 200003c:   eb02003f    cmp x1, x2
 2000040:   54ffffa3    b.cc    2000034 <_start+0x34>  // b.lo, b.ul, b.last
 2000044:   d2c00000    movz    x0, #0x0, lsl #32
 2000048:   f2a04000    movk    x0, #0x200, lsl #16
 200004c:   f2800000    movk    x0, #0x0
 2000050:   9100001f    mov sp, x0
 2000054:   d2c00001    movz    x1, #0x0, lsl #32
 2000058:   f2a04001    movk    x1, #0x200, lsl #16
 200005c:   f2800f01    movk    x1, #0x78
 2000060:   d61f0020    br  x1
 2000064:   d503205f    wfe
 2000068:   17ffffff    b   2000064 <_start+0x64>
 200006c:   00000000    .inst   0x00000000 ; undefined

0000000002000070 <BOOT_CORE_ID>:
    ...

0000000002000078 <_start_rust>:
 2000078:   94000417    bl  20010d4 <kernel::runtime_init::runtime_init>
 200007c:   d4200020    brk #0x1

...
00000000020010d4 <kernel::runtime_init::runtime_init>:
 20010d4:   90000008    adrp    x8, 2001000 <kernel::kernel_main+0x388>
 20010d8:   90000009    adrp    x9, 2001000 <kernel::kernel_main+0x388>
 20010dc:   f943e508    ldr x8, [x8, #1992]
 20010e0:   f943e929    ldr x9, [x9, #2000]
 20010e4:   eb08013f    cmp x9, x8
 20010e8:   54000109    b.ls    2001108 <kernel::runtime_init::runtime_init+0x34>  // b.plast
```

# make test

`ruby tests/qemu_minipush.rb qemu-system-aarch64 -M raspi3 -serial stdio -display none -kernel kernel8.img demo_payload_rpi3.img`

## qemu_minipush.rb

```ruby
QEMUMiniPush.new(qemu_cmd, binary_image_path).run

def initialize(qemu_cmd, binary_image_path) do
    super(nil, binary_image_path)
    @qemu_cmd = qemu_cmd
end

def run do
    open_serial                 # override
    wait_for_binary_request
    load_binary
    send_size
    send_binary
    terminal                    # override
end

def open_serial
                    # qemu_cmdの標準入出力との間にパイプラインを確立
    @target_serial = IO.popen(@qemu_cmd, 'r+', err: '/dev/null')
    puts "[#{@name_short}] ✅ Serial connected"
end

def terminal
                            # kernel処理が出力する最後のメッセージを待ち
    result = @target_serial.expect(EXPECTED_PRINT, TIMEOUT_SECS)
    puts result             # メッセージをすべて出力して
    quit_qemu_graceful      # 終了
end
```

```bash
$ make test

Compiling kernel - rpi3
    Finished release [optimized] target(s) in 0.10s

Testing chainloading - rpi3

# QEMUMiniPush.new("qemu-system-aarch64 -M raspi3 -serial stdio -display none -kernel kernel8.img", demo_payload_rpi3.img).run
QEMUMiniPush 1.0

[MP] ✅ Serial connected
[MP] 🔌 Please power the target now

 __  __ _      _ _                 _
|  \/  (_)_ _ (_) |   ___  __ _ __| |
| |\/| | | ' \| | |__/ _ \/ _` / _` |
|_|  |_|_|_||_|_|____\___/\__,_\__,_|

           Raspberry Pi 3

[ML] Requesting binary
                                                                                [MP] ⏩ Pushing 0 KiB 🦀                                             0% 0 KiB/s [MP] ⏩ Pushing 6 KiB ==========================================🦀 100% 0 KiB/s Time: 00:00:00
[ML] Loaded! Executing the payload now

[0] mingo version 0.5.0
[1] Booting on: Raspberry Pi 3
[2] Drivers loaded:
      1. BCM GPIO
      2. BCM PL011 UART
[3] Chars written: 117
[4] Echoing input now

[MP] Bye 👋
```

# make qemu

シリアルの接続を待ってしまうので使えない。

```bash
$ make qemu

Compiling kernel - rpi3
    Finished release [optimized] target(s) in 0.00s

Launching QEMU

 __  __ _      _ _                 _
|  \/  (_)_ _ (_) |   ___  __ _ __| |
| |\/| | | ' \| | |__/ _ \/ _` / _` |
|_|  |_|_|_||_|_|____\___/\__,_\__,_|

           Raspberry Pi 3

[ML] Requesting binary                  // ここでストール
```

## 実機による実行結果

```
$ make chainboot

Minipush 1.0

/Users/dspace/raspi_os/rust_raspi_os/.vendor/bundle/ruby/2.7.0/gems/serialport-1.3.1/lib/serialport.rb:25: warning: rb_secure will be removed in Ruby 3.0
[MP] ✅ Serial connected
[MP] 🔌 Please power the target now

 __  __ _      _ _                 _
|  \/  (_)_ _ (_) |   ___  __ _ __| |
| |\/| | | ' \| | |__/ _ \/ _` / _` |
|_|  |_|_|_||_|_|____\___/\__,_\__,_|

           Raspberry Pi 3

[ML] Requesting binary
[MP] ⏩ Pushing 0 KiB 🦀                                             0% 0 KiB/s
[MP] ⏩ Pushing 3 KiB =====================🦀                       51% 0 KiB/s
[MP] ⏩ Pushing 5 KiB ==================================🦀          81% 0 KiB/s
[MP] ⏩ Pushing 6 KiB ==========================================🦀 100% 0 KiB/s Time: 00:00:00
[ML] Loaded! Executing the payload now

[0] mingo version 0.5.0
[1] Booting on: Raspberry Pi 3
[2] Drivers loaded:
      1. BCM GPIO
      2. BCM PL011 UART
[3] Chars written: 117
[4] Echoing input now
albabldef

[MP] Bye 👋
```
