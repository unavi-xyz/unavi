//! Eases the sky toward the tone its surroundings ask for, so a change of
//! space or of sky colour never cuts.

use bevy::prelude::*;
use bevy_hsd::{
    attributes::sky::Sky,
    prim::{
        Prim,
        PrimOf,
    },
};
use unavi_space::{
    grid::ActiveSpace,
    membership::Space,
};

use crate::{
    camera::sky::{
        SkyMaterial,
        SkyParams,
        palette::{
            MAX_CHROMA,
            Palette,
            Tone,
        },
    },
    scene::SceneState,
};

/// Seconds for limbo's hue to go once around the wheel.
const LIMBO_HUE_PERIOD: f64 = 120.0;
/// Seconds the springs take to cover most of a change.
const SMOOTH_TIME: f32 = 1.5;

/// What the sky is easing toward.
#[derive(Resource, Clone, Copy, Debug)]
pub struct SkyTarget {
    tone:      Tone,
    /// How strongly limbo's graticule shows, from 0 to 1.
    graticule: f32,
}

impl SkyTarget {
    /// Limbo belongs to no space, so its hue keeps turning.
    fn limbo(elapsed: f64) -> Self {
        let turns = (elapsed / LIMBO_HUE_PERIOD).fract();
        Self {
            tone:      Tone {
                hue:    (turns * 360.0) as f32,
                chroma: MAX_CHROMA,
            },
            graticule: 1.0,
        }
    }
}

impl Default for SkyTarget {
    fn default() -> Self {
        Self::limbo(0.0)
    }
}

/// The sky as currently drawn, mid-ease.
#[derive(Resource, Debug)]
pub struct SkyBlend {
    hue:       Spring,
    chroma:    Spring,
    graticule: Spring,
}

impl SkyBlend {
    /// The tone as currently drawn.
    pub fn tone(&self) -> Tone {
        Tone {
            hue:    self.hue.value.rem_euclid(360.0),
            chroma: self.chroma.value.max(0.0),
        }
    }

    pub fn palette(&self) -> Palette {
        Palette::new(self.tone())
    }

    pub const fn graticule(&self) -> f32 {
        self.graticule.value.clamp(0.0, 1.0)
    }

    fn step(&mut self, target: &SkyTarget, dt: f32) {
        // Hue eases along the shorter way round the wheel.
        let delta = (target.tone.hue - self.hue.value + 180.0).rem_euclid(360.0) - 180.0;
        self.hue.step(self.hue.value + delta, dt);
        self.hue.value = self.hue.value.rem_euclid(360.0);
        self.chroma.step(target.tone.chroma, dt);
        self.graticule.step(target.graticule, dt);
    }
}

impl Default for SkyBlend {
    fn default() -> Self {
        let target = SkyTarget::default();
        Self {
            hue:       Spring::at(target.tone.hue),
            chroma:    Spring::at(target.tone.chroma),
            graticule: Spring::at(target.graticule),
        }
    }
}

/// A critically damped spring, so a target that moves mid-ease bends the
/// curve instead of restarting it.
#[derive(Clone, Copy, Debug)]
struct Spring {
    value:    f32,
    velocity: f32,
}

impl Spring {
    const fn at(value: f32) -> Self {
        Self {
            value,
            velocity: 0.0,
        }
    }

    fn step(&mut self, target: f32, dt: f32) {
        let omega = 2.0 / SMOOTH_TIME;
        let x = omega * dt;
        let decay = 1.0 / (0.235 * x * x).mul_add(x, (0.48 * x).mul_add(x, 1.0 + x));
        let offset = self.value - target;
        let pull = omega.mul_add(offset, self.velocity) * dt;
        self.velocity = omega.mul_add(-pull, self.velocity) * decay;
        self.value = (offset + pull).mul_add(decay, target);
    }
}

/// Picks the tone the sky should take: limbo's turning hue, or the active
/// space's sky colour, or the colour derived from its id.
pub fn target_sky(
    time: Res<Time>,
    scene: Res<State<SceneState>>,
    active: Res<ActiveSpace>,
    spaces: Query<&Space>,
    skies: Query<(&Sky, &PrimOf, &Prim)>,
    mut target: ResMut<SkyTarget>,
) {
    if *scene.get() == SceneState::Limbo {
        *target = SkyTarget::limbo(time.elapsed_secs_f64());
        return;
    }
    // Out of limbo the graticule goes even before a space is known, which
    // keeps whatever tone the sky had.
    target.graticule = 0.0;
    let Some(space) = active.0 else { return };

    // Only the space's own document sets its sky, not whatever a visitor
    // brought into it. The lowest prim id wins, so two skies settle the same
    // way every frame.
    let own = skies
        .iter()
        .filter(|(_, prim_of, _)| prim_of.0 == space)
        .min_by_key(|(_, _, prim)| prim.0)
        .map(|(sky, ..)| Tone::of(sky.0));
    let tone = match own {
        Some(tone) => tone,
        None => match spaces.get(space) {
            Ok(space) => Tone::of_space(space.id()),
            Err(_) => return,
        },
    };
    target.tone = tone;
}

