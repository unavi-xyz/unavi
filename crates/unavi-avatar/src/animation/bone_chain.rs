use bevy::{animation::AnimationTargetId, core::Name};

#[derive(Default, Clone)]
pub struct BoneChain<'a> {
    names: Vec<String>,
    prefix: &'a str,
}

impl<'a> BoneChain<'a> {
    pub fn new(names: Vec<String>, prefix: &'a str) -> Self {
        Self { names, prefix }
    }

    pub fn push_target(&mut self, name: &str) -> AnimationTargetId {
        let value = format!("{}{}", self.prefix, name);
        self.names.push(value);
        self.target()
    }

    pub fn target(&self) -> AnimationTargetId {
        let names = self
            .names
            .clone()
            .into_iter()
            .map(Name::new)
            .collect::<Vec<_>>();
        AnimationTargetId::from_names(names.iter())
    }
}
