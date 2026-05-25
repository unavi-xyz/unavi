//! Write Rust types into Loro containers.

pub(crate) mod impls;
pub mod list;
pub mod map;
pub mod movable_list;

use std::hash::Hash;

use loro::{
    Container,
    ContainerTrait,
    LoroList,
    LoroMap,
    LoroMovableList,
    LoroValue,
    ValueOrContainer,
};

use crate::error::ReconcileError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NoKey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadKey<K> {
    NoKey,
    KeyNotFound,
    Found(K),
}

impl<K> LoadKey<K> {
    pub fn into_found(self) -> Option<K> {
        match self {
            Self::Found(k) => Some(k),
            _ => None,
        }
    }
}

pub trait Reconcile {
    type Key: PartialEq + Eq + Hash;

    fn reconcile<R: Reconciler>(&self, reconciler: R) -> Result<(), ReconcileError>;

    fn key(&self) -> LoadKey<Self::Key> {
        LoadKey::NoKey
    }

    fn hydrate_key(_source: &ValueOrContainer) -> Result<LoadKey<Self::Key>, ReconcileError> {
        Ok(LoadKey::NoKey)
    }
}

pub trait Reconciler {
    fn null(self) -> Result<(), ReconcileError>;
    fn boolean(self, v: bool) -> Result<(), ReconcileError>;
    fn i64(self, v: i64) -> Result<(), ReconcileError>;
    fn f64(self, v: f64) -> Result<(), ReconcileError>;
    fn str(self, v: &str) -> Result<(), ReconcileError>;
    fn bytes(self, v: &[u8]) -> Result<(), ReconcileError>;
    /// Write an atomic [`LoroValue::List`] value (not a container).
    fn inline_list(self, v: LoroValue) -> Result<(), ReconcileError>;

    fn map(self) -> Result<MapReconciler, ReconcileError>;
    fn list(self) -> Result<ListReconciler, ReconcileError>;
    fn movable_list(self) -> Result<MovableListReconciler, ReconcileError>;
}

pub struct PropReconciler {
    action: PropAction,
}

enum PropAction {
    MapPut {
        map: LoroMap,
        key: String,
    },
    ListInsert {
        list:  LoroList,
        index: usize,
    },
    MovableListInsert {
        list:  LoroMovableList,
        index: usize,
    },
    MovableListSet {
        list:  LoroMovableList,
        index: usize,
    },
}

impl PropReconciler {
    #[must_use]
    pub const fn map_put(map: LoroMap, key: String) -> Self {
        Self {
            action: PropAction::MapPut { map, key },
        }
    }

    #[must_use]
    pub const fn list_insert(list: LoroList, index: usize) -> Self {
        Self {
            action: PropAction::ListInsert { list, index },
        }
    }

    #[must_use]
    pub const fn movable_list_insert(list: LoroMovableList, index: usize) -> Self {
        Self {
            action: PropAction::MovableListInsert { list, index },
        }
    }

    #[must_use]
    pub const fn movable_list_set(list: LoroMovableList, index: usize) -> Self {
        Self {
            action: PropAction::MovableListSet { list, index },
        }
    }

    fn put_value(self, value: impl Into<LoroValue>) -> Result<(), ReconcileError> {
        match self.action {
            PropAction::MapPut { map, key } => {
                let new_value = value.into();
                if let Some(ValueOrContainer::Value(existing)) = map.get(&key)
                    && existing == new_value
                {
                    return Ok(());
                }
                map.insert(&key, new_value)?;
            }
            PropAction::ListInsert { list, index } => {
                list.insert(index, value)?;
            }
            PropAction::MovableListInsert { list, index } => {
                list.insert(index, value)?;
            }
            PropAction::MovableListSet { list, index } => {
                let new_value = value.into();
                if let Some(ValueOrContainer::Value(existing)) = list.get(index)
                    && existing == new_value
                {
                    return Ok(());
                }
                list.set(index, new_value)?;
            }
        }
        Ok(())
    }

    fn get_or_create_container<C: ContainerTrait>(self, detached: C) -> Result<C, ReconcileError> {
        let container = match self.action {
            PropAction::MapPut { map, key } => map.get_or_create_container(&key, detached)?,
            PropAction::ListInsert { list, index } => list.insert_container(index, detached)?,
            PropAction::MovableListInsert { list, index } => {
                list.insert_container(index, detached)?
            }
            PropAction::MovableListSet { list, index } => list.set_container(index, detached)?,
        };
        Ok(container)
    }

