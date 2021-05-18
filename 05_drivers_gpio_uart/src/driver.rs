// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! ドライバサポート

//--------------------------------------------------------------------------------------------------
// パブリック定義
//--------------------------------------------------------------------------------------------------

/// ドライバインタフェース.
pub mod interface {
    /// デバイスドライバ関数
    pub trait DeviceDriver {
        /// ドライバを識別するための互換性文字列を返す
        fn compatible(&self) -> &'static str;

        /// デバイスを起動するためにカーネルから呼び出される
        ///
        /// # 安全性
        ///
        /// - initの間にドライバがシステム全体に影響を与えることをする可能性がある
        unsafe fn init(&self) -> Result<(), &'static str> {
            Ok(())
        }
    }

    /// デバイスドライバ管理関数
    ///
    /// `BSP`はグローバルインスタンスを一つ提供することが想定されている.
    pub trait DriverManager {
        /// `BSP`がインスタンス化したすべてのドライバへの参照のスライスを返す
        ///
        /// # 安全性
        ///
        /// - デバイスの順番はその`DeviceDriver::init()`が呼び出された順番
        fn all_device_drivers(&self) -> &[&'static (dyn DeviceDriver + Sync)];

        /// ドライバのinit後に実行される初期化コード
        ///
        /// たとえば、すでにオンラインになっている他のドライバに依存するデバイスドライバのコード.
        fn post_device_driver_init(&self);
    }
}
