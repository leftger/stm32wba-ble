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

