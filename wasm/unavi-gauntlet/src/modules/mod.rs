pub mod inventory;
pub mod nav;

pub use inventory::InventoryActive;
pub use nav::NavActive;

use wired_prelude::wired_math::types::Vec3;

use crate::{
    module::ModuleKind,
    wired::scene::types::{Document, Node},
};

pub enum ModuleActive {
    Inventory(InventoryActive),
    Nav(NavActive),
}

impl ModuleActive {
    pub fn activate(&self) {
        self.root().set_scale(Vec3::ONE);
        if let Self::Nav(nav) = self {
            nav.ring.set_scale(Vec3::ONE);
        }
    }

    pub fn deactivate(&self) {
        self.root().set_scale(Vec3::ZERO);
        if let Self::Nav(nav) = self {
            nav.ring.set_scale(Vec3::ZERO);
        }
    }

    pub const fn root(&self) -> &Node {
        match self {
            Self::Inventory(a) => &a.root,
            Self::Nav(a) => &a.root,
        }
    }
}

pub fn make_active(kind: ModuleKind, doc: &Document, color: [f32; 3]) -> ModuleActive {
    match kind {
        ModuleKind::Inventory => ModuleActive::Inventory(InventoryActive::new(doc, color)),
        ModuleKind::Nav => ModuleActive::Nav(NavActive::new(doc, color)),
    }
}
