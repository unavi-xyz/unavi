use std::sync::Arc;

use blake3::Hash;
use hsd::HsdMaterial;
use loro::{LoroDoc, LoroMap};
use loro_surgeon::{Hydrate, Reconcile};
use smol_str::SmolStr;

use crate::{
    firewall::Channel,
    runtime::shared::{Api, registry::firewall::validate_firewall},
};

#[derive(Clone)]
pub struct MaterialRes {
    pub doc: Arc<LoroDoc>,
    pub doc_id: Hash,
    pub id: SmolStr,
}

#[derive(Clone, Copy, Default)]
pub struct MaterialColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Clone, Copy)]
pub enum MaterialAlphaMode {
    Add,
    Blend,
    Mask,
    Multiply,
    Opaque,
    PreMultiplied,
}

pub fn clone(api: &Api, rep: u32) -> anyhow::Result<u32> {
    api.wired_scene
        .try_lock()?
        .materials
        .insert_clone(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid material"))
}

pub fn on_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_scene.try_lock()?.materials.remove(rep);
    Ok(())
}

fn get_material(api: &Api, rep: u32) -> anyhow::Result<MaterialRes> {
    api.wired_scene
        .try_lock()?
        .materials
        .get(rep)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("invalid material rep: {rep}"))
}

fn material_map(doc: &LoroDoc, id: &str) -> anyhow::Result<LoroMap> {
    doc.get_map("hsd")
        .get_or_create_container("materials", LoroMap::new())?
        .get_or_create_container(id, LoroMap::new())
        .map_err(Into::into)
}

fn hydrate_material(map: &LoroMap) -> HsdMaterial {
    HsdMaterial::hydrate(&map.get_deep_value()).unwrap_or_default()
}

pub fn id(api: &Api, rep: u32) -> anyhow::Result<String> {
    Ok(get_material(api, rep)?.id.to_string())
}

pub fn name(api: &Api, rep: u32) -> anyhow::Result<Option<String>> {
    let mat = get_material(api, rep)?;
    let map = material_map(&mat.doc, &mat.id)?;
    Ok(hydrate_material(&map).name.map(|s| s.to_string()))
}

pub fn set_name(api: &Api, rep: u32, value: Option<String>) -> anyhow::Result<()> {
    let mat = get_material(api, rep)?;
    validate_firewall(&api.doc_id, &mat.doc_id, Channel::SceneWrite)?;
    let map = material_map(&mat.doc, &mat.id)?;
    let mut data = hydrate_material(&map);
    data.name = value.map(SmolStr::from);
    data.reconcile(&map)?;
    Ok(())
}

pub fn alpha_cutoff(api: &Api, rep: u32) -> anyhow::Result<f32> {
    let mat = get_material(api, rep)?;
    let map = material_map(&mat.doc, &mat.id)?;
    Ok(hydrate_material(&map).alpha_cutoff.unwrap_or(0.5) as f32)
}

pub fn set_alpha_cutoff(api: &Api, rep: u32, value: f32) -> anyhow::Result<()> {
    let mat = get_material(api, rep)?;
    validate_firewall(&api.doc_id, &mat.doc_id, Channel::SceneWrite)?;
    let map = material_map(&mat.doc, &mat.id)?;
    let mut data = hydrate_material(&map);
    data.alpha_cutoff = Some(value as f64);
    data.reconcile(&map)?;
    Ok(())
}

pub fn alpha_mode(api: &Api, rep: u32) -> anyhow::Result<Option<MaterialAlphaMode>> {
    let mat = get_material(api, rep)?;
    let map = material_map(&mat.doc, &mat.id)?;
    Ok(hydrate_material(&map).alpha_mode.and_then(|s| {
        match s.as_str() {
            "add" => Some(MaterialAlphaMode::Add),
            "blend" => Some(MaterialAlphaMode::Blend),
            "mask" => Some(MaterialAlphaMode::Mask),
            "multiply" => Some(MaterialAlphaMode::Multiply),
            "opaque" => Some(MaterialAlphaMode::Opaque),
            "premultiplied" => Some(MaterialAlphaMode::PreMultiplied),
            _ => None,
        }
    }))
}

