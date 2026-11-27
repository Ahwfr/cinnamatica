//! Shared bounded schematic conversion and component publication, callable in-process on iOS.
pub mod import;
pub mod launch;
#[cfg(not(target_os = "ios"))]
#[cfg_attr(windows, path = "picker/windows.rs")]
#[cfg_attr(not(windows), path = "picker/unix.rs")]
pub mod picker;
use anyhow::{Context, Result, ensure};
use schematica_core::{BlockState, Schematic, blueprint};
use std::{
    borrow::Cow,
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};
use wasm_encoder::{ComponentNameSection, CustomSection, Encode};
pub fn convert(input: &Path) -> Result<PathBuf> {
    convert_selection(&[input.to_path_buf()])
}

pub fn convert_selection(inputs: &[PathBuf]) -> Result<PathBuf> {
    let first = inputs.first().context("choose at least one schematic")?;
    let output = if inputs.len() == 1 {
        first.with_extension("cinnabar.wasm")
    } else {
        first.with_file_name("schematica-library.cinnabar.wasm")
    };
    write_library(inputs, &output, false)?;
    Ok(output)
}

pub fn write_library(inputs: &[PathBuf], output: &Path, include_demo: bool) -> Result<()> {
    let count = inputs.len() + usize::from(include_demo);
    ensure!(
        (1..=blueprint::MAX_LIBRARY).contains(&count),
        "choose 1–{} schematics",
        blueprint::MAX_LIBRARY
    );
    let template_path = env::var_os("SCHEMATICA_GUEST_TEMPLATE")
        .map(PathBuf::from)
        .unwrap_or(env::current_exe()?.with_file_name("schematica-guest-template.bin"));
    let template = fs::read(&template_path).with_context(|| {
        format!(
            "read the matching guest template at {}",
            template_path.display()
        )
    })?;
    let mut entries = Vec::with_capacity(count);
    if include_demo {
        entries.push(encode_blueprint(&demo())?);
    }
    add_inputs(&mut entries, inputs)?;
    let revision = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos() as u64;
    let bytes = encode_component(&template, &entries, revision)?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .with_context(|| {
            format!(
                "create {} (choose another filename if this output already exists)",
                output.display()
            )
        })?;
    file.write_all(&bytes)?;
    Ok(())
}

fn add_inputs(entries: &mut Vec<Vec<u8>>, inputs: &[PathBuf]) -> Result<()> {
    ensure!(
        entries.len() + inputs.len() <= blueprint::MAX_LIBRARY,
        "this library can hold at most {} schematic files",
        blueprint::MAX_LIBRARY
    );
    for input in inputs {
        let schematic =
            import::import(input).with_context(|| format!("import {}", input.display()))?;
        entries.push(encode_blueprint(&schematic)?);
    }
    Ok(())
}

fn encode_component(template: &[u8], entries: &[Vec<u8>], revision: u64) -> Result<Vec<u8>> {
    let payload =
        blueprint::encode_library_revision(entries, revision).map_err(anyhow::Error::msg)?;
    component(&populate_template(template, &payload)?)
}

pub fn populate_template(template: &[u8], payload: &[u8]) -> Result<Vec<u8>> {
    let empty = blueprint::embedded_slot(&[]).map_err(anyhow::Error::msg)?;
    let mut offset = None;
    for section in wasmparser::Parser::new(0).parse_all(template) {
        if let wasmparser::Payload::DataSection(data) = section? {
            for segment in data {
                let segment = segment?;
                if !matches!(segment.kind, wasmparser::DataKind::Active { .. }) {
                    continue;
                }
                for (index, _) in
                    segment
                        .data
                        .windows(empty.len())
                        .enumerate()
                        .filter(|(_, bytes)| {
                            bytes.starts_with(blueprint::SLOT_MARKER) && *bytes == empty
                        })
                {
                    let position = segment.range.end - segment.data.len() + index;
                    ensure!(
                        offset.replace(position).is_none(),
                        "guest template contains duplicate blueprint slots"
                    );
                }
            }
        }
    }
    let offset = offset.context("guest template has no empty blueprint slot")?;
    let mut core = template.to_vec();
    let slot = blueprint::embedded_slot(payload).map_err(anyhow::Error::msg)?;
    core[offset..offset + slot.len()].copy_from_slice(&slot);
    Ok(core)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_encoder::{ConstExpr, DataSection, Module};

    #[test]
    fn fills_only_the_active_wasm_data_slot() -> Result<()> {
        let empty = blueprint::embedded_slot(&[]).unwrap();
        let payload = encode_blueprint(&demo())?;
        let mut data = DataSection::new();
        data.active(0, &ConstExpr::i32_const(0), empty.clone());
        let mut template = Module::new();
        template.section(&CustomSection {
            name: Cow::Borrowed("decoy"),
            data: Cow::Borrowed(&empty),
        });
        template.section(&data);
        let patched = populate_template(&template.finish(), &payload)?;
        for section in wasmparser::Parser::new(0).parse_all(&patched) {
            match section? {
                wasmparser::Payload::CustomSection(section) if section.name() == "decoy" => {
                    assert!(
                        blueprint::embedded_payload(section.data())
                            .unwrap()
                            .is_empty()
                    );
                }
                wasmparser::Payload::DataSection(section) => {
                    let segment = section.into_iter().next().unwrap()?;
                    assert_eq!(
                        blueprint::embedded_payload(segment.data),
                        Some(payload.as_slice())
                    );
                }
                _ => {}
            }
        }
        Ok(())
    }
}

