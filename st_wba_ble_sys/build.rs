use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn run(cmd: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(cmd).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn add_isystem_if_exists(builder: bindgen::Builder, p: &Path, label: &str) -> bindgen::Builder {
    if p.exists() {
        println!("cargo:warning=bindgen {}: {}", label, p.display());
        builder.clang_arg(format!("-isystem{}", p.display()))
    } else {
        builder
    }
}

fn main() {
    // ---- locate vendor tree from workspace root ----
    let workspace_dir = env::var("CARGO_WORKSPACE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
                .parent()
                .unwrap()
                .to_path_buf()
        });

    // Locate the BLE stack inside STM32CubeWBA. Prefer an env override; otherwise try common submodule paths.
    let root: PathBuf = if let Ok(p) = env::var("STM32CUBEWBA_DIR") {
        PathBuf::from(p)
            .join("Middlewares")
            .join("ST")
            .join("STM32_WPAN")
            .join("ble")
            .join("stack")
    } else {
        let candidates = [
            workspace_dir.join("STM32_WPAN/ble/stack"), // legacy symlink at repo root
            workspace_dir.join("external/STM32CubeWBA/Middlewares/ST/STM32_WPAN/ble/stack"),
            workspace_dir.join("vendor/STM32CubeWBA/Middlewares/ST/STM32_WPAN/ble/stack"),
            workspace_dir.join("third_party/STM32CubeWBA/Middlewares/ST/STM32_WPAN/ble/stack"),
        ];
        match candidates.into_iter().find(|p| p.exists()) {
            Some(p) => p,
            None => panic!(
                "Could not find STM32_WPAN/ble/stack.\n\
                 Set STM32CUBEWBA_DIR to your CubeWBA folder or add a submodule at one of:\n  \
                 external/STM32CubeWBA, vendor/STM32CubeWBA, third_party/STM32CubeWBA"
            ),
        }
    };
    let inc = root.join("include");
    let inc_auto = inc.join("auto");
    let lib = root.join("lib");

    assert!(inc.exists(), "Missing include dir: {}", inc.display());
    assert!(
        inc_auto.exists(),
        "Missing include/auto dir: {}",
        inc_auto.display()
    );
    assert!(lib.exists(), "Missing lib dir: {}", lib.display());

    // ---- link ST prebuilt static lib ----
    println!("cargo:rustc-link-search=native={}", lib.display());
    let libname = if cfg!(feature = "basic") {
        "stm32wba_ble_stack_basic"
    } else if cfg!(feature = "basic_plus") {
        "stm32wba_ble_stack_basic_plus"
    } else if cfg!(feature = "llo") {
        "stm32wba_ble_stack_llo"
    } else if cfg!(feature = "llobasic") {
        "stm32wba_ble_stack_llobasic"
    } else if cfg!(feature = "po") {
        "stm32wba_ble_stack_po"
    } else {
        "stm32wba_ble_stack_full"
    };
    println!("cargo:rustc-link-lib=static={}", libname);

    // ---- bindgen setup ----
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    let shim = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("shims")
        .join("bindgen_fix.h");

    let mut builder = bindgen::Builder::default()
        .use_core()
        .ctypes_prefix("cty")
        .generate_comments(true)
        .clang_arg("-x")
        .clang_arg("c")
        .clang_arg("-std=c11")
        .clang_arg("-target")
        .clang_arg("arm-none-eabi")
        // pre-include the checked-in shim so macros are defined before ST headers
        .clang_arg("-include")
        .clang_arg(shim.to_string_lossy().to_string())
        // Force robust definitions for all packed macro spellings to avoid header-order surprises
        .clang_arg("-D__PACKED_BEGIN=")
        .clang_arg("-D__PACKED_END=__attribute__((__packed__))")
        .clang_arg("-D__PACKED_STRUCT=struct __attribute__((__packed__))")
        .clang_arg("-D__PACKED_UNION=union __attribute__((__packed__))")
        .clang_arg("-DPACKED_STRUCT=struct __attribute__((__packed__))")
        .clang_arg("-DPACKED_UNION=union __attribute__((__packed__))")
        .clang_arg("-DPACKED=__attribute__((__packed__))")
        .clang_arg("-Wno-macro-redefined")
        // our header roots
        .clang_arg(format!("-I{}", inc.display()))
        .clang_arg(format!("-I{}", inc_auto.display()))
        // tame STM attribute macros so clang is happy
        // .clang_arg("-D__packed=__attribute__((__packed__))")
        // .clang_arg("-D__PACKED=__attribute__((__packed__))")
        .clang_arg("-D__weak=__attribute__((weak))")
        .clang_arg("-D__WEAK=__attribute__((weak))")
        .clang_arg("-D__ALIGN_BEGIN=")
        .clang_arg("-D__ALIGN_END=")
        // .clang_arg("-DPACKED=")
        // .clang_arg("-DPLACE_IN_SECTION(x)=")
        .clang_arg("-D__IO=volatile")
        // top headers (pulls in ble_types.h transitively)
        .header(inc.join("blestack.h").to_string_lossy())
        .header(inc.join("ble_core.h").to_string_lossy())
        .header(inc.join("ble_std.h").to_string_lossy())
        .header(inc_auto.join("ble_gap_aci.h").to_string_lossy())
        .header(inc_auto.join("ble_gatt_aci.h").to_string_lossy())
        .header(inc_auto.join("ble_hal_aci.h").to_string_lossy())
        .header(inc_auto.join("ble_l2cap_aci.h").to_string_lossy())
        .header(inc_auto.join("ble_hci_le.h").to_string_lossy())
        .header(inc_auto.join("ble_events.h").to_string_lossy())
        .allowlist_function("aci_.*")
        .allowlist_function("hci_.*")
        .allowlist_var("HCI_.*|ACI_.*|GAP_.*|GATT_.*|BLE_.*")
        .allowlist_type(".*(Handle|Event|Param|Service|Characteristic|Status|Opcode).*");

    // ---- feed clang the ARM toolchain headers ----
    let gcc = env::var("ARM_NONE_EABI_GCC").unwrap_or_else(|_| "arm-none-eabi-gcc".into());

    // GCC internals
    if let Some(gcc_inc) = run(&gcc, &["-print-file-name=include"]).map(PathBuf::from) {
        builder = add_isystem_if_exists(builder, &gcc_inc, "GCC include");
        if let Some(ver_dir) = gcc_inc.parent() {
            builder =
                add_isystem_if_exists(builder, &ver_dir.join("include-fixed"), "GCC include-fixed");
        }
        // derive toolchain root from ".../arm-none-eabi/lib/gcc/..."
        let s = gcc_inc.to_string_lossy();
        if let Some(idx) = s.find("/arm-none-eabi/lib/gcc/") {
            let tool_root = Path::new(&s[..idx]); // e.g. /Applications/ArmGNUToolchain/14.3.rel1
            // Try both newlib layouts:
            builder = add_isystem_if_exists(
                builder,
                &tool_root.join("arm-none-eabi").join("include"),
                "newlib include (A)",
            );
            builder = add_isystem_if_exists(
                builder,
                &tool_root
                    .join("arm-none-eabi")
                    .join("arm-none-eabi")
                    .join("include"),
                "newlib include (B)",
            );
        }
    }

    // Prefer canonicalized sysroot (removes any "/bin/../")
    if let Some(sysroot_raw) = run(&gcc, &["-print-sysroot"]) {
        let sysroot = fs::canonicalize(PathBuf::from(sysroot_raw.clone()))
            .unwrap_or(PathBuf::from(sysroot_raw));
        builder = add_isystem_if_exists(builder, &sysroot.join("include"), "sysroot include");
        builder = add_isystem_if_exists(
            builder,
            &sysroot.join("arm-none-eabi").join("include"),
            "sysroot arm-none-eabi/include",
        );
    }

    // Optional manual overrides
    if let Ok(extra) = env::var("BINDGEN_EXTRA_CLANG_ARGS") {
        for tok in extra.split_whitespace() {
            println!("cargo:warning=bindgen extra arg: {}", tok);
            builder = builder.clang_arg(tok.to_string());
        }
    }

    // ---- generate ----
    builder
        .generate()
        .expect("bindgen failed")
        .write_to_file(out.join("bindings.rs"))
        .unwrap();

    println!("cargo:rerun-if-changed={}", inc.display());
    println!("cargo:rerun-if-changed={}", inc_auto.display());
    println!("cargo:rerun-if-changed={}", shim.display());
    println!("cargo:rerun-if-env-changed=STM32CUBEWBA_DIR");
    println!("cargo:rerun-if-env-changed=ARM_NONE_EABI_GCC");
    println!("cargo:rerun-if-env-changed=BINDGEN_EXTRA_CLANG_ARGS");
}
