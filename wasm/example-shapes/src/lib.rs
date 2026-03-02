use crate::{
    exports::wired::script::guest_api::{Guest, GuestScript},
    unavi::shapes::api::Cuboid,
    wired::scene::context::self_document,
};

wit_bindgen::generate!({
    generate_all,
});

struct World;

impl Guest for World {
    type Script = Script;
}

struct Script;

impl GuestScript for Script {
    fn new() -> Self {
        let doc = self_document();
        let node = doc.create_node();

        let cube = Cuboid::new(1.0, 1.0, 1.0).mesh();
        node.set_mesh(Some(cube));

        let mat = doc.create_material();
        mat.set_base_color(&[0.3, 0.4, 0.8, 1.0]);
        node.set_material(Some(mat));

        Self
    }

    fn tick(&self) {}

    fn render(&self) {}

    fn drop(&self) {}
}

export!(World);
