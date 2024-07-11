# Dockerをインストール

```bash
$ brew install docker
$ brew install docker --cask
Warning: Cask 'docker' is already installed.

To re-install docker, run:
  brew reinstall docker
$ brew reinstall docker --cask
$ which docker
$ open Docker   # 初期設定
$ which docker
/usr/local/bin/docker
```

# aarch64 toolchainsをインストール

```bash
$ brew tap SergioBenitez/osxct
$ brew install aarch64-none-elf
Error: The contents of the SDKs in your Command Line Tools (CLT) installation do not match the SDK folder names.
$ sudo mv /Library/Developer/CommandLineTools /Library/Developer/CommandLineTools.old
$ wget https://download.developer.apple.com/Developer_Tools/Command_Line_Tools_for_Xcode_11.3.1/Command_Line_Tools_for_Xcode_11.3.1.dmg
$ open ~/Downloads/Command_Line_Tools_for_Xcode_11.3.1.dmg
$ brew install aarch64-none-elf
$ aarch64-none-elf-gcc --version
aarch64-none-elf-gcc (GCC) 7.2.0
Copyright (C) 2017 Free Software Foundation, Inc.
This is free software; see the source for copying conditions.  There is NO
warranty; not even for MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
```

# 実行

```bash
$ make qemu

Compiling kernel - rpi3
    Finished release [optimized] target(s) in 0.00s

Launching QEMU
----------------
IN:
0x00000000:  580000c0  ldr      x0, #0x18
0x00000004:  aa1f03e1  mov      x1, xzr
0x00000008:  aa1f03e2  mov      x2, xzr
0x0000000c:  aa1f03e3  mov      x3, xzr
0x00000010:  58000084  ldr      x4, #0x20
0x00000014:  d61f0080  br       x4

----------------
IN:
0x00080000:  d503205f  wfe
0x00080004:  17ffffff  b        #0x80000

----------------
IN:
0x00000300:  d2801b05  mov      x5, #0xd8
0x00000304:  d53800a6  mrs      x6, mpidr_el1
0x00000308:  924004c6  and      x6, x6, #3
0x0000030c:  d503205f  wfe
0x00000310:  f86678a4  ldr      x4, [x5, x6, lsl #3]
0x00000314:  b4ffffc4  cbz      x4, #0x30c

----------------
IN:
0x0000030c:  d503205f  wfe
0x00000310:  f86678a4  ldr      x4, [x5, x6, lsl #3]
0x00000314:  b4ffffc4  cbz      x4, #0x30c
```

# make targetを実行

```bash
$ make readelf

Compiling kernel - rpi3
    Finished release [optimized] target(s) in 0.00s

Launching readelf
ELF Header:
  Magic:   7f 45 4c 46 02 01 01 00 00 00 00 00 00 00 00 00
  Class:                             ELF64
  Data:                              2's complement, little endian
  Version:                           1 (current)
  OS/ABI:                            UNIX - System V
  ABI Version:                       0
  Type:                              EXEC (Executable file)
  Machine:                           AArch64
  Version:                           0x1
  Entry point address:               0x80000
  Start of program headers:          64 (bytes into file)
  Start of section headers:          65800 (bytes into file)
  Flags:                             0x0
  Size of this header:               64 (bytes)
  Size of program headers:           56 (bytes)
  Number of program headers:         1
  Size of section headers:           64 (bytes)
  Number of section headers:         7
  Section header string table index: 5

Section Headers:
  [Nr] Name              Type             Address           Offset
       Size              EntSize          Flags  Link  Info  Align
  [ 0]                   NULL             0000000000000000  00000000
       0000000000000000  0000000000000000           0     0     0
  [ 1] .text             PROGBITS         0000000000080000  00010000
       0000000000000008  0000000000000000  AX       0     0     1
  [ 2] .debug_aranges    PROGBITS         0000000000000000  00010008
       0000000000000000  0000000000000000           0     0     1
  [ 3] .comment          PROGBITS         0000000000000000  00010008
       0000000000000013  0000000000000001  MS       0     0     1
  [ 4] .symtab           SYMTAB           0000000000000000  00010020
       0000000000000078  0000000000000018           6     3     8
  [ 5] .shstrtab         STRTAB           0000000000000000  00010098
       0000000000000039  0000000000000000           0     0     1
  [ 6] .strtab           STRTAB           0000000000000000  000100d1
       0000000000000033  0000000000000000           0     0     1
Key to Flags:
  W (write), A (alloc), X (execute), M (merge), S (strings), I (info),
  L (link order), O (extra OS processing required), G (group), T (TLS),
  C (compressed), x (unknown), o (OS specific), E (exclude),
  p (processor specific)

Program Headers:
  Type           Offset             VirtAddr           PhysAddr
                 FileSiz            MemSiz              Flags  Align
  LOAD           0x0000000000010000 0x0000000000080000 0x0000000000080000
                 0x0000000000000008 0x0000000000000008  R E    0x10000

 Section to Segment mapping:
  Segment Sections...
   00     .text
$ make objdump

Compiling kernel - rpi3
    Finished release [optimized] target(s) in 0.00s

Launching objdump

target/aarch64-unknown-none-softfloat/release/kernel:     file format elf64-littleaarch64


Disassembly of section .text:

0000000000080000 <_start>:
   80000:       d503205f        wfe
   80004:       17ffffff        b       80000 <_start>

$ make nm

Compiling kernel - rpi3
    Finished release [optimized] target(s) in 0.00s

Launching nm
0000000000080000 0000000000000008 T _start
0000000000080000 A __rpi_load_addr
```
