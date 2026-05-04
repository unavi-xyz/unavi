use bevy::platform::collections::HashMap;

pub struct SlotMap<T> {
    pub items: HashMap<u32, T>,
    next: u32,
}

impl<T> Default for SlotMap<T> {
    fn default() -> Self {
        Self {
            items: HashMap::new(),
            next: 0,
        }
    }
}

impl<T> SlotMap<T> {
    pub fn get(&self, key: u32) -> Option<&T> {
        self.items.get(&key)
    }

    pub fn insert(&mut self, value: T) -> u32 {
        let key = self.next;
        self.next += 1;
        self.items.insert(key, value);
        key
    }

    pub fn remove(&mut self, key: u32) -> Option<T> {
        self.items.remove(&key)
    }
}

impl<T> SlotMap<T>
where
    T: Clone,
{
    /// Clone the given key into a new entry.
    pub fn insert_clone(&mut self, key: u32) -> Option<u32> {
        let value = self.get(key)?;
        let rep = self.insert(value.clone());
        Some(rep)
    }
}
