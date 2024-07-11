// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>

// Rust embedded logo for `make doc`.
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/rust-embedded/wg/master/assets/logo/ewg-logo-blue-white-on-transparent.png"
)]

//! `カーネル`バイナリ。
//!
//! # コードの構成とアーキテクチャ
//!
//! コードは複数のモジュールに分割されており、それぞれが`カーネル`の代表的な**サブシステム**を
//! 表しています。サブシステムのトップレベルのモジュールファイルは `src` フォルダ直下に格納
//! されています。たとえば、`src/memory.rs`には、メモリ管理全般に関するコードが含まれています。
//!
//! ## プロセッサアーキテクチャコードの可視化
//!
//! カーネルの`サブシステム`の中には、対象となるプロセッサアーキテクチャ固有の低レベルコードに
//! 依存するものがあります。それらはサポートされているプロセッサアーキテクチャごとに`src/_arch`
//! 配下にサブフォルダが存在します（たとえば、`src/_arch/aarch64`）。
//!
//! アーキテクチャのフォルダは`src`配下のサブシステムモジュールのレイアウトを踏襲しています。
//! たとえば、`カーネル`のMMUサブシステム（`src/memory/mmu.rs`）に関するアーキテクチャコードは
//! `src/_arch/aarch64/memory/mmu.rs`にあります。後者のファイルは、`path属性`を使って
//! `src/memory/mmu.rs`のモジュールとして読み込まれます。通常、選択されるモジュールの名前は
//! 汎用モジュールの名前の先頭に`arch_`を付けたものになります。
//!
//! たとえば、`src/memory/mmu.rs`の冒頭は次のようになっています。
//!
//! ```
//! #[cfg(target_arch = "aarch64")]
//! #[path = "../_arch/aarch64/memory/mmu.rs"]
//! mod arch_mmu;
//! ```
//!
//! 多くの場合、`arch_ モジュール`のアイテムは親モジュールによりpublicに再エクスポートされます。
//! このようにして、各アーキテクチャ固有のモジュールはアイテムの実装を提供することができ、
//! 呼び出し側はどのアーキテクチャが条件付きでコンパイルされているかを気にする必要がありません。
//!
//! ## BSPコード
//!
//! `BSP`はBoard Support Packageの略です。`BSP`のコードは`src/bsp.rs`としてまとめられており、
//! ターゲットボード固有の定義や機能が含まれています。これには、ボードのメモリマップや、各ボードに
//! 搭載されているデバイス用のドライバのインスタンスなどがあります。
//!
//! プロセッサアーキテクチャのコードと同様に、`BSP`のコードモジュール構造は`カーネル`の
//! サブシステムモジュールを踏襲していますが再エクスポートはしていません。つまり、
//! `bsp::driver::driver_manager()`のように、提供されているものを呼び出す際にはすべて
//! `bsp`名前空間を付ける必要があります。
//!
//!
//! ## カーネルインタフェース
//!
//! `arch`も`bsp`も、実際にカーネルがコンパイルされるターゲットやボードに応じて条件コンパイル
//! されるコードを含んでいます。たとえば、`Raspberry Pi 3`と`Raspberry Pi 4`では、`割り込み
//! コントローラ`のハードウェアが異なりますが、`カーネル`コードは、2つのうちのどちらとも
//! うまく動作するようにしたいものです。
//!
//! `arch`と`bsp`そして`汎用カーネルコード`の間でクリーンな抽象化を行うために、*可能な限り*、
//! *意味のある*ところには`interface`トレイトが提供されています。これらは各サブシステム
//! モジュールで定義されており、*実装ではなくインタフェースに対してプログラムする*という
//! イディオムを強制します。たとえば、Raspberryの2つの異なる割り込みコントローラ`ドライバ`が
//! 実装するべき共通のIRQ処理インタフェースを提供し、カーネルの他の部分にはそのインタフェース
//! だけをエクスポートしています。
//!
//! ```
//!         +-------------------+
//!         | インタフェース    |
//!         |    (トレイト)     |
//!         +--+-------------+--+
//!            ^             ^
//!            |             |
//!            |             |
//! +----------+--+       +--+----------+
//! | カーネル    |       |  bspコード  |
//! |   コード    |       |  archコード |
//! +-------------+       +-------------+
//! ```
//!
//! # まとめ
//!
//! 論理的な`カーネル`サブシステムは、対応するコードを複数の物理的な場所に分散配置//!
//! できます。ここでは**メモリ**サブシステムの例を示します。
//!
//! - `src/memory.rs` と `src/memory/**/*`
//!   - 対象となるプロセッサのアーキテクチャや`BSP`の特性に左右されない共通のコー//! ド
//!     - 例: メモリチャンクをゼロにする関数
//!   - `arch`や`BSP`のコードで実装されるメモリサブシステムのインタフェース
//!     - 例: `MMU`関数プロトタイプを定義する`MMU`インタフェース
//! - `src/bsp/__board_name__/memory.rs` と `src/bsp/__board_name__/memory/**/*`
//!   - `BSP`特有のコード。
//!     - 例: ボードのメモリマップ（DRAMやMMIOデバイスの物理アドレス）
//! - `src/_arch/__arch_name__/memory.rs` と `src/_arch/__arch_name__/memory/**/*`
//!   - プロセッサアーキテクチャ固有のコード
//!     - 例: `__arch_name__`プロセッサアーキテクチャ用の`MMU`インタフェースの実装
//!
//! 名前空間の観点から見ると、**メモリ**サブシステムのコードは以下になります。
//!
//! - `crate::memory::*`
//! - `crate::bsp::memory::*`
//!
//! # ブートフロー
//!
//! 1. カーネルのエントリポイントは関数 `cpu::boot::arch_boot::_start()`
//!     - 実装は `src/_arch/__arch_name__/cpu/boot.s` にある
//! 2. アーキテクチャのセットアップが終わったら、
//! アーキテクチャのコードは[`runtime_init::runtime_init()`]を呼び出す
//!
//! [`runtime_init::runtime_init()`]: runtime_init/fn.runtime_init.html

