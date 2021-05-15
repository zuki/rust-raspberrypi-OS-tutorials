# コードリーディング

## コアの識別

- raspberry piは4コア搭載。
- ブート時には1つだけ動かして、残りの3つは停止する必要があり、コアの識別が必要。
- コアの識別にはMultiprocessor Affinity Register　(MPIDR_EL1) の下位8ビットを見れば良い（[参照](http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0500g/BABHBJCI.html)）。

```assembly
	mrs	x1, MPIDR_EL1	        // MARの[7:0]がコア番号（raspi3/4は4つ搭載: 0x0-0x3）
	and	x1, x1, _core_id_mask // _code_id_mask = 0b11: 0-3を識別するのでビット[1:0]でAND
	ldr	x2, BOOT_CORE_ID      // BOOT_CORE_ID=0: ブートコアの番号は0
	cmp	x1, x2
	b.ne	1f		              // core0以外は1へジャンプ
```

# 実行

```bash
$ make qemu
Compiling kernel - rpi3
  Downloaded cortex-a v5.1.6
  Downloaded 1 crate (24.9 KB) in 1.26s
   Compiling tock-registers v0.6.0
   Compiling mingo v0.2.0 (/Users/dspace/raspi_os/02_runtime_init)
   Compiling register v1.0.2
   Compiling cortex-a v5.1.6
    Finished release [optimized] target(s) in 5.63s

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
0x00080000:  d53800a1  mrs      x1, mpidr_el1
0x00080004:  92400421  and      x1, x1, #3
0x00080008:  58000142  ldr      x2, #0x80030
0x0008000c:  eb02003f  cmp      x1, x2
0x00080010:  540000a1  b.ne     #0x80024

----------------
IN:
0x00080014:  90000000  adrp     x0, #0x80000
0x00080018:  91000000  add      x0, x0, #0
0x0008001c:  9100001f  mov      sp, x0
0x00080020:  14000006  b        #0x80038

----------------
IN:
0x00080038:  94000002  bl       #0x80040

----------------
IN:
0x00080040:  90000008  adrp     x8, #0x80000
0x00080044:  90000009  adrp     x9, #0x80000
0x00080048:  f940ad08  ldr      x8, [x8, #0x158]
0x0008004c:  f940b129  ldr      x9, [x9, #0x160]
0x00080050:  eb08013f  cmp      x9, x8
0x00080054:  54000109  b.ls     #0x80074

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
0x00000300:  d2801b05  mov      x5, #0xd8
0x00000304:  d53800a6  mrs      x6, mpidr_el1
0x00000308:  924004c6  and      x6, x6, #3
0x0000030c:  d503205f  wfe
0x00000310:  f86678a4  ldr      x4, [x5, x6, lsl #3]
0x00000314:  b4ffffc4  cbz      x4, #0x30c

----------------
IN:
0x00080074:  90000009  adrp     x9, #0x80000
0x00080078:  f940b129  ldr      x9, [x9, #0x160]
0x0008007c:  f800853f  str      xzr, [x9], #8
0x00080080:  eb08013f  cmp      x9, x8
0x00080084:  54ffffc9  b.ls     #0x8007c

----------------
IN:
0x00080088:  94000006  bl       #0x800a0

----------------
IN:
0x000800a0:  90000000  adrp     x0, #0x80000
0x000800a4:  90000002  adrp     x2, #0x80000
0x000800a8:  91032000  add      x0, x0, #0xc8
0x000800ac:  91036042  add      x2, x2, #0xd8
0x000800b0:  528001c1  mov      w1, #0xe
0x000800b4:  97fffff7  bl       #0x80090

----------------
IN:
0x00080090:  94000002  bl       #0x80098

----------------
IN:
0x00080098:  94000009  bl       #0x800bc

----------------
IN:
0x000800bc:  d503205f  wfe
0x000800c0:  17ffffff  b        #0x800bc
```

# シンボル一覧

```bash
$ make nm

Launching nm
0000000000000003 a _core_id_mask
0000000000080000 000000000000002c T _start
0000000000080000 A __rpi_load_addr
0000000000080000 T __boot_core_stack_end_exclusive
0000000000080030 0000000000000008 T BOOT_CORE_ID
0000000000080038 0000000000000008 T _start_rust
0000000000080040 0000000000000050 t kernel::runtime_init::runtime_init
0000000000080090 0000000000000008 t core::panicking::panic
0000000000080098 0000000000000008 t core::panicking::panic_fmt
00000000000800a0 000000000000001c t kernel::kernel_init
00000000000800bc 0000000000000008 t rust_begin_unwind
0000000000080168 B __bss_end_inclusive
0000000000080168 B __bss_start
```
