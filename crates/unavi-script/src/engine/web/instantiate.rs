//! Builds each script's [`crate::host::ScriptHost`] and starts transpiling
//! and instantiating it through `runtime.ts`.

use std::{
    cell::RefCell,
    rc::Rc,
};

use bevy::prelude::*;
use bevy_async::{
    AsyncWorld,
    task,
};
use bevy_hsd::{
    document::{
        Hsd,
        HsdDocId,
    },
    prim::{
        Prim,
        PrimOf,
    },
};
use unavi_policy::quota::Quota;
use unavi_space::{
    authority::SpaceView,
    identity::RootDocument,
};
use wasm_bindgen::JsValue;

use crate::{
    Script,
    ScriptStatus,
    bindings::web::{
        HostHandle,
        Runtime,
        instantiate_script,
    },
    host::{
        ScriptHost,
        ScriptIdentity,
        SharedResources,
    },
    load::asset::Wasm,
    quota::QuotaExempt,
};

/// Where a script's running instance lives once `runtime.ts` builds it.
///
/// `JsValue` is not `Send`/`Sync`, but every component on wasm32 is read on
/// the one JS thread there is, so holding it in a [`Component`] is sound.
#[derive(Component)]
pub struct ScriptInstance(pub JsValue);

// Safe: wasm32 has one thread, and Bevy's own `Component` bound is the only
// reason this needs to claim either.
unsafe impl Send for ScriptInstance {}
unsafe impl Sync for ScriptInstance {}

/// The script's host state, owned deterministically by this component:
/// dropping it (on despawn, or when a trapped script's components are torn
/// down) closes every handle the script still held. `engine::web::tick`
/// clones it to open and close each call's write boundary
/// ([`crate::host::ScriptHost::open_tick`]).
#[derive(Component, Clone)]
pub struct ScriptHostHandle(pub HostHandle);

// Safe: see `ScriptInstance`. `Rc`/`RefCell` are not `Send`/`Sync` only
// because they assume a multi-threaded world; wasm32 has none.
#[expect(
    clippy::non_send_fields_in_send_ty,
    reason = "see the comment above; HostHandle is Rc<RefCell<_>> by design"
)]
unsafe impl Send for ScriptHostHandle {}
unsafe impl Sync for ScriptHostHandle {}

/// A spawned `instantiateScript` call not yet resolved.
#[derive(Component)]
pub struct Instantiating(async_channel::Receiver<Result<JsValue, JsValue>>);

// Safe: see `ScriptInstance`.
unsafe impl Send for Instantiating {}
unsafe impl Sync for Instantiating {}

pub fn instantiate_scripts(
    wasms: Res<Assets<Wasm>>,
    scripts: Query<
        (Entity, &Script, &ScriptStatus, NameOrEntity, &Prim, &PrimOf),
        (Without<Instantiating>, Without<ScriptHostHandle>),
    >,
    docs: Query<(&HsdDocId, &Hsd, Has<QuotaExempt>)>,
    root: Option<Res<RootDocument>>,
    view: Option<Res<SpaceView>>,
    shared: SharedResources,
    async_world: Res<AsyncWorld>,
    mut commands: Commands,
) {
    let Some(view) = view else {
        return;
    };

    for (entity, script, status, name, prim, prim_of) in scripts {
        if status.is_trapped() {
            continue;
        }
        let Some(wasm) = wasms.get(&script.0) else {
            continue;
        };
        let Ok((doc_id, doc, exempt)) = docs.get(prim_of.0) else {
            continue;
        };

        let name = name.to_string();
        let quota = if exempt {
            Quota::unlimited()
        } else {
            view.document_quota(doc_id.0)
        };
        let host: HostHandle = Rc::new(RefCell::new(ScriptHost::new(
            ScriptIdentity {
                doc:      doc_id.0,
                prim:     prim.0,
                state:    std::sync::Arc::clone(&doc.0),
                view:     (*view).clone(),
                quota:    std::sync::Arc::clone(&quota),
                world:    (*async_world).clone(),
                root_doc: root.as_ref().map(|root| root.0),
            },
            shared.get(),
        )));

        let (tx, rx) = async_channel::bounded(1);
        let (bytes, hash) = (std::sync::Arc::clone(&wasm.bytes), wasm.hash);
        let runtime = Runtime::new(Rc::clone(&host));
        task::spawn(async move {
            let result = instantiate_script(&bytes, &hash.to_hex(), &name, runtime).await;
            let _ = tx.send(result).await;
        });

        commands
            .entity(entity)
            .insert((Instantiating(rx), ScriptHostHandle(host)));
    }
}

pub fn finish_instantiating(
    mut scripts: Query<(Entity, &mut Instantiating, &ScriptStatus, NameOrEntity)>,
    mut commands: Commands,
) {
    for (entity, instantiating, status, name) in &mut scripts {
        match instantiating.0.try_recv() {
            Ok(Ok(instance)) => {
                commands
                    .entity(entity)
                    .remove::<Instantiating>()
                    .insert(ScriptInstance(instance));
            }
            Ok(Err(err)) => {
                if status.trap() {
                    error!(
                        name = %name,
                        err = crate::bindings::web::describe_js_error(&err),
                        "Failed to instantiate the script; it will not run"
                    );
                }
                commands.entity(entity).remove::<Instantiating>();
            }
            Err(async_channel::TryRecvError::Empty) => {}
            Err(async_channel::TryRecvError::Closed) => {
                commands.entity(entity).remove::<Instantiating>();
            }
        }
    }
}
