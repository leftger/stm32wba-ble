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

fn feat(name: &str) -> bool {
    env::var(format!(
        "CARGO_FEATURE_{}",
        name.to_ascii_uppercase().replace('-', "_")
    ))
    .is_ok()
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

    // ---- locate Link Layer (LL) tree ----
    let ll_root: PathBuf = if let Ok(p) = env::var("STM32CUBEWBA_DIR") {
        PathBuf::from(p)
            .join("Middlewares")
            .join("ST")
            .join("STM32_WPAN")
            .join("link_layer")
            .join("ll_cmd_lib")
    } else {
        let candidates = [
            workspace_dir.join("STM32_WPAN/link_layer/ll_cmd_lib"),
            workspace_dir
                .join("external/STM32CubeWBA/Middlewares/ST/STM32_WPAN/link_layer/ll_cmd_lib"),
            workspace_dir
                .join("vendor/STM32CubeWBA/Middlewares/ST/STM32_WPAN/link_layer/ll_cmd_lib"),
            workspace_dir
                .join("third_party/STM32CubeWBA/Middlewares/ST/STM32_WPAN/link_layer/ll_cmd_lib"),
        ];
        match candidates.into_iter().find(|p| p.exists()) {
            Some(p) => p,
            None => panic!(
                "Could not find STM32_WPAN/link_layer/ll_cmd_lib.\n\
                 Set STM32CUBEWBA_DIR to your CubeWBA folder or add a submodule at one of:\n  \
                 external/STM32CubeWBA, vendor/STM32CubeWBA, third_party/STM32CubeWBA"
            ),
        }
    };
    let ll_inc = ll_root.join("inc");
    let ll_sys_inc = ll_root.parent().unwrap().join("ll_sys").join("inc");
    let ll_lib = ll_root.join("lib");

    assert!(
        ll_inc.exists(),
        "Missing LL include dir: {}",
        ll_inc.display()
    );
    assert!(
        ll_sys_inc.exists(),
        "Missing LL system include dir: {}",
        ll_sys_inc.display()
    );
    assert!(ll_lib.exists(), "Missing LL lib dir: {}", ll_lib.display());

    // ---- link ST prebuilt static lib (copy into OUT_DIR with lib prefix) ----
    // Always re-run if the vendor lib dir changes (e.g., submodule updates)
    println!("cargo:rerun-if-changed={}", lib.display());

    // Only attempt to link the ARM archives when building for an embedded target
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    if target_os == "none" {
        fs::create_dir_all(&out_dir).expect("create OUT_DIR failed");

        // Copy any stm32wba_ble_stack*.a archives into OUT_DIR with a `lib` prefix,
        // because `-lfoo` expects `libfoo.a` on disk.
        let mut copied_any = false;
        for entry in fs::read_dir(&lib).expect("read lib dir") {
            let p = entry.expect("dir entry").path();
            if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("stm32wba_ble_stack") && name.ends_with(".a") {
                    let dst = out_dir.join(format!("lib{}", name));
                    fs::copy(&p, &dst).expect("copy static lib to OUT_DIR");
                    copied_any = true;
                }
            }
        }
        assert!(
            copied_any,
            "No stm32wba_ble_stack*.a archives found in {}",
            lib.display()
        );

        // Copy Link Layer archive(s) into OUT_DIR with a `lib` prefix
        let mut ll_copied_any = false;
        for entry in fs::read_dir(&ll_lib).expect("read LL lib dir") {
            let p = entry.expect("dir entry").path();
            if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                if name.ends_with("_lib.a") && name.starts_with("LinkLayer_BLE_") {
                    let dst = out_dir.join(format!("lib{}", name));
                    fs::copy(&p, &dst).expect("copy LL static lib to OUT_DIR");
                    ll_copied_any = true;
                }
            }
        }
        assert!(
            ll_copied_any,
            "No LinkLayer_BLE_*_lib.a archives found in {}",
            ll_lib.display()
        );

        // === Select BLE stack & Link Layer by features (Option A) ===
        // Use CARGO_FEATURE_* env vars in build.rs (cfg!(feature) is evaluated for the host).
        let stack = if feat("basic") {
            "stm32wba_ble_stack_basic"
        } else if feat("basic_plus") {
            "stm32wba_ble_stack_basic_plus"
        } else if feat("llo") {
            "stm32wba_ble_stack_llo"
        } else if feat("llobasic") {
            "stm32wba_ble_stack_llobasic"
        } else if feat("po") {
            "stm32wba_ble_stack_po"
        } else {
            // default
            "stm32wba_ble_stack_full"
        };

        // Map BLE stack variant -> required Link Layer archive (per ST table)
        let ll = if feat("basic") || feat("basic_plus") || feat("llobasic") || feat("po") {
            "LinkLayer_BLE_Basic_lib"
        } else {
            // full and llo require the FULL LL
            "LinkLayer_BLE_Full_lib"
        };

        // Ensure the chosen archives exist in OUT_DIR now that we've copied files
        let stack_a = out_dir.join(format!("lib{}.a", stack));
        assert!(
            stack_a.exists(),
            "Requested BLE stack variant '{}' not present at {}",
            stack,
            stack_a.display()
        );
        let ll_a = out_dir.join(format!("lib{}.a", ll));
        assert!(
            ll_a.exists(),
            "Requested Link Layer '{}' not present at {}",
            ll,
            ll_a.display()
        );

        let repo_root = std::path::Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
            .parent()
            .unwrap()
            .to_path_buf();
        let built_glue = build_wba_glue(&repo_root);

        // === Link: group the two static archives so order doesn’t bite us ===
        println!("cargo:rustc-link-search=native={}", out_dir.display());
        println!("cargo:rustc-link-arg=-Wl,--start-group");
        if built_glue {
            println!("cargo:rustc-link-lib=static=st_wba_ble_glue");
        }
        println!("cargo:rustc-link-lib=static={}", ll); // LL first
        println!("cargo:rustc-link-lib=static={}", stack); // then BLE stack
        println!("cargo:rustc-link-arg=-Wl,--end-group");
    } else {
        println!(
            "cargo:warning=skipping ST BLE stack link on host target ({})",
            target_os
        );
    }

    // ---- bindgen setup ----
    let out_bindings = PathBuf::from(env::var("OUT_DIR").unwrap());

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
        .clang_arg(format!("-I{}", ll_inc.display()))
        .clang_arg(format!("-I{}", ll_sys_inc.display()))
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
        .write_to_file(out_bindings.join("bindings.rs"))
        .unwrap();

    println!("cargo:rerun-if-changed={}", inc.display());
    println!("cargo:rerun-if-changed={}", inc_auto.display());
    println!("cargo:rerun-if-changed={}", shim.display());
    println!("cargo:rerun-if-env-changed=STM32CUBEWBA_DIR");
    println!("cargo:rerun-if-env-changed=ARM_NONE_EABI_GCC");
    println!("cargo:rerun-if-env-changed=BINDGEN_EXTRA_CLANG_ARGS");
    println!("cargo:rerun-if-changed={}", ll_inc.display());
    println!("cargo:rerun-if-changed={}", ll_sys_inc.display());
    println!("cargo:rerun-if-changed={}", ll_lib.display());
}

