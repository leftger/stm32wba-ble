// st_wba_ble/src/gatt.rs
use crate::status::{BleStatus, Result};
use st_wba_ble_sys::ffi;

pub struct Service { pub handle: u16 }
pub struct Char    { pub handle: u16 }

/// Add a primary service with a 16-bit UUID.
pub fn add_primary_service(uuid16: u16, max_attr_records: u8) -> Result<Service> {
    let mut svc: u16 = 0;
    // Zero the Service_UUID_t and write the 16-bit value into the start of the union.
    let mut suuid: ffi::Service_UUID_t = unsafe { core::mem::zeroed() };
    unsafe {
        let p = (&mut suuid) as *mut _ as *mut u16;
        *p = uuid16.to_le();
    }
    let rc = unsafe {
        ffi::aci_gatt_add_service(
            0x01,                         // UUID_TYPE_16
            &suuid as *const _,
            0x01,                         // PRIMARY_SERVICE
            max_attr_records,
            &mut svc as *mut u16,
        )
    } as i32;
    if BleStatus::from(rc) == BleStatus::Ok {
        Ok(Service { handle: svc })
    } else {
        Err(BleStatus::from(rc))
    }
}

/// Add a characteristic with a 16-bit UUID to an existing service.
pub fn add_char(svc: &Service, uuid16: u16, props: u8, perm: u8, len: u16) -> Result<Char> {
    let mut ch: u16 = 0;
    let mut cuuid: ffi::Char_UUID_t = unsafe { core::mem::zeroed() };
    unsafe {
        let p = (&mut cuuid) as *mut _ as *mut u16;
        *p = uuid16.to_le();
    }
    let rc = unsafe {
        ffi::aci_gatt_add_char(
            svc.handle,
            0x01,                         // UUID_TYPE_16
            &cuuid as *const _,
            len,
            props,
            perm,
            1,                            // GATT_Evt_Mask
            0,                            // Encryption_Key_Size
            0,                            // Is_Variable
            &mut ch as *mut u16,
        )
    } as i32;
    if BleStatus::from(rc) == BleStatus::Ok {
        Ok(Char { handle: ch })
    } else {
        Err(BleStatus::from(rc))
    }
}
