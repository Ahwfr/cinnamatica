//! Runs the bundled converter on the library worker, away from the rendering thread.

use std::path::Path;
#[cfg(not(target_os = "ios"))]
use std::process::Command;

/// Opens the converter's native picker and waits for its atomic library publication.
#[cfg(not(target_os = "ios"))]
pub(super) fn import(mods: &Path) -> Result<bool, String> {
    let helper = std::env::var_os("CINNABAR_SCHEMATICA_IMPORTER")
        .ok_or("Start Cinnamatica from its launcher or start script to import schematics.")?;
    let mut command = Command::new(helper);
    command.arg("import-dialog").arg(mods);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    let result = command
        .output()
        .map_err(|error| format!("Could not start the schematic importer: {error}"))?;
    if !result.status.success() {
        return Err(format!(
            "Schematic import failed. {}",
            String::from_utf8_lossy(&result.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&result.stdout).contains("SCHEMATICA_IMPORTED"))
}

#[cfg(target_os = "ios")]
pub(super) fn import(mods: &Path) -> Result<bool, String> {
    let picked = crate::desktop::picker::pick_files(&crate::desktop::picker::Filter {
        title: "Import schematics",
        name: "Minecraft schematics",
        extensions: &[
            "litematic",
            "schem",
            "schematic",
            "mcstructure",
            "nbt",
            "gz",
            "scb",
        ],
        unavailable_hint: "Choose the schematic in Files or iCloud Drive.",
        max_bytes: Some(64 * 1024 * 1024),
    })?;
    if picked.is_empty() {
        return Ok(false);
    }
    let paths: Vec<_> = picked
        .iter()
        .map(|file| file.as_ref().to_path_buf())
        .collect();
    let layout = launcher::install_layout::InstallLayout::discover().map_err(|e| e.to_string())?;
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../../../../packaging/ios/runtime.json"))
            .map_err(|e| e.to_string())?;
    let template = manifest["builder_template"]
        .as_str()
        .ok_or("Missing iOS guest template path")?;
    let guest = std::fs::read(layout.resource_root.join(template)).map_err(|e| e.to_string())?;
    schematica_builder::launch::append_imports_with_template(mods, &paths, &guest)
        .map_err(|e| format!("{e:#}"))?;
    Ok(true)
}
