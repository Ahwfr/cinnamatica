//! Report-based test partitions. Status is a testing baseline, not a server guarantee.
use super::*;

fn status(name: &str) -> &'static str {
    if name.contains("double_")
        || name.ends_with("_candle_cake")
        || matches!(
            name,
            "minecraft:closed_eyeblossom"
                | "minecraft:open_eyeblossom"
                | "minecraft:pale_moss_carpet"
                | "minecraft:daylight_detector_inverted"
                | "minecraft:crafter"
        )
    {
        return "unverified-fixes";
    }
    if name.contains("coral")
        || name.ends_with("lightning_rod")
        || name.contains("deprecated")
        || name.ends_with("command_block")
        || matches!(
            name,
            "minecraft:barrel"
                | "minecraft:crafter"
                | "minecraft:piston"
                | "minecraft:sticky_piston"
                | "minecraft:dispenser"
                | "minecraft:dropper"
                | "minecraft:conduit"
                | "minecraft:allow"
                | "minecraft:deny"
                | "minecraft:border_block"
                | "minecraft:client_request_placeholder_block"
                | "minecraft:frosted_ice"
        )
    {
        return "known-issues";
    }
    "working"
}

fn category(name: &str) -> &'static str {
    if name.contains("deprecated")
        || name.contains("command_block")
        || name.contains("placeholder")
        || matches!(
            name,
            "minecraft:allow"
                | "minecraft:deny"
                | "minecraft:border_block"
                | "minecraft:barrier"
                | "minecraft:structure_void"
        )
    {
        return "technical";
    }
    if name.ends_with("slab") || name.ends_with("stairs") {
        return "stairs-and-slabs";
    }
    if name.ends_with("fence")
        || name.ends_with("wall")
        || name.ends_with("pane")
        || name.ends_with("bars")
    {
        return "connections";
    }
    if [
        "redstone",
        "piston",
        "observer",
        "crafter",
        "dispenser",
        "dropper",
        "comparator",
        "repeater",
        "detector",
        "lever",
        "button",
        "rail",
    ]
    .iter()
    .any(|part| name.contains(part))
    {
        return "redstone-and-mechanisms";
    }
    if name.contains("chest")
        || name.contains("shulker")
        || name.contains("hopper")
        || name.contains("barrel")
    {
        return "containers";
    }
    if [
        "flower",
        "sapling",
        "leaves",
        "moss",
        "grass",
        "dripleaf",
        "vines",
        "coral",
        "mushroom",
        "crop",
        "wheat",
        "bush",
        "stem",
        "fern",
        "eyeblossom",
    ]
    .iter()
    .any(|part| name.contains(part))
    {
        return "plants-and-natural";
    }
    if [
        "door", "sign", "banner", "bed", "head", "skull", "frame", "lantern", "torch", "conduit",
        "statue", "cake", "candle", "pot",
    ]
    .iter()
    .any(|part| name.contains(part))
    {
        return "decoration-and-attachments";
    }
    "building-blocks"
}

fn paired(
    block: &BlockState,
    groups: &BTreeMap<String, Vec<BlockState>>,
    p: [i32; 3],
) -> Option<([i32; 3], BlockState)> {
    let (cell, key) = if block.states.get("head_piece_bit").and_then(n) == Some(0) {
        let q = block
            .states
            .get("direction")
            .and_then(n)
            .map(|v| v as usize)
            .or_else(|| match block.states.get("minecraft:cardinal_direction") {
                Some(StateValue::Str(v)) => ["south", "west", "north", "east"]
                    .iter()
                    .position(|d| *d == v),
                _ => None,
            })?;
        let offset = [[0, 0, 1], [-1, 0, 0], [0, 0, -1], [1, 0, 0]][q];
        (std::array::from_fn(|i| p[i] + offset[i]), "head_piece_bit")
    } else if block.name != "minecraft:pale_moss_carpet"
        && block.states.get("upper_block_bit").and_then(n) == Some(0)
    {
        ([p[0], p[1] + 1, p[2]], "upper_block_bit")
    } else {
        return None;
    };
    let mut base = block.clone();
    base.states.remove(DISPLAY_COLOR_STATE);
    let mut part = groups[&block.name]
        .iter()
        .find(|state| {
            state.states.iter().all(|(k, v)| {
                if k == key {
                    n(v) == Some(1)
                } else {
                    base.states.get(k) == Some(v)
                }
            })
        })?
        .clone();
    if let Some(color) = block.states.get(DISPLAY_COLOR_STATE) {
        part.states
            .insert(DISPLAY_COLOR_STATE.into(), color.clone());
    }
    Some((cell, part))
}

