//! Persistent library removals preserve input files and recover on a fresh import.

use crate::{Options, Viewer};

impl Options {
    pub(super) fn select_library(&mut self, revision: u64) {
        if self.library_revision != revision {
            self.deleted_entries.clear();
            self.saved_placements.clear();
            self.library_revision = revision;
        }
    }
}

impl Viewer {
    pub(super) fn visible_entry(&self, row: usize) -> Option<usize> {
        (0..self.library.len())
            .filter(|index| !self.options.deleted_entries.contains(index))
            .nth(self.library_page * crate::browser::LIBRARY_ROWS + row)
    }

    pub(super) fn delete_entry(&mut self, row: usize) {
        let Some(index) = self.visible_entry(row) else {
            return;
        };
        self.options.deleted_entries.push(index);
        let visible = self
            .library
            .len()
            .saturating_sub(self.options.deleted_entries.len());
        self.library_page = self
            .library_page
            .min(visible.saturating_sub(1) / crate::browser::LIBRARY_ROWS);
        self.options.save();
        if self.request == Some(index) {
            self.request = None;
        }
        if self.active_entry == Some(index) {
            self.placement = None;
            self.active_entry = None;
            self.loaded = false;
            self.auto_place = false;
            self.materials.clear();
            self.dirty = true;
        }
    }
}

#[test]
fn paged_library_rows_keep_real_entry_indices_and_deleted_slots() {
    let mut viewer = crate::tests::reference_viewer();
    viewer.library = (0..10).map(|i| format!("Entry {i}")).collect();
    viewer.options.deleted_entries = vec![6];
    assert_eq!(viewer.visible_entry(7), Some(8));
    viewer.library_page = 1;
    assert_eq!(viewer.visible_entry(0), Some(9));
    assert_eq!(viewer.visible_entry(1), None);
    viewer.delete_entry(0);
    assert_eq!(viewer.options.deleted_entries, vec![6, 9]);
    assert_eq!(viewer.library_page, 0);
    assert_eq!(viewer.visible_entry(7), Some(8));
}
