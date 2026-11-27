//! Inventory-backed support placement. This never invents a clicked block or uses commands.
use super::*;
use bevy::prelude::Vec3;
use mod_host::CreativePlacementOffer as Offer;
use std::time::Duration;

use super::support_state::{Owned, Step, Waiting};

/// Documented resource preference, not an in-game market price. No falling,
/// interactive, fluid, fragile or valuable material is selected as expendable support.
fn cheap_rank(name: &str) -> Option<u8> {
    match name.strip_prefix("minecraft:")? {
        "dirt" | "coarse_dirt" => Some(0),
        "netherrack" => Some(1),
        "cobblestone" | "cobbled_deepslate" | "stone" | "deepslate" => Some(2),
        "diorite" | "andesite" | "granite" | "blackstone" | "tuff" => Some(3),
        n if n.ends_with("_planks") => Some(4),
        // Next best: other plain full blocks, so any building material can hold a block up.
        "mossy_cobblestone" | "sandstone" | "red_sandstone" | "smooth_stone" | "stone_bricks"
        | "bricks" | "mud_bricks" | "end_stone" | "terracotta" | "hardened_clay" | "clay"
        | "calcite" | "basalt" | "smooth_basalt" | "nether_bricks" | "red_nether_bricks"
        | "mud" | "moss_block" | "sandstone_smooth" => Some(5),
        n if n.ends_with("_terracotta") || n.ends_with("_concrete") || n.ends_with("_wool") => {
            Some(6)
        }
        n if n.ends_with("_log") || n.ends_with("_wood") || n.ends_with("_stem") => Some(7),
        _ => None,
    }
}

/// These supports must remain: removing them would drop/break the target. Readable
/// native survival facts supplement the audited vanilla gravity/mechanism families.
fn keep_below(name: &str) -> bool {
    matches!(
        name,
        "minecraft:sand"
            | "minecraft:red_sand"
            | "minecraft:gravel"
            | "minecraft:suspicious_sand"
            | "minecraft:suspicious_gravel"
            | "minecraft:dragon_egg"
            | "minecraft:anvil"
            | "minecraft:chipped_anvil"
            | "minecraft:damaged_anvil"
            | "minecraft:cake"
            | "minecraft:redstone_wire"
            | "minecraft:unpowered_repeater"
            | "minecraft:powered_repeater"
            | "minecraft:unpowered_comparator"
            | "minecraft:powered_comparator"
            | "minecraft:snow_layer"
            | "minecraft:bed"
            | "minecraft:straw_bed"
    ) || name.ends_with("_concrete_powder")
        || name.ends_with("_candle_cake")
        || name.ends_with("_door")
        || name.ends_with("_carpet")
        || name.ends_with("_pressure_plate")
}

/// Blocks that survive on a floor, wall or ceiling only while their attachment stays.
fn attached_stays(name: &str) -> bool {
    name.contains("wall_") || name == "minecraft:ladder" || name.contains("frame")
        || name.ends_with("amethyst_bud") || name == "minecraft:amethyst_cluster"
}

/// Heads and skulls place on their own, so they never get a support block.
fn is_head(name: &str) -> bool {
    name.ends_with("_skull")
        || matches!(
            name,
            "minecraft:player_head"
                | "minecraft:zombie_head"
                | "minecraft:creeper_head"
                | "minecraft:dragon_head"
                | "minecraft:piglin_head"
        )
}

const SOIL: &[&str] = &[
    "dirt", "grass_block", "grass", "podzol", "coarse_dirt", "dirt_with_roots", "rooted_dirt",
    "mycelium", "farmland", "moss_block", "pale_moss_block", "mud", "muddy_mangrove_roots",
];
const SAND: &[&str] = &["sand", "red_sand"];

