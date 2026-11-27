use super::{MAX_IMPORT_WRITES, State, cinnabar};
use anyhow::{Result, bail};

/// A use pulse staged by the running frame callback and published only when it succeeds.
#[derive(Default)]
pub(super) struct ItemUsePolicy {
    pending_pulse: bool,
    pending_placement: Option<crate::CreativePlacementOffer>,
    pending_supports: Option<Vec<crate::CreativePlacementOffer>>,
    pub supports: Option<Vec<crate::CreativePlacementOffer>>,
    pending_lock: bool,
    pub locked: bool,
    writes: u32,
    /// The current frame's committed pulse, read before the action filter runs.
    pub pulse: bool,
    pub placement: Option<crate::CreativePlacementOffer>,
    pending_prediction_wanted: bool,
    /// A committed callback read the prediction, so the client prepares the next one.
    pub prediction_wanted: bool,
}

impl ItemUsePolicy {
    /// Publishes a pulse only from a callback that saw a gameplay frame.
    pub fn commit(&mut self, gameplay: bool) {
        self.pulse = std::mem::take(&mut self.pending_pulse) && gameplay;
        self.placement = self.pending_placement.take().filter(|_| gameplay);
        self.supports = self
            .pending_supports
            .take()
            .filter(|_| gameplay && self.placement.is_some());
        self.locked = std::mem::take(&mut self.pending_lock) && gameplay;
        self.prediction_wanted = std::mem::take(&mut self.pending_prediction_wanted) && gameplay;
    }
}

impl cinnabar::extension::item_use::Host for State {
    fn offer_placement_supports(
        &mut self,
        blocks: Vec<cinnabar::extension::item_use::SupportBlock>,
    ) -> Result<Result<(), String>> {
        self.controls.budget(false)?;
        self.item_use_policy.writes += 1;
        if self.item_use_policy.writes > MAX_IMPORT_WRITES {
            bail!("item-use import budget exhausted");
        }
        if !self.grants.item_use
            || !self.grants.inventory
            || !self.grants.terrain
            || self.snapshot.is_none()
            || !self.controls.frame.focused
            || !self.controls.frame.gameplay
            || self.controls.frame.panel_open
        {
            return Ok(Err(
                "Support placement requires focused gameplay, item use, inventory and terrain"
                    .into(),
            ));
        }
        let Some(root) = self.item_use_policy.pending_placement.as_ref() else {
            return Ok(Err("No admitted placement offer in this frame".into()));
        };
        let mut accepted = Vec::with_capacity(blocks.len().min(32));
        let mut seen = std::collections::HashSet::new();
        let mut bytes = 0usize;
        if blocks.len() > 32 {
            return Ok(Err("Too many support guide cells".into()));
        }
        for block in blocks {
            let p = [block.position.0, block.position.1, block.position.2];
            let distance: u64 = p
                .iter()
                .zip(root.position)
                .map(|(a, b)| a.abs_diff(b) as u64)
                .sum();
            bytes = bytes
                .saturating_add(block.name.len())
                .saturating_add(block.states.len());
            if !(1..=2).contains(&distance)
                || !seen.insert(p)
                || p.iter().any(|v| v.unsigned_abs() > 30_000_000)
                || !block.name.starts_with("minecraft:")
                || block.name.len() > 128
                || block.states.len() > 2048
                || bytes > 16 * 1024
            {
                return Ok(Err("Invalid support guide cells".into()));
            }
            accepted.push(crate::CreativePlacementOffer {
                position: p,
                name: block.name,
                states: block.states,
                correct_aim: true,
                strict_state: true,
                repeat_while_held: true,
            });
        }
        self.item_use_policy.pending_supports = Some(accepted);
        Ok(Ok(()))
    }
    fn read_placement_cell(&mut self) -> Result<Result<Option<(i32, i32, i32)>, String>> {
        self.controls.budget(false)?;
        if !self.grants.item_use || !self.grants.terrain {
            return Ok(Err(
                "Reading the placement cell requires item use and terrain".into(),
            ));
        }
        Ok(Ok(self
            .gameplay_facts
            .placement_cell
            .filter(|_| {
                self.snapshot.is_some()
                    && self.controls.frame.focused
                    && self.controls.frame.gameplay
                    && !self.controls.frame.panel_open
            })
            .map(|[x, y, z]| (x, y, z))))
    }

    fn offer_creative_placement(
        &mut self,
        position: (i32, i32, i32),
        name: String,
        states: String,
    ) -> Result<Result<(), String>> {
        self.offer_placement(position, name, states, false, false)
    }

    fn offer_oriented_placement(
        &mut self,
        position: (i32, i32, i32),
        name: String,
        states: String,
    ) -> Result<Result<(), String>> {
        self.offer_placement(position, name, states, true, false)
    }

    fn offer_strict_placement(
        &mut self,
        position: (i32, i32, i32),
        name: String,
        states: String,
    ) -> Result<Result<(), String>> {
        self.offer_placement(position, name, states, true, true)
    }

