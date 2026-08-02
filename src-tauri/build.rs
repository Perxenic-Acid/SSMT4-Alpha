use std::{env, fs, path::PathBuf};

fn main() {
    configure_windows_delay_load();
    copy_windivert_runtime_files();
    tauri_build::build()
}

fn configure_windows_delay_load() {
    if env::var("CARGO_CFG_WINDOWS").is_ok() {
        println!("cargo:rustc-link-lib=delayimp");
        println!("cargo:rustc-link-arg=/DELAYLOAD:WinDivert.dll");
    }
}

fn copy_windivert_runtime_files() {
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("missing CARGO_MANIFEST_DIR"));
    let windivert_dir = manifest_dir.join("resources").join("WinDivert");
    let runtime_files = [
        windivert_dir.join("WinDivert.dll"),
        windivert_dir.join("WinDivert64.sys"),
    ];

    for runtime_file in &runtime_files {
        println!("cargo:rerun-if-changed={}", runtime_file.display());
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("missing OUT_DIR"));
    let Some(profile_dir) = out_dir
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
    else {
        return;
    };

    for runtime_file in runtime_files {
        if !runtime_file.exists() {
            continue;
        }

        let file_name = runtime_file
            .file_name()
            .expect("runtime file missing file name");
        let target_file = profile_dir.join(file_name);
        if let Err(error) = fs::copy(&runtime_file, &target_file) {
            panic!(
                "failed to copy {} to {}: {}",
                runtime_file.display(),
                target_file.display(),
                error
            );
        }
    }
}
