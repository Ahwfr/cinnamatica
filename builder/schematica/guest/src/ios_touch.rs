//! iOS-only retained HUD controls; the host layout editor owns their final positions.
use crate::{OPEN_AFTER_LOAD, cinnabar::extension::touch};
use std::cell::{Cell, RefCell};
const MENU: &str = "cinnamatica-menu";

/// Stable action ids, HUD labels and configuration titles, in grid order.
pub(super) const ACTIONS: [(&str, &str, &str); 15] = [
    ("previous-layer", "Layer -", "Previous layer"),
    ("next-layer", "Layer +", "Next layer"),
    ("mode", "Layers", "Cycle layer mode"),
    ("move-forward", "Forward", "Move forward"),
    ("move-back", "Back", "Move backward"),
    ("move-left", "Left", "Move left"),
    ("move-right", "Right", "Move right"),
    ("move-up", "Up", "Move up"),
    ("move-down", "Down", "Move down"),
    ("ccw", "Turn left", "Rotate left 90 degrees"),
    ("cw", "Turn right", "Rotate right 90 degrees"),
    ("mirror", "Mirror", "Mirror schematic"),
    ("visibility", "Ghosts", "Show or hide guide"),
    ("place", "Here", "Move guide here"),
    ("lock", "Lock", "Lock guide position"),
];
const ALL: u16 = (1 << ACTIONS.len()) - 1;
thread_local! {
    static ENABLED: Cell<u16> = const { Cell::new(7) };
    static PUBLISHED: Cell<Option<u16>> = const { Cell::new(None) };
    static PRESSES: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static MAINTENANCE: Cell<u8> = const { Cell::new(0) };
    static RENEW: Cell<u8> = const { Cell::new(0) };
}
pub(super) fn mask() -> u16 {
    ENABLED.get()
}
pub(super) fn load(value: Option<u64>) {
    ENABLED.set(value.map_or(7, |v| v as u16 & ALL));
}
pub(super) fn enabled(index: usize) -> bool {
    mask() & (1 << index) != 0
}
pub(super) fn set(index: usize, on: bool) {
    if index < ACTIONS.len() {
        ENABLED.set(if on {
            mask() | (1 << index)
        } else {
            mask() & !(1 << index)
        });
    }
}
fn buttons(mask: u16) -> Vec<touch::Button> {
    let mut buttons = vec![touch::Button {
        id: MENU.into(),
        label: "Build".into(),
        anchor: touch::Anchor::TopRight,
        offset: touch::Point { x: 8., y: 56. },
        size: touch::Point { x: 48., y: 28. },
    }];
    for (index, (action, label, _)) in ACTIONS.iter().enumerate() {
        if mask & (1 << index) != 0 {
            let slot = buttons.len() - 1;
            buttons.push(touch::Button {
                id: format!("cinnamatica-{action}"),
                label: (*label).into(),
                anchor: touch::Anchor::TopRight,
                offset: touch::Point {
                    x: 8. + (slot % 3) as f32 * 54.,
                    y: 90. + (slot / 3) as f32 * 34.,
                },
                size: touch::Point { x: 50., y: 28. },
            });
        }
    }
    buttons
}

pub(super) fn frame() {
    let mask = mask();
    // Occasional renewal also recovers a grant revoked and then restored without
    // reinitializing the component. Retained buttons need no per-frame allocation.
    RENEW.set(RENEW.get().saturating_add(1));
    if (PUBLISHED.get() != Some(mask) || RENEW.get() >= 120)
        && touch::set_buttons(&buttons(mask)).is_ok()
    {
        PUBLISHED.set(Some(mask));
        RENEW.set(0);
    }
    if let Ok(ids) = touch::read_presses() {
        for id in ids {
            if id == MENU {
                OPEN_AFTER_LOAD.set(true);
            } else if let Some(action) = id.strip_prefix("cinnamatica-") {
                if ACTIONS
                    .iter()
                    .enumerate()
                    .any(|(i, (id, _, _))| enabled(i) && *id == action)
                {
                    PRESSES.with(|p| {
                        let mut p = p.borrow_mut();
                        if p.len() < 16 {
                            p.push(action.into());
                        }
                    });
                }
            }
        }
    }
}
pub(super) fn take_actions() -> Vec<String> {
    PRESSES.with(|p| std::mem::take(&mut *p.borrow_mut()))
}

/// Retain existing geometry while a held gesture reserves callbacks for placement.
pub(super) fn defer_background(holding: bool) -> bool {
    if !holding {
        MAINTENANCE.set(0);
        return false;
    }
    let phase = (MAINTENANCE.get() + 1) % 4;
    MAINTENANCE.set(phase);
    phase != 0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_actions_fit_the_host_slots_with_unique_stable_ids() {
        let buttons = buttons(ALL);
        assert_eq!(buttons.len(), 16);
        let ids: std::collections::BTreeSet<_> = buttons.iter().map(|b| &b.id).collect();
        assert_eq!(ids.len(), buttons.len());
        assert!(buttons.iter().all(|b| b.id.len() <= 32 && b.size.y >= 28.));
    }
    #[test]
    fn toggles_hide_buttons_without_changing_other_identities() {
        load(Some(0));
        set(4, true);
        assert_eq!(buttons(mask())[1].id, "cinnamatica-move-back");
        set(4, false);
        assert_eq!(buttons(mask()).len(), 1);
        load(None);
        assert_eq!(mask(), 7);
    }
    #[test]
    fn held_placement_keeps_periodic_maintenance_and_release_resumes_it() {
        assert!(!defer_background(false));
        assert_eq!(
            (0..4).map(|_| defer_background(true)).collect::<Vec<_>>(),
            [true, true, true, false]
        );
        assert!(!defer_background(false));
    }
    #[test]
    fn menu_toggle_updates_the_hud_configuration_and_binding() {
        load(Some(0));
        let mut viewer = crate::tests::reference_viewer();
        viewer.browser_event("bw-tab-touch", 1.);
        viewer.browser_event("bw-set-touch-4", 1.);
        let mut controls = Vec::new();
        crate::browser::bindings(&viewer, &mut serde_json::Map::new(), &mut controls);
        assert!(
            controls
                .iter()
                .any(|v| v["id"] == "bw-set-touch-4" && v["value"] == true)
        );
        assert_eq!(buttons(mask())[1].id, "cinnamatica-move-back");
        viewer.browser_event("bw-set-touch-4", 0.);
        assert_eq!(buttons(mask()).len(), 1);
    }
}
