use std::path::{Path, PathBuf};
use std::{env, fs, io};

fn main() -> nice_plug_xtask::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let is_bundle = matches!(
        args.first().map(String::as_str),
        Some("bundle" | "bundle-universal")
    );

    nice_plug_xtask::main_with_args("cargo xtask", args)?;

    // `main_with_args` leaves us in the workspace root, so relative paths resolve from there
    if is_bundle {
        let target_dir =
            env::var_os("CARGO_TARGET_DIR").map_or_else(|| PathBuf::from("target"), PathBuf::from);
        install_vst3_bundles(&target_dir.join("bundled"))?;
    }

    Ok(())
}

/// The per-user VST3 plugin directory, as defined by the VST3 SDK for each platform.
fn vst3_install_dir() -> io::Result<PathBuf> {
    let (var, subdir) = if cfg!(target_os = "macos") {
        ("HOME", "Library/Audio/Plug-Ins/VST3")
    } else if cfg!(windows) {
        ("LOCALAPPDATA", "Programs/Common/VST3")
    } else {
        ("HOME", ".vst3")
    };

    let base = env::var_os(var).ok_or_else(|| io::Error::other(format!("{var} is not set")))?;
    Ok(Path::new(&base).join(subdir))
}

/// Copy every VST3 bundle in `bundled_dir` to the user's VST3 plugin directory, replacing any
/// previously installed copy.
fn install_vst3_bundles(bundled_dir: &Path) -> io::Result<()> {
    let install_dir = vst3_install_dir()?;
    fs::create_dir_all(&install_dir)?;

    for entry in fs::read_dir(bundled_dir)? {
        let bundle = entry?.path();
        if bundle.extension().is_none_or(|ext| ext != "vst3") {
            continue;
        }

        let destination =
            install_dir.join(bundle.file_name().expect("Bundle path doesn't end in .."));
        if destination.exists() {
            fs::remove_dir_all(&destination)?;
        }
        copy_bundle(&bundle, &destination)?;

        eprintln!("Installed VST3 bundle to '{}'", destination.display());
    }

    Ok(())
}

fn copy_bundle(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let destination = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_bundle(&entry.path(), &destination)?;
        } else {
            fs::copy(entry.path(), destination)?;
        }
    }

    Ok(())
}
