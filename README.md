# stm32wba-ble

Rust bindings & a thin wrapper for STMicroelectronics’ **STM32WBA** Bluetooth Low Energy (BLE) stack.

This repo contains:
- **`st_wba_ble_sys/`** — FFI bindings generated with `bindgen` against the prebuilt ST BLE stack headers.
- **`st_wba_ble/`** — a small, `no_std` wrapper that provides a safer surface (init, GATT helpers, etc).
- **`STM32_WPAN`** — symlink pointing into the STM32CubeWBA tree (added via Git submodule).
- **`external/STM32CubeWBA/`** — the STM32CubeWBA Git submodule (sparse-checked to the WPAN middleware).

---

## Quick start

```bash
# Clone with submodules (needed for STM32CubeWBA)
git clone --recurse-submodules <this-repo-url>
cd stm32wba-ble

# (Optional) limit the submodule checkout to the WPAN middleware
git -C external/STM32CubeWBA sparse-checkout init --cone
git -C external/STM32CubeWBA sparse-checkout set Middlewares/ST/STM32_WPAN


## Integration (STM32WBA BLE Stack)

- **Vendor tree**: Either set `STM32CUBEWBA_DIR` to your CubeWBA folder, or keep the provided `external/STM32CubeWBA` submodule. The build locates:
  - `Middlewares/ST/STM32_WPAN/ble/stack` and `link_layer/ll_cmd_lib`
  - `Drivers/CMSIS`, `Drivers/STM32WBAxx_HAL_Driver`

- **Required project files (if you enable C glue)**: Provide board/example glue at the repo root:
  - `STM32_WPAN/Target/`: `bleplat.c`, `linklayer_plat.c`, `ll_sys_if.c`, `host_stack_if.c`, `power_table.c`
  - `System/Config/`: `app_conf.h`, `ble_conf.h`, etc.
  - `System/Interfaces/`: `hw_rng.c`, `hw_aes.c`, `hw_pka.c`, `pka_p256.c`
  - `System/Modules/`: `ble_timer.c`, `stm_list.c`, memory manager, NVM, Flash, rf_timing_synchro
  - These are taken from an ST WBA BLE example (for your exact board).

- **Features** (choose one stack variant): `full` (default), `basic`, `basic-plus`, `llo`, `llobasic`, `po`.
  - Optional: `compile_glue` (builds the C glue listed above)
  - Optional: `shim_hal_tick` (provides `HAL_GetTick()` via `embassy-time`)

- **App Cargo.toml** (example):

```toml
[dependencies]
st_wba_ble = { path = "../stm32wba-ble/st_wba_ble", features = ["full", "compile_glue", "shim_hal_tick"] }
# Embassy selections depend on your MCU/board
embassy-executor = { git = "https://github.com/embassy-rs/embassy" }
embassy-time = { git = "https://github.com/embassy-rs/embassy" }
```

- **Embassy event pump** (periodically call host event processing):

```rust
use embassy_executor::{Spawner, task};
use embassy_time::{Timer, Duration};

#[task]
async fn ble_evt_task() {
    loop {
        st_wba_ble::evt::hci_user_evt_proc();
        Timer::after(Duration::from_millis(10)).await; // tune 1–10 ms as needed
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let _p = embassy_stm32::init(Default::default());
    spawner.spawn(ble_evt_task()).unwrap();
    // Init BLE, add services, start advertising...
}
```

- **Advertising/GATT** (helpers provided):

```rust
let _ble = st_wba_ble::Ble::init_peripheral("RustWBA").unwrap();
let svc = st_wba_ble::add_primary_service(0x180A, 4).unwrap();
let ch = st_wba_ble::add_char(&svc, 0x2A29, st_wba_ble::gatt::CHAR_PROP_READ, st_wba_ble::gatt::ATTR_PERMISSION_NONE, 20).unwrap();
st_wba_ble::update_char_value(&svc, &ch, b"ACME-Rust").unwrap();
st_wba_ble::start_fast_name("RustWBA").unwrap();
```

- **Environment/toolchain**:
  - Arm GNU toolchain available (`arm-none-eabi-gcc`) or set `ARM_NONE_EABI_GCC=/abs/path/to/arm-none-eabi-gcc`
  - Optional override: `STM32CUBEWBA_DIR=/abs/path/to/STM32CubeWBA`

- **Build**:
  - Host check: `cargo build`
  - MCU example: `cargo build -p st_wba_ble --target thumbv8m.main-none-eabihf --features full --example minimal`

Refer to ST’s wiki “Connectivity: STM32WBA BLE Stack Integration” for board-specific RCC/IPCC/radio bring‑up and copy the corresponding `STM32_WPAN/Target` and `System/*` trees from a working ST example.
