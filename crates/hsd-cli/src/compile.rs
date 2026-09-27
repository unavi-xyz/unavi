//! `.hsda` → `.hsdz`.
//!
//! Output is an entry set, not a prim tree: the same keys a live document
//! holds, with bytes inlined so the package is one self-contained blob.

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
    ensure,
};
use hsd::{
    format::{
        meta::DocMeta,
        package::Package,
        source::{
            Source,
            SourceAttributes,
            SourceCollider,
            SourceImage,
            SourceMaterial,
            SourcePrim,
            SourceRigidBody,
            SourceShader,
            SourceXform,
        },
    },
    id::{
        DocId,
        PrimId,
    },
    key,
    property::{
        Property,
        name::PropName,
        value::Value,
    },
    schema::{
        collider::ColliderKind,
        gravity_scale::GravityScaleAttr,
        image::{
            ImageData,
            ImageSampler,
        },
        material::{
            self,
            ColorVec,
            MaterialAttr,
        },
        name::NameAttr,
        parent::ParentAttr,
        reference::ReferenceAttr,
        rigid_body::{
            RigidBodyAttr,
            RigidBodyKind,
        },
        script::ScriptAttr,
        shader::{
            ShaderGraph,
            overrides::{
                GraphOverridesAttr,
                validate_overrides,
            },
            parse::parse as parse_hss,
            validate::validate,
        },
        spawn::SpawnAttr,
        xform::XformAttr,
    },
};

use crate::wasm::build_wasm_for_crate;

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

/// A build-time id is derived rather than minted, so every peer instancing
/// a file computes byte-identical prim ids and a cross-peer reference to an
/// authored prim resolves.
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

/// Compiles a `.hsda` file into a package, recursively compiling every file it
/// references.
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
    let doc = Source::parse(&src).with_context(|| format!("parsing {}", input_abs.display()))?;

    let source = source_identity(&input_abs)?;

    let mut names = HashMap::new();
    index_names(&doc.0, &source, &mut Vec::new(), &mut names)?;

    let mut compiler = Compiler {
        source,
        input_dir,
        names,
        entries: BTreeMap::new(),
        documents: BTreeMap::new(),
        built,
    };
    compiler.entries.insert(
        key::META.to_owned(),
        DocMeta::default().encode().context("encoding meta")?,
    );
    compiler.emit(&doc.0, ParentAttr::Root, &mut Vec::new())?;

    let mut package = Package::new(compiler.entries);
    package.documents = compiler.documents.into_iter().collect();
    Ok(package)
}

fn index_names(
    prims: &[SourcePrim],
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
    /// Documents this file references, and everything they reference, flat and
    /// keyed so two prims naming one file share a copy. Ordered, so an
    /// unchanged input still compiles to identical bytes.
    documents: BTreeMap<DocId, Vec<(String, Vec<u8>)>>,
    built:     &'a mut HashMap<String, Vec<u8>, S>,
}