    fn set_placement_repeat(&mut self, enabled: bool) -> Result<Result<(), String>> {
        self.controls.budget(false)?;
        let Some(offer) = self.item_use_policy.pending_placement.as_mut() else {
            return Ok(Err("No admitted placement offer in this frame".into()));
        };
        offer.repeat_while_held = enabled;
        Ok(Ok(()))
    }

    fn read_placement_prediction(
        &mut self,
    ) -> Result<Result<Option<cinnabar::extension::item_use::PlacementPrediction>, String>> {
        self.controls.budget(false)?;
        if !self.grants.item_use || !self.grants.inventory || self.snapshot.is_none() {
            return Ok(Err(
                "Placement prediction requires gameplay, item use and inventory".into(),
            ));
        }
        self.item_use_policy.pending_prediction_wanted = true;
        let tuple = |[x, y, z]: [i32; 3]| (x, y, z);
        Ok(Ok(self.placement_prediction.clone().map(|prediction| {
            cinnabar::extension::item_use::PlacementPrediction {
                ghost: tuple(prediction.ghost),
                block: prediction.block,
                states: prediction.states,
                item: prediction.item,
                source: prediction.source.to_owned(),
                support: prediction.support.map(tuple),
                face: prediction.face,
                facing: prediction.facing.map(|[pitch, yaw]| (pitch, yaw)),
                air_click: prediction.air_click,
                sneak: prediction.sneak,
                predicted_position: prediction.predicted_position.map(tuple),
                predicted_block: prediction.predicted_block,
                predicted_states: prediction.predicted_states,
                verdict: prediction.verdict,
            }
        })))
    }

    fn set_placement_lock(&mut self, enabled: bool) -> Result<Result<(), String>> {
        self.controls.budget(false)?;
        if !self.grants.item_use || !self.grants.inventory || self.snapshot.is_none() {
            return Ok(Err(
                "Placement lock requires gameplay, item use and inventory".into(),
            ));
        }
        self.item_use_policy.pending_lock = enabled;
        Ok(Ok(()))
    }

    /// Stages one use press edge while gameplay owns input and the use control is held.
    fn pulse_use(&mut self) -> Result<Result<(), String>> {
        self.item_use_policy.writes += 1;
        if self.item_use_policy.writes > MAX_IMPORT_WRITES {
            bail!("item-use import budget exhausted");
        }
        if !self.grants.item_use {
            return Ok(Err("item-use capability denied".into()));
        }
        if self.snapshot.is_none() || !self.gameplay_facts.use_held {
            return Ok(Err("use pulse requires a held current gameplay use".into()));
        }
        self.item_use_policy.pending_pulse = true;
        Ok(Ok(()))
    }
}

impl State {
    fn offer_placement(
        &mut self,
        position: (i32, i32, i32),
        name: String,
        states: String,
        correct_aim: bool,
        strict_state: bool,
    ) -> Result<Result<(), String>> {
        self.item_use_policy.writes += 1;
        if self.item_use_policy.writes > MAX_IMPORT_WRITES {
            bail!("item-use import budget exhausted");
        }
        if !self.grants.item_use || !self.grants.inventory || self.snapshot.is_none() {
            return Ok(Err(
                "Creative placement requires gameplay, item use and inventory".into(),
            ));
        }
        let position = [position.0, position.1, position.2];
        if position.iter().any(|v| v.unsigned_abs() > 30_000_000)
            || !name.starts_with("minecraft:")
            || name.len() > 128
            || states.len() > 2048
        {
            return Ok(Err("invalid Creative placement offer".into()));
        }
        // Support context belongs to exactly this offer, never to an earlier offer
        // overwritten by the same callback.
        self.item_use_policy.pending_supports = None;
        self.item_use_policy.pending_placement = Some(crate::CreativePlacementOffer {
            position,
            name,
            states,
            correct_aim,
            strict_state,
            repeat_while_held: true,
        });
        Ok(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cinnabar::extension::item_use::Host as _;

    /// A prediction delivered before a frame reaches that frame's callback, which runs on a
    /// transaction copy of the state, and reading it asks for the next one.
    #[test]
    fn a_delivered_prediction_is_read_in_the_frame_and_asks_for_the_next() {
        let grants = crate::ModGrants {
            item_use: true,
            inventory: true,
            ..Default::default()
        };
        let mut state = State::new(grants, String::new(), Default::default());
        state.snapshot = Some(crate::GameplaySnapshot {
            session: 1,
            dimension: 0,
            eye: crate::GameplayVector3 {
                x: 0.,
                y: 64.,
                z: 0.,
            },
            yaw: 0.,
            pitch: 0.,
            attack_held: false,
            frame_seconds: 0.016,
        });
        state.placement_prediction = Some(crate::PlacementPrediction {
            ghost: [1, 2, 3],
            block: "minecraft:stone".into(),
            verdict: "places".into(),
            source: "hotbar",
            ..Default::default()
        });
        let mut frame = state.frame_transaction();
        let read = frame.read_placement_prediction().unwrap().unwrap().unwrap();
        assert_eq!(read.ghost, (1, 2, 3));
        assert_eq!(read.verdict, "places");
        frame.item_use_policy.commit(true);
        assert!(frame.item_use_policy.prediction_wanted);
    }
}
