//! The root form: what the halo opens as.
//!
//! Three slots around a ring: tools, navigation and going home. Nothing sits
//! in the middle, so every slot is a direction.
//!
//! The cardinals are not settled until there are four of them. Home takes up,
//! which is its fixed sense, and the other two fall where a three-point ring
//! puts them. When more branches arrive the ring grows and they move once,
//! deliberately, rather than drifting a slot at a time.
//!
//! A slot is never reordered and never repurposed. That is the basis of
//! eyes-free use, and it is why the order below is written once and left
//! alone.

use wired_prelude::prelude::*;

use crate::{
    icon,
    palette,
    unavi::vui::api::{
        Arrange,
        Bearing,
        Kind,
        Mote,
        Mount,
        Orbit,
    },
};

/// Metres along the line of sight the halo appears.
const FORWARD: f32 = 0.7;

/// Summoned where the eye already is rather than level ahead: the halo is
/// called up to be used now, so it should not have to be found. It
/// world-locks there, so looking away afterwards leaves it where it was put.
const MOUNT: Mount = Mount {
    distance: FORWARD,
    height:   0.0,
    offset:   Vec2::ZERO,
    bearing:  Bearing::Sight,
};

/// Every slot is drawn at once: a root that paged would not be a root.
const CAPACITY: u32 = 3;

pub struct Root {
    /// Return, the fixed point.
    pub home:   Mote,
    /// Outward: the world.
    pub places: Mote,
    /// What can be at hand.
    pub tools:  Mote,
    pub orbit:  Orbit,
}

impl Root {
    pub fn new() -> anyhow::Result<Self> {
        let level = Mote::new(Kind::Group, "Halo");

        let home = Mote::new(Kind::Cast, "Home");
        home.describe("Travel to your own space. Hold to confirm.");
        home.set_icon(Some(&icon::build(&icon::home(), palette::HOME)?));

        let places = Mote::new(Kind::Group, "Places");
        places.describe("Spaces with people in them.");
        places.set_arrange(Arrange::Grid);
        places.set_icon(Some(&icon::build(&icon::places(), palette::PLACES)?));

        let tools = Mote::new(Kind::Group, "Tools");
        tools.describe("Take one to hold it. Let it go to put it away.");
        tools.set_icon(Some(&icon::build(&icon::tools(), palette::TOOLS)?));

        for slot in [&home, &places, &tools] {
            level.add_child(slot);
        }

        let orbit = Orbit::new(&level, MOUNT, CAPACITY)?;
        orbit.dismiss()?;

        // `level` is not kept: the orbit holds the tree from here, and every
        // slot beneath it is reached through the three handles above.
        Ok(Self {
            home,
            places,
            tools,
            orbit,
        })
    }
}
