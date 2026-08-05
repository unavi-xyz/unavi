//! `.hsda` → `.hsdz`.
//!
//! Output is an entry set, not a prim tree: the same keys a live document
//! holds, with bulk inlined so the package is one self-contained blob.

use std::{
    collections::{
        BTreeMap,
        HashMap,
    },
    path::{
        Path,
        PathBuf,
    },
};

use anyhow::{
    Context,
    Result,
    bail,
};
use hsd::{
    attributes::{
        Attribute,
        collider::ColliderAttr,
        gravity_scale::GravityScaleAttr,
        image::ImageAttr,
        material::{
            self,
            ColorVec,
            MaterialAttr,
        },
        material_graph::{
            self,
            GraphOverridesAttr,
            ShaderGraph,
        },
        name::NameAttr,
        rigid_body::{
            RigidBodyAttr,
            RigidBodyKind,
        },
        slots,
        spawn::SpawnAttr,
        xform::XformAttr,
    },
    id::PrimId,
    key,
    meta::DocMeta,
    package::Package,
    property::{
        Parent,
        Property,
    },
};

use crate::{
    hsda::{
        Hsda,
        HsdaAttributes,
        HsdaCollider,
        HsdaImage,
        HsdaMaterial,
        HsdaMaterialGraph,
        HsdaPrim,
        HsdaRigidBody,
        HsdaXform,
    },
    wasm::build_wasm_for_crate,
};

/// Identifies a source file in a way that is the same on every machine, so a
/// prim id derived from it does not depend on where the repo is checked out.
fn source_identity(input_abs: &Path) -> Result<String> {
    let dir = input_abs.parent().context("input has no parent dir")?;
    let crate_name = dir
        .file_name()
        .with_context(|| format!("input dir has no name: {}", dir.display()))?
        .to_string_lossy();
    let stem = input_abs
        .file_stem()
        .context("input has no file stem")?
        .to_string_lossy();
    Ok(format!("{crate_name}/{stem}"))
}

/// A build-time id is derived, not minted: every peer instancing the prefab
/// gets byte-identical prim ids, which is what makes a cross-peer reference to
/// an authored prim — a portal receptor, a `wired:kv` key — resolve at all.
fn derive_prim_id(source: &str, path: &[usize]) -> PrimId {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"hsd:prim");
    hasher.update(source.as_bytes());
    for index in path {
        hasher.update(b"/");
        hasher.update(index.to_string().as_bytes());
    }
    PrimId::from_digest(hasher.finalize().as_bytes())
}

#[must_use]
pub fn output_name(input_abs: &Path) -> String {
    input_abs.parent().and_then(Path::file_name).map_or_else(
        || "asset".to_owned(),
        |name| name.to_string_lossy().replace('-', "_"),
    )
}

/// Compiles a `.hsda` file into a package, recursively compiling any prefabs
/// it names.
pub fn compile_file<S: std::hash::BuildHasher>(
    input: &Path,
    built: &mut HashMap<String, Vec<u8>, S>,
) -> Result<Package> {
    let input_abs =
        std::fs::canonicalize(input).with_context(|| format!("resolving {}", input.display()))?;
    let input_dir = input_abs
        .parent()
        .context("input has no parent dir")?
        .to_path_buf();

    let src = std::fs::read_to_string(&input_abs)
        .with_context(|| format!("reading {}", input_abs.display()))?;
    let hsda = Hsda::parse(&src).with_context(|| format!("parsing {}", input_abs.display()))?;

    let source = source_identity(&input_abs)?;

    let mut names = HashMap::new();
    index_names(&hsda.0, &source, &mut Vec::new(), &mut names)?;

    let mut compiler = Compiler {
        source,
        input_dir,
        names,
        entries: BTreeMap::new(),
        built,
    };
    compiler.entries.insert(
        key::META.to_owned(),
        DocMeta::default().encode().context("encoding meta")?,
    );
    compiler.emit(&hsda.0, Parent::Root, &mut Vec::new())?;

    Ok(Package::new(compiler.entries))
}

fn index_names(
    prims: &[HsdaPrim],
    source: &str,
    path: &mut Vec<usize>,
    names: &mut HashMap<String, PrimId>,
) -> Result<()> {
    for (index, prim) in prims.iter().enumerate() {
        path.push(index);
        let id = derive_prim_id(source, path);
        if let Some(name) = &prim.attributes.name
            && names.insert(name.clone(), id).is_some()
        {
            bail!("duplicate prim name {name:?}; names must be unique within a file");
        }
        index_names(&prim.children, source, path, names)?;
        path.pop();
    }
    Ok(())
}

