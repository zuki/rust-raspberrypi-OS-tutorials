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
	// ブートコア上でのみ実行する。他のコアは止める。
	mrs	x1, MPIDR_EL1	      // MARの[7:0]がコア番号（raspi3/4はcoreを4つ搭載: 0x00-0x03）
	and	x1, x1, _core_id_mask // _code_id_mask = 0b11; このファイルの先頭で定義
	ldr	x2, BOOT_CORE_ID      // BOOT_CORE_ID=0: bsp/__board_name__/cpu.rs で定義
	cmp	x1, x2
	b.ne	1f		      // core0以外は1へジャンプ

	// 処理がここに来たらそれはブートコア。Rustコードにジャンプするための準備をする。

	// スタックポインタを設定する。
	ADR_REL	x0, __boot_core_stack_end_exclusive	// link.ldで定義 = 0x80000 .textの下に伸びる
	mov	sp, x0

	// Rustコードにジャンプする。
	b	_start_rust

	// イベントを無限に待つ（別名 "park the core"）
1:	wfe
	b	1b

.size	_start, . - _start
.type	_start, function
.global	_start
