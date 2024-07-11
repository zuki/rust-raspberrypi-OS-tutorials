// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>

//! ドライバサポート

use crate::{
    println,
    synchronization::{interface::Mutex, NullLock},
};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

const NUM_DRIVERS: usize = 5;

struct DriverManagerInner {
    next_index: usize,
    descriptors: [Option<DeviceDriverDescriptor>; NUM_DRIVERS],
}

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
}

<<<<<<< HEAD
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
=======
/// Tpye to be used as an optional callback after a driver's init() has run.
pub type DeviceDriverPostInitCallback = unsafe fn() -> Result<(), &'static str>;

/// A descriptor for device drivers.
#[derive(Copy, Clone)]
pub struct DeviceDriverDescriptor {
    device_driver: &'static (dyn interface::DeviceDriver + Sync),
    post_init_callback: Option<DeviceDriverPostInitCallback>,
}

/// Provides device driver management functions.
pub struct DriverManager {
    inner: NullLock<DriverManagerInner>,
}

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

static DRIVER_MANAGER: DriverManager = DriverManager::new();

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

impl DriverManagerInner {
    /// Create an instance.
    pub const fn new() -> Self {
        Self {
            next_index: 0,
            descriptors: [None; NUM_DRIVERS],
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl DeviceDriverDescriptor {
    /// Create an instance.
    pub fn new(
        device_driver: &'static (dyn interface::DeviceDriver + Sync),
        post_init_callback: Option<DeviceDriverPostInitCallback>,
    ) -> Self {
        Self {
            device_driver,
            post_init_callback,
        }
    }
}

/// Return a reference to the global DriverManager.
pub fn driver_manager() -> &'static DriverManager {
    &DRIVER_MANAGER
}

impl DriverManager {
    /// Create an instance.
    pub const fn new() -> Self {
        Self {
            inner: NullLock::new(DriverManagerInner::new()),
        }
    }

    /// Register a device driver with the kernel.
    pub fn register_driver(&self, descriptor: DeviceDriverDescriptor) {
        self.inner.lock(|inner| {
            inner.descriptors[inner.next_index] = Some(descriptor);
            inner.next_index += 1;
        })
    }

    /// Helper for iterating over registered drivers.
    fn for_each_descriptor<'a>(&'a self, f: impl FnMut(&'a DeviceDriverDescriptor)) {
        self.inner.lock(|inner| {
            inner
                .descriptors
                .iter()
                .filter_map(|x| x.as_ref())
                .for_each(f)
        })
    }

    /// Fully initialize all drivers.
    ///
    /// # Safety
    ///
    /// - During init, drivers might do stuff with system-wide impact.
    pub unsafe fn init_drivers(&self) {
        self.for_each_descriptor(|descriptor| {
            // 1. Initialize driver.
            if let Err(x) = descriptor.device_driver.init() {
                panic!(
                    "Error initializing driver: {}: {}",
                    descriptor.device_driver.compatible(),
                    x
                );
            }

            // 2. Call corresponding post init callback.
            if let Some(callback) = &descriptor.post_init_callback {
                if let Err(x) = callback() {
                    panic!(
                        "Error during driver post-init callback: {}: {}",
                        descriptor.device_driver.compatible(),
                        x
                    );
                }
            }
        });
    }

    /// Enumerate all registered device drivers.
    pub fn enumerate(&self) {
        let mut i: usize = 1;
        self.for_each_descriptor(|descriptor| {
            println!("      {}. {}", i, descriptor.device_driver.compatible());

            i += 1;
        });
>>>>>>> master
    }
}
