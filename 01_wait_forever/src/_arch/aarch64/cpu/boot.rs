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

<<<<<<< HEAD
// このファイルに対応するアセンブリファイル。
=======
use core::arch::global_asm;

// Assembly counterpart to this file.
>>>>>>> master
global_asm!(include_str!("boot.s"));
