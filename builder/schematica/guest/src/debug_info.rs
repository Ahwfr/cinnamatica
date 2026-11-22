//! Target info's Debug mode: a HUD panel listing the targeted planned block, the block really in
//! the world there, and Fast Place's prediction for its offer as the client works it out.

use crate::*;
use schematica_core::{BlockState, CellStatus};
use serde_json::{Map, Value, json};

/// Lines the panel holds; a longer readout is cut.
const LINES: usize = 22;
/// Characters per line before a state list wraps.
const WRAP: usize = 62;
/// Panel layout, read only when the precomputed document is regenerated.
#[cfg(test)]
const LINE_HEIGHT: f64 = 9.;
#[cfg(test)]
const WIDTH: f64 = 290.;
#[cfg(test)]
const PAD: f64 = 4.;
/// The panel's text is rebuilt on one frame in this many, so it costs the guide little.
const REFRESH_FRAMES: u32 = 4;
/// Fuel the text rebuild needs left in the frame. The guide spends the frame's fuel first; the
/// panel waits for a frame with room rather than running the mod out.
const REFRESH_FUEL: u64 = 420_000;
/// Fuel each of the per-frame reads, and the surface update, needs left. The world read and the
/// prediction are asked for every frame: the client keeps a terrain read and prepares the next
/// prediction only while the mod keeps asking.
const STEP_FUEL: u64 = 150_000;
/// The panel document, precomputed by `tests::precomputed_panel_matches_the_code` so showing it
/// builds no JSON in the frame.
const SURFACE: &str = include_str!("../ui/debug_panel.json");

/// The panel's retained surface and the world read at its target.
#[derive(Default)]
pub(crate) struct DebugPanel {
    shown: bool,
    lines: Vec<String>,
    frame: u32,
    /// The read of the target cell, and that cell.
    world: Option<([i32; 3], terrain::Region)>,
    /// The latest answer to the placement prediction read; none before the first.
    prediction: Option<Result<Option<item_use::PlacementPrediction>, String>>,
}

/// The binding one line of the panel reads.
fn binding(line: usize) -> String {
    format!("#dbg_{line}")
}

/// The panel document: a translucent card at the top left holding [`LINES`] labels.
#[cfg(test)]
fn surface() -> String {
    let mut controls = vec![json!({"card": {
        "type": "custom", "renderer": "cinnabar_rounded_rectangle",
        "anchor_from": "top_left", "anchor_to": "top_left",
        "size": [WIDTH, LINES as f64 * LINE_HEIGHT + PAD * 2.],
        "color": [0.08, 0.09, 0.11, 0.72], "radius": 2,
    }})];
    for line in 0..LINES {
        controls.push(json!({ format!("line_{line}"): {
            "type": "label", "text": binding(line),
            "bindings": [{"binding_name": binding(line)}],
            "anchor_from": "top_left", "anchor_to": "top_left",
            "offset": [PAD, PAD + line as f64 * LINE_HEIGHT],
            "size": [WIDTH - PAD * 2., LINE_HEIGHT],
            "color": [0.95, 0.95, 0.96, 1.0], "shadow": false, "localize": false,
            "font_type": "mod_panel_sans",
        }}));
    }
    let document = json!({"namespace": "hud", "root": {
        "type": "panel", "anchor_from": "top_left", "anchor_to": "top_left",
        "offset": [4, 4], "size": [WIDTH, LINES as f64 * LINE_HEIGHT + PAD * 2.],
        "controls": controls,
    }});
    let bindings: Map<String, Value> = (0..LINES).map(|line| (binding(line), json!(""))).collect();
    json!({"screen": "hud.root", "document": document.to_string(), "bindings": bindings})
        .to_string()
}

/// `minecraft:oak_fence` as `oak_fence`.
fn short(name: &str) -> &str {
    name.strip_prefix("minecraft:").unwrap_or(name)
}

/// Appends `key=value` pairs, wrapped to [`WRAP`] characters and indented.
fn push_states(out: &mut Vec<String>, block: &BlockState) {
    if block.states.is_empty() {
        out.push("    (no states)".into());
        return;
    }
    let mut line = String::from("   ");
    for (key, value) in &block.states {
        let pair = format!(" {}={value}", short(key));
        if line.len() + pair.len() > WRAP && line.len() > 3 {
            out.push(std::mem::replace(&mut line, String::from("   ")));
        }
        line.push_str(&pair);
    }
    out.push(line);
}

