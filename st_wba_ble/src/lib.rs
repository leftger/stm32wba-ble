#![no_std]
// st_wba_ble/src/lib.rs
use st_wba_ble_sys::ffi;

// Only include modules that actually exist and are meant to build.
pub mod evt;
pub mod gatt;
pub use gatt::{Char, Service, add_char, add_primary_service};

/// Lightweight status mapping for ACI return codes.
pub mod status {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum BleStatus {
        Ok,
        Other(i32),
    }
    impl From<i32> for BleStatus {
        fn from(v: i32) -> Self {
            if v == 0 {
                BleStatus::Ok
            } else {
                BleStatus::Other(v)
            }
        }
    }
    pub type Result<T> = core::result::Result<T, BleStatus>;
}

pub struct Ble {
    _priv: (),
}

impl Ble {
    /// Initialize the BLE stack for a GAP Peripheral role and optionally set the device name.
    pub fn init_peripheral(dev_name: &str) -> status::Result<Self> {
        unsafe {
            // 1) GATT init
            let rc = ffi::aci_gatt_init() as i32;
            if status::BleStatus::from(rc) != status::BleStatus::Ok {
                return Err(status::BleStatus::from(rc));
            }

            // 2) GAP init (Peripheral role, no privacy)
            let (mut svc, mut name_h, mut app_h) = (0u16, 0u16, 0u16);
            let rc = ffi::aci_gap_init(
                0x01, // GAP Peripheral role (replace with ffi::GAP_PERIPHERAL_ROLE if present)
                0,    // privacy off
                dev_name.len() as u8,
                &mut svc,
                &mut name_h,
                &mut app_h,
            ) as i32;
            if status::BleStatus::from(rc) != status::BleStatus::Ok {
                return Err(status::BleStatus::from(rc));
            }

            // 3) Optional: update the Device Name characteristic value
            if !dev_name.is_empty() {
                let rc = ffi::aci_gatt_update_char_value(
                    svc,
                    name_h,
                    0u8,                  // offset
                    dev_name.len() as u8, // length
                    dev_name.as_ptr(),    // value pointer
                ) as i32;
                if status::BleStatus::from(rc) != status::BleStatus::Ok {
                    return Err(status::BleStatus::from(rc));
                }
            }
        }
        Ok(Ble { _priv: () })
    }
}
