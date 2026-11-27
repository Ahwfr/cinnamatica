#![cfg_attr(windows, windows_subsystem = "windows")]
use anyhow::{Context, Result, bail, ensure};
use schematica_builder::{
    component, convert, demo, import, launch, picker, populate_template, write_blueprint,
    write_library,
};
use std::{
    env, fs,
    path::{Path, PathBuf},
};
/// One stable Windows identity shared with the game window and the shortcuts, so a pinned
/// taskbar icon keeps working across updates.
#[cfg(windows)]
fn set_app_identity() {
    let id: Vec<u16> = "Ahwfr.Cinnamatica"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    // SAFETY: `id` is a NUL-terminated UTF-16 string that outlives the call.
    unsafe {
        windows_sys::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID(id.as_ptr());
    }
}

fn main() -> Result<()> {
    #[cfg(windows)]
    set_app_identity();
    let result = run();
    if let Err(error) = &result
        && (env::args_os().len() == 1
            || env::args_os()
                .nth(1)
                .is_some_and(|arg| arg == "import-dialog"))
    {
        picker::show_error(&format!("{error:#}"));
    }
    result
}

fn run() -> Result<()> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    match args.as_slice() {
        [] => {
            let root = env::current_exe()?
                .parent()
                .context("launcher folder")?
                .to_path_buf();
            let package = launch::Package::portable(&root);
            if package.client.is_file()
                || env::current_exe()?.file_stem() == Some(std::ffi::OsStr::new("Cinnamatica"))
            {
                launch::start(&package, &root.join("profile"))?;
            } else {
                picker::run();
            }
            Ok(())
        }
        [cmd, checkout, profile] if cmd == "start-checkout" => {
            let checkout = Path::new(checkout);
            let package = launch::Package::checkout(checkout, &profile.to_string_lossy());
            let status = launch::start(&package, &checkout.join(".local/builder-user"))?.wait()?;
            ensure!(status.success(), "Cinnamatica exited with {status}.");
            Ok(())
        }
        [cmd, mods] if cmd == "import-dialog" => launch::import_dialog(Path::new(mods)),
        [cmd, mods, inputs @ ..] if cmd == "append-library" && !inputs.is_empty() => {
            let inputs: Vec<_> = inputs.iter().map(PathBuf::from).collect();
            launch::append_imports(Path::new(mods), &inputs)
        }
        [cmd, input, output] if cmd == "refresh-component" => {
            let bytes = launch::refresh(&fs::read(input)?)?;
            fs::write(output, bytes)?;
            Ok(())
        }
        [cmd, output] if cmd == "empty-library" => {
            let template = launch::template()?;
            fs::write(output, component(&template)?)?;
            Ok(())
        }
        [cmd, input] if cmd == "convert" => {
            let output = convert(Path::new(input))?;
            println!("Wrote {}", output.display());
            Ok(())
        }
        [cmd, output, inputs @ ..] if cmd == "library" || cmd == "demo-library" => {
            let paths: Vec<_> = inputs.iter().map(PathBuf::from).collect();
            write_library(&paths, Path::new(output), cmd == "demo-library")?;
            println!("Wrote {}", Path::new(output).display());
            Ok(())
        }
        [cmd, output] if cmd == "demo" => write_blueprint(&demo(), Path::new(output)),
        [cmd, input, output] if cmd == "import" => {
            let input = Path::new(input);
            let out =
                import::import(input).with_context(|| format!("import {}", input.display()))?;
            write_blueprint(&out, Path::new(output))
        }
        [cmd, input, output] if cmd == "pack" => {
            let core = fs::read(input)?;
            fs::write(output, component(&core)?)?;
            println!(
                "Wrote Cinnamatica component {}",
                Path::new(output).display()
            );
            Ok(())
        }
        [cmd, template, input, output] if cmd == "pack-blueprint" => {
            let core = populate_template(&fs::read(template)?, &fs::read(input)?)?;
            fs::write(output, component(&core)?)?;
            println!("Wrote Cinnabar component {}", Path::new(output).display());
            Ok(())
        }
        _ => bail!(
            "usage: schematica-builder convert INPUT | library OUTPUT.wasm INPUT... | demo OUTPUT.scb | import INPUT OUTPUT.scb | pack GUEST.wasm OUTPUT.wasm | start-checkout CHECKOUT PROFILE"
        ),
    }
}
