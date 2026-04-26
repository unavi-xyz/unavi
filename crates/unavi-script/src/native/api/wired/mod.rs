use wasmtime::component::{HasSelf, Linker};

use crate::{
    load::native::state::StoreState,
    permissions::{ApiName, ScriptPermissions},
};

pub mod agent;
pub mod event;
pub mod input;
pub mod scene;
pub mod wds;

pub fn add_to_linker(
    linker: &mut Linker<StoreState>,
    perms: &ScriptPermissions,
) -> wasmtime::Result<()> {
    if perms.api.contains(&ApiName::Scene) {
        scene::bindings::wired::scene::api::add_to_linker::<_, HasSelf<_>>(linker, |s| {
            &mut s.rt.wired_scene
        })?;
        scene::bindings::wired::scene::types::add_to_linker::<_, HasSelf<_>>(linker, |s| {
            &mut s.rt.wired_scene
        })?;
    }
    if perms.api.contains(&ApiName::Agent) {
        agent::bindings::wired::agent::types::add_to_linker::<_, HasSelf<_>>(linker, |s| {
            &mut s.rt
        })?;
    }
    if perms.api.contains(&ApiName::LocalAgent) {
        agent::bindings::wired::agent::api::add_to_linker::<_, HasSelf<_>>(linker, |s| &mut s.rt)?;
    }
    if perms.api.contains(&ApiName::Event) {
        event::bindings::wired::event::api::add_to_linker::<_, HasSelf<_>>(linker, |s| &mut s.rt)?;
        event::bindings::wired::event::types::add_to_linker::<_, HasSelf<_>>(linker, |s| {
            &mut s.rt
        })?;
    }
    if perms.api.contains(&ApiName::Input) {
        input::bindings::wired::input::api::add_to_linker::<_, HasSelf<_>>(linker, |s| &mut s.rt)?;
        input::bindings::wired::input::types::add_to_linker::<_, HasSelf<_>>(linker, |s| {
            &mut s.rt
        })?;
    }
    if perms.api.contains(&ApiName::InputContext) {
        input::bindings::wired::input::api::add_to_linker::<_, HasSelf<_>>(linker, |s| &mut s.rt)?;
    }
    if perms.api.contains(&ApiName::Wds) {
        wds::bindings::wired::wds::api::add_to_linker::<_, HasSelf<_>>(linker, |s| &mut s.rt)?;
        wds::bindings::wired::wds::types::add_to_linker::<_, HasSelf<_>>(linker, |s| &mut s.rt)?;
    }
    Ok(())
}