fn build_wba_glue(repo_root: &std::path::Path) -> bool {
    use std::path::PathBuf;
    if !feat("compile_glue") {
        return false;
    }

    let wpan = repo_root.join("STM32_WPAN");
    let target = wpan.join("Target");
    let system = repo_root.join("System");
    if !target.exists() {
        return false;
    }

    // Minimal set used by ST examples (add only if they exist locally)
    let mut files: Vec<PathBuf> = vec![
        target.join("bleplat.c"),
        target.join("linklayer_plat.c"),
        target.join("ll_sys_if.c"),
        target.join("host_stack_if.c"),
        target.join("power_table.c"),
        system.join("Modules/ble_timer.c"),
        system.join("Modules/stm_list.c"),
        system.join("Modules/utilities_common.c"),
        system.join("Modules/Flash/flash_driver.c"),
        system.join("Modules/Flash/flash_manager.c"),
        system.join("Modules/Flash/simple_nvm_arbiter.c"),
        system.join("Modules/Flash/rf_timing_synchro.c"),
        system.join("Modules/Nvm/nvm.c"),
        system.join("Modules/MemoryManager/stm32_mm.c"),
        system.join("Modules/MemoryManager/advanced_memory_manager.c"),
        system.join("Interfaces/hw_rng.c"),
        system.join("Interfaces/hw_aes.c"),
        system.join("Interfaces/hw_pka.c"),
        system.join("Interfaces/pka_p256.c"),
    ];
    files.retain(|p| p.exists());
    if files.is_empty() {
        return false;
    }

    println!("cargo:rerun-if-changed={}", target.display());
    println!("cargo:rerun-if-changed={}", system.display());

    let drivers = repo_root.join("Drivers");
    let mut b = cc::Build::new();
    for f in &files {
        b.file(f);
    }

    // Includes (Target/App/System + stack/LL + CMSIS/HAL)
    b.include(&target);
    b.include(wpan.join("App"));
    b.include(system.join("Config"));
    b.include(system.join("Interfaces"));
    b.include(system.join("Modules"));
    b.include(wpan.join("ble/stack/include"));
    b.include(wpan.join("ble/stack/include/auto"));
    b.include(wpan.join("link_layer/ll_cmd_lib/inc"));
    b.include(wpan.join("link_layer/ll_sys/inc"));
    b.include(drivers.join("CMSIS/Include"));
    b.include(drivers.join("CMSIS/Device/ST/STM32WBAxx/Include"));
    b.include(drivers.join("STM32WBAxx_HAL_Driver/Inc"));

    b.define("STM32WBAXX", None);
    b.flag_if_supported("-Wno-unused-parameter");
    b.flag_if_supported("-Wno-sign-compare");

    b.compile("st_wba_ble_glue");
    true
}
