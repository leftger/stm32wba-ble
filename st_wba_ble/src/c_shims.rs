// st_wba_ble/src/c_shims.rs
#![allow(non_snake_case)]

/// --- Optional: HAL tick stub -----------------------------------------------
/// If you’re not linking STM32 HAL, some ST utilities may call HAL_GetTick().
/// Provide a millisecond counter. With Embassy, you can map to its clock.
/// NOTE: remove this if you *do* link a real HAL that already defines it.

#[cfg(feature = "shim_hal_tick")]
#[unsafe(no_mangle)]
pub extern "C" fn HAL_GetTick() -> u32 {
    // Embassy Instant is monotonic; convert to ms (wraps at u32::MAX by design).
    (embassy_time::Instant::now().as_millis() as u64 % (u32::MAX as u64 + 1)) as u32
}

// --- Optional placeholders for RNG/AES/PKA ---------------------------------
// Only add these if you decide not to compile ST’s C "Interfaces" drivers and
// your chosen stack libs reference them. Leave commented until needed.
// #[unsafe(no_mangle)]
// pub extern "C" fn HW_RNG_Init() {}
//
// #[unsafe(no_mangle)]
// pub extern "C" fn HW_RNG_GetRandom32(pval: *mut u32) -> i32 {
//     unsafe { if !pval.is_null() { *pval = 0xDEADBEEF; } }
//     0 // return 0 as “OK” by convention
// }
//
// #[unsafe(no_mangle)]
// pub extern "C" fn HW_AES_Init() {}
//
// #[unsafe(no_mangle)]
// pub extern "C" fn HW_PKA_Init() {}
//
// // …add more exact signatures as needed (match ST headers in your repo)
