use bevy::prelude::*;

use self::shadows::ShadowQuality;

mod shadows;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UserSettings::default()).add_systems(
            FixedUpdate,
            shadows::set_shadow_config.run_if(shadow_quality_changed()),
        );
    }
}

#[derive(Resource)]
pub struct UserSettings {
    pub shadow_quality: ShadowQuality,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            shadow_quality: ShadowQuality::High,
        }
    }
}

fn shadow_quality_changed() -> impl Condition<()> {
    IntoSystem::into_system(
        |mut loaded_initial: Local<bool>,
         mut prev_settings: Local<UserSettings>,
         shadow_settings: Res<UserSettings>| {
            if !*loaded_initial {
                *loaded_initial = true;
                prev_settings.shadow_quality = shadow_settings.shadow_quality.clone();
                return true;
            }

            if prev_settings.shadow_quality != shadow_settings.shadow_quality {
                prev_settings.shadow_quality = shadow_settings.shadow_quality.clone();
                true
            } else {
                false
            }
        },
    )
}
