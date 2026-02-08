use bevy::{
    asset::io::web::WebAssetPlugin, light::light_consts::lux, log::LogPlugin, prelude::*,
    window::WindowTheme,
};

use bitflags::bitflags;
use tracing::Level;

#[cfg(not(target_family = "wasm"))]
mod assets;
mod async_commands;
#[cfg(feature = "devtools-network")]
mod devtools;
mod fade;
mod grab;
mod icon;
mod networking;
mod scene;
mod space;

bitflags! {
    #[derive(Clone, Copy, Debug, Default)]
    pub struct DebugFlags: u8 {
        const FPS     = 0b0001;
        const NETWORK = 0b0010;
        const PHYSICS = 0b0100;
    }
}

pub struct UnaviPlugin {
    pub debug: DebugFlags,
    pub in_memory: bool,
    pub log_level: Level,
}

impl Plugin for UnaviPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(target_family = "wasm"))]
        {
            assets::copy::copy_assets_to_dirs().expect("failed to copy assets");
            assets::download::download_web_assets().expect("failed to download web assets");
        }

        let default_plugins = DefaultPlugins
            .set(LogPlugin {
                filter: format!(
                    "{},loro_internal=off,offset_allocator=off",
                    bevy::log::DEFAULT_FILTER
                ),
                level: self.log_level,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    name: Some("unavi".to_string()),
                    title: "UNAVI".to_string(),
                    window_theme: Some(WindowTheme::Dark),
                    ..default()
                }),
                ..default()
            });

        #[cfg(not(target_family = "wasm"))]
        let default_plugins = default_plugins
            .set(AssetPlugin {
                file_path: assets::assets_dir().to_string_lossy().to_string(),
                ..default()
            })
            .disable::<WebAssetPlugin>();

        #[cfg(target_family = "wasm")]
        let default_plugins = default_plugins
            .set(AssetPlugin {
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            })
            .set(WebAssetPlugin {
                silence_startup_warning: true,
            });

        app.add_plugins((
            default_plugins,
            avian3d::PhysicsPlugins::default(),
            avian3d::picking::PhysicsPickingPlugin,
            fade::FadePlugin,
            bevy_wds::WdsPlugin,
            bevy_hsd::HsdPlugin,
            unavi_input::InputPlugin,
            unavi_locomotion::LocomotionPlugin,
            unavi_portal::PortalPlugin,
            #[cfg(not(target_family = "wasm"))]
            unavi_script::ScriptPlugin,
            networking::NetworkingPlugin {
                wds_in_memory: self.in_memory,
            },
            space::SpacePlugin,
        ))
        .insert_resource(AmbientLight {
            brightness: lux::OVERCAST_DAY,
            ..default()
        });

        #[cfg(feature = "devtools-bevy")]
        {
            if self.debug.contains(DebugFlags::FPS) {
                app.add_plugins(bevy::dev_tools::fps_overlay::FpsOverlayPlugin::default());
            }
            if self.debug.contains(DebugFlags::PHYSICS) {
                app.add_plugins(avian3d::debug_render::PhysicsDebugPlugin);
            }
        }

        #[cfg(feature = "devtools-network")]
        {
            if self.debug.contains(DebugFlags::NETWORK) {
                app.add_plugins(devtools::DevToolsPlugin { enabled: true });
            }
        }

        app.insert_resource(ClearColor(Color::BLACK))
            .init_resource::<grab::GrabbedObjects>()
            .init_resource::<grab::PointerLocations3d>()
            .add_observer(grab::handle_grab_click)
            .add_systems(Startup, (icon::set_window_icon, scene::spawn_scene))
            .add_systems(
                FixedUpdate,
                (async_commands::apply_async_commands, scene::spawn_agent),
            )
            .add_systems(
                Update,
                (grab::track_mouse_pointer, grab::move_grabbed_objects).chain(),
            );
    }
}
