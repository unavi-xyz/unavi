//! `wired:scene`.

use wasmtime::component::Resource;

use super::{
    HostCtx,
    convert,
    generated::wired::{
        core::{
            ids::{
                DocumentId,
                PrimId,
            },
            math::Transform,
        },
        scene::{
            document::{
                Anchor,
                Edit,
                Host,
                HostDocument,
                Layer,
            },
            properties::{
                self,
                Property,
                PropertyKey,
            },
        },
    },
};
use crate::{
    error::ScriptError,
    host::scene::{
        self,
        DocumentRes,
        documents,
        edit,
        place,
    },
};

impl properties::Host for HostCtx {}

impl HostDocument for HostCtx {
    fn id(&mut self, doc: Resource<DocumentRes>) -> wasmtime::Result<DocumentId> {
        Ok(convert::wit_doc_id(scene::id(&self.host, doc.rep())?))
    }

    fn owned(&mut self, doc: Resource<DocumentRes>) -> wasmtime::Result<bool> {
        Ok(scene::owned(&self.host, doc.rep())?)
    }

    fn contains(&mut self, doc: Resource<DocumentRes>, prim: PrimId) -> wasmtime::Result<bool> {
        Ok(scene::contains(
            &self.host,
            doc.rep(),
            convert::prim_id(prim),
        )?)
    }

    fn roots(&mut self, doc: Resource<DocumentRes>) -> wasmtime::Result<Vec<PrimId>> {
        Ok(scene::roots(&self.host, doc.rep())?
            .into_iter()
            .map(convert::wit_prim_id)
            .collect())
    }

    fn prims(&mut self, doc: Resource<DocumentRes>) -> wasmtime::Result<Vec<PrimId>> {
        Ok(scene::prims(&self.host, doc.rep())?
            .into_iter()
            .map(convert::wit_prim_id)
            .collect())
    }

    fn children(
        &mut self,
        doc: Resource<DocumentRes>,
        prim: PrimId,
    ) -> wasmtime::Result<Vec<PrimId>> {
        Ok(
            scene::children(&self.host, doc.rep(), convert::prim_id(prim))?
                .into_iter()
                .map(convert::wit_prim_id)
                .collect(),
        )
    }

    fn find_by_name(
        &mut self,
        doc: Resource<DocumentRes>,
        name: String,
    ) -> wasmtime::Result<Vec<PrimId>> {
        Ok(scene::find_by_name(&self.host, doc.rep(), &name)?
            .into_iter()
            .map(convert::wit_prim_id)
            .collect())
    }

    /// A key naming nothing a script could have written, such as a custom
    /// key in a built-in group, has no value.
    fn get(
        &mut self,
        doc: Resource<DocumentRes>,
        prim: PrimId,
        key: PropertyKey,
    ) -> wasmtime::Result<Option<Property>> {
        let Ok(key) = convert::property_key(key) else {
            return Ok(None);
        };
        Ok(
            scene::get(&self.host, doc.rep(), convert::prim_id(prim), &key)?
                .map(convert::wit_property),
        )
    }

    fn get_many(
        &mut self,
        doc: Resource<DocumentRes>,
        keys: Vec<(PrimId, PropertyKey)>,
    ) -> wasmtime::Result<Vec<Option<Property>>> {
        let keys = keys
            .into_iter()
            .map(|(prim, key)| Some((convert::prim_id(prim), convert::property_key(key).ok()?)))
            .collect::<Vec<_>>();
        let known = keys.iter().flatten().cloned().collect::<Vec<_>>();
        let mut values = scene::get_many(&self.host, doc.rep(), &known)?.into_iter();
        Ok(keys
            .into_iter()
            .map(|key| {
                key.and_then(|_| values.next().flatten())
                    .map(convert::wit_property)
            })
            .collect())
    }

    fn keys(
        &mut self,
        doc: Resource<DocumentRes>,
        prim: PrimId,
    ) -> wasmtime::Result<Vec<PropertyKey>> {
        Ok(scene::keys(&self.host, doc.rep(), convert::prim_id(prim))?
            .into_iter()
            .map(convert::wit_property_key)
            .collect())
    }