pub fn component(core: &[u8]) -> Result<Vec<u8>> {
    let build_version = guest_version(core)?.unwrap_or("legacy");
    let name = match guest_revision(core)? {
        Some(revision) => format!("Cinnamatica v{build_version} [{}]", &revision[..7]),
        None => "Cinnamatica [unstamped]".to_owned(),
    };
    let mut component = wit_component::ComponentEncoder::default()
        .module(core)?
        .validate(true)
        .encode()?;
    let section = CustomSection {
        name: Cow::Borrowed("cinnabar:grants"),
        data: Cow::Borrowed(include_bytes!("../../guest/grants.json")),
    };
    component.push(0);
    section.encode(&mut component);
    let mut names = ComponentNameSection::new();
    names.component(&name);
    component.push(0);
    names.encode(&mut component);
    if let Some(revision) = guest_revision(core)? {
        let version = format!("{build_version}+{}", &revision[..7]);
        component.push(0);
        CustomSection {
            name: Cow::Borrowed("version"),
            data: Cow::Borrowed(version.as_bytes()),
        }
        .encode(&mut component);
    }
    Ok(component)
}

fn guest_version(core: &[u8]) -> Result<Option<&str>> {
    for section in wasmparser::Parser::new(0).parse_all(core) {
        if let wasmparser::Payload::CustomSection(section) = section?
            && section.name() == "schematica:build-version"
        {
            let version = std::str::from_utf8(section.data())?;
            let parts: Vec<_> = version.split('.').collect();
            ensure!(
                parts.len() == 3
                    && parts
                        .iter()
                        .all(|part| !part.is_empty() && part.bytes().all(|ch| ch.is_ascii_digit())),
                "invalid guest release version"
            );
            return Ok(Some(version));
        }
    }
    Ok(None)
}

/// Reads the build stamp from the guest template, independent of the importer's version.
fn guest_revision(core: &[u8]) -> Result<Option<&str>> {
    for section in wasmparser::Parser::new(0).parse_all(core) {
        if let wasmparser::Payload::CustomSection(section) = section?
            && section.name() == "schematica:build-revision"
        {
            let revision = std::str::from_utf8(section.data())?;
            if revision.is_empty() {
                return Ok(None);
            }
            ensure!(
                revision.len() == 40 && revision.bytes().all(|b| b.is_ascii_hexdigit()),
                "invalid schematic guest build revision"
            );
            return Ok(Some(revision));
        }
    }
    Ok(None)
}

pub fn write_blueprint(schematic: &Schematic, output: &Path) -> Result<()> {
    let bytes = encode_blueprint(schematic)?;
    fs::write(output, bytes)?;
    println!(
        "{}: {:?}, {} blocks, {} palette entries",
        schematic.name,
        schematic.size(),
        schematic.block_count(),
        schematic.palette().len()
    );
    Ok(())
}

fn encode_blueprint(schematic: &Schematic) -> Result<Vec<u8>> {
    ensure!(
        schematic.block_count() > 0,
        "schematic contains no non-air blocks"
    );
    blueprint::encode(schematic).map_err(anyhow::Error::msg)
}

pub fn demo() -> Schematic {
    let mut out = Schematic::new("Demo arch", [5, 4, 2]);
    let stone = BlockState::new("minecraft:stone");
    let glass = BlockState::new("minecraft:glass");
    for x in 0..5 {
        for y in 0..4 {
            if x == 0 || x == 4 || y == 3 {
                out.set([x, y, 0], stone.clone());
            }
        }
    }
    out.set([2, 3, 1], glass);
    out
}