/// Bedrock's face numbering.
fn face_name(face: u8) -> &'static str {
    match face {
        0 => "down",
        1 => "up",
        2 => "north",
        3 => "south",
        4 => "west",
        5 => "east",
        _ => "?",
    }
}

/// Fences, panes, bars and walls, whose drawn arms depend on their neighbours.
fn connects(name: &str) -> bool {
    let name = short(name);
    (name.contains("fence") && !name.contains("gate"))
        || name.ends_with("_pane")
        || name == "glass_pane"
        || name == "iron_bars"
        || name.ends_with("_wall")
}

/// Whether a planned neighbour looks like something a fence, pane or wall would join. It is an
/// approximation from names: blocks are joined unless they are clearly not full or not solid.
fn joinable(name: &str) -> bool {
    let name = short(name);
    const LOOSE: [&str; 24] = [
        "slab",
        "stairs",
        "door",
        "sign",
        "button",
        "lever",
        "torch",
        "carpet",
        "rail",
        "sapling",
        "flower",
        "grass",
        "fern",
        "vine",
        "ladder",
        "pressure_plate",
        "banner",
        "head",
        "skull",
        "chain",
        "lantern",
        "rod",
        "candle",
        "bed",
    ];
    connects(name) || name.contains("fence_gate") || !LOOSE.iter().any(|loose| name.contains(loose))
}

/// Horizontal neighbours with their names.
const SIDES: [([i32; 3], &str); 4] = [
    ([0, 0, -1], "N"),
    ([1, 0, 0], "E"),
    ([0, 0, 1], "S"),
    ([-1, 0, 0], "W"),
];

/// The arms a selection outline reaches out with: boxes that touch a side of the cell.
fn arms(boxes: &[f64]) -> String {
    let mut sides = String::new();
    let sides_of: [(fn(&[f64]) -> bool, &str); 4] = [
        (|b| b[2] < 0.2, "N"),
        (|b| b[3] > 0.8, "E"),
        (|b| b[5] > 0.8, "S"),
        (|b| b[0] < 0.2, "W"),
    ];
    for (side, name) in sides_of {
        if boxes.chunks_exact(6).any(side) {
            sides.push_str(name);
            sides.push(' ');
        }
    }
    if sides.is_empty() {
        "none".into()
    } else {
        sides.trim_end().into()
    }
}

/// A one-cell region's primary and second layer palette indices, and whether it is loaded.
fn single_cell(region: &terrain::Region) -> (Option<usize>, Option<usize>, bool) {
    let loaded = region.loaded.first().is_some_and(|bits| bits & 1 != 0);
    let index = |slot: usize| {
        region
            .states
            .get(slot)
            .filter(|index| **index != u16::MAX)
            .map(|index| *index as usize)
    };
    let second = (region.layer_count > 1).then(|| index(1)).flatten();
    (index(0), second, loaded)
}

/// A palette entry as a block state.
fn palette_block(region: &terrain::Region, index: usize) -> Option<BlockState> {
    let state = region.palette.get(index)?;
    capture::block_from_terrain(
        state.identifier.as_deref()?,
        state.canonical_state.as_deref(),
    )
}

/// The selection boxes of a one-cell region's primary layer.
fn centre_boxes(region: &terrain::Region) -> Option<Vec<f64>> {
    let cell = 0u32;
    let shape = match region.overrides.iter().find(|o| o.slot == cell) {
        Some(over) => over.selection?,
        None => {
            let index = *region.states.get(cell as usize)? as usize;
            match region.palette.get(index)?.selection {
                terrain::ShapeBinding::Fixed(shape) => shape,
                _ => return None,
            }
        }
    };
    let terrain::ResolvedShape::Known(shape) = shape else {
        return None;
    };
    let start = *region.shape_offsets.get(shape as usize)? as usize;
    let end = *region.shape_offsets.get(shape as usize + 1)? as usize;
    region
        .shape_boxes
        .get(start * 6..end * 6)
        .map(<[f64]>::to_vec)
}

