use hsd::attributes::xform::XformAttr;
use wasmtime::component::Resource;

use crate::{
    error::ScriptError,
    runtime::{
        Runtime,
        native::wired::{
            error::Error,
            scene::bindings::wired::{
                math::types::Transform,
                scene::types::{
                    AlphaMode,
                    Collider,
                    ColliderCapsule,
                    ColliderCylinder,
                    ColliderTrimesh,
                    Color,
                    HostPrim,
                    Image,
                    Material,
                    Mesh,
                    Portal,
                    PortalDestination,
                    PortalReceptor,
                    RigidBody,
                    RigidBodyKind,
                    Spawn,
                    Topology,
                    Xform,
                },
            },
        },
        shared::{
            self,
            wired::scene::prim::{
                PrimAlphaMode,
                PrimCollider,
                PrimColor,
                PrimImage,
                PrimMaterial,
                PrimMesh,
                PrimPortal,
                PrimPortalDestination,
                PrimPortalReceptor,
                PrimRes,
                PrimRigidBody,
                PrimRigidBodyKind,
                PrimSpawn,
                PrimTopology,
            },
        },
    },
};

fn to_blob_array(bytes: Vec<u8>) -> wasmtime::Result<[u8; 32]> {
    bytes
        .as_slice()
        .try_into()
        .map_err(|_| wasmtime::Error::msg("blob id must be 32 bytes"))
}

/// Lowers a fallible host call into a WIT `result`, so firewall/quota/space
/// errors reach the guest instead of trapping.
fn lower<T>(result: anyhow::Result<T>) -> Result<T, Error> {
    result.map_err(|e| ScriptError::from(e).into())
}

const fn topology_wit(t: PrimTopology) -> Topology {
    match t {
        PrimTopology::PointList => Topology::PointList,
        PrimTopology::LineList => Topology::LineList,
        PrimTopology::LineStrip => Topology::LineStrip,
        PrimTopology::TriangleList => Topology::TriangleList,
        PrimTopology::TriangleStrip => Topology::TriangleStrip,
    }
}

const fn topology_shared(t: Topology) -> PrimTopology {
    match t {
        Topology::PointList => PrimTopology::PointList,
        Topology::LineList => PrimTopology::LineList,
        Topology::LineStrip => PrimTopology::LineStrip,
        Topology::TriangleList => PrimTopology::TriangleList,
        Topology::TriangleStrip => PrimTopology::TriangleStrip,
    }
}

const fn alpha_mode_wit(m: PrimAlphaMode) -> AlphaMode {
    match m {
        PrimAlphaMode::Add => AlphaMode::Add,
        PrimAlphaMode::Blend => AlphaMode::Blend,
        PrimAlphaMode::Mask => AlphaMode::Mask,
        PrimAlphaMode::Multiply => AlphaMode::Multiply,
        PrimAlphaMode::Opaque => AlphaMode::Opaque,
        PrimAlphaMode::PreMultiplied => AlphaMode::PreMultiplied,
    }
}

const fn alpha_mode_shared(m: AlphaMode) -> PrimAlphaMode {
    match m {
        AlphaMode::Add => PrimAlphaMode::Add,
        AlphaMode::Blend => PrimAlphaMode::Blend,
        AlphaMode::Mask => PrimAlphaMode::Mask,
        AlphaMode::Multiply => PrimAlphaMode::Multiply,
        AlphaMode::Opaque => PrimAlphaMode::Opaque,
        AlphaMode::PreMultiplied => PrimAlphaMode::PreMultiplied,
    }
}

const fn rigid_kind_wit(k: PrimRigidBodyKind) -> RigidBodyKind {
    match k {
        PrimRigidBodyKind::Dynamic => RigidBodyKind::Dynamic,
        PrimRigidBodyKind::Kinematic => RigidBodyKind::Kinematic,
        PrimRigidBodyKind::Static => RigidBodyKind::Static,
    }
}

const fn rigid_kind_shared(k: RigidBodyKind) -> PrimRigidBodyKind {
    match k {
        RigidBodyKind::Dynamic => PrimRigidBodyKind::Dynamic,
        RigidBodyKind::Kinematic => PrimRigidBodyKind::Kinematic,
        RigidBodyKind::Static => PrimRigidBodyKind::Static,
    }
}

const fn color_wit(c: PrimColor) -> Color {
    Color {
        r: c.r,
        g: c.g,
        b: c.b,
        a: c.a,
    }
}

