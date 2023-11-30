use bevy::ecs::schedule::States;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
pub enum AppState {
    #[default]
    LoadingWorld,
    InWorld,
}
