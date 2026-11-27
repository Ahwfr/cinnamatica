//! Built only into the iOS guest; desktop and Android keep their existing controls.
use crate::{OPEN_AFTER_LOAD, cinnabar::extension::touch};
const MENU: &str = "cinnamatica-menu";

fn button() -> touch::Button {
    touch::Button {
        id: MENU.into(),
        label: "Build".into(),
        anchor: touch::Anchor::TopRight,
        offset: touch::Point { x: 8., y: 56. },
        size: touch::Point { x: 48., y: 28. },
    }
}

pub(super) fn frame() {
    // Renew after world transitions, reactivation or a permission change. The host
    // handles safe areas, multi-touch ownership and the player's layout overrides.
    let _ = touch::set_buttons(&[button()]);
    if touch::read_presses().is_ok_and(|ids| ids.iter().any(|id| id == MENU)) {
        OPEN_AFTER_LOAD.set(true);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_menu_control_is_finger_sized_and_has_a_stable_layout_identity() {
        let button = super::button();
        assert_eq!(button.id, "cinnamatica-menu");
        assert!(button.size.x >= 24. && button.size.y >= 24.);
        assert!(button.offset.y >= 48.);
    }
}
