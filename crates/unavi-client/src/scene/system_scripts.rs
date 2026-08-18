use bevy::prelude::*;
use bevy_hsd::load::LoadHsd;
use unavi_policy::document::DocumentPolicy;
use unavi_script::quota::QuotaExempt;

const SHELL_HSD: &str = "hsd/unavi_halo.hsdz";
const TOOL_HSDS: &[&str] = &["hsd/unavi_spawner.hsdz", "hsd/unavi_physgun.hsdz"];

/// Loads the shell and the tools it ships with.
///
/// They need no reach configuration between them: all three are the local
/// user's own documents, and same-owner documents reach each other
/// unconditionally.
pub fn spawn_system_scripts(mut commands: Commands, asset_server: Res<AssetServer>) {
    for &path in TOOL_HSDS.iter().chain(std::iter::once(&SHELL_HSD)) {
        let handle = asset_server.load(path);
        commands.spawn((
            LoadHsd {
                handle,
                on_load: None,
            },
            DocumentPolicy::system(),
            QuotaExempt,
        ));
    }
}
