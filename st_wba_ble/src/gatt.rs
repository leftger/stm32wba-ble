// st_wba_ble/src/gatt.rs
use crate::status::{BleStatus, Result};
use st_wba_ble_sys::ffi;

pub struct Service {
    pub handle: u16,
}
pub struct Char {
    /// NOTE: ST returns the *declaration* handle from `aci_gatt_add_char`.
    /// The value handle is `handle + 1`; the CCCD (if NOTIFY/INDICATE is set)
    /// is at `handle + 2`. See ST's GATT ACI docs/comments.
    /// (Confirmed by ST community posts quoting `ble_gatt_aci.h`.)
    pub handle: u16,
}

// ===== Named constants (avoid magic numbers) =====
// Values are documented in ST's headers (e.g., ble_gatt_aci.h):
// UUID type: 0x01 = 16-bit, 0x02 = 128-bit. Primary service = 0x01.
pub const UUID_TYPE_16: u8 = 0x01;
pub const UUID_TYPE_128: u8 = 0x02;
pub const PRIMARY_SERVICE: u8 = 0x01;

// Common characteristic properties & permissions (convenience re-exports)
// These match CubeMX templates / ST examples.
pub const CHAR_PROP_READ: u8 = 0x02;
pub const CHAR_PROP_NOTIFY: u8 = 0x10;
pub const ATTR_PERMISSION_NONE: u8 = 0x00;

// Event mask examples for `aci_gatt_add_char` (8-bit).
// Pick the one(s) you need; default to 0 if unsure.
pub const GATT_NOTIFY_ATTRIBUTE_WRITE: u8 = 0x01;

// ===== Internal helpers for UUID packing =====
#[inline]
fn make_service_uuid16(uuid16: u16) -> ffi::Service_UUID_t {
    // Zero then write the LE 16-bit value into the start of the union.
    let mut suuid: ffi::Service_UUID_t = unsafe { core::mem::zeroed() };
    let bytes = uuid16.to_le_bytes();
    unsafe {
        let p = (&mut suuid) as *mut _ as *mut u8;
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), p, 2);
    }
    suuid
}

#[inline]
fn make_char_uuid16(uuid16: u16) -> ffi::Char_UUID_t {
    let mut cuuid: ffi::Char_UUID_t = unsafe { core::mem::zeroed() };
    let bytes = uuid16.to_le_bytes();
    unsafe {
        let p = (&mut cuuid) as *mut _ as *mut u8;
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), p, 2);
    }
    cuuid
}

#[inline]
fn make_service_uuid128(uuid128: &[u8; 16]) -> ffi::Service_UUID_t {
    let mut suuid: ffi::Service_UUID_t = unsafe { core::mem::zeroed() };
    unsafe {
        let p = (&mut suuid) as *mut _ as *mut u8;
        core::ptr::copy_nonoverlapping(uuid128.as_ptr(), p, 16);
    }
    suuid
}

#[inline]
fn make_char_uuid128(uuid128: &[u8; 16]) -> ffi::Char_UUID_t {
    let mut cuuid: ffi::Char_UUID_t = unsafe { core::mem::zeroed() };
    unsafe {
        let p = (&mut cuuid) as *mut _ as *mut u8;
        core::ptr::copy_nonoverlapping(uuid128.as_ptr(), p, 16);
    }
    cuuid
}

// ===== Services =====
/// Add a primary service with a 16-bit UUID. (Backwards-compatible API)
pub fn add_primary_service(uuid16: u16, max_attr_records: u8) -> Result<Service> {
    add_primary_service_uuid16(uuid16, max_attr_records)
}

/// Add a primary service with a 16-bit UUID.
pub fn add_primary_service_uuid16(uuid16: u16, max_attr_records: u8) -> Result<Service> {
    let mut svc_handle: u16 = 0;
    let suuid = make_service_uuid16(uuid16);
    let rc = unsafe {
        ffi::aci_gatt_add_service(
            UUID_TYPE_16 as u8,
            &suuid as *const _,
            PRIMARY_SERVICE as u8,
            max_attr_records,
            &mut svc_handle as *mut u16,
        )
    } as i32;
    if BleStatus::from(rc) == BleStatus::Ok {
        Ok(Service { handle: svc_handle })
    } else {
        Err(BleStatus::from(rc))
    }
}

/// Add a primary service with a 128-bit UUID.
pub fn add_primary_service_uuid128(uuid128: [u8; 16], max_attr_records: u8) -> Result<Service> {
    let mut svc_handle: u16 = 0;
    let suuid = make_service_uuid128(&uuid128);
    let rc = unsafe {
        ffi::aci_gatt_add_service(
            UUID_TYPE_128 as u8,
            &suuid as *const _,
            PRIMARY_SERVICE as u8,
            max_attr_records,
            &mut svc_handle as *mut u16,
        )
    } as i32;
    if BleStatus::from(rc) == BleStatus::Ok {
        Ok(Service { handle: svc_handle })
    } else {
        Err(BleStatus::from(rc))
    }
}