    fn try_get_existing_map(&self) -> Option<LoroMap> {
        match &self.action {
            PropAction::MovableListSet { list, index } => {
                if let Some(ValueOrContainer::Container(Container::Map(m))) = list.get(*index) {
                    Some(m)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Reconciler for PropReconciler {
    fn null(self) -> Result<(), ReconcileError> {
        self.put_value(LoroValue::Null)
    }

    fn boolean(self, v: bool) -> Result<(), ReconcileError> {
        self.put_value(v)
    }

    fn i64(self, v: i64) -> Result<(), ReconcileError> {
        self.put_value(v)
    }

    fn f64(self, v: f64) -> Result<(), ReconcileError> {
        self.put_value(v)
    }

    fn str(self, v: &str) -> Result<(), ReconcileError> {
        self.put_value(v)
    }

    fn bytes(self, v: &[u8]) -> Result<(), ReconcileError> {
        self.put_value(LoroValue::Binary(v.to_vec().into()))
    }

    fn inline_list(self, v: LoroValue) -> Result<(), ReconcileError> {
        self.put_value(v)
    }

    fn map(self) -> Result<MapReconciler, ReconcileError> {
        if let Some(existing) = self.try_get_existing_map() {
            return Ok(MapReconciler { map: existing });
        }
        let m = self.get_or_create_container(LoroMap::new())?;
        Ok(MapReconciler { map: m })
    }

    fn list(self) -> Result<ListReconciler, ReconcileError> {
        let l = self.get_or_create_container(LoroList::new())?;
        Ok(ListReconciler { list: l })
    }

    fn movable_list(self) -> Result<MovableListReconciler, ReconcileError> {
        let l = self.get_or_create_container(LoroMovableList::new())?;
        Ok(MovableListReconciler { list: l })
    }
}

pub struct RootReconciler {
    map: LoroMap,
}

impl RootReconciler {
    #[must_use]
    pub const fn new(map: LoroMap) -> Self {
        Self { map }
    }
}

impl Reconciler for RootReconciler {
    fn null(self) -> Result<(), ReconcileError> {
        Err(ReconcileError::TypeMismatch {
            expected: "map",
            found:    "null",
        })
    }
    fn boolean(self, _v: bool) -> Result<(), ReconcileError> {
        Err(ReconcileError::TypeMismatch {
            expected: "map",
            found:    "bool",
        })
    }
    fn i64(self, _v: i64) -> Result<(), ReconcileError> {
        Err(ReconcileError::TypeMismatch {
            expected: "map",
            found:    "i64",
        })
    }
    fn f64(self, _v: f64) -> Result<(), ReconcileError> {
        Err(ReconcileError::TypeMismatch {
            expected: "map",
            found:    "f64",
        })
    }
    fn str(self, _v: &str) -> Result<(), ReconcileError> {
        Err(ReconcileError::TypeMismatch {
            expected: "map",
            found:    "string",
        })
    }
    fn bytes(self, _v: &[u8]) -> Result<(), ReconcileError> {
        Err(ReconcileError::TypeMismatch {
            expected: "map",
            found:    "binary",
        })
    }
    fn inline_list(self, _v: LoroValue) -> Result<(), ReconcileError> {
        Err(ReconcileError::TypeMismatch {
            expected: "map",
            found:    "inline list",
        })
    }
    fn map(self) -> Result<MapReconciler, ReconcileError> {
        Ok(MapReconciler { map: self.map })
    }
    fn list(self) -> Result<ListReconciler, ReconcileError> {
        Err(ReconcileError::TypeMismatch {
            expected: "map",
            found:    "list",
        })
    }
    fn movable_list(self) -> Result<MovableListReconciler, ReconcileError> {
        Err(ReconcileError::TypeMismatch {
            expected: "map",
            found:    "movable_list",
        })
    }
}

pub struct MapReconciler {
    pub map: LoroMap,
}

pub struct ListReconciler {
    pub(crate) list: LoroList,
}

pub struct MovableListReconciler {
    pub(crate) list: LoroMovableList,
}
