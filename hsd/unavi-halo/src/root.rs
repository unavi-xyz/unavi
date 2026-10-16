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

use wired_guest::math::Vec2;

use crate::{
    glyph,
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

/// The most motes an orbit level draws on one page: the way back and eight
/// more, matching the cells of a grid level. The root has three slots, so it
/// is still drawn whole and a root that paged would not be a root.
const CAPACITY: u32 = 9;

pub struct Root {
    /// Return, the fixed point.
    pub home:  Mote,
    /// Outward: where you can go.
    pub nav:   Mote,
    /// What can be at hand.
    pub tools: Mote,
    pub orbit: Orbit,
}

/// Puts a glyph on a root slot. A refused glyph leaves the slot bare rather
/// than stopping the halo from existing.
fn wear(mote: &Mote, name: &str, rows: &glyph::Rows) {
    if let Err(err) = mote.set_glyph(Some(&glyph::from_rows(rows))) {
        eprintln!("halo: {name} glyph: {err:?}");
    }
}

impl Root {
    pub fn new() -> anyhow::Result<Self> {
        let level = Mote::new(Kind::Group, "Halo")?;

        let home = Mote::new(Kind::Cast, "Home")?;
        home.set_description("Go to your home space.")?;
        home.set_tint(Some(palette::HOME));
        wear(&home, "Home", &glyph::HOME);

        let nav = Mote::new(Kind::Group, "Nav")?;
        nav.set_description("Discover spaces to visit.")?;
        nav.set_arrange(Arrange::Grid);
        nav.set_tint(Some(palette::NAV));
        wear(&nav, "Nav", &glyph::NAV);

        let tools = Mote::new(Kind::Group, "Tools")?;
        tools.set_description("Tools you can pick up.")?;
        tools.set_tint(Some(palette::TOOLS));
        wear(&tools, "Tools", &glyph::TOOLS);

        for slot in [&home, &nav, &tools] {
            level.add_child(slot);
        }

        let orbit = Orbit::new(&level, MOUNT, CAPACITY)?;
        orbit.dismiss()?;

        // `level` is not kept: the orbit holds the tree from here, and every
        // slot beneath it is reached through the three handles above.
        Ok(Self {
            home,
            nav,
            tools,
            orbit,
        })
    }
}
