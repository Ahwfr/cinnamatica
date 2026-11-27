//! Candle use is an in-cell cake interaction, not adjacent block placement.
use assets::{
    facts::{BlockFact, BlockRenderKind},
    registry::BlockTraits,
};

pub(super) fn uses_cake(
    clicked: BlockFact,
    held: BlockFact,
    canonical: Option<&str>,
    holding: bool,
    can_build: bool,
) -> bool {
    holding
        && can_build
        && clicked.render.kind == BlockRenderKind::Cake
        && held.traits.contains(BlockTraits::CANDLE)
        && canonical.is_some_and(|canonical| {
            serde_json::from_str::<serde_json::Value>(canonical)
                .is_ok_and(|state| state["bite_counter"]["value"].as_u64() == Some(0))
        })
}
