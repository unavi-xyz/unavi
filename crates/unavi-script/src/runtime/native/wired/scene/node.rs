use wasmtime::component::Resource;

use crate::runtime::{
    Runtime,
    native::wired::scene::bindings::wired::{
        math::types::{Quat, Transform, Vec3},
        scene::types::{
            Collider, ColliderCapsule, ColliderCylinder, ColliderTrimesh, HostNode, RigidBodyKind,
        },
    },
    shared::{
        self,
        wired::scene::{
            material::MaterialRes,
            mesh::MeshRes,
            node::{NodeCollider, NodeRes, NodeRigidBody},
        },
    },
};

impl HostNode for Runtime {
    async fn id(&mut self, self_: Resource<NodeRes>) -> wasmtime::Result<String> {
        shared::wired::scene::node::id(&self.api, self_.rep()).map_err(wasmtime::Error::from_anyhow)
    }

    async fn clone(&mut self, self_: Resource<NodeRes>) -> wasmtime::Result<Resource<NodeRes>> {
        shared::wired::scene::node::clone(&self.api, self_.rep())
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn name(&mut self, self_: Resource<NodeRes>) -> wasmtime::Result<Option<String>> {
        shared::wired::scene::node::name(&self.api, self_.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn set_name(
        &mut self,
        self_: Resource<NodeRes>,
        value: Option<String>,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::node::set_name(&self.api, self_.rep(), value)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn translation(&mut self, self_: Resource<NodeRes>) -> wasmtime::Result<Vec3> {
        let [x, y, z] = shared::wired::scene::node::translation(&self.api, self_.rep())
            .map_err(wasmtime::Error::from_anyhow)?;
        Ok(Vec3 { x, y, z })
    }

    async fn set_translation(
        &mut self,
        self_: Resource<NodeRes>,
        value: Vec3,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::node::set_translation(
            &self.api,
            self_.rep(),
            [value.x, value.y, value.z],
        )
        .map_err(wasmtime::Error::from_anyhow)
    }

    async fn rotation(&mut self, self_: Resource<NodeRes>) -> wasmtime::Result<Quat> {
        let [x, y, z, w] = shared::wired::scene::node::rotation(&self.api, self_.rep())
            .map_err(wasmtime::Error::from_anyhow)?;
        Ok(Quat { x, y, z, w })
    }

    async fn set_rotation(
        &mut self,
        self_: Resource<NodeRes>,
        value: Quat,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::node::set_rotation(
            &self.api,
            self_.rep(),
            [value.x, value.y, value.z, value.w],
        )
        .map_err(wasmtime::Error::from_anyhow)
    }

    async fn scale(&mut self, self_: Resource<NodeRes>) -> wasmtime::Result<Vec3> {
        let [x, y, z] = shared::wired::scene::node::scale(&self.api, self_.rep())
            .map_err(wasmtime::Error::from_anyhow)?;
        Ok(Vec3 { x, y, z })
    }

    async fn set_scale(&mut self, self_: Resource<NodeRes>, value: Vec3) -> wasmtime::Result<()> {
        shared::wired::scene::node::set_scale(&self.api, self_.rep(), [value.x, value.y, value.z])
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn transform(&mut self, self_: Resource<NodeRes>) -> wasmtime::Result<Transform> {
        let t = shared::wired::scene::node::transform(&self.api, self_.rep())
            .map_err(wasmtime::Error::from_anyhow)?;
        Ok(Transform {
            translation: Vec3 {
                x: t.translation[0],
                y: t.translation[1],
                z: t.translation[2],
            },
            rotation: Quat {
                x: t.rotation[0],
                y: t.rotation[1],
                z: t.rotation[2],
                w: t.rotation[3],
            },
            scale: Vec3 {
                x: t.scale[0],
                y: t.scale[1],
                z: t.scale[2],
            },
        })
    }

    async fn set_transform(
        &mut self,
        self_: Resource<NodeRes>,
        value: Transform,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::node::set_transform(
            &self.api,
            self_.rep(),
            shared::wired::scene::node::NodeTransform {
                translation: [
                    value.translation.x,
                    value.translation.y,
                    value.translation.z,
                ],
                rotation: [
                    value.rotation.x,
                    value.rotation.y,
                    value.rotation.z,
                    value.rotation.w,
                ],
                scale: [value.scale.x, value.scale.y, value.scale.z],
            },
        )
        .map_err(wasmtime::Error::from_anyhow)
    }

    async fn global_transform(&mut self, self_: Resource<NodeRes>) -> wasmtime::Result<Transform> {
        let t = shared::wired::scene::node::global_transform(&self.api, self_.rep())
            .map_err(wasmtime::Error::from_anyhow)?;
        Ok(Transform {
            translation: Vec3 {
                x: t.translation[0],
                y: t.translation[1],
                z: t.translation[2],
            },
            rotation: Quat {
                x: t.rotation[0],
                y: t.rotation[1],
                z: t.rotation[2],
                w: t.rotation[3],
            },
            scale: Vec3 {
                x: t.scale[0],
                y: t.scale[1],
                z: t.scale[2],
            },
        })
    }

    async fn parent(
        &mut self,
        self_: Resource<NodeRes>,
    ) -> wasmtime::Result<Option<Resource<NodeRes>>> {
        shared::wired::scene::node::parent(&self.api, self_.rep())
            .map(|opt| opt.map(Resource::new_own))
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn children(
        &mut self,
        self_: Resource<NodeRes>,
    ) -> wasmtime::Result<Vec<Resource<NodeRes>>> {
        shared::wired::scene::node::children(&self.api, self_.rep())
            .map(|v| v.into_iter().map(Resource::new_own).collect())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn add_child(
        &mut self,
        self_: Resource<NodeRes>,
        child: Resource<NodeRes>,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::node::add_child(&self.api, self_.rep(), child.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn remove_child(
        &mut self,
        self_: Resource<NodeRes>,
        child: Resource<NodeRes>,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::node::remove_child(&self.api, self_.rep(), child.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn mesh(
        &mut self,
        self_: Resource<NodeRes>,
    ) -> wasmtime::Result<Option<Resource<MeshRes>>> {
        shared::wired::scene::node::mesh(&self.api, self_.rep())
            .map(|opt| opt.map(Resource::new_own))
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn set_mesh(
        &mut self,
        self_: Resource<NodeRes>,
        value: Option<Resource<MeshRes>>,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::node::set_mesh(&self.api, self_.rep(), value.map(|r| r.rep()))
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn material(
        &mut self,
        self_: Resource<NodeRes>,
    ) -> wasmtime::Result<Option<Resource<MaterialRes>>> {
        shared::wired::scene::node::material(&self.api, self_.rep())
            .map(|opt| opt.map(Resource::new_own))
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn set_material(
        &mut self,
        self_: Resource<NodeRes>,
        value: Option<Resource<MaterialRes>>,
    ) -> wasmtime::Result<()> {
        shared::wired::scene::node::set_material(&self.api, self_.rep(), value.map(|r| r.rep()))
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn collider(&mut self, self_: Resource<NodeRes>) -> wasmtime::Result<Option<Collider>> {
        let c = shared::wired::scene::node::collider(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)?;
        Ok(c.map(|c| match c {
            NodeCollider::Capsule { height, radius } => {
                Collider::Capsule(ColliderCapsule { height, radius })
            }
            NodeCollider::ConvexHull(points) => Collider::ConvexHull(points),
            NodeCollider::Cuboid([x, y, z]) => Collider::Cuboid(Vec3 { x, y, z }),
            NodeCollider::Cylinder { height, radius } => {
                Collider::Cylinder(ColliderCylinder { height, radius })
            }
            NodeCollider::Sphere(radius) => Collider::Sphere(radius),
            NodeCollider::Trimesh { indices, vertices } => {
                Collider::Trimesh(ColliderTrimesh { indices, vertices })
            }
        }))
    }

    async fn set_collider(
        &mut self,
        self_: Resource<NodeRes>,
        value: Option<Collider>,
    ) -> wasmtime::Result<()> {
        let c = value.map(|c| match c {
            Collider::Capsule(c) => NodeCollider::Capsule {
                height: c.height,
                radius: c.radius,
            },
            Collider::ConvexHull(points) => NodeCollider::ConvexHull(points),
            Collider::Cuboid(v) => NodeCollider::Cuboid([v.x, v.y, v.z]),
            Collider::Cylinder(c) => NodeCollider::Cylinder {
                height: c.height,
                radius: c.radius,
            },
            Collider::Sphere(radius) => NodeCollider::Sphere(radius),
            Collider::Trimesh(t) => NodeCollider::Trimesh {
                indices: t.indices,
                vertices: t.vertices,
            },
        });
        shared::wired::scene::node::set_collider(&self.api, self_.rep(), c)
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn rigid_body(
        &mut self,
        self_: Resource<NodeRes>,
    ) -> wasmtime::Result<Option<RigidBodyKind>> {
        let rb = shared::wired::scene::node::rigid_body(&self.api, self_.rep())
            .map_err(wasmtime::Error::from_anyhow)?;
        Ok(rb.map(|rb| match rb {
            NodeRigidBody::Dynamic => RigidBodyKind::Dynamic,
            NodeRigidBody::Fixed => RigidBodyKind::Fixed,
            NodeRigidBody::Kinematic => RigidBodyKind::Kinematic,
        }))
    }

    async fn set_rigid_body(
        &mut self,
        self_: Resource<NodeRes>,
        value: Option<RigidBodyKind>,
    ) -> wasmtime::Result<()> {
        let rb = value.map(|rb| match rb {
            RigidBodyKind::Dynamic => NodeRigidBody::Dynamic,
            RigidBodyKind::Fixed => NodeRigidBody::Fixed,
            RigidBodyKind::Kinematic => NodeRigidBody::Kinematic,
        });
        shared::wired::scene::node::set_rigid_body(&self.api, self_.rep(), rb)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn drop(&mut self, rep: Resource<NodeRes>) -> wasmtime::Result<()> {
        shared::wired::scene::node::on_drop(&self.api, rep.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }
}
