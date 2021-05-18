// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>

//! BSPドライバサポート

use crate::driver;

//--------------------------------------------------------------------------------------------------
// プライベート定義
//--------------------------------------------------------------------------------------------------

/// デバイスドライバマネージャ型
struct BSPDriverManager {
    device_drivers: [&'static (dyn DeviceDriver + Sync); 2],
}

//--------------------------------------------------------------------------------------------------
// グローバルインスンタンス
//--------------------------------------------------------------------------------------------------

static BSP_DRIVER_MANAGER: BSPDriverManager = BSPDriverManager {
    device_drivers: [&super::GPIO, &super::PL011_UART],
};

//--------------------------------------------------------------------------------------------------
// パブリックコード
//--------------------------------------------------------------------------------------------------

/// ドライバマネージャへの参照を返す
pub fn driver_manager() -> &'static impl driver::interface::DriverManager {
    &BSP_DRIVER_MANAGER
}

//------------------------------------------------------------------------------
// OSインタフェースコード
//------------------------------------------------------------------------------
use driver::interface::DeviceDriver;

impl driver::interface::DriverManager for BSPDriverManager {
    fn all_device_drivers(&self) -> &[&'static (dyn DeviceDriver + Sync)] {
        &self.device_drivers[..]
    }

    fn post_device_driver_init(&self) {
        // PL011Uartの出力ピンを構成する
        super::GPIO.map_pl011_uart();
    }
}
