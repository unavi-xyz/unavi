//! The root form: what the halo opens as.
//!
//! Four slots around a ring: going home, navigation, the self and tools.
//! Nothing sits in the middle, so every slot is a direction, and a ring of
//! four puts them on the cardinals: home up, navigation right, tools down and
//! the self left. A slot is a child of the root in that order, because an
//! orbit sets its first child at the top and advances clockwise.
//!
//! Home keeps up, which is its fixed sense. Before the self arrived the ring
//! held three and tools and navigation sat where a three-point ring puts
//! them; the fourth slot is the one deliberate move, made once so the ring
//! lands on the cardinals for good.
//!
//! A slot is never reordered and never repurposed after this. That is the
//! basis of eyes-free use, and it is why the order below is written once and
//! left alone.

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
/// more, matching the cells of a grid level. The root has four slots, so it
/// is still drawn whole and a root that paged would not be a root.
const CAPACITY: u32 = 9;

pub struct Root {
    /// Return, the fixed point.
    pub home:  Mote,
    /// Outward: where you can go.
    pub nav:   Mote,
    /// What can be at hand.
    pub tools: Mote,
    /// Who you are to the world: holds the visor.
    pub me:    Mote,
    /// The lenses worn over the world, one toggle each.
    pub visor: Mote,
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

        let me = Mote::new(Kind::Group, "Self")?;
        me.set_description("Your relation to the world.")?;
        me.set_tint(Some(palette::SELF));
        wear(&me, "Self", &glyph::SELF);

        let visor = Mote::new(Kind::Group, "Visor")?;
        visor.set_description("Lenses to augment your vision.")?;
        visor.set_tint(Some(palette::SELF));
        wear(&visor, "Visor", &glyph::VISOR);
        me.add_child(&visor);

        // Clockwise from the top: home up, nav right, tools down, self left.
        for slot in [&home, &nav, &tools, &me] {
            level.add_child(slot);
        }

        let orbit = Orbit::new(&level, MOUNT, CAPACITY)?;
        orbit.dismiss()?;

        // `level` is not kept: the orbit holds the tree from here, and every
        // slot beneath it is reached through the handles above.
        Ok(Self {
            home,
            nav,
            tools,
            me,
            visor,
            orbit,
        })
    }
}
