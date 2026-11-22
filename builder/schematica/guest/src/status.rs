use crate::*;

impl Viewer {
    pub(super) fn show_label(&mut self, message: &str) {
        if self.render_all && message != self.last_label {
            label(message);
            self.last_label = message.to_owned();
        }
    }

    pub(super) fn publish_label(&mut self, placement_cell: Option<[i32; 3]>) {
        if self.paste.is_some() {
            self.hide_debug();
            return;
        }
        if let Some((message, _)) = self.paste_notice.clone() {
            self.hide_debug();
            self.show_label(&message);
            return;
        }
        if let Some(text) = self.tool_label() {
            self.hide_debug();
            self.show_label(&text);
            return;
        }
        if self.options.target_info == TargetInfo::Off {
            self.hide_debug();
            self.show_label("");
            return;
        }
        if self.options.target_info == TargetInfo::Debug && self.placement.is_some() {
            let cell = self.label_target(placement_cell).map(|(_, pos, _)| pos);
            self.show_debug(cell);
        } else {
            self.hide_debug();
        }
        if let Some((block, [x, y, z], fast)) = self.label_target(placement_cell) {
            let name = block.name.strip_prefix("minecraft:").unwrap_or(&block.name);
            let name = name.replace('_', " ");
            let action = if fast && self.options.hold_place {
                "  |  Hold right-click"
            } else if fast {
                "  |  Right-click"
            } else {
                ""
            };
            self.show_label(&format!("{name}  |  {x}, {y}, {z}{action}"));
            return;
        }
        let pending = self
            .tiles
            .iter()
            .filter(|tile| tile.stamp.is_none())
            .count();
        let message = if let Some(p) = self.placement {
            let needed: u64 = self.tiles.iter().map(|t| t.tally.missing).sum();
            let wrong: u64 = self.tiles.iter().map(|t| t.tally.wrong).sum();
            let extra: u64 = self.tiles.iter().map(|t| t.tally.extra).sum();
            let unknown: u64 = self.tiles.iter().map(|t| t.tally.unloaded).sum();
            let layer = self.layers_text();
            let note = if !self.shown {
                "hidden".into()
            } else if let Some(error) = self.tiles.iter().find_map(|t| t.error.as_ref()) {
                error.clone()
            } else {
                format!("{pending} tiles pending")
            };
            format!(
                "{} {:?} | {} | missing {} wrong {} extra {} unknown {} | {}",
                self.schematic.name, p.origin, layer, needed, wrong, extra, unknown, note
            )
        } else {
            self.options.menu_hint(&self.schematic.name)
        };
        self.show_label(&message);
    }
}

impl Viewer {
    /// The block Target info describes: Fast Place's missing cell, else the aimed planned block.
    /// The flag is set for the Fast Place cell.
    fn label_target(
        &self,
        placement_cell: Option<[i32; 3]>,
    ) -> Option<(&schematica_core::BlockState, [i32; 3], bool)> {
        placement_cell
            .filter(|pos| self.tiles.iter().any(|tile| tile.is_missing(*pos)))
            .and_then(|pos| self.planned_block(pos).map(|block| (block, pos, true)))
            .or_else(|| self.aimed_block().map(|(block, pos)| (block, pos, false)))
    }
}

impl Options {
    pub(super) fn menu_hint(&self, title: &str) -> String {
        let menu = native_bindings::caption(native_bindings::Action::Main);
        format!("{title} | {menu} Menu")
    }
}