/// The block families a plant needs beneath it, with the warning shown when they are missing.
fn soil_rule(name: &str) -> Option<(Vec<&'static str>, &'static str)> {
    let n = name.strip_prefix("minecraft:")?;
    let mut ok: Vec<&'static str> = Vec::new();
    let message = if n == "sapling" || n.ends_with("_sapling") || n == "mangrove_propagule"
        || matches!(n, "red_flower" | "yellow_flower" | "wither_rose" | "tallgrass" | "short_grass"
            | "fern" | "sweet_berry_bush" | "torchflower" | "azalea" | "flowering_azalea"
            | "double_plant" | "cornflower" | "lily_of_the_valley")
    {
        ok.extend(SOIL);
        "Needs dirt, grass or similar soil underneath"
    } else if n == "cactus" {
        ok.extend(SAND);
        "Cactus needs sand underneath"
    } else if n == "reeds" || n == "sugar_cane" {
        ok.extend(SAND);
        ok.extend(SOIL);
        "Sugar cane needs sand or soil underneath"
    } else if n == "nether_wart" {
        ok.push("soul_sand");
        "Nether wart needs soul sand underneath"
    } else if matches!(n, "wheat" | "carrots" | "potatoes" | "beetroot" | "pumpkin_stem" | "melon_stem") {
        ok.push("farmland");
        "Crops need farmland underneath"
    } else if matches!(n, "crimson_roots" | "warped_roots" | "crimson_fungus" | "warped_fungus" | "nether_sprouts") {
        ok.extend(SOIL);
        ok.extend(["crimson_nylium", "warped_nylium", "soul_soil"]);
        "Needs nylium, soul soil or soil underneath"
    } else {
        return None;
    };
    Some((ok, message))
}

fn faces(offer: &Offer) -> Option<(Vec<u8>, bool)> {
    if is_head(&offer.name) {
        return None;
    }
    if offer.name.ends_with("_hanging_sign")
        || offer.name.contains("cave_vines")
        || offer.name.contains("weeping_vines")
        || offer.name == "minecraft:hanging_roots"
    {
        // Hangs from the block above, which must stay.
        return Some((vec![0], true));
    }
    if offer.name.contains("twisting_vines") {
        return Some((vec![1], true));
    }
    if offer.name == "minecraft:candle" || offer.name.ends_with("_candle") {
        return Some((vec![1], true));
    }
    if matches!(offer.name.as_str(), "minecraft:turtle_egg" | "minecraft:sniffer_egg") {
        return Some((vec![1], true));
    }
    if matches!(offer.name.as_str(), "minecraft:standing_banner" | "minecraft:banner") {
        return Some((vec![1], true));
    }
    if offer.name.ends_with("_button") {
        // The state names the face the button is mounted on: 0 ceiling, 1 floor, 2-5 walls.
        let state: serde_json::Value = serde_json::from_str(&offer.states).ok()?;
        let face = state["facing_direction"]["value"].as_u64().filter(|f| *f < 6)? as u8;
        return Some((vec![face], true));
    }
    if offer.name.ends_with("torch") && !offer.name.contains("flower") {
        let state: serde_json::Value = serde_json::from_str(&offer.states).ok()?;
        let face = match state["torch_facing_direction"]["value"].as_str() {
            // The state names the side of the torch that holds it, so the clicked face is opposite.
            Some("north") => 3,
            Some("south") => 2,
            Some("west") => 5,
            Some("east") => 4,
            _ => 1,
        };
        return Some((vec![face], true));
    }
    if matches!(offer.name.as_str(), "minecraft:glow_lichen" | "minecraft:sculk_vein") {
        // One clicked face per attached side; any existing neighbour already holds it.
        let state: serde_json::Value = serde_json::from_str(&offer.states).ok()?;
        let bits = state["multi_face_direction_bits"]["value"].as_u64()?;
        let faces: Vec<u8> = (0..6u8).filter(|i| bits & (1 << i) != 0).map(|i| i ^ 1).collect();
        return (!faces.is_empty()).then_some((faces, true));
    }
    if offer.name == "minecraft:standing_sign" || offer.name.ends_with("_standing_sign") {
        return Some((vec![1], true));
    }
    if offer.name.ends_with("lantern") {
        let state: serde_json::Value = serde_json::from_str(&offer.states).ok()?;
        let hanging = state["hanging"]["value"].as_bool().unwrap_or(false)
            || state["hanging"]["value"].as_u64() == Some(1);
        return Some((vec![if hanging { 0 } else { 1 }], true));
    }
    if keep_below(&offer.name) {
        return Some((vec![1], true));
    }
    if let Some(face) = orientation::wanted_face(offer) {
        return Some((vec![face], attached_stays(&offer.name)));
    }
    // Logs and other pillars take their axis from the clicked face, so a sideways one needs
    // a neighbour on that axis; the support can go once the pillar stands.
    if let Ok(state) = serde_json::from_str::<serde_json::Value>(&offer.states) {
        match state["pillar_axis"]["value"].as_str() {
            Some("x") => return Some((vec![4, 5], false)),
            Some("z") => return Some((vec![2, 3], false)),
            _ => {}
        }
    }
    if orientation::planned_placement_half(offer) == Some(true) {
        return Some((vec![0, 2, 3, 4, 5], false));
    }
    if offer.name == "minecraft:conduit" || offer.name.ends_with("_coral_block") {
        return Some((vec![1, 2, 3, 4, 5, 0], false));
    }
    None
}

