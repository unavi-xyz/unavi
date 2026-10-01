use bevy::prelude::*;
use bevy_hsd::package::ImportPackage;
use unavi_script::QuotaExempt;

const SHELL_HSD: &str = "hsd/unavi_halo.hsdz";
const TOOL_HSDS: &[&str] = &["hsd/unavi_spawner.hsdz", "hsd/unavi_physgun.hsdz"];

/// Loads the shell and the tools it ships with.
///
/// They hang under no space and no peer's pin, so they resolve as authored
/// here, and a document authored here is trusted at `Trust::Myself` — which
/// is the whole of what separates them from a document a peer brought.
pub fn spawn_system_scripts(mut commands: Commands, asset_server: Res<AssetServer>) {
    for &path in TOOL_HSDS.iter().chain(std::iter::once(&SHELL_HSD)) {
        let handle = asset_server.load(path);
        commands.spawn((ImportPackage(handle), QuotaExempt));
    }
}
