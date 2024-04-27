use std::collections::HashMap;

use crate::bindings::wired::ecs::types::{Component, ComponentInstance};

pub struct Store<T> {
    component: Component,
    map: HashMap<u32, T>,
}

impl<T> Store<T> {
    pub fn new(component: Component) -> Self {
        Self {
            component,
            map: HashMap::new(),
        }
    }

    pub fn get(&self, instance: &ComponentInstance) -> Option<&T> {
        self.map.get(&instance.handle())
    }

    pub fn insert(&mut self, instance: &ComponentInstance, data: T) {
        self.map.insert(instance.handle(), data);
    }

    pub fn insert_new(&mut self, data: T) -> ComponentInstance {
        let instance = self.component.new();
        self.insert(&instance, data);
        instance
    }
}
