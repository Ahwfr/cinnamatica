// Exercise the same named-state lookup as the client worker, not a permissive record scan.
fn main() {
    use mod_render::retained::blocks::{BlockCell, BlockGrid, BlockState};
    use schematica_core::{StateValue, blueprint};
    let input = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "outputs/Latest/Schematics/test/test.scb".into());
    let s = blueprint::decode(&std::fs::read(input).unwrap()).unwrap();
    let mut palette = vec![BlockState::Named {
        name: "minecraft:air".into(),
        states: "{}".into(),
    }];
    for b in s.palette() {
        let mut values = serde_json::Map::new();
        for (k, v) in &b.states {
            let v = match v {
                StateValue::Bool(v) => serde_json::json!({"type":"byte","value":u8::from(*v)}),
                StateValue::Byte(v) => serde_json::json!({"type":"byte","value":v}),
                StateValue::Int(v) => serde_json::json!({"type":"int","value":v}),
                StateValue::Str(v) => serde_json::json!({"type":"string","value":v}),
            };
            values.insert(k.clone(), v);
        }
        palette.push(BlockState::Named {
            name: b.name.clone(),
            states: serde_json::Value::Object(values).to_string(),
        });
    }
    let strings: usize = palette
        .iter()
        .map(|p| match p {
            BlockState::Named { name, states } => name.len() + states.len(),
            _ => 0,
        })
        .sum();
    println!(
        "{} palette entries, {} state/name bytes, native byte cap {}",
        palette.len(),
        strings,
        mod_api::drawing::MAX_BLOCK_STATE_BYTES
    );
    let size = s.size();
    let mut cells = Vec::new();
    for x in 0..size[0] {
        for z in 0..size[2] {
            for y in 0..size[1] {
                let id = s.palette_index([x, y, z]).unwrap();
                let b = &s.palette()[id as usize];
                cells.push(BlockCell {
                    palette: if b.is_air() || b.is_placement_artifact() {
                        0
                    } else {
                        id + 1
                    },
                    sky_light: Some(15),
                    block_light: Some(0),
                });
            }
        }
    }
    let g = BlockGrid {
        revision: 0,
        size: size.map(|v| v as u32),
        palette,
        cells,
    };
    let records = assets::read_registry_for_protocol(
        assets::pinned_block_registry_bytes(),
        assets::active_content_registry_protocol(),
    )
    .unwrap();
    let a = assets::RuntimeAssets::decode(
        &std::fs::read("outputs/Latest/Play/profile/data/assets/compiled/vanilla-v2225.mcbea")
            .unwrap(),
    )
    .unwrap();
    let collisions = gameplay::movement::PhysicsCollisionRegistries::from_assets(
        assets::pinned_block_registry_bytes(),
        &records,
        &std::fs::read("outputs/Latest/Play/profile/data/assets/compiled/block-physics-v2225.bin")
            .unwrap(),
        assets::active_content_registry_protocol(),
    )
    .unwrap();
    let mut failed = false;
    for mode in [
        assets::NetworkIdMode::Sequential,
        assets::NetworkIdMode::Hashed,
    ] {
        let named = |name: &str, states: &str| {
            let states = serde_json::from_str(states).ok()?;
            let found = collisions
                .block_state_runtime_id(
                    mode,
                    assets::facts::BlockStateIdentity::Named(name),
                    &states,
                )
                .or_else(|| collisions.block_default_runtime_id(mode, name));
            if found.is_none() {
                println!("Unresolved {mode:?}: {name}");
            }
            found
        };
        match g.mesh(&a, mode, named, Default::default()) {
            Ok(m) => println!(
                "Mesh {mode:?} OK: {} sections, {} triangles",
                m.sections.len(),
                m.triangles
            ),
            Err(e) => {
                failed = true;
                println!("Mesh {mode:?} FAILED: {e:?}")
            }
        }
    }
    if failed {
        std::process::exit(1);
    }
}
