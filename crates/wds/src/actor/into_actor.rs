use crate::actor::Actor;

pub trait IntoActor {
    fn into_actor(self) -> Option<Actor>;
}

impl IntoActor for Option<Actor> {
    fn into_actor(self) -> Option<Actor> {
        self
    }
}

impl IntoActor for Actor {
    fn into_actor(self) -> Option<Actor> {
        Some(self)
    }
}

impl IntoActor for &Actor {
    fn into_actor(self) -> Option<Actor> {
        Some(self.clone())
    }
}
