// st_wba_ble/src/status.rs
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BleStatus {
    Ok,
    Busy,
    InvalidParam,
    // …
    Other(i32),
}
impl From<i32> for BleStatus {
    fn from(v: i32) -> Self {
        match v {
            0 => BleStatus::Ok,
            // fill in using your ffi:: constants (e.g., ffi::BLE_STATUS_INVALID_PARAMS)
            x => BleStatus::Other(x),
        }
    }
}
pub type Result<T> = core::result::Result<T, BleStatus>;