/// Lists the states that differ between two canonical state lists, as `key: plan → other`.
fn differences(planned: &BlockState, other: &BlockState) -> Vec<String> {
    let mut keys: Vec<&String> = planned.states.keys().chain(other.states.keys()).collect();
    keys.sort();
    keys.dedup();
    keys.into_iter()
        .filter(|key| planned.states.get(*key) != other.states.get(*key))
        .map(|key| {
            let value = |block: &BlockState| {
                block
                    .states
                    .get(key)
                    .map_or_else(|| "-".into(), |value| value.to_string())
            };
            format!("{}: {} -> {}", short(key), value(planned), value(other))
        })
        .collect()
}

impl Viewer {
    /// Reads Fast Place's prediction while the Debug panel is on. It runs every frame, right
    /// after the offer, because the client prepares the next prediction only while it is read.
    pub(super) fn keep_prediction(&mut self) {
        if self.options.target_info != TargetInfo::Debug || !self.options.fast_place {
            self.debug.prediction = None;
        } else if budget::remaining_fuel() >= STEP_FUEL {
            self.debug.prediction = Some(item_use::read_placement_prediction());
        }
    }

    /// Removes the panel.
    pub(super) fn hide_debug(&mut self) {
        if self.debug.shown {
            let _ = hud::set_surface("");
            self.debug = DebugPanel::default();
        }
    }

    /// Draws the panel for the targeted cell and Fast Place's offer, refreshing it on one frame
    /// in [`REFRESH_FRAMES`] that still has [`REFRESH_FUEL`] left.
    pub(super) fn show_debug(&mut self, target: Option<[i32; 3]>) {
        if !self.render_all {
            self.hide_debug();
            return;
        }
        if !self.debug.shown {
            if hud::set_surface(SURFACE).is_err() {
                return;
            }
            self.debug.shown = true;
            self.debug.lines = vec![String::new(); LINES];
            self.debug.frame = 0;
        }
        // Keep the world read alive every frame; only the text waits for a refresh frame.
        match target {
            Some(pos) => {
                let outline = self
                    .planned_block(pos)
                    .is_some_and(|block| connects(&block.name));
                self.read_debug_world(pos, outline);
            }
            None => self.debug.world = None,
        }
        self.debug.frame = self.debug.frame.wrapping_add(1);
        if self.debug.frame % REFRESH_FRAMES != 1 || budget::remaining_fuel() < REFRESH_FUEL {
            return;
        }
        let mut lines = Vec::new();
        match target {
            Some(pos) => self.target_lines(pos, &mut lines),
            None => {
                self.debug.world = None;
                lines.push("TARGET  none: aim at a planned block".into());
            }
        }
        self.prediction_lines(&mut lines);
        lines.truncate(LINES);
        lines.resize(LINES, String::new());
        if budget::remaining_fuel() < STEP_FUEL {
            return;
        }
        let changes: Map<String, Value> = lines
            .iter()
            .enumerate()
            .filter(|(line, text)| self.debug.lines.get(*line) != Some(*text))
            .map(|(line, text)| (binding(line), json!(text)))
            .collect();
        if !changes.is_empty()
            && hud::update_surface(&json!({ "binding_changes": changes }).to_string()).is_ok()
        {
            self.debug.lines = lines;
        }
    }

