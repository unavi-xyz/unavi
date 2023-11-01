use bevy::{
    pbr::{CascadeShadowConfig, CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
};

use super::UserSettings;

#[derive(Debug, PartialEq, Clone)]
pub enum ShadowQuality {
    Low,
    Medium,
    High,
}

impl ShadowQuality {
    pub fn size(&self) -> usize {
        match self {
            ShadowQuality::Low => 1024,
            ShadowQuality::Medium => 2048,
            ShadowQuality::High => 4096,
        }
    }

    pub fn num_cascades(&self) -> usize {
        #[cfg(target_arch = "wasm32")]
        let value = 1;

        #[cfg(not(target_arch = "wasm32"))]
        let value = match self {
            ShadowQuality::Low => 2,
            ShadowQuality::Medium => 3,
            ShadowQuality::High => 3,
        };

        value
    }

    pub fn first_cascade_far_bound(&self) -> f32 {
        #[cfg(target_arch = "wasm32")]
        let value = self.maximum_distance();

        #[cfg(not(target_arch = "wasm32"))]
        let value = match self {
            ShadowQuality::Low => 15.0,
            ShadowQuality::Medium => 15.0,
            ShadowQuality::High => 15.0,
        };

        value
    }

    pub fn maximum_distance(&self) -> f32 {
        #[cfg(target_arch = "wasm32")]
        let value = match self {
            ShadowQuality::Low => 20.0,
            ShadowQuality::Medium => 30.0,
            ShadowQuality::High => 40.0,
        };

        #[cfg(not(target_arch = "wasm32"))]
        let value = match self {
            ShadowQuality::Low => 30.0,
            ShadowQuality::Medium => 50.0,
            ShadowQuality::High => 80.0,
        };

        value
    }
}

pub fn set_shadow_config(
    settings: Res<UserSettings>,
    mut directional_shadow_config: ResMut<DirectionalLightShadowMap>,
    mut directional_lights: Query<&mut CascadeShadowConfig, With<DirectionalLight>>,
) {
    info!("Updating shadow config to {:?}", settings.shadow_quality);

    directional_shadow_config.size = settings.shadow_quality.size();

    let target_config: CascadeShadowConfig = CascadeShadowConfigBuilder {
        num_cascades: settings.shadow_quality.num_cascades(),
        first_cascade_far_bound: settings.shadow_quality.first_cascade_far_bound(),
        maximum_distance: settings.shadow_quality.maximum_distance(),
        ..Default::default()
    }
    .into();

    for mut config in directional_lights.iter_mut() {
        config.bounds = target_config.bounds.clone();
        config.minimum_distance = target_config.minimum_distance;
        config.overlap_proportion = target_config.overlap_proportion;
    }
}
