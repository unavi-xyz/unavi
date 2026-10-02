//! `wired:peer`.

use wasmtime::component::Resource;

use super::{
    HostCtx,
    generated::wired::peer::{
        authority,
        identity,
    },
};
use crate::{
    error::ScriptError,
    host::{
        peer,
        scene::DocumentRes,
    },
};

impl identity::Host for HostCtx {
    fn self_did(&mut self) -> Result<String, ScriptError> {
        peer::self_did(&self.host)
    }
}

impl authority::Host for HostCtx {
    fn author(&mut self, doc: Resource<DocumentRes>) -> Result<Option<String>, ScriptError> {
        peer::author(&self.host, doc.rep())
    }

    fn holder(&mut self, doc: Resource<DocumentRes>) -> Result<Option<String>, ScriptError> {
        peer::holder(&self.host, doc.rep())
    }

    fn is_author(&mut self, doc: Resource<DocumentRes>) -> Result<bool, ScriptError> {
        peer::is_author(&self.host, doc.rep())
    }

    fn is_holder(&mut self, doc: Resource<DocumentRes>) -> Result<bool, ScriptError> {
        peer::is_holder(&self.host, doc.rep())
    }

    fn take_hold(&mut self, doc: Resource<DocumentRes>) -> Result<(), ScriptError> {
        peer::take_hold(&self.host, doc.rep())
    }

    fn release_hold(
        &mut self,
        doc: Resource<DocumentRes>,
        to: Option<String>,
    ) -> Result<(), ScriptError> {
        peer::release_hold(&self.host, doc.rep(), to.as_deref())
    }
}