fn current(context: &BlockUseContext, offer: &Offer) -> Option<u32> {
    let stream = context.client_world.stream.as_ref()?;
    PaletteWorld::new(
        stream.collision_store(),
        context.collisions.registry(stream.network_id_mode()),
        stream.current_dimension(),
        Default::default(),
    )
    .primary_runtime_id(offer.position)
    .ok()
}

fn matches(context: &BlockUseContext, offer: &Offer, runtime: u32) -> bool {
    let Some(stream) = context.client_world.stream.as_ref() else {
        return false;
    };
    let mode = stream.network_id_mode();
    context.collisions.block_identifier(mode, runtime) == Some(offer.name.as_str())
        && context
            .collisions
            .block_canonical_state(mode, runtime)
            .is_some_and(|s| orientation::strict_state_matches(offer, s))
}

fn cheapest(player: &client_state::PlayerRuntime, context: &BlockUseContext) -> Option<Offer> {
    let stream = context.client_world.stream.as_ref()?;
    let mut candidates = Vec::new();
    for slot in 0..36 {
        let Some(stack) = player.inventory.ledger().backing_stack(slot) else {
            continue;
        };
        if stack.is_empty() {
            continue;
        }
        let Some(block) = super::build::fast_place_block(
            &context.collisions,
            stream,
            &context.ui,
            stack.block_runtime_id,
            || stream.authority().item_identifier(stack.network_id),
        ) else {
            continue;
        };
        let Some(name) = context
            .collisions
            .block_identifier(stream.network_id_mode(), block)
        else {
            continue;
        };
        let Some(rank) = cheap_rank(name) else {
            continue;
        };
        let Some(states) = context
            .collisions
            .block_canonical_state(stream.network_id_mode(), block)
        else {
            continue;
        };
        candidates.push((
            rank,
            std::cmp::Reverse(stack.count),
            slot >= 9,
            slot,
            name.to_owned(),
            states.to_owned(),
        ));
    }
    candidates.sort();
    candidates
        .into_iter()
        .next()
        .map(|(_, _, _, _, name, states)| Offer {
            position: [0; 3],
            name,
            states,
            correct_aim: true,
            strict_state: true,
            repeat_while_held: true,
        })
}

/// Plans the supports of one placement, including both halves of a bed and the soil check.
fn plan(
    player: &client_state::PlayerRuntime,
    context: &BlockUseContext,
    root: &Offer,
    guides: &[Offer],
    depth: usize,
    steps: &mut Vec<Step>,
) -> Result<(), &'static str> {
    if depth == 0 {
        match check_soil(context, root) {
            Ok(None) => {}
            Ok(Some(soil)) => {
                // Creative players take the missing soil from the catalogue.
                if player.facts.player_game_mode() == Some(PlayerGameMode::Creative) {
                    steps.push(Step {
                        offer: soil,
                        temporary: false,
                    });
                } else {
                    return Err("Needs the right block underneath: place soil or sand first");
                }
            }
            Err(message) => return Err(message),
        }
    }
    plan_cell(player, context, root, guides, depth, steps)?;
    if let Some(other) = bed_partner(root) {
        plan_cell(player, context, &other, guides, depth, steps)?;
    }
    Ok(())
}

