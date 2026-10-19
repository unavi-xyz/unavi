//! The local user's seal: a colour chosen once per device, drawn by
//! `Node::Seal` only in documents this user authors.

use bevy::prelude::*;
use bevy_hsd::{
    attributes::shader::{
        Sealed,
        ShellSeal,
    },
    document::{
        HsdDocId,
        HsdNamespace,
    },
};
use rand::Rng;
use unavi_local::DeviceStorage;
use unavi_space::authority::SpaceView;

const HUE_KEY: &str = "seal";

/// Bright and chromatic enough to read as a pane's border line.
const LIGHTNESS: f32 = 0.75;
/// The chroma asked for. Some hues cannot show it at this lightness, and give
/// up chroma until they fit.
const CHROMA: f32 = 0.15;
const CHROMA_STEP: f32 = 0.005;

pub struct SealPlugin {
    pub storage: DeviceStorage,
}

impl Plugin for SealPlugin {
    fn build(&self, app: &mut App) {
        let hue = load_hue(&self.storage);
        app.insert_resource(ShellSeal(seal_color(hue)))
            .add_systems(Update, seal_authored_documents);
    }
}

/// The seal for `hue` in degrees, with the most chroma up to [`CHROMA`] that
/// stays inside the display's gamut, since a channel outside `0..=1` would
/// reach the shader verbatim.
fn seal_color(hue: f32) -> LinearRgba {
    let mut chroma = CHROMA;
    loop {
        let linear = LinearRgba::from(Color::oklcha(LIGHTNESS, chroma, hue, 1.0));
        let in_gamut = [linear.red, linear.green, linear.blue]
            .iter()
            .all(|channel| (0.0..=1.0).contains(channel));
        if in_gamut || chroma <= 0.0 {
            return linear;
        }
        chroma = (chroma - CHROMA_STEP).max(0.0);
    }
}

/// The device's hue in degrees, generated at random on first use. A device
/// whose storage fails still gets a seal, just not a stable one.
fn load_hue(storage: &DeviceStorage) -> f32 {
    match storage.read(HUE_KEY) {
        Ok(Some(text)) => {
            if let Ok(hue) = text.trim().parse::<f32>()
                && (0.0..360.0).contains(&hue)
            {
                return hue;
            }
            warn!("unreadable seal hue; choosing a new one");
        }
        Ok(None) => {}
        Err(err) => warn!(?err, "failed to read the seal hue"),
    }

    let hue = rand::rng().random_range(0.0..360.0);
    if let Err(err) = storage.write(HUE_KEY, &hue.to_string()) {
        warn!(?err, "failed to persist the seal hue");
    }
    hue
}

/// Marks every namespace-backed document the local user itself authors. A
/// reference instance, or a document hosted by another, is never sealed:
/// resolving its author through the host would hand a stranger's prefab the
/// seal.
///
/// Authorship can resolve after a document loads, so every document is
/// re-checked whenever the view says who authors what has changed, and a new
/// one when it appears.
fn seal_authored_documents(
    view: Option<Res<SpaceView>>,
    docs: Query<(Entity, &HsdDocId, Has<Sealed>), With<HsdNamespace>>,
    fresh: Query<Entity, Or<(Added<HsdDocId>, Added<HsdNamespace>)>>,
    mut seen: Local<Option<u64>>,
    mut commands: Commands,
) {
    let Some(view) = view else {
        return;
    };

    let generation = view.author_generation();
    let resweep = *seen != Some(generation);
    *seen = Some(generation);

    for (doc, id, sealed) in &docs {
        if !resweep && !fresh.contains(doc) {
            continue;
        }
        let mine = view.authored_by_me(id.0);
        if mine && !sealed {
            commands.entity(doc).insert(Sealed);
        } else if !mine && sealed {
            commands.entity(doc).remove::<Sealed>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_hue_has_a_seal_inside_the_gamut() {
        for degrees in 0..360 {
            let seal = seal_color(degrees as f32);
            for channel in [seal.red, seal.green, seal.blue] {
                assert!(
                    (0.0..=1.0).contains(&channel),
                    "hue {degrees} has channel {channel}"
                );
            }
        }
    }

    #[test]
    fn the_hue_is_chosen_once_and_kept() {
        let storage = DeviceStorage::memory();
        let first = load_hue(&storage);
        assert!((0.0..360.0).contains(&first));
        assert!((load_hue(&storage) - first).abs() < f32::EPSILON);
    }

    #[test]
    fn a_garbled_hue_is_replaced() {
        let storage = DeviceStorage::memory();
        storage.write(HUE_KEY, "not a number").expect("write");
        let hue = load_hue(&storage);
        assert!((0.0..360.0).contains(&hue));
        assert_eq!(storage.read(HUE_KEY).expect("read"), Some(hue.to_string()));
    }
}