// ===== Characteristics =====
/// Add a 16-bit UUID characteristic to an existing service.
/// `evt_mask` lets you request callbacks (e.g., `GATT_NOTIFY_ATTRIBUTE_WRITE`).
pub fn add_char_with_mask(
    svc: &Service,
    uuid16: u16,
    props: u8,
    perm: u8,
    len: u16,
    evt_mask: u8,
) -> Result<Char> {
    let mut ch_decl_handle: u16 = 0;
    let cuuid = make_char_uuid16(uuid16);
    let rc = unsafe {
        ffi::aci_gatt_add_char(
            svc.handle,
            UUID_TYPE_16 as u8,
            &cuuid as *const _,
            len,
            props,
            perm,
            evt_mask, // GATT event mask (0 for none)
            0,        // Encryption_Key_Size (0 = default)
            0,        // Is_Variable (0 = fixed length)
            &mut ch_decl_handle as *mut u16,
        )
    } as i32;
    if BleStatus::from(rc) == BleStatus::Ok {
        Ok(Char {
            handle: ch_decl_handle,
        })
    } else {
        Err(BleStatus::from(rc))
    }
}

/// Backwards-compatible helper with no event mask.
pub fn add_char(svc: &Service, uuid16: u16, props: u8, perm: u8, len: u16) -> Result<Char> {
    add_char_with_mask(svc, uuid16, props, perm, len, 0)
}

/// Add a 128-bit UUID characteristic.
pub fn add_char_uuid128_with_mask(
    svc: &Service,
    uuid128: [u8; 16],
    props: u8,
    perm: u8,
    len: u16,
    evt_mask: u8,
) -> Result<Char> {
    let mut ch_decl_handle: u16 = 0;
    let cuuid = make_char_uuid128(&uuid128);
    let rc = unsafe {
        ffi::aci_gatt_add_char(
            svc.handle,
            UUID_TYPE_128 as u8,
            &cuuid as *const _,
            len,
            props,
            perm,
            evt_mask,
            0,
            0,
            &mut ch_decl_handle as *mut u16,
        )
    } as i32;
    if BleStatus::from(rc) == BleStatus::Ok {
        Ok(Char {
            handle: ch_decl_handle,
        })
    } else {
        Err(BleStatus::from(rc))
    }
}

// ===== Updates / Notifications =====
/// Update the characteristic value (<=255 bytes) and let the stack notify if CCCD is enabled.
/// For longer values, prefer `update_char_value_chunked_notify` (uses the EXT API when enabled).
pub fn update_char_value(svc: &Service, ch: &Char, val: &[u8]) -> Result<()> {
    let value_handle = ch.handle + 1; // value attribute lives at decl+1
    let rc = unsafe {
        ffi::aci_gatt_update_char_value(
            svc.handle,
            value_handle,
            0,               // Offset
            val.len() as u8, // Val length (<= 255)
            val.as_ptr(),
        )
    } as i32;
    if BleStatus::from(rc) == BleStatus::Ok {
        Ok(())
    } else {
        Err(BleStatus::from(rc))
    }
}

/// Update long values by chunking (and optionally sending notifications).
/// If the `use_update_ext` Cargo feature is enabled and your bindings expose
/// `aci_gatt_update_char_value_ext`, this will use it to send notifications
/// in one go; otherwise it falls back to multiple basic updates.
#[cfg(feature = "use_update_ext")]
pub fn update_char_value_chunked_notify(svc: &Service, ch: &Char, val: &[u8]) -> Result<()> {
    let value_handle = ch.handle + 1;
    // Update type 0x01 is commonly used to send notifications (per ST posts).
    const GATT_CHAR_UPDATE_SEND_NOTIFICATION: u8 = 0x01;

    let total = val.len() as u16;
    let mut off: u16 = 0;
    while off < total {
        let chunk = core::cmp::min(255u16, total - off) as u8;
        let ptr = unsafe { val.as_ptr().add(off as usize) };
        let rc = unsafe {
            ffi::aci_gatt_update_char_value_ext(
                svc.handle, // Some stacks take a conn handle first; adjust your bindings if needed
                value_handle,
                GATT_CHAR_UPDATE_SEND_NOTIFICATION,
                total,
                off,
                chunk,
                ptr,
            )
        } as i32;
        if BleStatus::from(rc) != BleStatus::Ok {
            return Err(BleStatus::from(rc));
        }
        off += chunk as u16;
    }
    Ok(())
}

#[cfg(not(feature = "use_update_ext"))]
pub fn update_char_value_chunked_notify(svc: &Service, ch: &Char, val: &[u8]) -> Result<()> {
    // Fallback: perform multiple basic updates with increasing offset.
    // Depending on stack configuration, this may generate multiple notifications.
    let value_handle = ch.handle + 1;
    let total = val.len() as u16;
    let mut off: u16 = 0;
    while off < total {
        let chunk = core::cmp::min(255u16, total - off) as u8;
        let ptr = unsafe { val.as_ptr().add(off as usize) };
        let rc = unsafe {
            ffi::aci_gatt_update_char_value(
                svc.handle,
                value_handle,
                // aci_gatt_update_char_value expects an 8-bit offset on WB/WBA stacks.
                off as u8,
                chunk,
                ptr,
            )
        } as i32;
        if BleStatus::from(rc) != BleStatus::Ok {
            return Err(BleStatus::from(rc));
        }
        off += chunk as u16;
    }
    Ok(())
}
