//! A dry run of a Fast Place press, for the builder's debug readout. It follows the same steps
//! as a real press (target, item, aim correction, sneak and the build prediction) through the
//! same functions, and stops before anything is sent, selected or changed.

use super::*;
use bevy::prelude::Vec3;
use mod_host::PlacementPrediction;
use semantic_input::identity::{InputAuthorityGeneration, InputSequence};

/// Predicts what a use press now would do with the current Fast Place offer.
#[allow(
    clippy::too_many_arguments,
    reason = "shares the vanilla build context and its owners"
)]
pub(super) fn preview(
    player: &client_state::PlayerRuntime,
    context: &BlockUseContext,
    runtime: &mut BlockUseRuntime,
    item_use: &mut client_interaction::item_use::ItemUseRuntime,
    movement: &MovementTicker,
    pick: Option<FramePick>,
    input_mode: PlayerInputMode,
    authority: (InputAuthorityGeneration, InputSequence),
) -> Option<PlacementPrediction> {
    let offer = context.creative.as_ref()?.offer.clone()?;
    let stream = context.client_world.stream.as_ref()?;
    let mode = stream.network_id_mode();
    // Targets are chosen for the offer as the mod made it; the build uses its full block.
    let raw = offer.clone();
    let offer = super::creative::full_block_offer(&context.collisions, stream, offer);
    let mut prediction = PlacementPrediction {
        ghost: offer.position,
        block: offer.name.clone(),
        states: offer.states.clone(),
        source: "none",
        ..Default::default()
    };
    let finish = |mut prediction: PlacementPrediction, verdict: &str| {
        prediction.verdict = verdict.to_owned();
        Some(prediction)
    };
    let Some(game_mode @ (PlayerGameMode::Creative | PlayerGameMode::Survival)) =
        player.facts.player_game_mode()
    else {
        return finish(prediction, "not in Creative or Survival");
    };
    let Some(state) = movement.build_action_state() else {
        return finish(prediction, "waiting for the movement state");
    };

    // The press's target: the crosshair hit when it is on the planned face, else the nearby
    // support Fast Place bridges to, as the held and ordered presses choose it.
    let observed = observe_use_target(
        player,
        context,
        &context.origin,
        input_mode,
        game_mode == PlayerGameMode::Survival,
        authority,
        movement.position_authority_identity().1,
        runtime,
        pick,
        ItemUseTrigger::PlayerInput,
        &state,
    );
    let native = observed.filter(|hit| super::target::on_planned_face(&raw, hit));
    let bridge = native.is_none();
    let target = match native {
        Some(native) => native,
        None => match super::target::bridge_target(
            player,
            context,
            &context.origin,
            input_mode,
            authority,
            movement.position_authority_identity().1,
            &state,
        ) {
            Ok(bridge) => bridge,
            Err(miss) => {
                let verdict = format!("{}; a click puts the item in your hand", miss.describe());
                return finish(prediction, &verdict);
            }
        },
    };
    prediction.support = Some(target.target.position);
    prediction.face = Some(target.target.face);

    let interacting = target.target.position == offer.position
        && super::orientation::needs_toggle(
            &offer,
            context
                .collisions
                .block_identifier(mode, target.target.runtime_id),
            context
                .collisions
                .block_canonical_state(mode, target.target.runtime_id),
        );
    if interacting {
        return finish(
            prediction,
            "uses the existing block to toggle it to the plan",
        );
    }
    let use_air = offer.strict_state
        && bridge
        && target.target.runtime_id == stream.air_block_id()
        && target.target.position == offer.position;
    let replacing_plant = gameplay::placement_plants::supported(&offer.name)
        && target.target.position == offer.position
        && gameplay::placement_plants::replaceable(
            context
                .collisions
                .block_facts(mode, target.target.runtime_id),
        );
    if target.target.face > 5
        || (!use_air
            && !replacing_plant
            && placement_cell(target.target.position, target.target.face) != offer.position)
    {
        return finish(
            prediction,
            "needs the support face adjoining the ghost; a click puts the item in your hand",
        );
    }

    // The item the press would select.
    let ledger = player.inventory.ledger();
    let materials = super::creative::find_materials(
        ledger,
        &context.collisions,
        stream,
        &context.ui,
        &offer,
        target.selection.slot,
    );
    let chosen = if let Some((slot, stack)) = materials.hotbar {
        prediction.source = if slot == target.selection.slot {
            "hotbar (held)"
        } else {
            "hotbar"
        };
        stack
    } else if let Some((_, stack)) = materials.carried {
        prediction.source = "inventory";
        stack
    } else if let Some((_, stack)) = materials
        .catalog
        .filter(|_| game_mode == PlayerGameMode::Creative)
    {
        prediction.source = "creative";
        stack
    } else {
        return finish(
            prediction,
            "no usable item in the hotbar, inventory or catalog",
        );
    };
    prediction.item = stream
        .authority()
        .item_identifier(chosen.network_id)
        .map(|name| name.to_string());
    let Ok(item) = protocol::VerifiedNetworkItemStack::try_new(chosen.clone(), chosen.nbt_digest)
    else {
        return finish(prediction, "the chosen item stack is not valid");
    };

    // The aim the press sends, then the target seen along it.
    let mut view = *context.view;
    let mut observed = target;
    observed.selection.item = item;
    if offer.correct_aim {
        let placement =
            super::orientation::correction(player, context, &observed, &state, &offer, bridge)
                .or_else(|| {
                    bridge.then(|| {
                        let eye = Vec3::from_array(observed.ray.origin);
                        let direction = Vec3::from_array(observed.ray.direction);
                        super::orientation::PlacementAim::observed(
                            client_presentation::camera::look_at_target(eye, eye + direction),
                            (
                                observed.target.position,
                                observed.target.face,
                                observed.target.runtime_id,
                            ),
                            direction,
                        )
                    })
                });
        if let Some(aim) = placement {
            prediction.facing = Some(super::orientation::wire_facing(aim.rotation));
            let selection = observed.selection.clone();
            let Some(mut adjusted) =
                super::orientation::recapture(player, context, observed, &state, aim)
            else {
                return finish(prediction, "the corrected aim does not reach the support");
            };
            adjusted.selection = selection;
            view.set_rotation(aim.rotation);
            observed = adjusted;
            prediction.support = Some(observed.target.position);
            prediction.face = Some(observed.target.face);
        }
    }
    let use_air = observed.target.position == offer.position
        && observed.target.runtime_id == stream.air_block_id();
    prediction.air_click = offer.strict_state && use_air;
    let mut state = state;
    if !use_air
        && context
            .collisions
            .block_facts(mode, observed.target.runtime_id)
            .traits
            .contains(assets::registry::BlockTraits::INTERACTIVE)
        && !state.sneaking
    {
        prediction.sneak = true;
        state.sneaking = true;
    }

    let clock = RepeatClock::for_state(context.time.elapsed(), &state, Some(game_mode));
    let mut build = build::BuildInput::new(
        observed,
        state,
        ItemUseTrigger::PlayerInput,
        clock.now,
        clock,
        view,
    );
    build.air_place = prediction.air_click;
    build.expected = Some(offer);
    build.report = Some(build::PredictionReport::default());
    build::dry_run(player, context, runtime, item_use, movement, &mut build);
    let report = build.report.unwrap_or_default();
    if let Some((position, block, states)) = report.predicted {
        prediction.predicted_position = Some(position);
        prediction.predicted_block = block;
        prediction.predicted_states = states;
    }
    let verdict = report
        .verdict
        .unwrap_or_else(|| "the world here is not loaded".into());
    finish(prediction, &verdict)
}
