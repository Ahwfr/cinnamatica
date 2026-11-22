//! The committed Fast Place requests of one mod and the client's prediction delivered to it.

use super::Instance;

impl Instance {
    pub(crate) fn placement_locked(&self) -> bool {
        self.store.data().item_use_policy.locked
    }

    pub(crate) fn creative_placement_offer(&self) -> Option<&crate::CreativePlacementOffer> {
        self.is_active()
            .then(|| self.store.data().item_use_policy.placement.as_ref())
            .flatten()
    }

    /// Whether the mod read the placement prediction in its last committed frame.
    pub(crate) fn placement_prediction_wanted(&self) -> bool {
        self.is_active() && self.store.data().item_use_policy.prediction_wanted
    }

    /// Delivers the client's latest dry run for the next frame's reads.
    pub(crate) fn set_placement_prediction(
        &mut self,
        prediction: Option<crate::PlacementPrediction>,
    ) {
        self.store.data_mut().placement_prediction = prediction;
    }
}
