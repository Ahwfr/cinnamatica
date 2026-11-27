//! The admitted multi-step placement sequence and its owner lifetime.
use super::creative;
use mod_host::CreativePlacementOffer as Offer;
use std::time::Duration;
#[derive(Default)]
pub(crate) struct SupportAssist {
    pub(super) root: Option<Offer>,
    pub(super) guides: Option<Vec<Offer>>,
    pub(super) scope: Option<(u64, i32)>,
    pub(super) steps: Vec<Step>,
    pub(super) waiting: Option<Waiting>,
    pub(super) owned: Vec<Owned>,
    pub(super) blocked: bool,
    pub(super) deadline: Duration,
    pub(super) owner: Option<u64>,
    pub(super) authorized: bool,
}

#[derive(Clone)]
pub(super) struct Step {
    pub(super) offer: Offer,
    pub(super) temporary: bool,
}
pub(super) struct Waiting {
    pub(super) offer: Offer,
    pub(super) serial: u64,
    pub(super) expires: Duration,
    pub(super) temporary: bool,
}
pub(super) struct Owned {
    pub(super) position: [i32; 3],
    pub(super) runtime: u32,
    pub(super) serial: u64,
}

impl creative::CreativePlacement {
    pub(crate) fn support_owner(&self) -> Option<u64> {
        self.support.owner
    }
    pub(crate) fn authorize_support(&mut self, enabled: bool) {
        if !enabled {
            self.support.authorized = false;
            self.cancel_support();
            self.offer = None;
        }
    }
    pub(super) fn support_continuing(&self) -> bool {
        !self.support.steps.is_empty() || self.support.waiting.is_some()
    }
    pub(super) fn cancel_support(&mut self) {
        self.support.steps.clear();
        self.support.waiting = None;
        self.offer = self.support.root.clone();
    }
    /// Collect the support context from the same mod that owns this offer. An effective
    /// support offer is retained across frames while its original crosshair target stays.
    pub(crate) fn set_offer(
        &mut self,
        root: Option<Offer>,
        guides: Option<Vec<Offer>>,
        owner: Option<u64>,
    ) {
        // One admitted V3 click includes its support and target stages. A newly
        // placed support can briefly obscure the guest's ray; omission during
        // that admitted sequence must not abandon its confirmation/cleanup record.
        if root.is_none() && self.support.authorized && self.support_continuing() {
            self.offer = self.support.steps.first().map(|step| step.offer.clone()).or_else(|| self.support.root.clone());
            return;
        }
        if root.is_some() {
            self.support.authorized = guides.is_some();
            if owner != self.support.owner {
                self.support.owned.clear();
            }
            self.support.owner = owner;
        }
        let changed =
            self.support.root != root || self.support.guides.is_some() != guides.is_some();
        if changed {
            self.support.steps.clear();
            self.support.waiting = None;
            self.support.blocked = false;
            self.offer = root.clone();
        } else if self.support.steps.is_empty() && self.support.waiting.is_none() {
            self.offer = root.clone();
        }
        self.support.root = root;
        self.support.guides = guides;
        self.offer = self.support.steps.first().map(|step| step.offer.clone()).or_else(|| self.support.root.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn offer(x: i32) -> Offer {
        Offer {position:[x,64,0],name:"minecraft:stone".into(),states:"{}".into(),correct_aim:true,strict_state:true,repeat_while_held:true}
    }
    #[test]
    fn a_frame_reset_republishes_the_effective_support_not_the_root() {
        let mut policy=creative::CreativePlacement::default();
        let root=offer(0); let support=offer(1);
        policy.set_offer(Some(root.clone()),Some(vec![]),Some(7));
        policy.support.steps.push(Step {offer:support.clone(),temporary:true});
        policy.offer=None; // the driver clears frame outputs before gathering new ones
        policy.set_offer(Some(root),Some(vec![]),Some(7));
        assert_eq!(policy.offer,Some(support.clone()));
        policy.offer=None;
        policy.set_offer(None,None,None); // a new support briefly obscures the ray
        assert_eq!(policy.offer,Some(support));
    }
    #[test]
    fn disabling_the_owner_withdraws_steps_and_does_not_resurrect_the_root() {
        let mut policy=creative::CreativePlacement::default();
        policy.set_offer(Some(offer(0)),Some(vec![]),Some(7));
        policy.support.steps.push(Step {offer:offer(1),temporary:true});
        policy.authorize_support(false);
        assert!(policy.offer.is_none() && policy.support.steps.is_empty());
        assert!(!policy.support.authorized);
    }
    #[test]
    fn an_owner_change_does_not_inherit_another_instances_cleanup_records() {
        let mut policy=creative::CreativePlacement::default();
        policy.set_offer(Some(offer(0)),Some(vec![]),Some(7));
        policy.support.owned.push(Owned {position:[1,64,0],runtime:1,serial:1});
        policy.set_offer(Some(offer(0)),Some(vec![]),Some(8));
        assert!(policy.support.owned.is_empty());
    }
    #[test]
    fn v1_and_v2_offers_remain_usable_without_support_authority() {
        let mut policy=creative::CreativePlacement::default();
        let root=offer(0);
        policy.set_offer(Some(root.clone()),None,Some(7));
        policy.authorize_support(true);
        assert_eq!(policy.offer,Some(root));
        assert!(!policy.support.authorized);
    }
}

