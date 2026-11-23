//! Items whose identifier differs from the block they place, from the pinned route table.

use std::{collections::HashMap, sync::OnceLock};

const SOURCE: &str = include_str!("../../data/block-item-routes-v2225.json");

#[derive(serde::Deserialize)]
struct Table {
    routes: Vec<Route>,
}

#[derive(serde::Deserialize)]
struct Route {
    identifier: String,
    block_name: String,
}

/// Every routed item and the block it places, read once from the pinned table.
fn routes() -> &'static HashMap<String, String> {
    static ROUTES: OnceLock<HashMap<String, String>> = OnceLock::new();
    ROUTES.get_or_init(|| {
        let table: Table = serde_json::from_str(SOURCE).expect("valid block item routes");
        table
            .routes
            .into_iter()
            .map(|route| (route.identifier, route.block_name))
            .collect()
    })
}

/// The block an item places when the route table names one other than the item itself, such
/// as `minecraft:oak_sign` placing `minecraft:standing_sign`. Items whose route names their
/// own identifier, and items without a route, answer `None`.
pub fn item_block_route(identifier: &str) -> Option<&'static str> {
    item_places_block(identifier).filter(|block| *block != identifier)
}

/// The block an item places according to the route table, including items such as
/// `minecraft:hopper` that place the same-named block without carrying it in their stack.
pub fn item_places_block(identifier: &str) -> Option<&'static str> {
    match identifier {
        "minecraft:sweet_berries" => Some("minecraft:sweet_berry_bush"),
        "minecraft:glow_berries" => Some("minecraft:cave_vines"),
        "minecraft:pitcher_pod" => Some("minecraft:pitcher_crop"),
        _ => routes().get(identifier).map(String::as_str),
    }
}

#[cfg(test)]
mod tests {
    use super::item_block_route;

    #[test]
    fn modern_seed_and_berry_routes_name_registered_blocks() {
        let records = crate::read_registry_for_protocol(
            crate::pinned_block_registry_bytes(),
            crate::active_content_registry_protocol(),
        )
        .unwrap();
        for (item, block) in [
            ("sweet_berries", "sweet_berry_bush"),
            ("glow_berries", "cave_vines"),
            ("pitcher_pod", "pitcher_crop"),
        ] {
            let expected = format!("minecraft:{block}");
            assert_eq!(
                item_block_route(&format!("minecraft:{item}")),
                Some(expected.as_str())
            );
            assert!(
                records
                    .iter()
                    .any(|record| record.name.as_ref() == expected)
            );
        }
    }

    #[test]
    fn sign_and_banner_items_route_to_their_standing_blocks() {
        assert_eq!(
            item_block_route("minecraft:oak_sign"),
            Some("minecraft:standing_sign")
        );
        assert_eq!(
            item_block_route("minecraft:dark_oak_sign"),
            Some("minecraft:darkoak_standing_sign")
        );
        assert_eq!(
            item_block_route("minecraft:banner"),
            Some("minecraft:standing_banner")
        );
        assert_eq!(item_block_route("minecraft:stone"), None);
    }

    #[test]
    fn same_named_routes_still_name_their_block() {
        use super::item_places_block;
        assert_eq!(item_block_route("minecraft:hopper"), None);
        assert_eq!(
            item_places_block("minecraft:hopper"),
            Some("minecraft:hopper")
        );
        assert_eq!(
            item_places_block("minecraft:campfire"),
            Some("minecraft:campfire")
        );
        assert_eq!(
            item_places_block("minecraft:wheat_seeds"),
            Some("minecraft:wheat")
        );
        assert_eq!(item_places_block("minecraft:diamond"), None);
    }
}
