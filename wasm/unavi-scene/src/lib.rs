use bindings::{
    exports::unavi::scene::api::{Guest, GuestRoot, Scene, SceneBorrow},
    wired::scene::glxf::{get_root, Asset, AssetBorrow, Children, GlxfScene},
};
use scene::SceneImpl;

#[allow(warnings)]
mod bindings;

mod scene;
mod wired_math_impls;
mod wired_scene_impls;

struct RootImpl;

impl RootImpl {
    fn add_scene_impl(value: &SceneImpl) {
        let root = get_root();
        root.add_asset(&AssetBorrow::Gltf(&value.asset));
        root.add_node(&value.node);

        let scene = root.active_scene().unwrap_or_else(|| {
            let value = GlxfScene::new();
            root.add_scene(&value);
            root.set_default_scene(&value);
            root.set_active_scene(Some(&value));
            value
        });

        scene.add_node(&value.node);
    }
}

impl GuestRoot for RootImpl {
    fn list_scenes() -> Vec<Scene> {
        let root = get_root();

        let scene = match root.active_scene() {
            Some(v) => v,
            None => return Vec::new(),
        };

        scene
            .nodes()
            .into_iter()
            .filter_map(|n| {
                if let Some(Children::Asset(Asset::Gltf(asset))) = n.children() {
                    Some((n, asset))
                } else {
                    None
                }
            })
            .filter_map(|(node, asset)| {
                let gltf = asset.document();
                let scene = match gltf
                    .active_scene()
                    .or_else(|| gltf.default_scene())
                    .or_else(|| gltf.list_scenes().into_iter().next())
                {
                    Some(v) => v,
                    None => return None,
                };

                let scene = SceneImpl {
                    asset,
                    gltf,
                    node,
                    scene,
                };

                Some(Scene::new(scene))
            })
            .collect()
    }

    fn add_scene(value: SceneBorrow) {
        let value = value.get::<SceneImpl>();
        RootImpl::add_scene_impl(value);
    }

    fn remove_scene(value: SceneBorrow) {
        let value = value.get::<SceneImpl>();

        let root = get_root();
        root.remove_asset(&AssetBorrow::Gltf(&value.asset));
        root.remove_node(&value.node);

        for scene in root.list_scenes() {
            scene.remove_node(&value.node)
        }
    }
}

struct GuestImpl;

impl Guest for GuestImpl {
    type Root = RootImpl;
    type Scene = SceneImpl;
}

bindings::export!(GuestImpl with_types_in bindings);
