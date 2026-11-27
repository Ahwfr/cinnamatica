//! Verify the generated build-equivalent table against the pinned registry and carrier.
//! Arguments: prepared asset directory, optionally an output .rs file to regenerate.
use assets::{NetworkIdMode as M, facts::PlacementRule as R};
use std::{collections::BTreeMap, path::Path};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let prepared = Path::new(args.get(1).expect("prepared asset directory"));
    let records = assets::read_registry_for_protocol(
        assets::pinned_block_registry_bytes(),
        assets::active_content_registry_protocol(),
    )
    .unwrap();
    let runtime = assets::RuntimeAssets::decode(
        &std::fs::read(prepared.join("vanilla-v2225.mcbea")).unwrap(),
    )
    .unwrap();
    let collisions = gameplay::movement::PhysicsCollisionRegistries::from_assets(
        assets::pinned_block_registry_bytes(),
        &records,
        &std::fs::read(prepared.join("block-physics-v2225.bin")).unwrap(),
        assets::active_content_registry_protocol(),
    )
    .unwrap();
    let mut rows = BTreeMap::new();
    for record in &records {
        if collisions
            .block_facts(M::Sequential, record.sequential_id)
            .placement
            != R::DoubleSlab
            || rows.contains_key(record.name.as_ref())
        {
            continue;
        }
        let single = gameplay::placement_items::single_slab_for_double(
            &collisions,
            M::Sequential,
            &record.name,
        )
        .unwrap();
        let base = single.strip_suffix("_slab").unwrap();
        // Reviewed legacy identifiers. These are checked against the registry, and
        // every newly registered double slab must resolve or this probe fails.
        let base = match base {
            "minecraft:normal_stone" => "minecraft:stone",
            "minecraft:petrified_oak" => "minecraft:oak_planks",
            "minecraft:end_stone_brick" => "minecraft:end_bricks",
            _ => base,
        };
        let candidates = [
            format!("{base}_planks"),
            base.to_owned(),
            format!("{base}s"),
            format!("{base}_block"),
        ];
        let full = candidates
            .iter()
            .filter_map(|name| collisions.block_default_runtime_id(M::Sequential, name))
            .find(|id| {
                collisions.block_is_full_cube(M::Sequential, *id)
                    || runtime
                        .resolve_known(M::Sequential, *id)
                        .is_some_and(|v| v.kind() == assets::VisualKind::Cube)
            })
            .unwrap_or_else(|| panic!("No full-block equivalent for {}", record.name));
        let name = collisions
            .block_identifier(M::Sequential, full)
            .unwrap()
            .to_owned();
        let states = collisions
            .block_canonical_state(M::Sequential, full)
            .unwrap();
        let pillar = match states {
            "{}" => false,
            r#"{"pillar_axis":{"type":"string","value":"y"}}"# => true,
            _ => panic!("Review extra properties on {name}: {states}"),
        };
        rows.insert(record.name.to_string(), (name, pillar));
    }
    if let Some(path) = args.get(2) {
        let mut output = format!(
            "// Generated for protocol {}; verify with full-block-equivalents.rs.\n&[\n",
            assets::active_content_registry_protocol()
        );
        for (source, (target, pillar)) in &rows {
            output += &format!("    ({source:?}, {target:?}, {pillar}),\n");
        }
        output += "]\n";
        std::fs::write(path, output).unwrap();
    } else {
        assert_eq!(
            rows.len(),
            schematica_core::normalization::DOUBLE_SLAB_EQUIVALENTS.len()
        );
        for (source, (target, pillar)) in &rows {
            let actual = schematica_core::normalization::for_building(
                schematica_core::BlockState::new(source),
            );
            assert_eq!(&actual.name, target, "{source}");
            assert_eq!(actual.states.contains_key("pillar_axis"), *pillar);
        }
    }
    println!(
        "PASS: {} registered double slabs map to verified full-block identities",
        rows.len()
    );
}
