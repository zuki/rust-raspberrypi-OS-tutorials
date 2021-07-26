// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021 Andre Richter <andre.o.richter@gmail.com>

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

.equ _core_id_mask, 0b11

//--------------------------------------------------------------------------------------------------
// パブリックコード
//--------------------------------------------------------------------------------------------------
.section .text._start

//------------------------------------------------------------------------------
// fn _start()
//------------------------------------------------------------------------------
_start:
	// ブートコア上でのみ実行する。他のコアは止める。
	mrs	x1, MPIDR_EL1	      // MARの[7:0]がコア番号（raspi3/4はcoreを4つ搭載: 0x00-0x03）
	and	x1, x1, _core_id_mask // _code_id_mask = 0b11; このファイルの先頭で定義
	ldr	x2, BOOT_CORE_ID      // BOOT_CORE_ID=0: bsp/__board_name__/cpu.rs で定義
	cmp	x1, x2
	b.ne	2f		      // core0以外は2へジャンプ

	// 処理がここに来たらそれはブートコア。

	// 次に、バイナリを再配置する
	ADR_REL	x0, __binary_nonzero_start         // バイナリのロードアドレス
	ADR_ABS	x1, __binary_nonzero_start         // バイナリのリンクアドレス
	ADR_ABS	x2, __binary_nonzero_end_exclusive

1:	ldr	x3, [x0], #8	// x3 <- [x0]; x0+=8
	str	x3, [x1], #8	// x3 -> [x1]; x1+=8
	cmp	x1, x2		// x1 - x2
	b.lo	1b		// goto 1b if x1 < x2

	// スタックポインタを設定する。
	ADR_ABS	x0, __boot_core_stack_end_exclusive
	mov	sp, x0

	// 再配置されたRustコードにジャンプする
	ADR_ABS	x1, _start_rust
	br	x1

	// イベントを無限に待つ（別名 "park the core"）
2:	wfe
	b	2b

.size	_start, . - _start
.type	_start, function
.global	_start
