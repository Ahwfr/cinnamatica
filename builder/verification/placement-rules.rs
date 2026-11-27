// Runs owning production modules against the pinned registry without a game/UI test build.
#![allow(dead_code)]
mod block_use {
    pub type BoxBounds = ([f64; 3], [f64; 3]);
}
mod placement_connections {
    use assets::facts::BlockProperty as P;
    pub const CONNECTION_STATES: &[(&str, u8)] = &[
        (P::MinecraftConnectionWest.name(), 4),
        (P::MinecraftConnectionEast.name(), 5),
        (P::MinecraftConnectionNorth.name(), 2),
        (P::MinecraftConnectionSouth.name(), 3),
    ];
}
#[path = "../../crates/assets/src/facts/item_route.rs"]
mod item_route;
#[path = "../../crates/gameplay/src/placement_facing.rs"]
mod placement_facing;
#[path = "../../crates/gameplay/src/placement_state.rs"]
mod placement_state;
fn main() {
    use serde_json::{Value, json};
    let records = assets::read_registry_for_protocol(
        assets::pinned_block_registry_bytes(),
        assets::active_content_registry_protocol(),
    )
    .unwrap();
    let facts = assets::facts::BlockFacts::new(&records).unwrap();
    let record = |name: &str| records.iter().find(|r| r.name.as_ref() == name).unwrap();
    for (item, block) in [
        ("sweet_berries", "sweet_berry_bush"),
        ("glow_berries", "cave_vines"),
        ("pitcher_pod", "pitcher_crop"),
        ("hopper", "hopper"),
        ("campfire", "campfire"),
    ] {
        let name = format!("minecraft:{block}");
        let _ = record(&name);
        assert_eq!(
            item_route::item_places_block(&format!("minecraft:{item}")),
            Some(name.as_str())
        );
    }
    for name in [
        "minecraft:bone_block",
        "minecraft:hay_block",
        "minecraft:creaking_heart",
    ] {
        let r = record(name);
        let fact = facts.get(r.sequential_id);
        for (face, axis) in ["y", "y", "z", "z", "x", "x"].into_iter().enumerate() {
            let state = placement_state::resolve_placement_state(
                fact,
                &r.canonical_state,
                placement_state::PlacementInput {
                    face: face as u8,
                    click_position: [0.5; 3],
                    yaw: 0.,
                    pitch: 0.,
                },
            )
            .unwrap();
            assert_eq!(state["pillar_axis"]["value"], json!(axis));
            if state.contains_key("deprecated") {
                assert_eq!(state["deprecated"]["value"], json!(0));
            }
        }
    }
    let player = ([0.2, 0., 0.2], [0.8, 1.8, 0.8]);
    for r in records
        .iter()
        .filter(|r| r.name.ends_with("glazed_terracotta"))
    {
        for (q, face) in [2, 5, 3, 4].into_iter().enumerate() {
            let state = placement_state::resolve_placement_state(
                facts.get(r.sequential_id),
                &r.canonical_state,
                placement_state::PlacementInput {
                    face: 1,
                    click_position: [0.5; 3],
                    yaw: q as f32 * 90.,
                    pitch: 0.,
                },
            )
            .unwrap();
            assert_eq!(state["facing_direction"]["value"], json!(face));
        }
    }
    for name in [
        "minecraft:calibrated_sculk_sensor",
        "minecraft:campfire",
        "minecraft:soul_campfire",
    ] {
        let r = record(name);
        let original = serde_json::from_str(&r.canonical_state).unwrap();
        for (q, direction) in ["south", "west", "north", "east"].into_iter().enumerate() {
            let state =
                placement_facing::named_states(name, &original, [1, 0, 0], player, q as f32 * 90.)
                    .unwrap();
            assert_eq!(
                state["minecraft:cardinal_direction"]["value"],
                json!(direction)
            );
        }
    }
    for name in [
        "minecraft:chiseled_bookshelf",
        "minecraft:beehive",
        "minecraft:bee_nest",
    ] {
        let r = record(name);
        let original = serde_json::from_str(&r.canonical_state).unwrap();
        for (q, direction) in [2, 3, 0, 1].into_iter().enumerate() {
            let state =
                placement_facing::named_states(name, &original, [1, 0, 0], player, q as f32 * 90.)
                    .unwrap();
            assert_eq!(state["direction"]["value"], json!(direction));
        }
    }
    let barrel = record("minecraft:barrel");
    let original = serde_json::from_str(&barrel.canonical_state).unwrap();
    for (q, face) in [2, 5, 3, 4].into_iter().enumerate() {
        let state = placement_facing::named_states(
            "minecraft:barrel",
            &original,
            [1, 0, 0],
            player,
            q as f32 * 90.,
        )
        .unwrap();
        assert_eq!(state["facing_direction"]["value"], Value::from(face));
    }
    let state =
        placement_facing::named_states("minecraft:barrel", &original, [1, -1, 0], player, 0.)
            .unwrap();
    assert_eq!(state["facing_direction"]["value"], json!(1));
    let measured: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("body-facing-v2225.json")).unwrap();
    assert_eq!(measured.len(), 24);
    for fixture in &measured {
        let name = fixture["block"].as_str().unwrap();
        let original = serde_json::from_str(&record(name).canonical_state).unwrap();
        let position = serde_json::from_value(fixture["position"].clone()).unwrap();
        let bounds = (
            serde_json::from_value(fixture["player_min"].clone()).unwrap(),
            serde_json::from_value(fixture["player_max"].clone()).unwrap(),
        );
        let actual = placement_facing::named_states(
            name,
            &original,
            position,
            bounds,
            fixture["yaw"].as_f64().unwrap() as f32,
        )
        .unwrap();
        assert_eq!(
            serde_json::to_value(&actual).unwrap(),
            fixture["canonical"],
            "{name}: {fixture}"
        );
        assert!(
            records.iter().filter(|r| r.name.as_ref() == name).any(|r| {
                serde_json::from_str::<serde_json::Value>(&r.canonical_state).unwrap()
                    == fixture["canonical"]
            }),
            "predicted an unregistered state"
        );
    }
    println!(
        "PASS: 24 captured stock-server placements including all 12 crafter orientations and barrel directions"
    );
    println!(
        "PASS: real-registry item routes, all bone/hay axes, measured sensor/camp/hive/bookshelf directions and body-height barrel orientation"
    );
}