/// The other half of a bed: a foot's head lies along its facing, a head's foot behind it.
fn bed_partner(root: &Offer) -> Option<Offer> {
    if root.name != "minecraft:bed" && !root.name.ends_with("_bed") {
        return None;
    }
    let state: serde_json::Value = serde_json::from_str(&root.states).ok()?;
    let direction = match state["minecraft:cardinal_direction"]["value"].as_str() {
        Some(name) => ["south", "west", "north", "east"].iter().position(|d| *d == name)? as u64,
        None => state["direction"]["value"].as_u64().or_else(|| state["direction"].as_u64())?,
    };
    let head = state["head_piece_bit"]["value"].as_bool().unwrap_or(false)
        || state["head_piece_bit"]["value"].as_u64() == Some(1);
    let [dx, dz] = [[0, 1], [-1, 0], [0, -1], [1, 0]].get(direction as usize).copied()?;
    let sign = if head { -1 } else { 1 };
    let mut other = root.clone();
    other.position = [root.position[0] + dx * sign, root.position[1], root.position[2] + dz * sign];
    Some(other)
}

/// Warns instead of placing a plant that cannot survive on the block below it.
fn check_soil(context: &BlockUseContext, root: &Offer) -> Result<Option<Offer>, &'static str> {
    let Some((allowed, message)) = soil_rule(&root.name) else {
        return Ok(None);
    };
    let Some(stream) = context.client_world.stream.as_ref() else {
        return Ok(None);
    };
    let mode = stream.network_id_mode();
    let world = PaletteWorld::new(
        stream.collision_store(),
        context.collisions.registry(mode),
        stream.current_dimension(),
        Default::default(),
    );
    let below = [root.position[0], root.position[1] - 1, root.position[2]];
    let Ok(id) = world.primary_runtime_id(below) else {
        return Ok(None);
    };
    let name = context.collisions.block_identifier(mode, id).unwrap_or("");
    let short = name.strip_prefix("minecraft:").unwrap_or(name);
    if allowed.iter().any(|ok| *ok == short) {
        return Ok(None);
    }
    // An empty cell can be filled with the rule's first placeable block (farmland cannot be placed).
    let placeable = allowed.iter().find(|name| **name != "farmland");
    match placeable {
        Some(name) if world.is_air(below).unwrap_or(false) => Ok(Some(Offer {
            position: below,
            name: format!("minecraft:{name}"),
            states: "{}".into(),
            correct_aim: true,
            strict_state: true,
            repeat_while_held: true,
        })),
        _ => Err(message),
    }
}

fn plan_cell(
    player: &client_state::PlayerRuntime,
    context: &BlockUseContext,
    root: &Offer,
    guides: &[Offer],
    depth: usize,
    steps: &mut Vec<Step>,
) -> Result<(), &'static str> {
    let Some((faces, retained)) = faces(root) else {
        return Ok(());
    };
    let stream = context
        .client_world
        .stream
        .as_ref()
        .ok_or("World is not ready")?;
    let mode = stream.network_id_mode();
    let world = PaletteWorld::new(
        stream.collision_store(),
        context.collisions.registry(mode),
        stream.current_dimension(),
        Default::default(),
    );
    for &face in &faces {
        let position = placement_cell(root.position, face ^ 1);
        if world.primary_runtime_id(position).is_ok_and(|id| {
            gameplay::placement_items::placement_anchor(
                context.collisions.block_facts(mode, id),
                &root.name,
            )
        }) {
            return Ok(());
        }
    }
    if depth >= 2 {
        return Err("This target needs more supports: place a foundation nearby first");
    }
    let creative = player.facts.player_game_mode() == Some(PlayerGameMode::Creative);
    let cheap = cheapest(player, context).or_else(|| {
        // Creative players always have dirt in the catalogue.
        creative.then(|| Offer {
            position: [0; 3],
            name: "minecraft:dirt".into(),
            states: "{}".into(),
            correct_aim: true,
            strict_state: true,
            repeat_while_held: true,
        })
    });
    for face in faces {
        let position = placement_cell(root.position, face ^ 1);
        if steps.iter().any(|step| step.offer.position == position) {
            continue;
        }
        if !world.is_air(position).unwrap_or(false) {
            continue;
        }
        let Some(ray) = context.origin.outbound_ray() else {
            return Err("Player view is not ready");
        };
        let centre = Vec3::from_array(position.map(|n| n as f32 + 0.5));
        if centre.distance_squared(ray.origin()) > 4.5 * 4.5 {
            continue;
        }
        let intended = guides.iter().find(|offer| offer.position == position);
        let mut offer = intended
            .cloned()
            .or_else(|| cheap.clone())
            .ok_or("No support blocks available: carry dirt, cobblestone, stone or planks")?;
        offer.position = position;
        offer.repeat_while_held = true;
        let materials = creative::find_materials(
            player.inventory.ledger(),
            &context.collisions,
            stream,
            &context.ui,
            &offer,
            player.selected_hotbar_slot().unwrap_or(0),
        );
        if materials.hotbar.is_none()
            && materials.carried.is_none()
            && !(player.facts.player_game_mode() == Some(PlayerGameMode::Creative)
                && materials.catalog.is_some())
        {
            return Err("The schematic's required support material is missing from your inventory");
        }
        let mut dependencies = Vec::new();
        if plan_cell(
            player,
            context,
            &offer,
            guides,
            depth + 1,
            &mut dependencies,
        )
        .is_err()
        {
            continue;
        }
        steps.extend(dependencies);
        steps.push(Step {
            offer,
            temporary: intended.is_none() && !retained,
        });
        return Ok(());
    }
    Err("No empty support position is loaded and within reach")
}

