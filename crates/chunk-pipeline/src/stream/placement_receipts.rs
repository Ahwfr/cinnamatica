//! A small, explicitly watched set of server block receipts. Local prediction never
//! advances it. Used by multi-step placement to avoid building on an unconfirmed block.
use super::*;

#[derive(Default)]
pub(super) struct PlacementReceipts {
    clock: u64,
    cells: Vec<(i32, [i32; 3], u64, Option<u32>)>,
}

impl PlacementReceipts {
    pub(super) fn received(&mut self, update: &BlockUpdateEvent) {
        if update.layer != 0 {
            return;
        }
        self.clock = self.clock.saturating_add(1);
        if let Some(cell) = self
            .cells
            .iter_mut()
            .find(|cell| cell.0 == update.dimension && cell.1 == update.position)
        {
            cell.2 = self.clock;
            cell.3 = Some(update.network_id);
        }
    }
}

impl WorldStream {
    /// Starts watching a real cell and returns its current receipt serial.
    pub fn watch_placement_cell(&mut self, position: [i32; 3]) -> u64 {
        let dim = self.current_dimension();
        let receipts = &mut self.placement_receipts;
        if let Some(cell) = receipts
            .cells
            .iter()
            .find(|cell| cell.0 == dim && cell.1 == position)
        {
            return cell.2;
        }
        if receipts.cells.len() == 64 {
            receipts.cells.remove(0);
        }
        receipts.cells.push((dim, position, receipts.clock, None));
        receipts.clock
    }

    /// The newest authoritative wire state received for this watched cell.
    pub fn placement_cell_receipt(&self, position: [i32; 3]) -> Option<(u64, u32)> {
        self.placement_receipts
            .cells
            .iter()
            .find(|cell| cell.0 == self.current_dimension() && cell.1 == position)
            .and_then(|cell| cell.3.map(|id| (cell.2, id)))
    }
}
