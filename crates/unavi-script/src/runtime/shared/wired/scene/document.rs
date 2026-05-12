use std::sync::Arc;

use blake3::Hash;
use hsd::topology::HydratedTopology;
use loro::{LoroDoc, LoroMap, LoroTree, LoroValue, TreeParentId, ValueOrContainer};
use loro_surgeon::Reconcile;

use crate::{
    firewall::Channel,
    runtime::shared::{
        Api,
        registry::firewall::validate_firewall,
        wired::scene::{material::MaterialRes, mesh::MeshRes, node::NodeRes},
    },
    util::gen_id,
};

#[derive(Clone)]
pub struct DocRes {
    pub doc: Arc<LoroDoc>,
    pub id: Hash,
}

pub fn id(api: &Api, rep: u32) -> anyhow::Result<Vec<u8>> {
    api.wired_scene
        .try_lock()?
        .docs
        .get(rep)
        .map(|d| d.id.as_bytes().to_vec())
        .ok_or_else(|| anyhow::anyhow!("invalid doc"))
}

pub fn clone(api: &Api, rep: u32) -> anyhow::Result<u32> {
    api.wired_scene
        .try_lock()?
        .docs
        .insert_clone(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid doc"))
}

pub fn on_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_scene.try_lock()?.docs.remove(rep);
    Ok(())
}

