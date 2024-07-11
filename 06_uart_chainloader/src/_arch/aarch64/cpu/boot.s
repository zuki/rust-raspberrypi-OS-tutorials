// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021-2022 Andre Richter <andre.o.richter@gmail.com>

//--------------------------------------------------------------------------------------------------
// 定義
//--------------------------------------------------------------------------------------------------

// シンボルのアドレス（PC-相対アドレス）をレジスタにロードする。
//
// シンボルはプログラムカウンタの +/- 4GiB以内になければならない。
//
// # 参考資料
//
// - https://sourceware.org/binutils/docs-2.36/as/AArch64_002dRelocations.html
.macro ADR_REL register, symbol
	adrp	\register, \symbol
	add	\register, \register, #:lo12:\symbol
.endm

// シンボルのアドレス（絶対アドレス）をレジスタにロードする
//
// # Resources
//
// - https://sourceware.org/binutils/docs-2.36/as/AArch64_002dRelocations.html
.macro ADR_ABS register, symbol
	movz	\register, #:abs_g2:\symbol
	movk	\register, #:abs_g1_nc:\symbol
	movk	\register, #:abs_g0_nc:\symbol
.endm

//--------------------------------------------------------------------------------------------------
// パブリックコード
//--------------------------------------------------------------------------------------------------
.section .text._start

//------------------------------------------------------------------------------
// fn _start()
//------------------------------------------------------------------------------
_start:
	// ブートコア上でのみ実行する。他のコアは止める
	mrs	x0, MPIDR_EL1			// MARの[7:0]がコア番号（raspi3/4はcoreを4つ搭載: 0x00-0x03）
	and	x0, x0, {CONST_CORE_ID_MASK}
	ldr	x1, BOOT_CORE_ID      // BOOT_CORE_ID=0: bsp/__board_name__/cpu.rs で定義
	cmp	x0, x1
	b.ne	.L_parking_loop

	// 処理がここに来たらそれはブートコア。

	// DRAMを初期化する.
	ADR_ABS	x0, __bss_start
	ADR_ABS x1, __bss_end_exclusive

.L_bss_init_loop:
	cmp	x0, x1
	b.eq	.L_relocate_binary
	stp	xzr, xzr, [x0], #16
	b	.L_bss_init_loop

	// バイナリをリロケートする.
.L_relocate_binary:
	ADR_REL	x0, __binary_nonzero_start         // バイナリがロードされるアドレス
	ADR_ABS	x1, __binary_nonzero_start         // バイナリがリンクされていたアドレス
	ADR_ABS	x2, __binary_nonzero_end_exclusive

.L_copy_loop:
	ldr	x3, [x0], #8
	str	x3, [x1], #8
	cmp	x1, x2
	b.lo	.L_copy_loop

	// Rustコードにジャンプするための準備をする.
	// スタックポインタを設定する.
	ADR_ABS	x0, __boot_core_stack_end_exclusive
	mov	sp, x0

	// 再配置されたRustコードにジャンプする
	ADR_ABS	x1, _start_rust
	br	x1

	// イベントを無限に待つ（別名 "park the core"）
.L_parking_loop:
	wfe
	b	.L_parking_loop

.size	_start, . - _start
.type	_start, function
.global	_start