    /// The target, its planned block and the block really there.
    fn target_lines(&mut self, pos: [i32; 3], lines: &mut Vec<String>) {
        let [x, y, z] = pos;
        let status = self.tiles.iter().find_map(|tile| tile.status_at(pos));
        let status = match status {
            Some(CellStatus::Missing) => "missing",
            Some(CellStatus::Wrong) => "wrong block",
            Some(CellStatus::WrongState) => "wrong state",
            Some(CellStatus::Correct) => "correct",
            Some(CellStatus::Extra) => "extra",
            Some(CellStatus::Unloaded) => "not loaded",
            None => "outside the guide",
        };
        lines.push(format!("TARGET  {x}, {y}, {z}   guide: {status}"));
        let planned = self
            .planned_block(pos)
            .cloned()
            .map(|block| match self.placement {
                Some(placement) => placement.transform_state(&block),
                None => block,
            });
        match &planned {
            Some(block) => {
                lines.push(format!("PLAN  {}", short(&block.name)));
                push_states(lines, block);
            }
            None => lines.push("PLAN  nothing planned here".into()),
        }

        let region = self
            .debug
            .world
            .as_ref()
            .filter(|(cell, _)| *cell == pos)
            .map(|(_, region)| region);
        let mut world_block = None;
        match region {
            Some(region) => {
                let (primary, second, loaded) = single_cell(region);
                if !loaded {
                    lines.push("WORLD  not loaded".into());
                } else {
                    match primary.and_then(|index| palette_block(region, index)) {
                        Some(block) => {
                            lines.push(format!("WORLD  {}", short(&block.name)));
                            push_states(lines, &block);
                            world_block = Some(block);
                        }
                        None => lines.push("WORLD  unknown block".into()),
                    }
                    if let Some(block) = second.and_then(|index| palette_block(region, index)) {
                        lines.push(format!("    second layer: {}", short(&block.name)));
                    }
                }
            }
            None => lines.push("WORLD  reading...".into()),
        }

        if let (Some(plan), Some(world)) = (&planned, &world_block)
            && plan.name == world.name
        {
            let diff = differences(plan, world);
            if !diff.is_empty() {
                lines.push(format!("    differs: {}", diff.join(", ")));
            }
        }
        let fence = planned
            .as_ref()
            .or(world_block.as_ref())
            .is_some_and(|block| connects(&block.name));
        if fence {
            let plan: Vec<&str> = SIDES
                .iter()
                .filter(|(offset, _)| {
                    let side = [0, 1, 2].map(|axis| pos[axis] + offset[axis]);
                    self.planned_block(side)
                        .is_some_and(|block| joinable(&block.name))
                })
                .map(|(_, name)| *name)
                .collect();
            let plan = if plan.is_empty() {
                "none".into()
            } else {
                plan.join(" ")
            };
            let world = self
                .debug
                .world
                .as_ref()
                .and_then(|(_, region)| centre_boxes(region))
                .map_or_else(|| "?".into(), |boxes| arms(&boxes));
            lines.push(format!(
                "CONNECTS  should (approx): {plan}   world: {world}"
            ));
        }
    }

    /// Reads the cell at `pos`, with its selection outline when `outline`, keeping the last read
    /// while a new one is pending or the frame has no fuel for it.
    fn read_debug_world(&mut self, pos: [i32; 3], outline: bool) {
        let [x, y, z] = pos;
        let query = terrain::RegionQuery {
            min: terrain::Position { x, y, z },
            max: terrain::Position {
                x: x + 1,
                y: y + 1,
                z: z + 1,
            },
            geometry: if outline {
                terrain::Geometry::SELECTION
            } else {
                terrain::Geometry::empty()
            },
        };
        if budget::remaining_fuel() < STEP_FUEL {
            return;
        }
        let after = self
            .debug
            .world
            .as_ref()
            .filter(|(centre, _)| *centre == pos)
            .map(|(_, region)| region.stamp);
        match terrain::read_region(query, after) {
            Ok(terrain::ReadResult::Changed(region)) => self.debug.world = Some((pos, region)),
            Ok(terrain::ReadResult::Unchanged(_)) => {}
            Ok(terrain::ReadResult::Pending) | Err(_) => {
                if after.is_none() {
                    self.debug.world = None;
                }
            }
        }
    }