pub fn assets(api: &Api, rep: u32) -> anyhow::Result<Vec<(String, Vec<u8>)>> {
    let scene = api.wired_scene.try_lock()?;
    let res = scene
        .docs
        .get(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid doc rep: {rep}"))?
        .clone();
    validate_firewall(&api.doc_id, &res.id, Channel::SceneWrite)?;

    let mut items = Vec::new();
    res.doc
        .get_map("hsd")
        .get_or_create_container("assets", LoroMap::new())?
        .for_each(|name, v| {
            if let ValueOrContainer::Value(LoroValue::Binary(bytes)) = v {
                if bytes.len() == 32 {
                    items.push((name.to_string(), bytes.to_vec()));
                }
            }
        });
    drop(scene);

    Ok(items)
}

pub fn add_asset(api: &Api, rep: u32, name: String, blob_id: Vec<u8>) -> anyhow::Result<()> {
    let res = api
        .wired_scene
        .try_lock()?
        .docs
        .get(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid doc rep: {rep}"))?
        .clone();
    validate_firewall(&api.doc_id, &res.id, Channel::SceneWrite)?;

    let assets_map = res
        .doc
        .get_map("hsd")
        .get_or_create_container("assets", LoroMap::new())?;
    assets_map.insert(name.as_str(), LoroValue::Binary(blob_id.into()))?;
    Ok(())
}

pub fn remove_asset(api: &Api, rep: u32, name: String) -> anyhow::Result<()> {
    let res = api
        .wired_scene
        .try_lock()?
        .docs
        .get(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid doc rep: {rep}"))?
        .clone();
    validate_firewall(&api.doc_id, &res.id, Channel::SceneWrite)?;

    let assets_map = res
        .doc
        .get_map("hsd")
        .get_or_create_container("assets", LoroMap::new())?;
    assets_map.insert(name.as_str(), LoroValue::Null)?;
    Ok(())
}

pub fn roots(api: &Api, rep: u32) -> anyhow::Result<Vec<u32>> {
    let res = api
        .wired_scene
        .try_lock()?
        .docs
        .get(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid doc rep: {rep}"))?
        .clone();

    let tree_ids = res
        .doc
        .get_map("hsd")
        .get_or_create_container("nodes", LoroTree::new())?
        .roots();

    let mut scene = api.wired_scene.try_lock()?;
    Ok(tree_ids
        .into_iter()
        .map(|id| {
            scene.nodes.insert(NodeRes {
                doc: Arc::clone(&res.doc),
                doc_id: res.id,
                id,
                is_proxy: false,
            })
        })
        .collect())
}

pub fn nodes(api: &Api, rep: u32) -> anyhow::Result<Vec<u32>> {
    let res = api
        .wired_scene
        .try_lock()?
        .docs
        .get(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid doc rep: {rep}"))?
        .clone();

    let tree_ids = res
        .doc
        .get_map("hsd")
        .get_or_create_container("nodes", LoroTree::new())?
        .nodes();

    let mut scene = api.wired_scene.try_lock()?;
    Ok(tree_ids
        .into_iter()
        .map(|id| {
            scene.nodes.insert(NodeRes {
                doc: Arc::clone(&res.doc),
                doc_id: res.id,
                id,
                is_proxy: false,
            })
        })
        .collect())
}

pub fn create_node(api: &Api, rep: u32) -> anyhow::Result<u32> {
    let mut scene = api.wired_scene.try_lock()?;
    let res = scene
        .docs
        .get(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid doc rep: {rep}"))?
        .clone();
    validate_firewall(&api.doc_id, &res.id, Channel::SceneWrite)?;

    let tree = res
        .doc
        .get_map("hsd")
        .get_or_create_container("nodes", LoroTree::new())?;
    let tree_id = tree.create(TreeParentId::Root)?;

    Ok(scene.nodes.insert(NodeRes {
        doc: res.doc,
        doc_id: res.id,
        id: tree_id,
        is_proxy: false,
    }))
}

pub fn remove_node(api: &Api, rep: u32) -> anyhow::Result<()> {
    let Some(node) = api.wired_scene.try_lock()?.nodes.remove(rep) else {
        return Ok(());
    };
    if node.is_proxy {
        return Ok(());
    }
    validate_firewall(&api.doc_id, &node.doc_id, Channel::SceneWrite)?;

    let tree = node
        .doc
        .get_map("hsd")
        .get_or_create_container("nodes", LoroTree::new())?;
    tree.delete(node.id)?;
    Ok(())
}

pub fn meshes(api: &Api, rep: u32) -> anyhow::Result<Vec<u32>> {
    let mut scene = api.wired_scene.try_lock()?;
    let res = scene
        .docs
        .get(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid doc rep: {rep}"))?
        .clone();
    validate_firewall(&api.doc_id, &res.id, Channel::SceneWrite)?;

    let mut ids = Vec::new();
    res.doc
        .get_map("hsd")
        .get_or_create_container("meshes", LoroMap::new())?
        .for_each(|id, v| match v {
            ValueOrContainer::Container(_) => {
                ids.push(scene.meshes.insert(MeshRes {
                    doc: Arc::clone(&res.doc),
                    doc_id: res.id,
                    id: id.into(),
                }));
            }
            ValueOrContainer::Value(_) => {}
        });
    drop(scene);

    Ok(ids)
}

pub fn create_mesh(api: &Api, rep: u32) -> anyhow::Result<u32> {
    let mut scene = api.wired_scene.try_lock()?;
    let res = scene
        .docs
        .get(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid doc rep: {rep}"))?
        .clone();
    validate_firewall(&api.doc_id, &res.id, Channel::SceneWrite)?;

    let id = gen_id();
    let meshes_map = res
        .doc
        .get_map("hsd")
        .get_or_create_container("meshes", LoroMap::new())?;
    let mesh_map = meshes_map.get_or_create_container(id.as_str(), LoroMap::new())?;
    HydratedTopology::default().reconcile_field(&mesh_map, "topology")?;

    Ok(scene.meshes.insert(MeshRes {
        doc: res.doc,
        doc_id: res.id,
        id,
    }))
}

pub fn remove_mesh(api: &Api, rep: u32) -> anyhow::Result<()> {
    let Some(mesh) = api.wired_scene.try_lock()?.meshes.remove(rep) else {
        return Ok(());
    };
    validate_firewall(&api.doc_id, &mesh.doc_id, Channel::SceneWrite)?;

    let meshes_map = mesh
        .doc
        .get_map("hsd")
        .get_or_create_container("meshes", LoroMap::new())?;
    meshes_map.insert(mesh.id.as_str(), LoroValue::Null)?;
    Ok(())
}

pub fn materials(api: &Api, rep: u32) -> anyhow::Result<Vec<u32>> {
    let mut scene = api.wired_scene.try_lock()?;
    let res = scene
        .docs
        .get(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid doc rep: {rep}"))?
        .clone();
    validate_firewall(&api.doc_id, &res.id, Channel::SceneWrite)?;

    let mut ids = Vec::new();
    res.doc
        .get_map("hsd")
        .get_or_create_container("materials", LoroMap::new())?
        .for_each(|id, v| match v {
            ValueOrContainer::Container(_) => {
                ids.push(scene.materials.insert(MaterialRes {
                    doc: Arc::clone(&res.doc),
                    doc_id: res.id,
                    id: id.into(),
                }));
            }
            ValueOrContainer::Value(_) => {}
        });
    drop(scene);

    Ok(ids)
}

pub fn create_material(api: &Api, rep: u32) -> anyhow::Result<u32> {
    let mut scene = api.wired_scene.try_lock()?;
    let res = scene
        .docs
        .get(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid doc rep: {rep}"))?
        .clone();
    validate_firewall(&api.doc_id, &res.id, Channel::SceneWrite)?;

    let id = gen_id();
    let materials_map = res
        .doc
        .get_map("hsd")
        .get_or_create_container("materials", LoroMap::new())?;
    materials_map.get_or_create_container(id.as_str(), LoroMap::new())?;

    Ok(scene.materials.insert(MaterialRes {
        doc: res.doc,
        doc_id: res.id,
        id,
    }))
}

pub fn remove_material(api: &Api, rep: u32) -> anyhow::Result<()> {
    let Some(mat) = api.wired_scene.try_lock()?.materials.remove(rep) else {
        return Ok(());
    };
    validate_firewall(&api.doc_id, &mat.doc_id, Channel::SceneWrite)?;

    let materials_map = mat
        .doc
        .get_map("hsd")
        .get_or_create_container("materials", LoroMap::new())?;
    materials_map.insert(mat.id.as_str(), LoroValue::Null)?;
    Ok(())
}
