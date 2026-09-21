use bevy::prelude::*;
use bevy_hsd::load::LoadHsd;
use unavi_policy::permissions::Permissions;
use unavi_script::quota::QuotaExempt;

const SHELL_HSD: &str = "hsd/unavi_halo.hsdz";
const TOOL_HSDS: &[&str] = &["hsd/unavi_spawner.hsdz", "hsd/unavi_physgun.hsdz"];

/// Loads the shell and the tools it ships with.
///
/// [`Permissions::system`] is what separates them from a document a peer
/// brought; nothing else on this node holds the privileged half of the API.
pub fn spawn_system_scripts(mut commands: Commands, asset_server: Res<AssetServer>) {
    for &path in TOOL_HSDS.iter().chain(std::iter::once(&SHELL_HSD)) {
        let handle = asset_server.load(path);
        commands.spawn((
            LoadHsd {
                handle,
                on_load: None,
            },
            Permissions::system(),
            QuotaExempt,
        ));
    }
}
