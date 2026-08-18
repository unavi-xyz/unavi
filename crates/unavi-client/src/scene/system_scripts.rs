use bevy::prelude::*;
use bevy_hsd::load::LoadHsd;
use unavi_policy::{
    document::DocumentPolicy,
    reach::Reach,
};
use unavi_script::quota::QuotaExempt;

const SHELL_HSD: &str = "hsd/unavi_halo.hsdz";
const TOOL_HSDS: &[&str] = &["hsd/unavi_spawner.hsdz", "hsd/unavi_physgun.hsdz"];

/// Loads the shell and the tools it ships with.
///
/// `own_only` refuses every other peer, so a document a peer brought can
/// neither write the shell's scene nor speak on the channels its tool registry
/// listens to. Same-owner writes answer before any rung, so it costs the three
/// nothing between themselves — and by the same rule it does not hold off a
/// document the user pinned but did not author, which needs provenance rather
/// than a rung.
pub fn spawn_system_scripts(mut commands: Commands, asset_server: Res<AssetServer>) {
    for &path in TOOL_HSDS.iter().chain(std::iter::once(&SHELL_HSD)) {
        let handle = asset_server.load(path);
        commands.spawn((
            LoadHsd {
                handle,
                on_load: None,
            },
            DocumentPolicy::system(),
            Reach::own_only(),
            QuotaExempt,
        ));
    }
}