    fn world_transform(
        &mut self,
        doc: Resource<DocumentRes>,
        prim: PrimId,
    ) -> Result<Transform, ScriptError> {
        scene::world_transform(&self.host, doc.rep(), convert::prim_id(prim))
            .map(convert::wit_transform)
    }

    fn offset_to(
        &mut self,
        doc: Resource<DocumentRes>,
        other: Resource<DocumentRes>,
    ) -> wasmtime::Result<Option<Transform>> {
        Ok(scene::offset_to(&self.host, doc.rep(), other.rep())?.map(convert::wit_transform))
    }

    async fn create_prim(
        &mut self,
        doc: Resource<DocumentRes>,
        layer: Layer,
        parent: Option<PrimId>,
    ) -> Result<PrimId, ScriptError> {
        edit::create_prim(
            &self.host,
            doc.rep(),
            convert::layer(layer),
            parent.map(convert::prim_id),
        )
        .await
        .map(convert::wit_prim_id)
    }

    async fn apply(
        &mut self,
        doc: Resource<DocumentRes>,
        layer: Layer,
        edits: Vec<Edit>,
    ) -> Result<(), ScriptError> {
        if edits.len() > edit::MAX_EDITS {
            return Err(ScriptError::invalid("at most 4096 edits per apply"));
        }
        let edits = edits
            .into_iter()
            .map(convert::edit)
            .collect::<Result<_, _>>()?;
        edit::apply(&self.host, doc.rep(), convert::layer(layer), edits).await
    }

    async fn commit(
        &mut self,
        doc: Resource<DocumentRes>,
        keys: Vec<(PrimId, PropertyKey)>,
    ) -> Result<(), ScriptError> {
        let keys = keys
            .into_iter()
            .map(|(prim, key)| Ok((convert::prim_id(prim), convert::property_key(key)?)))
            .collect::<Result<_, ScriptError>>()?;
        edit::commit(&self.host, doc.rep(), keys).await
    }

    async fn place(
        &mut self,
        doc: Resource<DocumentRes>,
        anchor: Anchor,
        offset: Transform,
    ) -> Result<(), ScriptError> {
        let anchor = match anchor {
            Anchor::Space => place::Anchor::Space,
            Anchor::Prim(target) => {
                let (doc, prim) = convert::prim_ref(target);
                place::Anchor::Prim(doc, prim)
            }
        };
        place::place(&self.host, doc.rep(), anchor, convert::xform(offset)).await
    }

    fn drop(&mut self, doc: Resource<DocumentRes>) -> wasmtime::Result<()> {
        Ok(scene::drop_document(&mut self.host, doc.rep())?)
    }
}

impl Host for HostCtx {
    fn script_document(&mut self) -> Result<Resource<DocumentRes>, ScriptError> {
        scene::script_document(&mut self.host).map(Resource::new_own)
    }

    fn script_prim(&mut self) -> wasmtime::Result<PrimId> {
        Ok(convert::wit_prim_id(scene::script_prim(&self.host)))
    }

    async fn open_document(
        &mut self,
        id: DocumentId,
    ) -> Result<Option<Resource<DocumentRes>>, ScriptError> {
        Ok(scene::open_document(&mut self.host, convert::doc_id(id))
            .await?
            .map(Resource::new_own))
    }

    async fn create_document(&mut self) -> Result<Resource<DocumentRes>, ScriptError> {
        documents::create_document(&mut self.host)
            .await
            .map(Resource::new_own)
    }

    async fn copy_document(
        &mut self,
        source: Resource<DocumentRes>,
    ) -> Result<Resource<DocumentRes>, ScriptError> {
        documents::copy_document(&mut self.host, source.rep())
            .await
            .map(Resource::new_own)
    }

    async fn delete_document(&mut self, doc: Resource<DocumentRes>) -> Result<(), ScriptError> {
        documents::delete_document(&mut self.host, doc.rep()).await
    }
}
