//! The root form: what the halo opens as.
//!
//! The middle of a bed and the four keys on its cross: the hand in the
//! middle, going home above it, places to its right, the pocket below and
//! the self to its left. The corners stay empty. A slot is a child of the
//! root in that order, because a bed puts its first child in the middle and
//! the next four up, right, down and left.
//!
//! Home keeps up, which is its fixed sense. A slot is never reordered and
//! never repurposed after this. That is the basis of eyes-free use, and it
//! is why the order below is written once and left alone.

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

/// The most motes a level draws on one page: the way back and eight more,
/// the keys of a bed. The root has five, so it is drawn whole and a root
/// that paged would not be a root.
const CAPACITY: u32 = 9;

pub struct Root {
    /// Return, the fixed point.
    pub home:  Mote,
    /// Outward: where you can go.
    pub nav:   Mote,
    /// What can be at hand, in the middle.
    pub tools: Mote,
    /// Under Hand, before the tools: take back and put back the last edit.
    pub undo:  Mote,
    pub redo:  Mote,
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
        level.set_arrange(Arrange::Bed);

        let home = Mote::new(Kind::Cast, "Home")?;
        home.set_description("Go to your home space.")?;
        home.set_tint(Some(palette::HOME));
        wear(&home, "Home", &glyph::HOME);

        let nav = Mote::new(Kind::Group, "Places")?;
        nav.set_description("Discover spaces to visit.")?;
        nav.set_arrange(Arrange::Bed);
        nav.set_tint(Some(palette::NAV));
        wear(&nav, "Places", &glyph::NAV);

        let tools = Mote::new(Kind::Group, "Hand")?;
        tools.set_description("Tools you can pick up.")?;
        tools.set_arrange(Arrange::Bed);
        tools.set_tint(Some(palette::TOOLS));
        wear(&tools, "Hand", &glyph::TOOLS);

        let undo = Mote::new(Kind::Action, "Undo")?;
        undo.set_description("Takes back the last change you made.")?;
        wear(&undo, "Undo", &glyph::UNDO);
        let redo = Mote::new(Kind::Action, "Redo")?;
        redo.set_description("Puts back the last change you took back.")?;
        wear(&redo, "Redo", &glyph::REDO);
        // First under Hand, so they keep their place as tools arrive after.
        tools.add_child(&undo);
        tools.add_child(&redo);

        // What you carry. Nothing can be filed into it yet, and its place is
        // held so the cross is whole.
        let pocket = Mote::new(Kind::Group, "Pocket")?;
        pocket.set_description("What you carry.")?;
        pocket.set_arrange(Arrange::Bed);
        pocket.set_tint(Some(palette::POCKET));
        wear(&pocket, "Pocket", &glyph::POCKET);

        let me = Mote::new(Kind::Group, "Self")?;
        me.set_description("Your relation to the world.")?;
        me.set_arrange(Arrange::Bed);
        me.set_tint(Some(palette::SELF));
        wear(&me, "Self", &glyph::SELF);

        let visor = Mote::new(Kind::Group, "Visor")?;
        visor.set_description("Lenses to augment your vision.")?;
        visor.set_arrange(Arrange::Bed);
        visor.set_tint(Some(palette::SELF));
        wear(&visor, "Visor", &glyph::VISOR);
        me.add_child(&visor);

        // The middle, then clockwise from the top: hand, home up, places
        // right, pocket down, self left.
        for slot in [&tools, &home, &nav, &pocket, &me] {
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
            undo,
            redo,
            me,
            visor,
            orbit,
        })
    }
}