struct Compiler<'a, S: std::hash::BuildHasher> {
    source:    String,
    input_dir: PathBuf,
    names:     HashMap<String, PrimId>,
    entries:   BTreeMap<String, Vec<u8>>,
    built:     &'a mut HashMap<String, Vec<u8>, S>,
}

impl<S: std::hash::BuildHasher> Compiler<'_, S> {
    fn emit(&mut self, prims: &[HsdaPrim], parent: Parent, path: &mut Vec<usize>) -> Result<()> {
        for (index, prim) in prims.iter().enumerate() {
            path.push(index);
            let id = derive_prim_id(&self.source, path);

            self.entries.insert(key::parent(id), parent.encode());
            self.emit_attributes(id, &prim.attributes)?;

            for (name, target) in &prim.relationships {
                let target = self.resolve(target)?;
                self.set_property(id, name, Property::Relationship(target));
            }

            self.emit(&prim.children, Parent::Prim(id), path)?;
            path.pop();
        }
        Ok(())
    }

    fn emit_attributes(&mut self, id: PrimId, attrs: &HsdaAttributes) -> Result<()> {
        if let Some(name) = &attrs.name {
            self.set_attribute(id, &NameAttr(name.clone()))?;
        }
        if let Some(scale) = attrs.gravity_scale {
            self.set_attribute(id, &GravityScaleAttr { scale })?;
        }
        if let Some(spawn) = &attrs.spawn {
            self.set_attribute(
                id,
                &SpawnAttr {
                    radius: spawn.radius,
                },
            )?;
        }
        if let Some(xform) = &attrs.xform {
            self.set_attribute(id, &compile_xform(xform))?;
        }
        if let Some(collider) = &attrs.collider {
            self.set_attribute(id, &compile_collider(collider))?;
        }
        if let Some(rigid_body) = &attrs.rigid_body {
            self.set_attribute(id, &compile_rigid_body(rigid_body)?)?;
        }
        if let Some(image) = &attrs.image {
            self.emit_image(id, image)?;
        }
        if let Some(mat) = &attrs.material {
            self.emit_material(id, mat)?;
        }
        if let Some(graph) = &attrs.material_graph {
            self.emit_material_graph(id, graph)?;
        }
        if let Some(rel) = &attrs.script {
            let bytes = self.compile_script(rel)?;
            self.set_bulk(id, slots::SCRIPT, bytes);
        }
        if let Some(rel) = &attrs.prefab {
            let bytes = self.compile_prefab(rel)?;
            self.set_bulk(id, slots::PREFAB, bytes);
        }
        Ok(())
    }

    fn emit_image(&mut self, id: PrimId, image: &HsdaImage) -> Result<()> {
        let path = self.input_dir.join(&image.data);
        let bytes =
            std::fs::read(&path).with_context(|| format!("reading image {}", path.display()))?;
        self.set_bulk(id, slots::IMAGE_DATA, bytes);
        self.set_attribute(
            id,
            &ImageAttr {
                address_mode_u: image.address_mode_u,
                address_mode_v: image.address_mode_v,
                address_mode_w: image.address_mode_w,
                mag_filter:     image.mag_filter,
                min_filter:     image.min_filter,
                mipmap_filter:  image.mipmap_filter,
                srgb:           image.srgb,
            },
        )
    }

    fn emit_material(&mut self, id: PrimId, mat: &HsdaMaterial) -> Result<()> {
        self.set_attribute(
            id,
            &MaterialAttr {
                alpha_cutoff: mat.alpha_cutoff,
                alpha_mode:   mat.alpha_mode.clone(),
                base_color:   mat.base_color.clone().map(ColorVec),
                double_sided: mat.double_sided,
                emissive:     mat.emissive.clone().map(ColorVec),
                metallic:     mat.metallic,
                roughness:    mat.roughness,
            },
        )?;

        for (name, slot) in [
            (material::BASE_COLOR_TEXTURE, &mat.base_color_texture),
            (material::EMISSIVE_TEXTURE, &mat.emissive_texture),
            (
                material::METALLIC_ROUGHNESS_TEXTURE,
                &mat.metallic_roughness_texture,
            ),
            (material::NORMAL_TEXTURE, &mat.normal_texture),
            (material::OCCLUSION_TEXTURE, &mat.occlusion_texture),
        ] {
            if let Some(target) = slot {
                let target = self.resolve(target)?;
                self.set_property(id, name, Property::Relationship(target));
            }
        }
        Ok(())
    }

    /// Compiles a `.shader` file to bulk content and, if the prim specifies
    /// overrides, an attribute alongside it. The graph itself never appears
    /// in the attribute payload — see `hsd::attributes::material_graph`.
    fn emit_material_graph(&mut self, id: PrimId, graph: &HsdaMaterialGraph) -> Result<()> {
        let path = self.input_dir.join(&graph.path);
        let src = std::fs::read_to_string(&path)
            .with_context(|| format!("reading shader graph {}", path.display()))?;
        let parsed: ShaderGraph = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(&src)
            .with_context(|| format!("parsing shader graph {}", path.display()))?;
        material_graph::validate(&parsed)
            .with_context(|| format!("validating shader graph {}", path.display()))?;

        let bytes = parsed.encode().context("encoding shader graph")?;
        self.set_bulk(id, slots::MATERIAL_GRAPH_DATA, bytes);

        if !graph.overrides.is_empty() {
            material_graph::validate_overrides(
                &parsed,
                &GraphOverridesAttr {
                    overrides: graph.overrides.clone(),
                },
            )
            .with_context(|| format!("validating overrides for {}", path.display()))?;
            self.set_attribute(
                id,
                &GraphOverridesAttr {
                    overrides: graph.overrides.clone(),
                },
            )?;
        }
        Ok(())
    }

    fn compile_script(&mut self, rel: &str) -> Result<Vec<u8>> {
        let cargo_path = self.input_dir.join(rel);
        let crate_dir = cargo_path
            .parent()
            .context("Cargo.toml has no parent dir")?;
        build_wasm_for_crate(crate_dir, self.built)
    }

    fn compile_prefab(&mut self, rel: &str) -> Result<Vec<u8>> {
        let path = self.input_dir.join(rel);
        let package = compile_file(&path, self.built)
            .with_context(|| format!("compiling prefab {}", path.display()))?;
        package.encode().context("encoding prefab package")
    }

    /// A dangling reference in hand-written source is an author bug, so it
    /// fails the build rather than passing through as a literal.
    fn resolve(&self, name: &str) -> Result<PrimId> {
        self.names
            .get(name)
            .copied()
            .with_context(|| format!("reference {name:?} does not match any named prim"))
    }

    fn set_attribute<A: Attribute>(&mut self, id: PrimId, value: &A) -> Result<()> {
        let payload = value
            .encode()
            .with_context(|| format!("encoding {} attribute", A::KEY))?;
        self.set_property(id, A::KEY, Property::Attribute(payload));
        Ok(())
    }

    fn set_property(&mut self, id: PrimId, name: &str, value: Property) {
        self.entries.insert(key::prop(id, name), value.encode());
    }

    fn set_bulk(&mut self, id: PrimId, slot: &str, bytes: Vec<u8>) {
        self.entries.insert(key::bulk(id, slot), bytes);
    }
}

