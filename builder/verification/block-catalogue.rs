// Generate an isolated, complete Minecraft catalogue from the pinned registry.
// Internal registry placeholders are not blocks and must never enter a schematic.
use schematica_core::{BlockState, Schematic, StateValue, blueprint, state::DISPLAY_COLOR_STATE};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};
fn read(name: &str, json: &str) -> BlockState {
    let v: serde_json::Value = serde_json::from_str(json).unwrap();
    let mut b = BlockState::new(name);
    for (k, v) in v.as_object().unwrap() {
        let value = match v["type"].as_str().unwrap() {
            "byte" => StateValue::Byte(v["value"].as_i64().unwrap() as i8),
            "int" => StateValue::Int(v["value"].as_i64().unwrap() as i32),
            "string" => StateValue::Str(v["value"].as_str().unwrap().into()),
            other => panic!("unknown type {other}"),
        };
        b.states.insert(k.clone(), value);
    }
    b
}
fn n(v: &StateValue) -> Option<i32> {
    match v {
        StateValue::Byte(x) => Some(*x as i32),
        StateValue::Int(x) => Some(*x),
        StateValue::Bool(x) => Some(*x as i32),
        _ => None,
    }
}
fn score(b: &BlockState) -> i32 {
    b.states
        .iter()
        .map(|(k, v)| {
            let want = match k.as_str() {
                "wall_post_bit" => 1,
                "hanging" | "attached_bit" if b.name.ends_with("hanging_sign") => 1,
                _ => 0,
            };
            if k.contains("connection") {
                return match v {
                    StateValue::Str(s) => {
                        if s == "none" {
                            0
                        } else {
                            20
                        }
                    }
                    _ => n(v).unwrap_or(0).abs() * 20,
                };
            }
            match v {
                StateValue::Str(s) => {
                    if matches!(s.as_str(), "none" | "bottom" | "south" | "y" | "standing") {
                        0
                    } else {
                        1
                    }
                }
                _ => {
                    if n(v) == Some(want) {
                        0
                    } else {
                        1
                    }
                }
            }
        })
        .sum()
}
fn main() {
    let records = assets::read_registry_for_protocol(
        assets::pinned_block_registry_bytes(),
        assets::active_content_registry_protocol(),
    )
    .unwrap();
    let mut groups: BTreeMap<String, Vec<BlockState>> = BTreeMap::new();
    for r in &records {
        if !r.name.starts_with("minecraft:") {
            continue;
        }
        groups
            .entry(r.name.to_string())
            .or_default()
            .push(read(&r.name, &r.canonical_state));
    }
    let mut samples = Vec::new();
    for (name, states) in &groups {
        let mut b = states.iter().min_by_key(|b| score(b)).unwrap().clone();
        if name == "minecraft:bed" {
            b.states
                .insert(DISPLAY_COLOR_STATE.into(), StateValue::Byte(14));
        }
        if matches!(
            name.as_str(),
            "minecraft:standing_banner" | "minecraft:wall_banner"
        ) {
            b.states
                .insert(DISPLAY_COLOR_STATE.into(), StateValue::Byte(0));
        }
        samples.push(b.clone());
        if matches!(
            name.as_str(),
            "minecraft:bed" | "minecraft:standing_banner" | "minecraft:wall_banner"
        ) {
            let initial = if name == "minecraft:bed" { 14 } else { 0 };
            for dye in 0..16 {
                if dye != initial {
                    samples.push(b.clone().with(DISPLAY_COLOR_STATE, StateValue::Byte(dye)));
                }
            }
        }
    }
    // Leave the former internal-placeholder station empty, preserving every real
    // sample's position for users who already placed blocks against the first gallery.
    const FIRST_STATION: usize = 1;
    let rows = (samples.len() + FIRST_STATION + 39) / 40;
    let size = [120, 4, (rows * 3) as i32];
    let mut schematic = Schematic::new("test", size);
    let mut owners = BTreeMap::new();
    let mut manifest = Vec::new();
    for (index, b) in samples.iter().enumerate() {
        let station = index + FIRST_STATION;
        let p = [
            (station % 40 * 3 + 1) as i32,
            1,
            (station / 40 * 3 + 1) as i32,
        ];
        schematic.set(p, b.clone());
        if !b.is_air() {
            assert!(owners.insert(p, index).is_none());
        }
        let mut cells = vec![p];
        let mut pair = None;
        if b.name == "minecraft:bed" && b.states.get("head_piece_bit").and_then(n) == Some(0) {
            let q = b.states.get("direction").and_then(n).unwrap() as usize;
            let off = [[0, 0, 1], [-1, 0, 0], [0, 0, -1], [1, 0, 0]][q];
            pair = Some((std::array::from_fn(|i| p[i] + off[i]), "head_piece_bit"));
        } else if b.states.get("upper_block_bit").and_then(n) == Some(0) {
            pair = Some(([p[0], p[1] + 1, p[2]], "upper_block_bit"));
        }
        if let Some((cell, key)) = pair {
            let mut base = b.clone();
            base.states.remove(DISPLAY_COLOR_STATE);
            if let Some(part) = groups[&b.name].iter().find(|s| {
                s.states.iter().all(|(k, v)| {
                    if k == key {
                        n(v) == Some(1)
                    } else {
                        base.states.get(k) == Some(v)
                    }
                })
            }) {
                let mut part = part.clone();
                if let Some(color) = b.states.get(DISPLAY_COLOR_STATE) {
                    part.states
                        .insert(DISPLAY_COLOR_STATE.into(), color.clone());
                }
                schematic.set(cell, part);
                assert!(owners.insert(cell, index).is_none());
                cells.push(cell);
            }
        }
        manifest.push(serde_json::json!({"sample":index+1,"block":b.name,"states":b.states.iter().map(|(k,v)|(k,format!("{v:?}"))).collect::<BTreeMap<_,_>>(),"position":p,"cells":cells}));
    }
    // Prove six-direction adjacency occurs only inside the same paired sample.
    for (p, owner) in &owners {
        for d in [
            [1, 0, 0],
            [-1, 0, 0],
            [0, 1, 0],
            [0, -1, 0],
            [0, 0, 1],
            [0, 0, -1],
        ] {
            let next = std::array::from_fn(|i| p[i] + d[i]);
            if let Some(other) = owners.get(&next) {
                assert_eq!(owner, other, "different samples touch at {p:?}");
            }
        }
    }
    let encoded = blueprint::encode(&schematic).unwrap();
    let restored = blueprint::decode(&encoded).unwrap();
    let names: BTreeSet<_> = restored.palette().iter().map(|b| b.name.as_str()).collect();
    assert_eq!(names.len(), groups.len());
    for name in groups.keys() {
        assert!(names.contains(name.as_str()), "missing {name}");
    }
    assert_eq!(
        restored.layer_cells(schematica_core::LayerMode::All),
        schematic.layer_cells(schematica_core::LayerMode::All)
    );
    let directory = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "outputs/Latest/Schematics/test".into());
    let out = Path::new(&directory);
    fs::create_dir_all(out).unwrap();
    fs::write(out.join("test.scb"), &encoded).unwrap();
    fs::write(
        out.join("blocks.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    let note = format!(
        "test — all registered Minecraft block types\n\n{} Minecraft identifiers from protocol {} ({} raw registry states, internal placeholders excluded).\n{} samples including every bed/banner dye; {} palette states including paired pieces.\nSize: {} x {} x {}. Sample origins are on a 3-block grid at local Y=1.\nNo floors, ceilings or supports. Six-direction spacing permits adjacency only within one paired sample.\nAir and technical/invisible blocks are included but may not visibly render or be placeable in Survival. Every identifier is covered, not every state/NBT combination.\nSee blocks.json for local sample coordinates and states.\n",
        groups.len(),
        assets::active_content_registry_protocol(),
        records.len(),
        samples.len(),
        schematic.palette().len(),
        size[0],
        size[1],
        size[2]
    );
    fs::write(out.join("README.txt"), &note).unwrap();
    println!("{note}");
}
