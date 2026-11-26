//! Fast CPU validation of the production ghost shaders, using the renderer's own composer.
#![allow(dead_code)]
#[path = "../../crates/render/src/material_shader.rs"]
mod material_shader;
#[path = "../../crates/render/tests/it/support/shader_source.rs"]
mod shader_source;
fn main() {
    for name in ["chunk", "model", "liquid"] {
        let source = std::fs::read_to_string(format!("crates/render/src/{name}.wesl")).unwrap();
        for flags in [
            vec!["MOD_BLOCKS", "MOD_BLOCK_TINT", "MOD_BLOCK_OUTLINE"],
            vec![
                "MOD_BLOCKS",
                "MOD_BLOCK_TINT",
                "MOD_BLOCK_OUTLINE",
                "MOD_BLOCK_OPACITY",
            ],
            vec!["MOD_BLOCKS", "MOD_BLOCK_TINT"],
        ] {
            shader_source::composed(&source, &flags);
            println!("PASS {name} {flags:?}");
        }
    }
    let source = std::fs::read_to_string("crates/render/src/block_entity/block_entity.wesl")
        .unwrap()
        .replace(
            "BLOCK_ENTITY_VERTEX_WORDS",
            &format!("{}u", render_model::BLOCK_ENTITY_VERTEX_WORDS),
        )
        .replace(
            "BLOCK_SELECTION_VERTICES_PER_EDGE",
            &format!("{}u", render_model::BLOCK_SELECTION_VERTICES_PER_EDGE),
        );
    for flags in [vec![], vec!["ALPHA_TO_COVERAGE"]] {
        shader_source::composed(&source, &flags);
        println!("PASS entity {flags:?}");
    }
}