const fn color_shared(c: Color) -> PrimColor {
    PrimColor {
        r: c.r,
        g: c.g,
        b: c.b,
        a: c.a,
    }
}

const fn xform_wit(x: XformAttr) -> Xform {
    use crate::runtime::native::wired::scene::bindings::wired::math::types::{
        Quat,
        Vec3,
    };
    Xform {
        translation: Vec3 {
            x: x.translation[0],
            y: x.translation[1],
            z: x.translation[2],
        },
        rotation:    Quat {
            x: x.rotation[0],
            y: x.rotation[1],
            z: x.rotation[2],
            w: x.rotation[3],
        },
        scale:       Vec3 {
            x: x.scale[0],
            y: x.scale[1],
            z: x.scale[2],
        },
    }
}

const fn xform_shared(x: Xform) -> XformAttr {
    XformAttr {
        translation: [x.translation.x, x.translation.y, x.translation.z],
        rotation:    [x.rotation.x, x.rotation.y, x.rotation.z, x.rotation.w],
        scale:       [x.scale.x, x.scale.y, x.scale.z],
    }
}

fn mesh_wit(m: PrimMesh) -> Mesh {
    Mesh {
        topology:   topology_wit(m.topology),
        attributes: m
            .attributes
            .into_iter()
            .map(|(k, v)| (k, v.to_vec()))
            .collect(),
        indices:    m.indices.map(|b| b.to_vec()),
    }
}

fn mesh_shared(m: Mesh) -> wasmtime::Result<PrimMesh> {
    let attributes = m
        .attributes
        .into_iter()
        .map(|(k, v)| to_blob_array(v).map(|b| (k, b)))
        .collect::<wasmtime::Result<Vec<_>>>()?;
    let indices = m.indices.map(to_blob_array).transpose()?;
    Ok(PrimMesh {
        topology: topology_shared(m.topology),
        attributes,
        indices,
    })
}

fn material_wit(m: PrimMaterial) -> Material {
    Material {
        alpha_cutoff:               m.alpha_cutoff,
        alpha_mode:                 m.alpha_mode.map(alpha_mode_wit),
        base_color:                 m.base_color.map(color_wit),
        base_color_texture:         m.base_color_texture,
        double_sided:               m.double_sided,
        emissive:                   m.emissive.map(color_wit),
        emissive_texture:           m.emissive_texture,
        metallic:                   m.metallic,
        metallic_roughness_texture: m.metallic_roughness_texture,
        normal_texture:             m.normal_texture,
        occlusion_texture:          m.occlusion_texture,
        roughness:                  m.roughness,
    }
}

fn material_shared(m: Material) -> PrimMaterial {
    PrimMaterial {
        alpha_cutoff:               m.alpha_cutoff,
        alpha_mode:                 m.alpha_mode.map(alpha_mode_shared),
        base_color:                 m.base_color.map(color_shared),
        base_color_texture:         m.base_color_texture,
        double_sided:               m.double_sided,
        emissive:                   m.emissive.map(color_shared),
        emissive_texture:           m.emissive_texture,
        metallic:                   m.metallic,
        metallic_roughness_texture: m.metallic_roughness_texture,
        normal_texture:             m.normal_texture,
        occlusion_texture:          m.occlusion_texture,
        roughness:                  m.roughness,
    }
}

fn image_wit(img: PrimImage) -> Image {
    Image {
        data:           img.data.to_vec(),
        address_mode_u: img.address_mode_u,
        address_mode_v: img.address_mode_v,
        address_mode_w: img.address_mode_w,
        mag_filter:     img.mag_filter,
        min_filter:     img.min_filter,
        mipmap_filter:  img.mipmap_filter,
        srgb:           img.srgb,
    }
}

fn image_shared(img: Image) -> wasmtime::Result<PrimImage> {
    Ok(PrimImage {
        data:           to_blob_array(img.data)?,
        address_mode_u: img.address_mode_u,
        address_mode_v: img.address_mode_v,
        address_mode_w: img.address_mode_w,
        mag_filter:     img.mag_filter,
        min_filter:     img.min_filter,
        mipmap_filter:  img.mipmap_filter,
        srgb:           img.srgb,
    })
}

