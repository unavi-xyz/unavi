use bevy::asset::LoadState;
use bevy::core_pipeline::Skybox;
use bevy::prelude::*;
use bevy::render::render_resource::{TextureViewDescriptor, TextureViewDimension};

#[derive(Resource)]
pub struct Cubemap {
    is_loaded: bool,
    image_handle: Handle<Image>,
}

pub fn create_skybox(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(Cubemap {
        is_loaded: false,
        image_handle: asset_server.load("images/skybox.png"),
    });
}

/// Adds the Skybox component to all cameras
pub fn add_skybox_to_cameras(
    mut commands: Commands,
    cubemap: ResMut<Cubemap>,
    cameras: Query<Entity, Without<Skybox>>,
) {
    for camera in cameras.iter() {
        commands.entity(camera).insert(Skybox {
            image: cubemap.image_handle.clone(),
            brightness: 1500.0,
        });
    }
}

/// Loads the skybox cubemap and adds it to the skybox resource
pub fn process_cubemap(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut cubemap: ResMut<Cubemap>,
    mut skyboxes: Query<&mut Skybox>,
) {
    if cubemap.is_loaded {
        return;
    }

    match asset_server.get_load_state(&cubemap.image_handle) {
        Some(load_state) => {
            if load_state != LoadState::Loaded {
                return;
            }
        }
        None => return,
    }

    // Load cubemap
    let image = match images.get_mut(&cubemap.image_handle) {
        Some(image) => image,
        None => {
            warn!("Failed to get skybox image");
            return;
        }
    };

    if image.texture_descriptor.array_layer_count() == 1 {
        image.reinterpret_stacked_2d_as_array(image.height() / image.width());
        image.texture_view_descriptor = Some(TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..default()
        });
    }

    // Update skyboxes
    for mut skybox in &mut skyboxes {
        skybox.image = cubemap.image_handle.clone();
    }

    cubemap.is_loaded = true;
}
