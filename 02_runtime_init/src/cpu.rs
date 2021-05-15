// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2021 Andre Richter <andre.o.richter@gmail.com>

//! プロセッサコード

#[cfg(target_arch = "aarch64")]
#[path = "_arch/aarch64/cpu.rs"]
mod arch_cpu;

mod boot;

//--------------------------------------------------------------------------------------------------
// アーキテクチャのパブリック再エクスポート
//--------------------------------------------------------------------------------------------------
pub use arch_cpu::wait_forever;