fn collider_wit(c: PrimCollider) -> Collider {
    use crate::runtime::native::wired::scene::bindings::wired::math::types::Vec3;
    match c {
        PrimCollider::Capsule { height, radius } => {
            Collider::Capsule(ColliderCapsule { height, radius })
        }
        PrimCollider::ConvexHull(hash) => Collider::ConvexHull(hash.to_vec()),
        PrimCollider::Cuboid([x, y, z]) => Collider::Cuboid(Vec3 { x, y, z }),
        PrimCollider::Cylinder { height, radius } => {
            Collider::Cylinder(ColliderCylinder { height, radius })
        }
        PrimCollider::Sphere(r) => Collider::Sphere(r),
        PrimCollider::Trimesh { indices, vertices } => Collider::Trimesh(ColliderTrimesh {
            indices:  indices.to_vec(),
            vertices: vertices.to_vec(),
        }),
    }
}

fn collider_shared(c: Collider) -> wasmtime::Result<PrimCollider> {
    Ok(match c {
        Collider::Capsule(c) => PrimCollider::Capsule {
            height: c.height,
            radius: c.radius,
        },
        Collider::ConvexHull(hash) => PrimCollider::ConvexHull(to_blob_array(hash)?),
        Collider::Cuboid(v) => PrimCollider::Cuboid([v.x, v.y, v.z]),
        Collider::Cylinder(c) => PrimCollider::Cylinder {
            height: c.height,
            radius: c.radius,
        },
        Collider::Sphere(r) => PrimCollider::Sphere(r),
        Collider::Trimesh(t) => PrimCollider::Trimesh {
            indices:  to_blob_array(t.indices)?,
            vertices: to_blob_array(t.vertices)?,
        },
    })
}

fn portal_wit(p: PrimPortal) -> Portal {
    Portal {
        destination: p.destination.map(|d| PortalDestination {
            receptor: d.receptor.map(|r| PortalReceptor {
                document: r.document.to_vec(),
                prim:     r.prim,
            }),
            space:    d.space.to_vec(),
        }),
        size_x:      p.size_x,
        size_y:      p.size_y,
    }
}

fn portal_shared(p: Portal) -> wasmtime::Result<PrimPortal> {
    Ok(PrimPortal {
        destination: p
            .destination
            .map(|d| -> wasmtime::Result<_> {
                Ok(PrimPortalDestination {
                    receptor: d
                        .receptor
                        .map(|r| -> wasmtime::Result<_> {
                            Ok(PrimPortalReceptor {
                                document: to_blob_array(r.document)?,
                                prim:     r.prim,
                            })
                        })
                        .transpose()?,
                    space:    to_blob_array(d.space)?,
                })
            })
            .transpose()?,
        size_x:      p.size_x,
        size_y:      p.size_y,
    })
}

const fn rigid_body_wit(rb: PrimRigidBody) -> RigidBody {
    RigidBody {
        kind:            rigid_kind_wit(rb.kind),
        angular_damping: rb.angular_damping,
        friction:        rb.friction,
        linear_damping:  rb.linear_damping,
        mass:            rb.mass,
        restitution:     rb.restitution,
    }
}

const fn rigid_body_shared(rb: RigidBody) -> PrimRigidBody {
    PrimRigidBody {
        kind:            rigid_kind_shared(rb.kind),
        angular_damping: rb.angular_damping,
        friction:        rb.friction,
        linear_damping:  rb.linear_damping,
        mass:            rb.mass,
        restitution:     rb.restitution,
    }
}

