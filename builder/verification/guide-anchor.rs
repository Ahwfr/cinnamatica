//! Check the production camera-relative anchor resolver with an unloaded origin.
#![allow(dead_code)]
pub use render_scene::ActorRenderFrame;
#[path = "../../crates/render-scene/src/mod_scene/retained.rs"] mod retained;
fn main() {
    use bevy::prelude::{GlobalTransform,Vec3};
    use mod_render::retained::{Anchor,Instance,Blend,Depth,Transform,WidthMode,Join,Cap};
    let mut drawing = Instance {
        geometry: mod_api::drawing::DrawingHandle {owner:1,generation:1,slot:1}, material: None, clip: None, params: [0.;16],
        anchor: Anchor::World {dimension:0,origin:[-128.,70.,256.]},
        transform:Transform::default(),tint:[1.;4],depth:Depth::Test,blend:Blend::Premultiplied,
        width:1.,width_mode:WidthMode::World,join:Join::Bevel,cap:Cap::Butt,
    };
    let mut frame=retained::DrawingFrame{connection:1,..Default::default()};
    for x in [-128.,0.,128.,512.] {
        let camera=GlobalTransform::from_translation(Vec3::new(x,70.,256.));
        let resolved=retained::resolve(&drawing,&frame,&camera).expect("Independent guide disappeared");
        assert_eq!(resolved.transform[3][0],-128.-x);
    }
    let camera=GlobalTransform::IDENTITY;
    drawing.anchor=Anchor::Block{dimension:0,position:[-128,70,256]};
    assert!(retained::resolve(&drawing,&frame,&camera).is_none(),"Control must reproduce old unloaded-origin behavior");
    frame.loaded_blocks.insert([-128,70,256]);
    let old_transform=retained::resolve(&drawing,&frame,&camera).unwrap();
    drawing.anchor=Anchor::World{dimension:0,origin:[-128.,70.,256.]};
    assert_eq!(retained::resolve(&drawing,&frame,&camera),Some(old_transform));
    frame.dimension=1;
    assert!(retained::resolve(&drawing,&frame,&camera).is_none());
    frame.dimension=0;frame.connection=0;
    assert!(retained::resolve(&drawing,&frame,&camera).is_none());
    println!("PASS: world guide survives an unloaded origin from four camera positions; block-anchor control reproduces the old disappearance; world/session fences remain");
}