impl<S: std::hash::BuildHasher> Compiler<'_, S> {
    fn emit(
        &mut self,
        prims: &[SourcePrim],
        parent: ParentAttr,
        path: &mut Vec<usize>,
    ) -> Result<()> {
        for (index, prim) in prims.iter().enumerate() {
            path.push(index);
            let id = derive_prim_id(&self.source, path);

            self.entries.insert(
                key::Key::prop(id, &ParentAttr::NAME).to_string(),
                ParentAttr::to_wire(Some(parent)),
            );
            self.emit_attributes(id, &prim.attributes)?;

            self.emit(&prim.children, ParentAttr::Prim(id), path)?;
            path.pop();
        }
        Ok(())
    }

    fn emit_attributes(&mut self, id: PrimId, attrs: &SourceAttributes) -> Result<()> {
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
            self.set_attribute(id, &compile_xform(xform)?)?;
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
        if let Some(shader) = &attrs.shader {
            self.emit_shader(id, shader)?;
        }
        if let Some(rel) = &attrs.script {
            let bytes = self.compile_script(rel)?;
            self.set_attribute(id, &ScriptAttr(bytes))?;
        }
        if let Some(rel) = &attrs.reference {
            let target = self.compile_reference(rel)?;
            self.set_attribute(id, &ReferenceAttr(target))?;
        }
        Ok(())
    }

    fn emit_image(&mut self, id: PrimId, image: &SourceImage) -> Result<()> {
        let path = self.input_dir.join(&image.data);
        let bytes =
            std::fs::read(&path).with_context(|| format!("reading image {}", path.display()))?;
        self.set_attribute(
            id,
            &ImageSampler {
                address_mode_u: image.address_mode_u,
                address_mode_v: image.address_mode_v,
                address_mode_w: image.address_mode_w,
                mag_filter:     image.mag_filter,
                min_filter:     image.min_filter,
                mipmap_filter:  image.mipmap_filter,
                srgb:           image.srgb,
            },
        )?;
        self.set_attribute(id, &ImageData(bytes))
    }

    fn emit_material(&mut self, id: PrimId, mat: &SourceMaterial) -> Result<()> {
        self.set_attribute(
            id,
            &MaterialAttr {
                alpha_cutoff: mat.alpha_cutoff,
                alpha_mode:   mat.alpha_mode,
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
                self.set_property(id, &name, Value::Relationship(target));
            }
        }
        Ok(())
    }

    /// Compiles a `.hss` (Hyper-Space Shader) file to the `shader/graph`
    /// attribute and, if the prim specifies overrides, a `shader/overrides`
    /// attribute alongside it.
    fn emit_shader(&mut self, id: PrimId, shader: &SourceShader) -> Result<()> {
        let path = self.input_dir.join(&shader.path);
        let src = std::fs::read_to_string(&path)
            .with_context(|| format!("reading shader graph {}", path.display()))?;
        let parsed: ShaderGraph =
            parse_hss(&src).with_context(|| format!("parsing shader graph {}", path.display()))?;
        validate(&parsed).with_context(|| format!("validating shader graph {}", path.display()))?;
        self.set_attribute(id, &parsed)?;

        if !shader.overrides.is_empty() {
            let overrides = GraphOverridesAttr {
                overrides: shader.overrides.clone(),
            };
            validate_overrides(&parsed, &overrides)
                .with_context(|| format!("validating overrides for {}", path.display()))?;
            self.set_attribute(id, &overrides)?;
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

    /// Compiles the referenced file into a document carried beside the root,
    /// and answers the placeholder that names it.
    ///
    /// Its own referenced documents are hoisted rather than nested: the list
    /// is flat so that minting at load is one pass, and so that two files
    /// naming one file share a single copy of it.
    fn compile_reference(&mut self, rel: &str) -> Result<DocId> {
        let path = self.input_dir.join(rel);
        let package = compile_file(&path, self.built)
            .with_context(|| format!("compiling reference {}", path.display()))?;
        let placeholder = Package::placeholder(&source_identity(&std::fs::canonicalize(&path)?)?);

        self.documents.extend(package.documents);
        self.documents.insert(placeholder, package.entries);
        Ok(placeholder)
    }

    /// A dangling reference in hand-written source is an author bug, so it
    /// fails the build rather than passing through as a literal.
    fn resolve(&self, name: &str) -> Result<PrimId> {
        self.names
            .get(name)
            .copied()
            .with_context(|| format!("reference {name:?} does not match any named prim"))
    }

    fn set_attribute<A: Property>(&mut self, id: PrimId, value: &A) -> Result<()> {
        let payload = value
            .encode()
            .with_context(|| format!("encoding {} attribute", A::NAME))?;
        self.set_property(id, &A::NAME, Value::Attribute(payload));
        Ok(())
    }

    fn set_property(&mut self, id: PrimId, name: &PropName, value: Value) {
        self.entries
            .insert(key::Key::prop(id, name).to_string(), value.encode());
    }
}

const fn compile_collider(c: &SourceCollider) -> ColliderKind {
    match *c {
        SourceCollider::Capsule { height, radius } => ColliderKind::Capsule { height, radius },
        SourceCollider::Cuboid { x, y, z } => ColliderKind::Cuboid { x, y, z },
        SourceCollider::Cylinder { height, radius } => ColliderKind::Cylinder { height, radius },
        SourceCollider::Sphere(r) => ColliderKind::Sphere(r),
    }
}

fn compile_rigid_body(rb: &SourceRigidBody) -> Result<RigidBodyAttr> {
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

fn compile_xform(x: &SourceXform) -> Result<XformAttr> {
    let mut out = XformAttr::default();
    if let Some(t) = &x.translation {
        ensure!(
            t.len() == 3,
            "translation must have 3 components, got {}",
            t.len()
        );
        out.translation.copy_from_slice(t);
    }
    if let Some(r) = &x.rotation {
        ensure!(
            r.len() == 4,
            "rotation must have 4 components, got {}",
            r.len()
        );
        out.rotation.copy_from_slice(r);
    }
    if let Some(s) = &x.scale {
        ensure!(
            s.len() == 3,
            "scale must have 3 components, got {}",
            s.len()
        );
        out.scale.copy_from_slice(s);
    }
    Ok(out)
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