#![allow(clippy::upper_case_acronyms)]
#![feature(asm_const)]
#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![feature(trait_alias)]
#![no_main]
#![no_std]

mod bsp;
mod console;
mod cpu;
mod driver;
mod panic_wait;
mod print;
mod synchronization;

/// 最初の初期化コード
///
/// # 安全性
///
/// - アクティブなコアはこの関数を実行しているコアだけでなければならない
/// - この関数内のinitコールは正しい順番でなければならない
unsafe fn kernel_init() -> ! {
    // Initialize the BSP driver subsystem.
    if let Err(x) = bsp::driver::init() {
        panic!("Error initializing BSP driver subsystem: {}", x);
    }

    // Initialize all device drivers.
    driver::driver_manager().init_drivers();
    // println! is usable from here on.

    // unsafeからsafeに移行
    kernel_main()
}

const MINILOAD_LOGO: &str = r#"
 __  __ _      _ _                 _
|  \/  (_)_ _ (_) |   ___  __ _ __| |
| |\/| | | ' \| | |__/ _ \/ _` / _` |
|_|  |_|_|_||_|_|____\___/\__,_\__,_|
"#;

/// 最初の初期化後に実行するメイン関数
fn kernel_main() -> ! {
    use console::console;

    println!("{}", MINILOAD_LOGO);
    println!("{:^37}", bsp::board_name());
    println!();
    println!("[ML] Requesting binary");
    console().flush();

    // エコーモードに移行する前に受信したスプリアス文字を破棄する
    console().clear_rx();

    // `Minipush`にバイナリを送信するよう通知する
    for _ in 0..3 {
        console().write_char(3 as char);
    }

    // バイナリサイズを読み込む
    let mut size: u32 = u32::from(console().read_char() as u8);
    size |= u32::from(console().read_char() as u8) << 8;
    size |= u32::from(console().read_char() as u8) << 16;
    size |= u32::from(console().read_char() as u8) << 24;

    // サイズが巨大でないことを信じる
    console().write_char('O');
    console().write_char('K');

    let kernel_addr: *mut u8 = bsp::memory::board_default_load_addr() as *mut u8;
    unsafe {
        // カーネルをバイトごとに読み込む
        for i in 0..size {
            core::ptr::write_volatile(kernel_addr.offset(i as isize), console().read_char() as u8)
        }
    }

    println!("[ML] Loaded! Executing the payload now\n");
    console().flush();

    // 関数ポインタを作成するために黒魔術を使用する
    let kernel: fn() -> ! = unsafe { core::mem::transmute(kernel_addr) };

    // ロードしたカーネルにジャンプする!
    kernel()
}
