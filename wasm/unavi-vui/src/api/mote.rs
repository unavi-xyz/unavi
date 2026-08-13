use crate::{
    exports::unavi::vui::api::{
        Arrange,
        GuestMote,
        Kind,
        Mote as Handle,
        MoteBorrow,
    },
    mote,
    scene::draw,
    tree,
    wired::scene::types::Prim,
};

/// The `mote` resource: a handle onto a mote in some tree.
pub struct Mote(pub tree::Mote);

impl GuestMote for Mote {
    fn new(kind: Kind, label: String) -> Self {
        Self(tree::Mote::new(held(kind), &label))
    }

    fn is(&self, other: MoteBorrow<'_>) -> bool {
        self.0.is(&other.get::<Self>().0)
    }

    fn label(&self) -> String {
        self.0.label().to_string()
    }

    fn set_label(&self, value: String) {
        self.0.set_label(&value);
    }

    fn describe(&self, text: String) {
        self.0.describe(&text);
    }

    /// The handle is cloned, so the consumer keeps its own and the icon stays
    /// drawable after that one is dropped.
    ///
    /// Hidden on the way in: a prim nothing has parented is a root of the
    /// document, and would otherwise stand at the document's origin at its
    /// authored size until some surface happens to draw it.
    fn set_icon(&self, value: Option<&Prim>) {
        if let Some(prim) = value
            && let Err(err) = prim.set_xform(Some(draw::hidden()))
        {
            // Not fatal, and visibly wrong: the icon stands at the document's
            // origin, at its authored size, until a surface draws it.
            eprintln!("vui: could not hide the icon for '{}': {err}", self.label());
        }
        self.0.set_icon(value.map(Prim::clone));
    }

    fn unique(&self) -> bool {
        self.0.is_unique()
    }

    fn set_unique(&self, value: bool) {
        self.0.set_unique(value);
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
    Handle::new(Mote(mote))
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
