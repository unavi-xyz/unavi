use std::sync::Arc;

use bevy::prelude::*;
use dwn::{actor::Actor, store::SurrealStore, DWN};
use surrealdb::{engine::local::Mem, Surreal};

use crate::UserActor;

pub async fn init_user_actor(world: &mut World) {
    let actor = {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        let store = SurrealStore::new(db).await.unwrap();
        let dwn = DWN::new(store.clone(), store);
        Actor::new_did_key(dwn).unwrap()
    };

    world.init_resource::<Assets<Mesh>>();
    world.insert_resource(UserActor(Arc::new(actor)));
}
