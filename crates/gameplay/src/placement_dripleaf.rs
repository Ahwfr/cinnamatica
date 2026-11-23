//! Small dripleaf places a two-cell plant, with dry moss/clay or submerged soil.

use crate::{
    block_use::placement_cell,
    movement::PhysicsCollisionRegistries,
    placement_prediction::{PlacementContext, PredictedPlacement},
    placement_state::{set_bit, set_value, yaw_quadrant},
};
use assets::{NetworkIdMode, facts::BlockStateIdentity};
use sim::{CollisionWorld, PaletteWorld};

pub(crate) fn predict(
    collisions: &PhysicsCollisionRegistries,
    mode: NetworkIdMode,
    held: u32,
    world: &PaletteWorld<'_>,
    context: &PlacementContext<'_>,
) -> Option<PredictedPlacement> {
    if context.input.face > 5 {
        return None;
    }
    let clicked = world.primary_runtime_id(context.clicked).ok()?;
    let position =
        if crate::placement_items::replaceable_destination(collisions.block_facts(mode, clicked)) {
            context.clicked
        } else {
            placement_cell(context.clicked, context.input.face)
        };
    let upper = placement_cell(position, 1);
    for cell in [position, upper] {
        if !context.build_height.contains(&cell[1]) {
            return None;
        }
        let existing = world.primary_runtime_id(cell).ok()?;
        if !crate::placement_items::replaceable_destination(collisions.block_facts(mode, existing))
        {
            return None;
        }
    }
    let below = world.primary_runtime_id(placement_cell(position, 0)).ok()?;
    let soil = collisions.block_identifier(mode, below)?;
    let wet = world
        .block_physics(position)
        .ok()?
        .layers
        .iter()
        .any(|layer| layer.flags.contains(sim::BlockPhysicsFlags::WATER));
    if !soil_supported(soil, wet) {
        return None;
    }
    let identity = BlockStateIdentity::Compiled(collisions.block_facts(mode, held).identity?);
    let mut states = serde_json::from_str(collisions.block_canonical_state(mode, held)?).ok()?;
    set_value(
        &mut states,
        "minecraft:cardinal_direction",
        serde_json::Value::from(
            ["north", "east", "south", "west"][yaw_quadrant(context.input.yaw)?],
        ),
    )?;
    set_bit(&mut states, "upper_block_bit", false)?;
    let lower_id = collisions.block_state_runtime_id(mode, identity, &states)?;
    set_bit(&mut states, "upper_block_bit", true)?;
    let upper_id = collisions.block_state_runtime_id(mode, identity, &states)?;
    Some(PredictedPlacement {
        position,
        block: lower_id,
        additional: vec![(upper, upper_id)],
    })
}

fn soil_supported(soil: &str, wet: bool) -> bool {
    matches!(soil, "minecraft:clay" | "minecraft:moss_block")
        || wet && crate::placement_plants::accepts_soil(soil)
}

#[cfg(test)]
mod tests {
    #[test]
    fn dry_moss_clay_and_submerged_soil_have_distinct_rules() {
        for soil in ["minecraft:clay", "minecraft:moss_block"] {
            assert!(super::soil_supported(soil, false));
        }
        assert!(!super::soil_supported("minecraft:grass_block", false));
        assert!(super::soil_supported("minecraft:grass_block", true));
        assert!(!super::soil_supported("minecraft:stone", true));
    }
}
