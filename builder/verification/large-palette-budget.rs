// Standalone real-WASM regression probe; see README.md for fixture and build requirements.
use mod_host::{
    ControlEvent, GameplaySnapshot, GameplayVector3, ModGrants, ModHost, WorldObservation,
    empty_controls,
};
use mod_host::{
    GameplayFacts, TerrainBlockState, TerrainCheck, TerrainQuery, TerrainRefresh, TerrainRegion,
    TerrainScheduler, TerrainShapeBinding, TerrainSource, TerrainStamp,
};
use std::{cell::Cell, path::Path};

struct Terrain(bool);
impl TerrainSource for Terrain {
    fn check(&mut self, _: &TerrainQuery, _: Option<&TerrainRegion>) -> TerrainCheck {
        TerrainCheck::Stale { cells: 1000 }
    }
    fn sample(&mut self, q: &TerrainQuery) -> TerrainRefresh {
        let [sx, sy, sz] = [q.max.x - q.min.x, q.max.y - q.min.y, q.max.z - q.min.z];
        let len = (sx * sy * sz) as usize;
        let states = vec![u16::from(self.0); len];
        TerrainRefresh::Replace(Box::new(TerrainRegion {
            stamp: TerrainStamp {
                session: 1,
                dimension: 0,
                registry: 1,
                revision: 0,
            },
            chunks: vec![],
            layer_count: 1,
            loaded: vec![u64::MAX; len.div_ceil(64)],
            states,
            palette: ["minecraft:air", "minecraft:stone"]
                .into_iter()
                .enumerate()
                .map(|(id, name)| TerrainBlockState {
                    runtime_id: id as u32,
                    identifier: Some(name.into()),
                    canonical_state: Some("{}".into()),
                    tags: vec![],
                    physics: None,
                    collision: TerrainShapeBinding::NotRequested,
                    selection: TerrainShapeBinding::NotRequested,
                })
                .collect(),
            shape_offsets: vec![0, 0],
            shape_boxes: vec![],
            overrides: vec![],
        }))
    }
}

fn main() {
    let file = std::env::args().nth(1).unwrap();
    let source = std::fs::read(&file).unwrap();
    let mut host = ModHost::prepare_snapshot_in_session(
        Path::new(&file),
        &source,
        ModGrants {
            terrain: true,
            controls: true,
            hud: true,
            settings: true,
            world_draw: true,
            render_material: true,
            player_state: true,
            item_use: true,
            ..Default::default()
        },
        None,
        Some(mod_api::drawing::DrawingSession {
            connection: 1,
            dimension: 0,
            local: true,
            permissions: mod_api::drawing::DrawingPermissions::ALL,
        }),
    )
    .unwrap();
    host.activate_settings(None);
    let wrong = Cell::new(true);
    let tick = Cell::new(0);
    let frame = |host: &mut ModHost, gameplay, pressed: Option<&str>, button: Option<&str>| {
        let world = WorldObservation {
            session: 1,
            dimension: 0,
            world: "fixture".into(),
            position: (0., 63., 0.),
            alive: true,
            runtime_id: 1,
        };
        let snapshot = GameplaySnapshot {
            session: 1,
            dimension: 0,
            eye: GameplayVector3 {
                x: 0.,
                y: 64.,
                z: 0.,
            },
            yaw: 0.,
            pitch: 0.,
            attack_held: false,
            frame_seconds: 0.016,
        };
        host.observe_presentation(Some(world), None, None, Vec::new())
            .unwrap();
        let mut keys = empty_controls();
        keys.focused = true;
        keys.gameplay = gameplay;
        keys.panel_open = host.panel_open();
        if let Some(key) = pressed {
            keys.keys_pressed.push(key.into());
            keys.keys_held.push(key.into());
        }
        if let Some(id) = button {
            keys.events.push(ControlEvent {
                id: id.into(),
                value: 1.,
            });
        }
        host.observe_gameplay(GameplayFacts {
            block_target: wrong.get().then_some(GameplayVector3 {
                x: -1.5,
                y: 62.5,
                z: -6.5,
            }),
            ..Default::default()
        })
        .unwrap();
        TerrainScheduler::default().refresh(&mut [host], 10000, &mut Terrain(wrong.get()));
        host.frame_with_controls(false, Some(snapshot), keys)
            .unwrap_or_else(|e| panic!("frame {} button {:?}: {e:?}", tick.get() + 1, button));
        tick.set(tick.get() + 1);
        assert!(
            !host.is_recovering(),
            "guest ran out of fuel at frame {} button {:?}",
            tick.get(),
            button
        );
    };
    for _ in 0..14 {
        frame(&mut host, true, None, None);
    }
    frame(&mut host, true, Some("KeyM"), None);
    frame(&mut host, true, None, None);
    assert!(host.take_panel_open_request());
    host.set_panel_open(true);
    frame(&mut host, false, None, Some("bw-tab-import"));
    frame(&mut host, false, None, Some("toggle-0"));
    host.set_panel_open(false);
    let mut large = false;
    for _ in 0..400 {
        frame(&mut host, true, None, None);
        for resources in &host.render().0.retained {
            for geometry in resources.geometry.values() {
                if let Some(blocks) = geometry.blocks() {
                    large |= blocks.grid.size.iter().copied().product::<u32>() >= 10_000;
                    blocks.accept_status(blocks.grid.revision, Ok(()));
                }
            }
        }
    }
    host.set_panel_open(true);
    frame(&mut host, false, None, Some("bw-tab-materials"));
    assert!(large, "Fixture did not load a large schematic");
    for _ in 0..600 {
        frame(&mut host, false, None, None);
    }
    assert_eq!(
        format!(
            "{:?}",
            host.panel()
                .unwrap()
                .surface
                .bindings
                .get("#bw_row0_visible")
        ),
        "Some(Bool(true))",
        "Materials never populated"
    );
    assert!(
        host.item_highlights()
            .is_some_and(|items| items.len() >= 800),
        "Wide fixture highlights did not finish"
    );
    assert!(host.panel_open(), "Materials closed the panel");
    assert!(host.is_active() && !host.is_recovering());
    host.set_panel_open(false);
    for _ in 0..3000 {
        frame(&mut host, true, None, None);
    }
    println!(
        "Large-guide verification and Materials callbacks completed without restarting the guest."
    );
}
