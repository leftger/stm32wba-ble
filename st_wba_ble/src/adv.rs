use crate::status::{BleStatus, Result};
use st_wba_ble_sys::ffi;

/// Start undirected connectable advertising quickly with a given local name.
///
/// Uses 20–40 ms advertising interval, public address, no filter policy.
pub fn start_fast_name(name: &str) -> Result<()> {
    let adv_type_undirected: u8 = 0x00; // ADV_IND
    let own_addr_public: u8 = 0x00;
    let filter_allow_all: u8 = 0x00;
    let min: u16 = 0x0020; // 20 ms
    let max: u16 = 0x0040; // 40 ms
    let name_len: u8 = name.len() as u8;
    let name_ptr: *const u8 = name.as_bytes().as_ptr();
    let uuid_len: u8 = 0;
    let uuid_ptr: *const u8 = core::ptr::null();
    let slave_ci_min: u16 = 0; // let stack choose
    let slave_ci_max: u16 = 0; // let stack choose

    let rc = unsafe {
        ffi::aci_gap_set_discoverable(
            adv_type_undirected,
            min,
            max,
            own_addr_public,
            filter_allow_all,
            name_len,
            name_ptr,
            uuid_len,
            uuid_ptr,
            slave_ci_min,
            slave_ci_max,
        )
    } as i32;
    if BleStatus::from(rc) != BleStatus::Ok {
        return Err(BleStatus::from(rc));
    }

    // Ensure advertising is enabled.
    let rc = unsafe { ffi::hci_le_set_advertising_enable(1) } as i32;
    if BleStatus::from(rc) != BleStatus::Ok {
        return Err(BleStatus::from(rc));
    }
    Ok(())
}

/// Stop advertising.
pub fn stop() -> Result<()> {
    let rc = unsafe { ffi::hci_le_set_advertising_enable(0) } as i32;
    if BleStatus::from(rc) == BleStatus::Ok {
        Ok(())
    } else {
        Err(BleStatus::from(rc))
    }
}

/// Replace current advertising data (<=31 bytes). Optional helper.
pub fn set_adv_data(data: &[u8]) -> Result<()> {
    let len: u8 = core::cmp::min(31, data.len()) as u8;
    let ptr = data.as_ptr();
    let rc = unsafe { ffi::hci_le_set_advertising_data(len, ptr) } as i32;
    if BleStatus::from(rc) == BleStatus::Ok {
        Ok(())
    } else {
        Err(BleStatus::from(rc))
    }
}
