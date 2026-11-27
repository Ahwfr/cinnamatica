//! Buildable equivalents requested by the user, applied before rendering and counting.
use crate::{BlockState, StateValue};

pub const DOUBLE_SLAB_EQUIVALENTS: &[(&str, &str, bool)] = include!("full_blocks_generated.rs");

/// Double slabs occupy one full cell. Normalize their identity, not just their
/// selected inventory item, so guides, Materials, verification and paste agree.
pub fn for_building(state: BlockState) -> BlockState {
    let Ok(index) = DOUBLE_SLAB_EQUIVALENTS.binary_search_by_key(&state.name.as_str(), |row| row.0)
    else {
        return state;
    };
    let (_, full, pillar) = DOUBLE_SLAB_EQUIVALENTS[index];
    let mut replacement = BlockState::new(full);
    if pillar {
        replacement
            .states
            .insert("pillar_axis".into(), StateValue::Str("y".into()));
    }
    replacement
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Schematic, blueprint, material_list};
    #[test]
    fn legacy_slab_libraries_load_as_full_blocks_before_material_counting() {
        assert_eq!(DOUBLE_SLAB_EQUIVALENTS.len(), 101);
        assert!(DOUBLE_SLAB_EQUIVALENTS.windows(2).all(|w| w[0].0 < w[1].0));
        for &(source, target, _) in DOUBLE_SLAB_EQUIVALENTS {
            // Bypass construction normalization to simulate an existing old library.
            let old = Schematic::from_validated(
                "legacy",
                [1; 3],
                vec![
                    BlockState::new(source)
                        .with("minecraft:vertical_half", StateValue::Str("top".into())),
                ],
                vec![0],
            );
            let loaded = blueprint::decode(&blueprint::encode(&old).unwrap()).unwrap();
            assert_eq!(loaded.get([0; 3]).unwrap().name, target, "{source}");
            let placed = crate::Placed::new(&loaded, crate::Placement::new([0; 3]));
            assert_eq!(
                material_list(&placed, None, crate::MatchMode::Exact)[0].name,
                target
            );
            assert_eq!(loaded.size(), [1; 3]);
        }
    }
    #[test]
    fn constructing_and_importing_states_use_the_same_replacements() {
        let mut s = Schematic::new("new", [1; 3]);
        s.set([0; 3], BlockState::new("minecraft:bamboo_double_slab"));
        assert_eq!(s.get([0; 3]).unwrap().name, "minecraft:bamboo_planks");
        let raw = Schematic::from_raw(
            "raw",
            [1; 3],
            vec![BlockState::new("minecraft:stone_brick_double_slab")],
            vec![0],
        )
        .unwrap();
        assert_eq!(raw.get([0; 3]).unwrap().name, "minecraft:stone_bricks");
    }
}