/// Advances only on actual use, fresh terrain and a server receipt. True pauses use while
/// waiting; false lets the existing confirmed inventory/aim/placement pipeline run the step.
pub(super) fn prepare(
    player: &client_state::PlayerRuntime,
    context: &mut BlockUseContext,
    held: bool,
) -> bool {
    let Some(policy) = context.creative.as_deref_mut() else {
        return false;
    };
    let Some(stream) = context.client_world.stream.as_ref() else {
        return false;
    };
    let scope = (
        stream.authority().actor_session_id(),
        stream.current_dimension(),
    );
    if policy.support.scope != Some(scope) {
        policy.support.steps.clear();
        policy.support.waiting = None;
        policy.support.owned.clear();
        policy.support.scope = Some(scope);
    }
    policy.support.owned.retain(|owned| {
        stream
            .placement_cell_receipt(owned.position)
            .is_some_and(|(serial, wire)| {
                serial == owned.serial && stream.resolve_block_network_id(wire) == owned.runtime
            })
    });
    if !held && !policy.support_continuing() {
        policy.support.blocked = false;
        return false;
    }
    if policy.support.blocked {
        return true;
    }
    let root = policy.support.root.clone();
    let guides = policy.support.guides.clone();
    let waiting = policy.support.waiting.take();
    // End the mutable policy borrow before querying the shared world/placement context.
    let _ = policy;
    if let Some(waiting) = waiting {
        let receipt = context
            .client_world
            .stream
            .as_ref()
            .and_then(|s| s.placement_cell_receipt(waiting.offer.position));
        let accepted = receipt.is_some_and(|(serial, wire)| {
            serial > waiting.serial
                && context.client_world.stream.as_ref().is_some_and(|stream| {
                    let id = stream.resolve_block_network_id(wire);
                    current(context, &waiting.offer) == Some(id)
                        && matches(context, &waiting.offer, id)
                })
        });
        if accepted {
            let (serial, wire) = receipt.unwrap();
            let id = context
                .client_world
                .stream
                .as_ref()
                .unwrap()
                .resolve_block_network_id(wire);
            let policy = context.creative.as_deref_mut().unwrap();
            if waiting.temporary && policy.support.owned.len() < 32 {
                policy.support.owned.push(Owned {
                    position: waiting.offer.position,
                    runtime: id,
                    serial,
                });
            }
            if policy
                .support
                .steps
                .first()
                .is_some_and(|step| step.offer == waiting.offer)
            {
                policy.support.steps.remove(0);
            }
            policy.offer = policy
                .support
                .steps
                .first()
                .map(|step| step.offer.clone())
                .or(root.clone());
        } else if context.time.elapsed() < waiting.expires {
            context.creative.as_deref_mut().unwrap().support.waiting = Some(waiting);
            return true;
        } else {
            fail(
                context,
                "Support placement was not confirmed by the server; try again",
            );
            return true;
        }
    }
    let Some(root) = root else {
        return false;
    };
    let Some(guides) = guides else {
        return false;
    };
    if !context.creative.as_ref().unwrap().support.authorized {
        return false;
    }
    if current(context, &root).is_some_and(|id| matches(context, &root, id)) {
        return true;
    }
    if !context.creative.as_ref().unwrap().support.steps.is_empty() {
        if context.time.elapsed() > context.creative.as_ref().unwrap().support.deadline {
            fail(
                context,
                "Placement could not finish; check target material and support access, then retry",
            );
            return true;
        }
        return false;
    }
    // A later support-dependent target may now rely on one of our earlier
    // expendable blocks. Promote that support rather than breaking the new target.
    if let Some((faces, true)) = faces(&root) {
        let positions: Vec<_> = faces
            .into_iter()
            .map(|face| placement_cell(root.position, face ^ 1))
            .collect();
        context
            .creative
            .as_deref_mut()
            .unwrap()
            .support
            .owned
            .retain(|owned| !positions.contains(&owned.position));
    }
    if context.creative.as_ref().unwrap().support.owned.len() >= 32 {
        fail(
            context,
            "Release Use to clear temporary supports before placing more",
        );
        return true;
    }
    let mut steps = Vec::new();
    if let Err(message) = plan(player, context, &root, &guides, 0, &mut steps) {
        fail(context, message);
        return true;
    }
    if !steps.is_empty() {
        steps.push(Step {
            offer: root,
            temporary: false,
        });
        let policy = context.creative.as_deref_mut().unwrap();
        policy.offer = Some(steps[0].offer.clone());
        policy.support.steps = steps;
        policy.support.deadline = context.time.elapsed() + Duration::from_secs(10);
    }
    false
}

