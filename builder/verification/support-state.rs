//! Owning-source controller lifetime regression without a client/Bevy launch.
#![allow(dead_code)]
#[path="../../app/src/block_use/support_state.rs"]
mod support_state;
mod creative {
    #[derive(Default)]
    pub(crate) struct CreativePlacement {
        pub(crate) support: super::support_state::SupportAssist,
        pub(crate) offer: Option<mod_host::CreativePlacementOffer>,
    }
}
