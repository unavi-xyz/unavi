//! The halo's document, carried with the user's position.

use wired_guest::math::Transform;

use crate::wired::{
    agent::local::{
        Attachment,
        attach,
        camera_transform,
    },
    core::error::Error,
    scene::document::script_document,
};

/// Attaches the script's document to the user's body node, which exists a few
/// ticks after the agent spawns and is replaced when the agent respawns.
#[derive(Default)]
pub struct Body {
    attached: bool,
    /// Whether a refusal has been reported, so it is reported once.
    refused:  bool,
}

impl Body {
    /// Tries to attach until it holds, and again after the agent respawns. The
    /// body not existing yet is not a failure. A refused attach leaves the halo
    /// world-locked rather than stopping its tick.
    pub fn step(&mut self) -> anyhow::Result<()> {
        if self.attached {
            // The camera is untracked while the agent is gone, and the node
            // the document hung from goes with it.
            self.attached = camera_transform().is_ok();
            return Ok(());
        }
        match attach(&script_document()?, Attachment::Body, Transform::IDENTITY) {
            Ok(()) => self.attached = true,
            Err(Error::NotReady) => {}
            Err(err) => {
                if !self.refused {
                    self.refused = true;
                    eprintln!("halo: body attach: {err:?}");
                }
            }
        }
        Ok(())
    }
}