const fn compile_collider(c: &HsdaCollider) -> ColliderAttr {
    match *c {
        HsdaCollider::Capsule { height, radius } => ColliderAttr::Capsule { height, radius },
        HsdaCollider::Cuboid { x, y, z } => ColliderAttr::Cuboid { x, y, z },
        HsdaCollider::Cylinder { height, radius } => ColliderAttr::Cylinder { height, radius },
        HsdaCollider::Sphere(r) => ColliderAttr::Sphere(r),
    }
}

fn compile_rigid_body(rb: &HsdaRigidBody) -> Result<RigidBodyAttr> {
    let kind = match rb.kind.as_str() {
        "Static" => RigidBodyKind::Static,
        "Kinematic" => RigidBodyKind::Kinematic,
        "Dynamic" => RigidBodyKind::Dynamic,
        other => bail!("unknown rigid body kind {other:?}; expected Static, Kinematic, or Dynamic"),
    };
    Ok(RigidBodyAttr {
        kind:            Some(kind),
        angular_damping: rb.angular_damping,
        friction:        rb.friction,
        linear_damping:  rb.linear_damping,
        mass:            rb.mass,
        restitution:     rb.restitution,
    })
}

fn compile_xform(x: &HsdaXform) -> XformAttr {
    let mut out = XformAttr::default();
    if let Some(t) = &x.translation {
        out.translation.copy_from_slice(t);
    }
    if let Some(r) = &x.rotation {
        out.rotation.copy_from_slice(r);
    }
    if let Some(s) = &x.scale {
        out.scale.copy_from_slice(s);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_ids_are_stable_and_distinct() {
        let a = derive_prim_id("unavi-gate/asset", &[0, 1]);
        assert_eq!(a, derive_prim_id("unavi-gate/asset", &[0, 1]));
        assert_ne!(a, derive_prim_id("unavi-gate/asset", &[0, 2]));
        assert_ne!(a, derive_prim_id("unavi-gate/asset", &[1]));
        assert_ne!(a, derive_prim_id("unavi-shapes/asset", &[0, 1]));
    }

    /// `1/1` and `11` must not collide, or two prims share a key.
    #[test]
    fn multi_digit_indices_do_not_collide() {
        assert_ne!(derive_prim_id("a/b", &[1, 1]), derive_prim_id("a/b", &[11]));
    }
}
