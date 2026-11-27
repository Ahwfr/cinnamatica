//! Compile the owning mesh emitter with cached dependencies; no whole-client test build.
#![allow(dead_code)]
pub use render_model::{celestial,pack_actor_light};
mod atlas { pub use render_model::{AtlasRect,BlockEntityAtlas,TextureRef}; }
#[path = "../../crates/render-model/src/block_entity/mesh.rs"] mod mesh;
fn main() {
    let mut builder = mesh::MeshBuilder::new([16,16]);
    let ghost = (1 << 28) | (1 << 27) | 0x55_77aa;
    builder.actor_light = ghost;
    assert!(!builder.actor_lit());
    let rgb = [0.5,0.25,0.75];
    assert_eq!(&builder.gamma_tint(rgb)[..3], &celestial::rgb_to_linear(rgb));
    builder.quad(mesh::Layer::Solid, [[0.,0.,0.],[1.,0.,0.],[1.,1.,0.],[0.,1.,0.]], [0.,0.,16.,16.], [1.;4]);
    assert_eq!(builder.solid.len(),6);
    assert!(builder.solid.iter().all(|v|v.actor_light==ghost));
    assert!(builder.solid.iter().all(|v|v.normal[0]>=0. && v.normal[0]<=1. && v.normal[1]>=0. && v.normal[1]<=1.));
    builder.actor_light = pack_actor_light(8,15);
    assert!(builder.actor_lit());
    assert_eq!(&builder.gamma_tint(rgb)[..3], &rgb);
    println!("PASS: native ghost emitter retains wash/outline and quad coordinates, converts dye to linear, and keeps real entity lighting in gamma");
}