fn fail(context: &mut BlockUseContext, message: &str) {
    if let Some(policy) = context.creative.as_deref_mut() {
        policy.support.blocked = true;
        policy.support.steps.clear();
        policy.support.waiting = None;
    }
    context.ui.push_local_chat_line(
        format!("Cinnamatica: {message}").into(),
        context.time.elapsed().as_millis() as u64,
    );
}

/// Called only after the normal packet batch was successfully admitted.
pub(super) fn sent(context: &mut BlockUseContext, expected: &Offer) {
    let Some(policy) = context.creative.as_deref_mut() else {
        return;
    };
    if policy.support.guides.is_none() {
        return;
    }
    let Some(stream) = context.client_world.stream.as_mut() else {
        return;
    };
    let serial = stream.watch_placement_cell(expected.position);
    let temporary = policy
        .support
        .steps
        .first()
        .is_some_and(|step| step.temporary && step.offer.position == expected.position);
    policy.support.waiting = Some(Waiting {
        offer: expected.clone(),
        serial,
        expires: context.time.elapsed() + Duration::from_secs(3),
        temporary,
    });
}

/// Only our unchanged, confirmed, expendable blocks are cleanup candidates. Native
/// mining still checks reach, line of sight, game mode, tool and the real break time.
pub(crate) fn cleanup_goal(
    policy: &creative::CreativePlacement,
    stream: &chunk_pipeline::WorldStream,
) -> Option<([i32; 3], u32)> {
    if !policy.support.authorized {
        return None;
    }
    if policy.support.scope
        != Some((
            stream.authority().actor_session_id(),
            stream.current_dimension(),
        ))
    {
        return None;
    }
    if !policy.support.steps.is_empty() || policy.support.waiting.is_some() {
        return None;
    }
    if policy.support.root.is_some() && policy.support.guides.is_none() {
        return None;
    }
    policy.support.owned.iter().rev().find_map(|owned| {
        stream
            .placement_cell_receipt(owned.position)
            .filter(|(serial, wire)| {
                *serial == owned.serial && stream.resolve_block_network_id(*wire) == owned.runtime
            })
            .map(|_| (owned.position, owned.runtime))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expendable_materials_exclude_falling_and_valuable_blocks() {
        for name in [
            "sand",
            "gravel",
            "oak_leaves",
            "diamond_block",
            "chest",
            "tnt",
            "glass",
        ] {
            assert_eq!(cheap_rank(&format!("minecraft:{name}")), None);
        }
        assert!(cheap_rank("minecraft:dirt") < cheap_rank("minecraft:oak_planks"));
        assert!(keep_below("minecraft:red_concrete_powder"));
        assert!(keep_below("minecraft:powered_repeater"));
        assert!(!keep_below("minecraft:blue_candle"));
        assert!(!keep_below("minecraft:oak_stairs"));
    }
}
