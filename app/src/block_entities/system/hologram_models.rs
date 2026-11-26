//! Native block-entity models selected from a retained grid's canonical states.
//! Block-entity NBT is not carried by BlockGrid, so these use plain/default artwork.

use super::Piece;
use assets::facts::{BlockEntityFacts, BlockEntityRenderer as R};
use render_model::{BlockEntityKind, Facing};

pub(super) fn piece(name: &str, states: &str) -> Option<Piece> {
    let entry = BlockEntityFacts::pinned().by_block_identifier(name)?;
    let state = super::super::super::state::BlockState::parse(states);
    Some(match entry.renderer {
        R::Bed => Piece::Bed(super::super::super::describe::bed(
            &state,
            state
                .int(mod_api::drawing::BLOCK_DISPLAY_COLOR_STATE)
                .filter(|color| (0..16).contains(color)),
        )?),
        R::Banner => Piece::Banner {
            wall: name
                .contains("wall_")
                .then(|| super::facing(states))
                .flatten(),
            rotation: state
                .int("ground_sign_direction")
                .unwrap_or(0)
                .rem_euclid(16) as f32
                * 22.5,
            color: state
                .int(mod_api::drawing::BLOCK_DISPLAY_COLOR_STATE)
                .and_then(|value| u8::try_from(value).ok())
                .filter(|value| *value < 16)
                .unwrap_or(0),
        },
        R::Skull => Piece::Skull(render_model::SkullModel {
            kind: render_model::SkullKind::from_block_identifier(name)?,
            mount: super::facing(states).map_or(
                render_model::SkullMount::Floor {
                    rotation_degrees: state.int("ground_sign_direction").unwrap_or(0) as f32 * 22.5,
                },
                render_model::SkullMount::Wall,
            ),
        }),
        R::DecoratedPot => Piece::Pot(super::facing(states).unwrap_or(Facing::North)),
        R::Conduit => Piece::Conduit,
        R::ItemFrame | R::GlowItemFrame => Piece::Frame(render_model::ItemFrameModel {
            glow: entry.renderer == R::GlowItemFrame,
            outward: state.int("facing_direction").unwrap_or(2).clamp(0, 5) as u8,
            full_face: ["item_frame_map_bit", "item_frame_photo_bit"]
                .iter()
                .any(|key| state.int(key).is_some_and(|value| value != 0)),
            map: None,
        }),
        // Block grids currently carry states, not tile-entity NBT. Use the actual
        // default model; unknown text, size and pose must not become an oak cube.
        R::Chalkboard => Piece::Chalkboard(render_model::ChalkboardModel {
            size: assets::block_entity_geometry::chalkboard::ChalkboardSize::Small,
            direction: state.int("direction").unwrap_or(0).rem_euclid(16) as u8,
            on_ground: true,
        }),
        R::Statue => Piece::Statue(render_model::StatueModel {
            pose: render_model::StatuePose::Standing,
            oxidation: render_model::Oxidation::from_block_name(name)?,
            facing: super::facing(states).unwrap_or(Facing::North),
        }),
        R::Bell => Piece::Bell(
            state
                .text("attachment")
                .and_then(render_model::BellAttachment::from_state)
                .unwrap_or(render_model::BellAttachment::Standing),
        ),
        _ => return None,
    })
}

impl Piece {
    pub(in super::super) fn kind(self) -> BlockEntityKind {
        match self {
            Self::Chest(model) => BlockEntityKind::Chest(model),
            Self::Shulker(model) => BlockEntityKind::Shulker(model),
            Self::Bed(model) => BlockEntityKind::Bed(model),
            Self::Skull(model) => BlockEntityKind::Skull(model),
            Self::Banner {
                wall,
                rotation,
                color,
            } => BlockEntityKind::Banner(render_model::BannerModel {
                mount: wall.map_or(
                    render_model::BannerMount::Standing {
                        rotation_degrees: rotation,
                    },
                    render_model::BannerMount::Wall,
                ),
                base: assets::banner::color_linear(i64::from(15 - color)),
                layers: std::sync::Arc::from([]),
            }),
            Self::Pot(facing) => BlockEntityKind::DecoratedPot(render_model::DecoratedPotModel {
                facing,
                sherds: Default::default(),
            }),
            Self::Conduit => BlockEntityKind::Conduit(render_model::ConduitModel {
                active: false,
                hunting: false,
                viewer_yaw_degrees: 0.,
            }),
            Self::Bell(attachment) => BlockEntityKind::Bell(render_model::BellModel {
                attachment,
                ring_direction: 0,
                seconds_since_ring: 1000.,
            }),
            Self::Frame(model) => BlockEntityKind::ItemFrame(model),
            Self::Chalkboard(model) => BlockEntityKind::Chalkboard(model),
            Self::Statue(model) => BlockEntityKind::Statue(model),
        }
    }
}