fn write_group(
    out: &Path,
    title: &str,
    samples: &[BlockState],
    groups: &BTreeMap<String, Vec<BlockState>>,
) -> Vec<serde_json::Value> {
    let mut ordered: Vec<_> = samples.iter().collect();
    ordered.sort_by(|a, b| (category(&a.name), &a.name).cmp(&(category(&b.name), &b.name)));
    let mut row = 0usize;
    let mut column = 0usize;
    let mut last = "";
    let mut positions = Vec::new();
    for block in ordered {
        let cat = category(&block.name);
        if last != cat && column > 0 {
            row += 1;
            column = 0;
        }
        last = cat;
        positions.push((block, [1 + (column * 3) as i32, 1, 1 + (row * 3) as i32]));
        column += 1;
        if column == 42 {
            column = 0;
            row += 1;
        }
    }
    let height = positions.iter().map(|(_, p)| p[2] + 3).max().unwrap_or(1);
    assert!(height <= 128, "Categorized catalogue exceeds bounds");
    let mut schematic = Schematic::new(title, [128, 4, height]);
    let mut manifest = Vec::new();
    for (block, p) in positions {
        schematic.set(p, block.clone());
        let mut cells = vec![p];
        if let Some((q, part)) = paired(block, groups, p) {
            schematic.set(q, part);
            cells.push(q);
        }
        let state_json = |block: &BlockState| -> serde_json::Value {
            block
                .states
                .iter()
                .map(|(key, value)| {
                    let value = match value {
                        StateValue::Bool(value) => {
                            serde_json::json!({"type":"byte","value":i8::from(*value)})
                        }
                        StateValue::Byte(value) => serde_json::json!({"type":"byte","value":value}),
                        StateValue::Int(value) => serde_json::json!({"type":"int","value":value}),
                        StateValue::Str(value) => {
                            serde_json::json!({"type":"string","value":value})
                        }
                    };
                    (key.clone(), value)
                })
                .collect::<serde_json::Map<_, _>>()
                .into()
        };
        let loaded = schematic.get(p).unwrap();
        manifest.push(serde_json::json!({"source_block":block.name,"source_states":state_json(block),"loaded_block":loaded.name,"loaded_states":state_json(loaded),"category":category(&block.name),"status":status(&block.name),"position":p,"cells":cells}));
    }
    fs::create_dir_all(out).unwrap();
    fs::write(
        out.join(format!("{title}.scb")),
        blueprint::encode(&schematic).unwrap(),
    )
    .unwrap();
    fs::write(
        out.join(format!("{title}.json")),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    manifest
}

pub(super) fn export(
    out: &Path,
    samples: &[BlockState],
    groups: &BTreeMap<String, Vec<BlockState>>,
) {
    let mut total = 0;
    let mut report = String::from(
        "Categorized test schematics\n\nWorking = baseline samples not in the latest failure report; not a guarantee of every state or unsupported placement.\nKnown Issues = reported failures or permission/support restrictions.\nUnverified Fixes = changed in this build and awaiting your confirmation.\nEach set is sorted into eight block-type groups; exact coordinates are in the matching JSON.\nSlabs are replaced with full blocks before Materials and rendering; source names are kept in JSON.\nCushions are entities, not block-registry samples.\n\n",
    );
    for group in ["working", "known-issues", "unverified-fixes"] {
        let selected: Vec<_> = samples
            .iter()
            .filter(|b| status(&b.name) == group)
            .cloned()
            .collect();
        total += selected.len();
        write_group(out, &format!("test-{group}"), &selected, groups);
        report += &format!("test-{group}.scb: {} samples\n", selected.len());
    }
    assert_eq!(total, samples.len());
    write_group(out, "test-all", samples, groups);
    let rotations: Vec<_> = ["minecraft:barrel", "minecraft:crafter"]
        .into_iter()
        .flat_map(|name| {
            groups[name]
                .iter()
                .filter(|block| {
                    ["open_bit", "crafting", "triggered_bit"]
                        .iter()
                        .all(|key| block.states.get(*key).and_then(n).is_none_or(|v| v == 0))
                })
                .cloned()
        })
        .collect();
    assert_eq!(rotations.len(), 18);
    write_group(out, "test-orientation", &rotations, groups);
    report += "\ntest-orientation.scb: 18 extra stations covering all six barrel directions and all twelve crafter orientations. Vertical fronts still require the measured body-height conditions; exact states are in JSON. This optional export is not added to your installed library automatically.\n";
    fs::write(out.join("README.txt"), report).unwrap();
}
