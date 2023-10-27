use bevy::asset::LoadState;
use bevy::core_pipeline::Skybox;
use bevy::prelude::*;
use bevy::render::render_resource::{TextureViewDescriptor, TextureViewDimension};

#[derive(Resource)]
pub struct Cubemap {
    is_loaded: bool,
    image_handle: Handle<Image>,
}

pub fn setup_skybox(mut commands: Commands) {
    commands.insert_resource(Cubemap {
        is_loaded: false,
        image_handle: Default::default(),
    });
}

const SKYBOX_URI: &str = "skybox-1-512.png";

pub fn create_skybox(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut cubemap: ResMut<Cubemap>,
    cameras: Query<Entity, (With<Camera3d>, Without<Skybox>)>,
) {
    if cameras.is_empty() {
        return;
    }

    info!("Loading skybox {}", SKYBOX_URI);

    let skybox_handle = asset_server.load(SKYBOX_URI);

    let ent = cameras.single();

    commands.entity(ent).insert(Skybox(skybox_handle.clone()));

    cubemap.is_loaded = false;
    cubemap.image_handle = skybox_handle;
}

pub fn process_skybox(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut cubemap: ResMut<Cubemap>,
    mut skyboxes: Query<&mut Skybox>,
) {
    if cubemap.is_loaded {
        return;
    }

    if asset_server.get_load_state(cubemap.image_handle.clone_weak()) != LoadState::Loaded {
        return;
    }

    // Load cubemap
    let image = images.get_mut(&cubemap.image_handle);

    let image = match image {
        Some(image) => image,
        None => {
            warn!("Failed to load skybox image");
            return;
        }
    };

    // NOTE: PNGs do not have any metadata that could indicate they contain a cubemap texture,
    // so they appear as one texture. The following code reconfigures the texture as necessary.
    if image.texture_descriptor.array_layer_count() == 1 {
        image.reinterpret_stacked_2d_as_array(
            image.texture_descriptor.size.height / image.texture_descriptor.size.width,
        );
        image.texture_view_descriptor = Some(TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..default()
        });
    }

    for mut skybox in &mut skyboxes {
        skybox.0 = cubemap.image_handle.clone();
    }

    cubemap.is_loaded = true;
}
