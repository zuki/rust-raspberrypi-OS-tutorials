// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021-2023 Andre Richter <andre.o.richter@gmail.com>

//! アーキテクチャ固有のブートコード。
//!
//! # オリエンテーション
//!
//! archモジュールはpath属性を使って汎用モジュールにインポートされるので
//! このファイルのパスは次の通り:
//!
//! crate::cpu::boot::arch_boot

use core::arch::global_asm;

<<<<<<< HEAD
// このファイルに対応するアセンブリファイル。
global_asm!(include_str!("boot.s"));
=======
// Assembly counterpart to this file.
global_asm!(
    include_str!("boot.s"),
    CONST_CORE_ID_MASK = const 0b11
);
>>>>>>> master

//--------------------------------------------------------------------------------------------------
// パブリックコード
//--------------------------------------------------------------------------------------------------

/// `kernel`バイナリのRust側エントリ。
///
<<<<<<< HEAD
/// この関数はアセンブリファイルの`_start`関数から呼び出される。
///
/// # 安全性
///
/// - `bss`セクションはまだ初期化されていない。コードはbssをいかなる方法であれ、使用または参照してはならない。
=======
/// The function is called from the assembly `_start` function.
>>>>>>> master
#[no_mangle]
pub unsafe fn _start_rust() -> ! {
    crate::kernel_init()
}