/// Eases the sky toward its target and hands the result to the sky material
/// and every camera's fog.
pub fn blend_sky(
    time: Res<Time>,
    target: Res<SkyTarget>,
    mut blend: ResMut<SkyBlend>,
    skies: Query<&MeshMaterial3d<SkyMaterial>>,
    mut materials: ResMut<Assets<SkyMaterial>>,
    mut fogs: Query<&mut DistanceFog>,
) {
    blend.step(&target, time.delta_secs());
    let palette = blend.palette();

    for mut fog in &mut fogs {
        fog.color = palette.fog().into();
    }
    for handle in &skies {
        if let Some(mut material) = materials.get_mut(&handle.0) {
            material.params = SkyParams::new(&palette, blend.graticule(), time.elapsed_secs_f64());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style;

    #[test]
    fn limbo_fog_at_hue_zero_matches_the_loader_cover() {
        let fog = Palette::new(SkyTarget::limbo(0.0).tone).fog();
        // loader.html paints this colour until the app's first frames.
        assert_eq!(Srgba::from(fog).to_hex().to_lowercase(), "#e4d2df");
    }

    /// Every token colour as the lower-case hex a stylesheet writes.
    fn token_hexes() -> Vec<(&'static str, String)> {
        [
            ("BG", style::BG),
            ("BG_DARK", style::BG_DARK),
            ("FG", style::FG),
            ("FG_2", style::FG_2),
            ("MUTED", style::MUTED),
            ("LIGHT", style::LIGHT),
            ("PANEL", style::PANEL),
            ("TINT", style::TINT),
            ("RULE", style::RULE),
            ("BLUE", style::BLUE),
            ("RED", style::RED),
            ("FIELD_COVER", style::FIELD_COVER),
        ]
        .map(|(name, token)| {
            let hex = Color::from(token).to_srgba().to_hex().to_lowercase();
            (name, hex)
        })
        .to_vec()
    }

    /// The `#rrggbb` literals in `text`.
    fn hex_literals(text: &str) -> Vec<String> {
        text.split('#')
            .skip(1)
            .filter_map(|rest| {
                let digits = rest.get(..6)?;
                let ends = rest
                    .get(6..7)
                    .is_none_or(|next| !next.chars().all(|c| c.is_ascii_alphanumeric()));
                (ends && digits.chars().all(|c| c.is_ascii_hexdigit()))
                    .then(|| format!("#{}", digits.to_lowercase()))
            })
            .collect()
    }

    #[test]
    fn the_loader_and_page_write_the_token_colours() {
        let loader = include_str!("../../../loader.html");
        let page = include_str!("../../../public/style.css");
        let tokens = token_hexes();
        let hex = |name: &str| {
            tokens
                .iter()
                .find(|(token, _)| *token == name)
                .map(|(_, hex)| hex.clone())
                .unwrap_or_default()
        };

        for (file, text, wanted) in [
            (
                "loader.html",
                loader,
                ["FIELD_COVER", "FG", "MUTED", "RED", "BG", "BLUE"].as_slice(),
            ),
            ("style.css", page, ["FIELD_COVER", "FG"].as_slice()),
        ] {
            let written = hex_literals(text);
            for literal in &written {
                assert!(
                    tokens.iter().any(|(_, hex)| hex == literal),
                    "{file} writes {literal}, which is no token"
                );
            }
            for name in wanted {
                assert!(
                    written.contains(&hex(name)),
                    "{file} does not write {name} ({})",
                    hex(name)
                );
            }
        }

        let [r, g, b, _] = Color::from(style::FG).to_srgba().to_u8_array();
        assert!(
            loader.contains(&format!("rgba({r}, {g}, {b},")),
            "loader.html's dot rgba is not FG"
        );
    }

    #[test]
    fn hue_eases_the_short_way_across_zero() {
        let mut blend = SkyBlend {
            hue:       Spring::at(350.0),
            chroma:    Spring::at(MAX_CHROMA),
            graticule: Spring::at(0.0),
        };
        let target = SkyTarget {
            tone:      Tone {
                hue:    10.0,
                chroma: MAX_CHROMA,
            },
            graticule: 0.0,
        };

        blend.step(&target, 0.2);
        let hue = blend.hue.value;
        assert!(hue > 350.0 || hue < 10.0, "went the long way: {hue}");

        for _ in 0..600 {
            blend.step(&target, 1.0 / 60.0);
        }
        assert!(
            (blend.hue.value - 10.0).abs() < 0.05,
            "settled at {}",
            blend.hue.value
        );
    }
}
