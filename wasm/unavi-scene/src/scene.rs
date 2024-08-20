use crate::bindings::{
    exports::unavi::scene::api::GuestScene,
    wired::{
        math::types::Transform,
        scene::{
            gltf::{Gltf, Scene},
            glxf::{AssetBorrow, AssetGltf, ChildrenBorrow, GlxfNode},
            node::Node,
        },
    },
};

pub struct SceneImpl {
    pub asset: AssetGltf,
    pub gltf: Gltf,
    pub node: GlxfNode,
    pub scene: Scene,
}

impl GuestScene for SceneImpl {
    fn new() -> Self {
        let gltf = Gltf::new();
        let scene = Scene::new();
        gltf.add_scene(&scene);
        gltf.set_default_scene(&scene);
        gltf.set_active_scene(Some(&scene));

        let asset = AssetGltf::new(&gltf);
        let node = GlxfNode::new();
        node.set_children(Some(&ChildrenBorrow::Asset(AssetBorrow::Gltf(&asset))));

        Self {
            asset,
            gltf,
            node,
            scene,
        }
    }

    fn active(&self) -> bool {
        if let Some(v) = self.gltf.active_scene() {
            v == self.scene
        } else {
            false
        }
    }
    fn set_active(&self, value: bool) {
        if value {
            self.gltf.set_active_scene(Some(&self.scene));
        } else {
            self.gltf.set_active_scene(None);
        }
    }

    fn transform(&self) -> Transform {
        self.node.transform()
    }
    fn set_transform(&self, value: Transform) {
        self.node.set_transform(value)
    }

    fn list_nodes(&self) -> Vec<Node> {
        self.scene.nodes()
    }
    fn create_node(&self) -> Node {
        let node = Node::new();
        self.add_node(&node);
        node
    }
    fn add_node(&self, value: &Node) {
        self.scene.add_node(value)
    }
    fn remove_node(&self, value: &Node) {
        self.scene.remove_node(value)
    }
}
