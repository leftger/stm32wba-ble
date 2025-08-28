// st_wba_ble/src/evt.rs
#![allow(dead_code)]
//! Minimal C hooks some ST BLE stacks expect.
//! These are no-ops so the crate compiles and links; wire them up later if your drop requires it.

use core::ffi::c_void;

// If your stack expects a periodic event processor, keep the symbol present as a no-op.
#[unsafe(no_mangle)]
pub extern "C" fn hci_user_evt_proc() {}

// Optional RX callback some drops expose; keep signature-compatible and unused for now.
#[unsafe(no_mangle)]
pub extern "C" fn hci_user_evt_rx(_pdata: *mut c_void) {}

// Notify/wait hooks pattern — provide harmless stubs.
#[unsafe(no_mangle)]
pub extern "C" fn hci_notify_asynch_evt(_pdata: *mut core::ffi::c_void) {}

#[unsafe(no_mangle)]
pub extern "C" fn hci_cmd_resp_release(_flag: u32) {}

#[unsafe(no_mangle)]
pub extern "C" fn hci_cmd_resp_wait(_timeout: u32) {}