    /// Fast Place's dry run of a press on its offer.
    fn prediction_lines(&self, lines: &mut Vec<String>) {
        if !self.options.fast_place {
            lines.push("FAST PLACE  off".into());
            return;
        }
        let prediction = match &self.debug.prediction {
            Some(Ok(Some(prediction))) => prediction,
            None => {
                lines.push("FAST PLACE  waiting for the client's prediction".into());
                return;
            }
            Some(Ok(None)) => {
                lines
                    .push("FAST PLACE  not aimed at a ghost block that still needs placing".into());
                return;
            }
            Some(Err(error)) => {
                lines.push(format!("FAST PLACE  {error}"));
                return;
            }
        };
        let (x, y, z) = prediction.ghost;
        lines.push(format!(
            "FAST PLACE  ghost {x}, {y}, {z}: {}",
            prediction.verdict
        ));
        let planned = capture::block_from_terrain(&prediction.block, Some(&prediction.states));
        match &planned {
            Some(block) => {
                lines.push(format!("  builds  {}", short(&block.name)));
                push_states(lines, block);
            }
            None => lines.push(format!("  builds  {}", short(&prediction.block))),
        }
        lines.push(format!(
            "  item  {} ({})",
            prediction.item.as_deref().map_or("none", short),
            prediction.source
        ));
        let mut click = match (prediction.support, prediction.face) {
            (Some((x, y, z)), Some(face)) => {
                format!("  click  {x}, {y}, {z} face {}", face_name(face))
            }
            _ => "  click  no support".into(),
        };
        if prediction.air_click {
            click.push_str(", into air");
        }
        if prediction.sneak {
            click.push_str(", sneaking");
        }
        if let Some((pitch, yaw)) = prediction.facing {
            click.push_str(&format!(", turns to pitch {pitch:.1} yaw {yaw:.1}"));
        }
        lines.push(click);
        let predicted = prediction.predicted_block.as_deref().and_then(|name| {
            capture::block_from_terrain(name, prediction.predicted_states.as_deref())
        });
        match (&predicted, prediction.predicted_position) {
            (Some(block), Some((x, y, z))) => {
                lines.push(format!(
                    "  predicts  {} at {x}, {y}, {z}",
                    short(&block.name)
                ));
                push_states(lines, block);
                if let Some(planned) = &planned {
                    let diff = differences(planned, block);
                    if planned.name != block.name {
                        lines.push("    a different block than planned".into());
                    } else if diff.is_empty() {
                        lines.push("    matches the plan".into());
                    } else {
                        lines.push(format!("    differs: {}", diff.join(", ")));
                    }
                }
            }
            _ => lines.push("  predicts  no client prediction (the server decides)".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use schematica_core::StateValue;

    #[test]
    fn states_wrap_and_differences_name_each_changed_state() {
        let planned = BlockState::new("minecraft:oak_trapdoor")
            .with("direction", StateValue::Int(2))
            .with("open_bit", StateValue::Byte(1))
            .with("upside_down_bit", StateValue::Byte(0));
        let placed = BlockState::new("minecraft:oak_trapdoor")
            .with("direction", StateValue::Int(3))
            .with("open_bit", StateValue::Byte(1))
            .with("upside_down_bit", StateValue::Byte(0));
        assert_eq!(differences(&planned, &placed), ["direction: 2 -> 3"]);
        assert!(differences(&planned, &planned).is_empty());

        let mut lines = Vec::new();
        let mut long = BlockState::new("minecraft:stone_brick_wall");
        for side in ["east", "north", "south", "west"] {
            long = long.with(
                &format!("wall_connection_type_{side}"),
                StateValue::Str("short".into()),
            );
        }
        push_states(&mut lines, &long);
        assert!(lines.len() > 1);
        assert!(lines.iter().all(|line| line.len() <= WRAP + 40));
    }

    #[test]
    fn arms_read_the_outline_and_connections_cover_fences_panes_and_walls() {
        // A fence post with an arm to the north and one to the east.
        let boxes = [
            0.375, 0., 0.375, 0.625, 1.5, 0.625, //
            0.4375, 0.75, 0., 0.5625, 0.9375, 0.375, //
            0.625, 0.75, 0.4375, 1., 0.9375, 0.5625,
        ];
        assert_eq!(arms(&boxes), "N E");
        assert_eq!(arms(&boxes[..6]), "none");
        for name in [
            "oak_fence",
            "glass_pane",
            "red_stained_glass_pane",
            "iron_bars",
            "cobblestone_wall",
        ] {
            assert!(connects(name), "{name}");
        }
        for name in ["oak_fence_gate", "oak_planks", "wall_banner"] {
            assert!(!connects(name), "{name}");
        }
        assert!(joinable("minecraft:stone") && joinable("minecraft:oak_fence_gate"));
        assert!(!joinable("minecraft:oak_slab") && !joinable("minecraft:torch"));
    }

    #[test]
    fn precomputed_panel_matches_the_code() {
        if std::env::var_os("SCHEMATICA_BROWSER_REGENERATE").is_some() {
            let path = concat!(env!("CARGO_MANIFEST_DIR"), "/ui/debug_panel.json");
            std::fs::write(path, surface()).unwrap();
            return;
        }
        assert_eq!(SURFACE, surface(), "regenerate ui/debug_panel.json");
    }

    #[test]
    fn the_panel_document_binds_every_line() {
        let surface: Value = serde_json::from_str(&surface()).unwrap();
        let document = surface["document"].as_str().unwrap();
        for line in 0..LINES {
            assert!(document.contains(&format!("\"{}\"", binding(line))));
            assert_eq!(surface["bindings"][binding(line)], "");
        }
    }
}