pub fn set_alpha_mode(api: &Api, rep: u32, value: Option<MaterialAlphaMode>) -> anyhow::Result<()> {
    let mat = get_material(api, rep)?;
    validate_firewall(&api.doc_id, &mat.doc_id, Channel::SceneWrite)?;
    let map = material_map(&mat.doc, &mat.id)?;
    let mut data = hydrate_material(&map);
    data.alpha_mode = value.map(|m| {
        SmolStr::from(match m {
            MaterialAlphaMode::Add => "add",
            MaterialAlphaMode::Blend => "blend",
            MaterialAlphaMode::Mask => "mask",
            MaterialAlphaMode::Multiply => "multiply",
            MaterialAlphaMode::Opaque => "opaque",
            MaterialAlphaMode::PreMultiplied => "premultiplied",
        })
    });
    data.reconcile(&map)?;
    Ok(())
}

pub fn base_color(api: &Api, rep: u32) -> anyhow::Result<MaterialColor> {
    let mat = get_material(api, rep)?;
    let map = material_map(&mat.doc, &mat.id)?;
    let c = hydrate_material(&map)
        .base_color
        .unwrap_or_else(|| vec![1.0, 1.0, 1.0, 1.0]);
    Ok(MaterialColor {
        r: c.first().copied().unwrap_or(1.0) as f32,
        g: c.get(1).copied().unwrap_or(1.0) as f32,
        b: c.get(2).copied().unwrap_or(1.0) as f32,
        a: c.get(3).copied().unwrap_or(1.0) as f32,
    })
}

pub fn set_base_color(api: &Api, rep: u32, value: MaterialColor) -> anyhow::Result<()> {
    let mat = get_material(api, rep)?;
    validate_firewall(&api.doc_id, &mat.doc_id, Channel::SceneWrite)?;
    let map = material_map(&mat.doc, &mat.id)?;
    let mut data = hydrate_material(&map);
    data.base_color = Some(vec![
        value.r as f64,
        value.g as f64,
        value.b as f64,
        value.a as f64,
    ]);
    data.reconcile(&map)?;
    Ok(())
}

pub fn metallic(api: &Api, rep: u32) -> anyhow::Result<f32> {
    let mat = get_material(api, rep)?;
    let map = material_map(&mat.doc, &mat.id)?;
    Ok(hydrate_material(&map).metallic.unwrap_or(0.0) as f32)
}

pub fn set_metallic(api: &Api, rep: u32, value: f32) -> anyhow::Result<()> {
    let mat = get_material(api, rep)?;
    validate_firewall(&api.doc_id, &mat.doc_id, Channel::SceneWrite)?;
    let map = material_map(&mat.doc, &mat.id)?;
    let mut data = hydrate_material(&map);
    data.metallic = Some(value as f64);
    data.reconcile(&map)?;
    Ok(())
}

pub fn roughness(api: &Api, rep: u32) -> anyhow::Result<f32> {
    let mat = get_material(api, rep)?;
    let map = material_map(&mat.doc, &mat.id)?;
    Ok(hydrate_material(&map).roughness.unwrap_or(0.5) as f32)
}

pub fn set_roughness(api: &Api, rep: u32, value: f32) -> anyhow::Result<()> {
    let mat = get_material(api, rep)?;
    validate_firewall(&api.doc_id, &mat.doc_id, Channel::SceneWrite)?;
    let map = material_map(&mat.doc, &mat.id)?;
    let mut data = hydrate_material(&map);
    data.roughness = Some(value as f64);
    data.reconcile(&map)?;
    Ok(())
}

pub fn double_sided(api: &Api, rep: u32) -> anyhow::Result<bool> {
    let mat = get_material(api, rep)?;
    let map = material_map(&mat.doc, &mat.id)?;
    Ok(hydrate_material(&map).double_sided.unwrap_or(false))
}

pub fn set_double_sided(api: &Api, rep: u32, value: bool) -> anyhow::Result<()> {
    let mat = get_material(api, rep)?;
    validate_firewall(&api.doc_id, &mat.doc_id, Channel::SceneWrite)?;
    let map = material_map(&mat.doc, &mat.id)?;
    let mut data = hydrate_material(&map);
    data.double_sided = Some(value);
    data.reconcile(&map)?;
    Ok(())
}

pub fn unlit(api: &Api, rep: u32) -> anyhow::Result<bool> {
    let mat = get_material(api, rep)?;
    let map = material_map(&mat.doc, &mat.id)?;
    Ok(hydrate_material(&map).unlit.unwrap_or(false))
}

pub fn set_unlit(api: &Api, rep: u32, value: bool) -> anyhow::Result<()> {
    let mat = get_material(api, rep)?;
    validate_firewall(&api.doc_id, &mat.doc_id, Channel::SceneWrite)?;
    let map = material_map(&mat.doc, &mat.id)?;
    let mut data = hydrate_material(&map);
    data.unlit = Some(value);
    data.reconcile(&map)?;
    Ok(())
}
