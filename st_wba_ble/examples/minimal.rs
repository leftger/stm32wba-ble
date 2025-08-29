#![no_std]
#![no_main]

use core::panic::PanicInfo;
use st_wba_ble::{Ble, add_char, add_primary_service, start_fast_name, update_char_value};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// NOTE: This is a placeholder "bare-metal" entry. In your board crate, use your
// actual startup/runtime with clocks/IPCC/radio initialized before calling BLE.
#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    let _ble = Ble::init_peripheral("RustWBA").ok();

    // Add a simple service and characteristic (UUIDs are examples)
    if let Ok(svc) = add_primary_service(0x180A, 4) {
        if let Ok(ch) = add_char(
            &svc,
            0x2A29,
            st_wba_ble::gatt::CHAR_PROP_READ,
            st_wba_ble::gatt::ATTR_PERMISSION_NONE,
            20,
        ) {
            let _ = update_char_value(&svc, &ch, b"ACME-Rust");
        }
    }

    let _ = start_fast_name("RustWBA");

    loop {
        // In a real app, periodically call the HCI event pump or run the C glue.
        // For now, spin.
    }
}
