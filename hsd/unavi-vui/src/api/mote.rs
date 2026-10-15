//! The `mote` resource: `unavi:vui`'s view onto a [`tree::Mote`].

use wired_guest::math::Color;

use crate::{
    exports::unavi::vui::api::{
        Arrange,
        GuestMote,
        Kind,
        Mote as Handle,
        MoteBorrow,
    },
    mote,
    render::draw,
    tree,
    wired::{
        core::error::Error,
        scene::{
            document::Document,
            properties::Property,
        },
    },
};

/// `mote.set-label`: at most 256 bytes.
const MAX_LABEL_BYTES: usize = 256;
/// `mote.set-description`: at most 1024 bytes.
const MAX_DESCRIPTION_BYTES: usize = 1024;

/// The `mote` resource: a handle onto a mote in some tree.
pub struct MoteRes(pub tree::Mote);

impl GuestMote for MoteRes {
    fn new(kind: Kind, label: String) -> Result<Self, Error> {
        bounded(&label, MAX_LABEL_BYTES, "label")?;
        Ok(Self(tree::Mote::new(held(kind), &label)))
    }

    fn is(&self, other: MoteBorrow<'_>) -> bool {
        self.0.is(&other.get::<Self>().0)
    }

    fn label(&self) -> String {
        self.0.label().to_string()
    }

    fn set_label(&self, value: String) -> Result<(), Error> {
        bounded(&value, MAX_LABEL_BYTES, "label")?;
        self.0.set_label(&value);
        Ok(())
    }

    fn description(&self) -> Option<String> {
        self.0.description().map(|text| text.to_string())
    }

    fn set_description(&self, text: String) -> Result<(), Error> {
        bounded(&text, MAX_DESCRIPTION_BYTES, "description")?;
        self.0.describe(&text);
        Ok(())
    }

    fn id(&self) -> u64 {
        self.0.id()
    }

    /// Hidden on the way in: a prim nothing has parented is a root of the
    /// document, and would otherwise stand at the document's origin at its
    /// authored size until some surface happens to draw it.
    fn set_icon(&self, doc: &Document, value: Option<(u64, u64)>) {
        if let Some(prim) = value
            && let Err(err) = doc
                .local()
                .set(prim, Property::Transform(draw::hidden()))
                .flush()
        {
            // Not fatal, and visibly wrong: the icon stands at the document's
            // origin, at its authored size, until a surface draws it.
            eprintln!(
                "vui: could not hide the icon for '{}': {err}",
                self.0.label()
            );
        }
        self.0.set_icon(value);
    }

    fn unique(&self) -> bool {
        self.0.is_unique()
    }

    fn set_unique(&self, value: bool) {
        self.0.set_unique(value);
    }

    fn set_tint(&self, value: Option<Color>) {
        self.0.set_tint(value);
    }

    fn set_film(&self, value: f32) {
        self.0.set_film(value);
    }

    fn set_frost(&self, value: f32) {
        self.0.set_frost(value);
    }

    fn arrange(&self) -> Arrange {
        match self.0.arrange() {
            mote::Arrange::Orbit => Arrange::Orbit,
            mote::Arrange::Grid => Arrange::Grid,
        }
    }

    fn set_arrange(&self, value: Arrange) {
        self.0.set_arrange(match value {
            Arrange::Orbit => mote::Arrange::Orbit,
            Arrange::Grid => mote::Arrange::Grid,
        });
    }

    fn active(&self) -> bool {
        self.0.is_active()
    }

    fn set_active(&self, value: bool) {
        self.0.set_active(value);
    }

    fn parent(&self) -> Option<Handle> {
        self.0.parent().map(handle)
    }

    fn children(&self) -> Vec<Handle> {
        self.0.children().into_iter().map(handle).collect()
    }

    fn add_child(&self, child: MoteBorrow<'_>) -> bool {
        self.0.add_child(&child.get::<Self>().0)
    }

    fn remove_child(&self, child: MoteBorrow<'_>) {
        self.0.remove_child(&child.get::<Self>().0);
    }

    fn clear(&self) {
        self.0.clear();
    }
}

/// A fresh handle onto `mote`, for a consumer that has no handle of its own —
/// what an event carries.
pub fn handle(mote: tree::Mote) -> Handle {
    Handle::new(MoteRes(mote))
}

const fn held(kind: Kind) -> tree::Kind {
    match kind {
        Kind::Action => tree::Kind::Action,
        Kind::Toggle => tree::Kind::Toggle,
        Kind::Item => tree::Kind::Item,
        Kind::Cast => tree::Kind::Cast,
        Kind::Group => tree::Kind::Group,
    }
}

/// Refuses `value` once it is over `max` bytes, naming `what` for the caller.
fn bounded(value: &str, max: usize, what: &str) -> Result<(), Error> {
    if value.len() > max {
        return Err(Error::InvalidArgument(format!(
            "{what} is {} bytes, over the limit of {max}",
            value.len()
        )));
    }
    Ok(())
}
