use std::{
    collections::HashSet,
    sync::Arc,
    time::Duration,
};

use anyhow::Context;
use bevy::{
    asset::{
        AssetLoader,
        AsyncReadExt,
        LoadContext,
        io::Reader,
    },
    platform::collections::HashMap,
    prelude::*,
    reflect::TypePath,
    tasks::{
        BoxedFuture,
        ConditionalSendFuture,
    },
};
use bevy_wds::{
    LocalActor,
    LocalBlobs,
    record::{
        read::ReadRecord,
        write::{
            SchemaDef,
            WriteRecord,
        },
    },
};
use blake3::Hash;
use hsd::{
    HSD_CONTAINER_ID,
    attributes::{
        collider::ColliderAttr,
        hydrate_attr,
        subdocument::SubdocumentAttr,
    },
    file::{
        HsdFile,
        HsdFilePrim,
    },
};
use loro::LoroDoc;
use loro_surgeon::bytes::ByteArray;
use unavi_util::{
    async_commands::AsyncCommands,
    async_task::spawn_async_task,
};
use wds::{
    Blobs,
    actor::Actor,
};
use wired_schemas::SCHEMA_HSD;

use crate::{
    Hsd,
    HsdChild,
    HsdRecordId,
    Prim,
    attributes::subdocument::HsdSubdocument,
};

const DEFAULT_TTL: Duration = Duration::from_hours(7 * 24);

#[derive(Asset, TypePath)]
pub struct HsdAsset {
    pub file: Arc<HsdFile>,
    pub deps: HashMap<Hash, Handle<BlobAsset>>,
}

#[derive(Asset, Debug, Deref, DerefMut, TypePath)]
pub struct BlobAsset(pub Vec<u8>);

#[derive(Default, TypePath)]
pub struct HsdLoader;

impl AssetLoader for HsdLoader {
    type Asset = HsdAsset;
    type Settings = ();
    type Error = anyhow::Error;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut s = String::new();
            reader.read_to_string(&mut s).await?;
            let file = Arc::new(HsdFile::from_ron(&s)?);

            let parent = load_context
                .path()
                .path()
                .parent()
                .expect("asset parent dir")
                .to_path_buf();

            let mut deps = HashMap::new();
            let mut seen_assets: HashSet<Hash> = HashSet::new();
            let mut queue: Vec<Arc<HsdFile>> = vec![Arc::clone(&file)];

            while let Some(current) = queue.pop() {
                collect_blob_deps(&current, &mut |hash| {
                    let h = Hash::from_bytes(hash);
                    let path = parent.join(h.to_string());
                    deps.entry(h).or_insert_with(|| load_context.load(path));
                });

                let mut nested = Vec::new();
                collect_asset_deps(&current, &mut |hash| {
                    let h = Hash::from_bytes(hash);
                    if seen_assets.insert(h) {
                        nested.push(h);
                    }
                });

                for asset_hash in nested {
                    let asset_path = parent.join(asset_hash.to_string());
                    let bytes = load_context.read_asset_bytes(asset_path).await?;
                    let asset_str = std::str::from_utf8(&bytes)?;
                    let asset_file = Arc::new(HsdFile::from_ron(asset_str)?);
                    queue.push(asset_file);
                }
            }

            Ok(HsdAsset { file, deps })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["hsd"]
    }
}

#[derive(Default, TypePath)]
pub struct BlobLoader;

impl AssetLoader for BlobLoader {
    type Asset = BlobAsset;
    type Settings = ();
    type Error = anyhow::Error;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            Ok(BlobAsset(bytes))
        })
    }

    fn extensions(&self) -> &[&str] {
        &[]
    }
}

fn collect_blob_deps(file: &HsdFile, push: &mut impl FnMut([u8; 32])) {
    for prim in &file.0 {
        walk_prim(prim, push);
    }
}

fn collect_asset_deps(file: &HsdFile, push: &mut impl FnMut([u8; 32])) {
    for prim in &file.0 {
        walk_asset_prim(prim, push);
    }
}

fn walk_asset_prim(prim: &HsdFilePrim, push: &mut impl FnMut([u8; 32])) {
    if let Some(asset) = &prim.attributes.asset {
        push(asset.0.0);
    }
    if let Some(SubdocumentAttr::Template(blob)) = &prim.attributes.subdocument {
        push(blob.0);
    }
    for child in &prim.children {
        walk_asset_prim(child, push);
    }
}

