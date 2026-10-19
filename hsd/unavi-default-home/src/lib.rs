//! The default home space: a ground plane tinted by the document's own id,
//! so every space reads as itself without an author having to pick a colour.

use wired_guest::{
    color::{
        desaturate,
        document_seed,
        generate_color,
    },
    math::{
        Transform,
        Vec3,
    },
};

use crate::{
    unavi::shapes::api::Cuboid,
    wired::scene::{
        document::script_document,
        properties::{
            Material,
            Property,
            PropertyKey,
            Relation,
            RigidBody,
        },
    },
};

wired_guest::generate_script!(Script);

const GROUND_SIZE: f32 = 30.0;
const GROUND_THICK: f32 = 0.5;

struct Script;

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        let doc = script_document()?;

        let shape = Cuboid::new(Vec3::new(GROUND_SIZE, GROUND_THICK, GROUND_SIZE));
        let prim = shape.mesh()?;

        let mut batch = doc
            .local()
            .set(prim, Property::Collider(shape.collider()))
            .set(prim, Property::RigidBody(RigidBody::static_body()))
            .set(
                prim,
                Property::Transform(Transform::from_translation(Vec3::new(
                    0.0,
                    -GROUND_THICK / 2.0,
                    0.0,
                ))),
            );

        let base_color = desaturate(generate_color(document_seed(doc.id())), 0.6);

        let ground_root = doc.find_by_name("ground").into_iter().next();
        let mut material = ground_root
            .and_then(|p| match doc.get(p, &PropertyKey::Material) {
                Some(Property::Material(m)) => Some(m),
                _ => None,
            })
            .unwrap_or_else(|| Material {
                metallic: Some(0.1),
                roughness: Some(0.85),
                ..Material::default()
            });
        material.base_color = Some(base_color);

        // Material payloads cannot carry texture relationships, so the mesh
        // binds to the authored prim instead.
        batch = match ground_root {
            Some(ground) => batch
                .set(ground, Property::Material(material))
                .set(prim, Property::Relation((Relation::ShaderBinding, ground))),
            None => batch.set(prim, Property::Material(material)),
        };

        batch.flush()?;

        println!("Welcome home! =)");

        Ok(Self)
    }
}