impl HostPrim for Runtime {
    async fn id(&mut self, self_: Resource<PrimRes>) -> wasmtime::Result<String> {
        shared::wired::scene::prim::id(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn clone(&mut self, self_: Resource<PrimRes>) -> wasmtime::Result<Resource<PrimRes>> {
        shared::wired::scene::prim::clone(&self.api, self_.rep())
            .await
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn parent(
        &mut self,
        self_: Resource<PrimRes>,
    ) -> wasmtime::Result<Option<Resource<PrimRes>>> {
        shared::wired::scene::prim::parent(&self.api, self_.rep())
            .await
            .map(|r| r.map(Resource::new_own))
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn children(
        &mut self,
        self_: Resource<PrimRes>,
    ) -> wasmtime::Result<Vec<Resource<PrimRes>>> {
        shared::wired::scene::prim::children(&self.api, self_.rep())
            .await
            .map(|v| v.into_iter().map(Resource::new_own).collect())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn add_child(
        &mut self,
        self_: Resource<PrimRes>,
        child: Resource<PrimRes>,
    ) -> wasmtime::Result<Result<(), Error>> {
        Ok(lower(
            shared::wired::scene::prim::add_child(&self.api, self_.rep(), child.rep()).await,
        ))
    }

    async fn remove_child(
        &mut self,
        self_: Resource<PrimRes>,
        child: Resource<PrimRes>,
    ) -> wasmtime::Result<Result<(), Error>> {
        Ok(lower(
            shared::wired::scene::prim::remove_child(&self.api, self_.rep(), child.rep()).await,
        ))
    }

    async fn name(&mut self, self_: Resource<PrimRes>) -> wasmtime::Result<Option<String>> {
        shared::wired::scene::prim::name(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn set_name(
        &mut self,
        self_: Resource<PrimRes>,
        value: Option<String>,
    ) -> wasmtime::Result<Result<(), Error>> {
        Ok(lower(
            shared::wired::scene::prim::set_name(&self.api, self_.rep(), value).await,
        ))
    }

    async fn asset(&mut self, self_: Resource<PrimRes>) -> wasmtime::Result<Option<Vec<u8>>> {
        shared::wired::scene::prim::asset(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn set_asset(
        &mut self,
        self_: Resource<PrimRes>,
        value: Option<Vec<u8>>,
    ) -> wasmtime::Result<Result<(), Error>> {
        Ok(lower(
            shared::wired::scene::prim::set_asset(&self.api, self_.rep(), value).await,
        ))
    }

    async fn xform(&mut self, self_: Resource<PrimRes>) -> wasmtime::Result<Option<Xform>> {
        Ok(shared::wired::scene::prim::xform(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)?
            .map(xform_wit))
    }

    async fn set_xform(
        &mut self,
        self_: Resource<PrimRes>,
        value: Option<Xform>,
    ) -> wasmtime::Result<Result<(), Error>> {
        Ok(lower(
            shared::wired::scene::prim::set_xform(&self.api, self_.rep(), value.map(xform_shared))
                .await,
        ))
    }

    async fn global_xform(&mut self, self_: Resource<PrimRes>) -> wasmtime::Result<Transform> {
        let x = shared::wired::scene::prim::global_xform(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)?;
        let Xform {
            translation,
            rotation,
            scale,
        } = xform_wit(x);
        Ok(Transform {
            translation,
            rotation,
            scale,
        })
    }

    async fn gravity_scale(&mut self, self_: Resource<PrimRes>) -> wasmtime::Result<f32> {
        shared::wired::scene::prim::gravity_scale(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn set_gravity_scale(
        &mut self,
        self_: Resource<PrimRes>,
        value: f32,
    ) -> wasmtime::Result<Result<(), Error>> {
        Ok(lower(
            shared::wired::scene::prim::set_gravity_scale(&self.api, self_.rep(), value).await,
        ))
    }

    async fn mesh(&mut self, self_: Resource<PrimRes>) -> wasmtime::Result<Option<Mesh>> {
        Ok(shared::wired::scene::prim::mesh(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)?
            .map(mesh_wit))
    }

    async fn set_mesh(
        &mut self,
        self_: Resource<PrimRes>,
        value: Option<Mesh>,
    ) -> wasmtime::Result<Result<(), Error>> {
        let shared_mesh = value.map(mesh_shared).transpose()?;
        Ok(lower(
            shared::wired::scene::prim::set_mesh(&self.api, self_.rep(), shared_mesh).await,
        ))
    }

    async fn set_mesh_stream(
        &mut self,
        self_: Resource<PrimRes>,
        key: String,
        values: Option<Vec<f32>>,
    ) -> wasmtime::Result<Result<(), Error>> {
        Ok(lower(
            shared::wired::scene::prim::set_mesh_stream(&self.api, self_.rep(), key, values).await,
        ))
    }

    async fn set_mesh_indices_u32(
        &mut self,
        self_: Resource<PrimRes>,
        values: Option<Vec<u32>>,
    ) -> wasmtime::Result<Result<(), Error>> {
        Ok(lower(
            shared::wired::scene::prim::set_mesh_indices_u32(&self.api, self_.rep(), values).await,
        ))
    }

    async fn material(&mut self, self_: Resource<PrimRes>) -> wasmtime::Result<Option<Material>> {
        Ok(shared::wired::scene::prim::material(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)?
            .map(material_wit))
    }

    async fn set_material(
        &mut self,
        self_: Resource<PrimRes>,
        value: Option<Material>,
    ) -> wasmtime::Result<Result<(), Error>> {
        Ok(lower(
            shared::wired::scene::prim::set_material(
                &self.api,
                self_.rep(),
                value.map(material_shared),
            )
            .await,
        ))
    }

    async fn image(&mut self, self_: Resource<PrimRes>) -> wasmtime::Result<Option<Image>> {
        Ok(shared::wired::scene::prim::image(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)?
            .map(image_wit))
    }

    async fn set_image(
        &mut self,
        self_: Resource<PrimRes>,
        value: Option<Image>,
    ) -> wasmtime::Result<Result<(), Error>> {
        let shared_img = value.map(image_shared).transpose()?;
        Ok(lower(
            shared::wired::scene::prim::set_image(&self.api, self_.rep(), shared_img).await,
        ))
    }

    async fn collider(&mut self, self_: Resource<PrimRes>) -> wasmtime::Result<Option<Collider>> {
        Ok(shared::wired::scene::prim::collider(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)?
            .map(collider_wit))
    }

    async fn set_collider(
        &mut self,
        self_: Resource<PrimRes>,
        value: Option<Collider>,
    ) -> wasmtime::Result<Result<(), Error>> {
        let shared_c = value.map(collider_shared).transpose()?;
        Ok(lower(
            shared::wired::scene::prim::set_collider(&self.api, self_.rep(), shared_c).await,
        ))
    }

    async fn rigid_body(
        &mut self,
        self_: Resource<PrimRes>,
    ) -> wasmtime::Result<Option<RigidBody>> {
        Ok(
            shared::wired::scene::prim::rigid_body(&self.api, self_.rep())
                .await
                .map_err(wasmtime::Error::from_anyhow)?
                .map(rigid_body_wit),
        )
    }

    async fn set_rigid_body(
        &mut self,
        self_: Resource<PrimRes>,
        value: Option<RigidBody>,
    ) -> wasmtime::Result<Result<(), Error>> {
        Ok(lower(
            shared::wired::scene::prim::set_rigid_body(
                &self.api,
                self_.rep(),
                value.map(rigid_body_shared),
            )
            .await,
        ))
    }

    async fn portal(&mut self, self_: Resource<PrimRes>) -> wasmtime::Result<Option<Portal>> {
        Ok(shared::wired::scene::prim::portal(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)?
            .map(portal_wit))
    }

    async fn set_portal(
        &mut self,
        self_: Resource<PrimRes>,
        value: Option<Portal>,
    ) -> wasmtime::Result<Result<(), Error>> {
        let value = value.map(portal_shared).transpose()?;
        Ok(lower(
            shared::wired::scene::prim::set_portal(&self.api, self_.rep(), value).await,
        ))
    }

    async fn spawn(&mut self, self_: Resource<PrimRes>) -> wasmtime::Result<Option<Spawn>> {
        Ok(shared::wired::scene::prim::spawn(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)?
            .map(|s| Spawn { radius: s.radius }))
    }

    async fn set_spawn(
        &mut self,
        self_: Resource<PrimRes>,
        value: Option<Spawn>,
    ) -> wasmtime::Result<Result<(), Error>> {
        let value = value.map(|s| PrimSpawn { radius: s.radius });
        Ok(lower(
            shared::wired::scene::prim::set_spawn(&self.api, self_.rep(), value).await,
        ))
    }

    async fn relationships(
        &mut self,
        self_: Resource<PrimRes>,
    ) -> wasmtime::Result<Vec<(String, String)>> {
        shared::wired::scene::prim::relationships(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn get_relationship(
        &mut self,
        self_: Resource<PrimRes>,
        key: String,
    ) -> wasmtime::Result<Option<String>> {
        shared::wired::scene::prim::get_relationship(&self.api, self_.rep(), key)
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn set_relationship(
        &mut self,
        self_: Resource<PrimRes>,
        key: String,
        target: Option<String>,
    ) -> wasmtime::Result<Result<(), Error>> {
        Ok(lower(
            shared::wired::scene::prim::set_relationship(&self.api, self_.rep(), key, target).await,
        ))
    }

    async fn drop(&mut self, rep: Resource<PrimRes>) -> wasmtime::Result<()> {
        shared::wired::scene::prim::on_drop(&self.api, rep.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }
}