fn walk_prim(prim: &HsdFilePrim, push: &mut impl FnMut([u8; 32])) {
    let a = &prim.attributes;
    if let Some(asset) = &a.asset {
        push(asset.0.0);
    }
    if let Some(SubdocumentAttr::Template(blob)) = &a.subdocument {
        push(blob.0);
    }
    if let Some(script) = &a.script {
        push(script.0.0);
    }
    if let Some(img) = &a.image {
        push(img.data.0);
    }
    if let Some(mesh) = &a.mesh {
        for b in mesh.attributes.values() {
            push(b.0);
        }
        if let Some(idx) = &mesh.indices {
            push(idx.0);
        }
    }
    if let Some(c) = &a.collider {
        match c {
            ColliderAttr::ConvexHull(b) => push(b.0),
            ColliderAttr::Trimesh { indices, vertices } => {
                push(indices.0);
                push(vertices.0);
            }
            _ => {}
        }
    }
    for child in &prim.children {
        walk_prim(child, push);
    }
}

pub struct OnLoadCtx {
    pub doc:       LoroDoc,
    pub entity:    Entity,
    pub record_id: Hash,
}

pub type OnLoadFn =
    Box<dyn FnOnce(OnLoadCtx) -> BoxedFuture<'static, anyhow::Result<()>> + Send + Sync>;

#[derive(Component)]
pub struct LoadHsd {
    pub handle:        Handle<HsdAsset>,
    pub extra_schemas: Option<Vec<SchemaDef>>,
    pub on_load:       Option<OnLoadFn>,
    pub public:        bool,
}

pub fn instance_hsd(
    asset_server: Res<AssetServer>,
    hsds: Res<Assets<HsdAsset>>,
    blobs: Res<Assets<BlobAsset>>,
    loading: Query<(Entity, &mut LoadHsd)>,
    actors: Query<(&LocalActor, &LocalBlobs)>,
    mut commands: Commands,
) {
    let Ok((actor, local_blobs)) = actors.single() else {
        return;
    };

    'loading: for (entity, mut load) in loading {
        let Some(asset) = hsds.get(&load.handle) else {
            continue;
        };

        let mut blob_map = HashMap::new();
        let mut failed = false;
        for (hash, blob_handle) in &asset.deps {
            if let Some(blob) = blobs.get(blob_handle) {
                blob_map.insert(*hash, blob.0.clone());
                continue;
            }
            if let Some(bevy::asset::LoadState::Failed(err)) =
                asset_server.get_load_state(blob_handle)
            {
                error!(blob = %hash, ?err, "hsd blob dependency failed to load; aborting instance");
                failed = true;
            }
        }
        if failed {
            commands.entity(entity).remove::<LoadHsd>();
            continue;
        }
        if blob_map.len() != asset.deps.len() {
            continue 'loading;
        }

        let actor = actor.0.clone();
        let local_blobs = local_blobs.0.clone();
        let prims = asset.file.0.clone();
        let public = load.public;
        let extra_schemas = load.extra_schemas.take().unwrap_or_default();
        let on_load = load.on_load.take();

        spawn_async_task(async move {
            if let Err(err) = build_and_instance(
                actor,
                local_blobs,
                blob_map,
                prims,
                public,
                extra_schemas,
                entity,
                on_load,
            )
            .await
            {
                error!(?err, "Failed to build HSD record");
            }
        });

        commands.entity(entity).remove::<LoadHsd>();
    }
}

#[expect(clippy::too_many_arguments)]
// n0_future futures are intentionally !Send on wasm (single-threaded, no
// Send needed there); Send-bounded elsewhere.
#[cfg_attr(target_family = "wasm", expect(clippy::future_not_send))]
async fn build_and_instance(
    actor: Actor,
    local_blobs: Blobs,
    blob_map: HashMap<Hash, Vec<u8>>,
    mut prims: Vec<HsdFilePrim>,
    public: bool,
    extra_schemas: Vec<SchemaDef>,
    entity: Entity,
    on_load: Option<OnLoadFn>,
) -> anyhow::Result<()> {
    for blob in blob_map.values() {
        local_blobs.add_slice(blob).await?;
    }

    materialize_subdocuments(&mut prims, &blob_map, public).await?;

    let record_id = write_hsd_record(prims, public, extra_schemas).await?;
    let doc = actor.read(record_id).send().await?;

    if let Some(on_load) = on_load {
        on_load(OnLoadCtx {
            doc,
            entity,
            record_id,
        })
        .await
        .context("on_load")?;
    }

    Ok(())
}

