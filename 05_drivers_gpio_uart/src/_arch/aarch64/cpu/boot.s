// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021-2022 Andre Richter <andre.o.richter@gmail.com>

//--------------------------------------------------------------------------------------------------
// 定義
//--------------------------------------------------------------------------------------------------

// シンボルのアドレスをレジスタにロードする（PC-相対）。
//
// シンボルはプログラムカウンタの +/- 4GiB以内になければならない。
//
// # リソース
//
// - https://sourceware.org/binutils/docs-2.36/as/AArch64_002dRelocations.html
.macro ADR_REL register, symbol
	adrp	\register, \symbol
	add	\register, \register, #:lo12:\symbol
.endm

//--------------------------------------------------------------------------------------------------
// パブリックコード
//--------------------------------------------------------------------------------------------------
.section .text._start

//------------------------------------------------------------------------------
// fn _start()
//------------------------------------------------------------------------------
_start:
	// ブートコア上でのみ実行する。他のコアは止める.
	mrs	x0, MPIDR_EL1
	and	x0, x0, {CONST_CORE_ID_MASK}
	ldr	x1, BOOT_CORE_ID      // provided by bsp/__board_name__/cpu.rs
	cmp	x0, x1
	b.ne	.L_parking_loop

	// 処理がここに来たらそれはブートコア.

	// DRAMを初期化する.
	ADR_REL	x0, __bss_start
	ADR_REL x1, __bss_end_exclusive

.L_bss_init_loop:
	cmp	x0, x1
	b.eq	.L_prepare_rust
	stp	xzr, xzr, [x0], #16
	b	.L_bss_init_loop

	// Rustコードにジャンプするための準備をする.
.L_prepare_rust:
	// スタックポインタを設定する
	ADR_REL	x0, __boot_core_stack_end_exclusive
	mov	sp, x0

	// Rustコードにジャンプする。
	b	_start_rust

	// イベントを無限に待つ（別名 "park the core"）
1:	wfe
	b	1b

.size	_start, . - _start
.type	_start, function
.global	_start