/// Writes each `Template` subdocument (its own subdocuments first) as a record,
/// rewriting the prim's attribute to the resulting `Record` id.
fn materialize_subdocuments<'a>(
    prims: &'a mut [HsdFilePrim],
    blob_map: &'a HashMap<Hash, Vec<u8>>,
    public: bool,
) -> BoxedFuture<'a, anyhow::Result<()>> {
    Box::pin(async move {
        for prim in prims.iter_mut() {
            let template = match &prim.attributes.subdocument {
                Some(SubdocumentAttr::Template(blob)) => Some(Hash::from_bytes(blob.0)),
                _ => None,
            };
            if let Some(blob_hash) = template {
                let bytes = blob_map
                    .get(&blob_hash)
                    .with_context(|| format!("subdocument blob {blob_hash} missing"))?;
                let mut sub_prims = HsdFile::from_ron(std::str::from_utf8(bytes)?)?.0;
                materialize_subdocuments(&mut sub_prims, blob_map, public).await?;
                let sub_id = write_hsd_record(sub_prims, public, Vec::new()).await?;
                prim.attributes.subdocument =
                    Some(SubdocumentAttr::Record(ByteArray::new(*sub_id.as_bytes())));
            }
            materialize_subdocuments(&mut prim.children, blob_map, public).await?;
        }
        Ok(())
    })
}

async fn write_hsd_record(
    prims: Vec<HsdFilePrim>,
    public: bool,
    extra_schemas: Vec<SchemaDef>,
) -> anyhow::Result<Hash> {
    let file = HsdFile(prims);
    let (mut write, rx, cancel) = WriteRecord::new(None);
    write.ttl = Some(DEFAULT_TTL);
    write.public = public;
    write.schemas = vec![SchemaDef {
        container: "hsd".into(),
        schema:    (&*SCHEMA_HSD).into(),
        f:         Arc::new(move |doc| {
            file.load_into_doc(doc)?;
            Ok(())
        }),
    }];
    write.schemas.extend(extra_schemas);
    AsyncCommands::default().trigger(write).send().await?;
    let id = rx.recv().await?;
    drop(cancel);
    Ok(id)
}

/// Records which subdocument record a prim currently has instanced, so a
/// changed target re-instances and a detached one is torn down.
#[derive(Component)]
pub struct SubdocLoaded(pub Hash);

pub fn instance_subdocuments(
    subdoc_prims: Query<
        (
            Entity,
            &Prim,
            &HsdChild,
            Option<&SubdocLoaded>,
            Option<&Children>,
        ),
        With<HsdSubdocument>,
    >,
    detached: Query<(Entity, Option<&Children>), (With<SubdocLoaded>, Without<HsdSubdocument>)>,
    hsd_docs: Query<(), With<Hsd>>,
    docs: Query<&Hsd>,
    mut commands: Commands,
) {
    for (prim_ent, children) in &detached {
        despawn_subdoc_docs(&mut commands, &hsd_docs, children);
        commands.entity(prim_ent).remove::<SubdocLoaded>();
    }

    for (prim_ent, prim, doc_ent, loaded, children) in &subdoc_prims {
        let Ok(parent) = docs.get(doc_ent.0) else {
            continue;
        };
        let Ok(meta) = parent.0.get_tree(&*HSD_CONTAINER_ID).get_meta(prim.0) else {
            continue;
        };
        let id = match hydrate_attr::<SubdocumentAttr>(&meta) {
            Ok(SubdocumentAttr::Record(id)) => Hash::from_bytes(id.0),
            _ => continue,
        };

        if let Some(loaded) = loaded {
            if loaded.0 == id {
                continue;
            }
            despawn_subdoc_docs(&mut commands, &hsd_docs, children);
        }

        commands.entity(prim_ent).insert(SubdocLoaded(id));

        let (mut event, rx, cancel) = ReadRecord::new(id);
        event.ttl = Some(DEFAULT_TTL);
        event.retries = 5;
        commands.trigger(event);

        spawn_async_task(async move {
            let _cancel = cancel;
            if let Ok(doc) = rx.recv().await {
                let _ = AsyncCommands::default()
                    .push(move |world: &mut World| {
                        let current = world
                            .get_entity(prim_ent)
                            .ok()
                            .and_then(|e| e.get::<SubdocLoaded>().map(|l| l.0));
                        if current == Some(id) {
                            world.spawn((Hsd(Arc::new(doc)), HsdRecordId(id), ChildOf(prim_ent)));
                        }
                    })
                    .send()
                    .await;
            }
        });
    }
}

fn despawn_subdoc_docs(
    commands: &mut Commands,
    hsd_docs: &Query<(), With<Hsd>>,
    children: Option<&Children>,
) {
    let Some(children) = children else {
        return;
    };
    for child in children.iter() {
        if hsd_docs.contains(child) {
            commands.entity(child).despawn();
        }
    }
}

#[must_use]
pub fn on_load_spawn_doc(ctx: OnLoadCtx) -> BoxedFuture<'static, anyhow::Result<()>> {
    Box::pin(async move {
        AsyncCommands::default()
            .push(move |world: &mut World| {
                world
                    .entity_mut(ctx.entity)
                    .insert((Hsd(Arc::new(ctx.doc)), HsdRecordId(ctx.record_id)));
            })
            .send()
            .await?;
        Ok(())
    })
}
