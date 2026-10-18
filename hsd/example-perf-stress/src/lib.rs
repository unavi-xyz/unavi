//! A reproducible stress space for the web client: many world-space text
//! labels, two portals, and two of the example scripts.
//!
//! # Shape
//!
//! `.hsda` carries names, transforms and script attachments only, because
//! `text` and `portal` are properties rather than source attributes. The
//! labels are minted in `init`; the portals are authored prims this script
//! finds by name.
//!
//! Nothing moves after `init`, so two runs of the same scene render the same
//! frames.

use wired_guest::math::{
    Color,
    Transform,
    Vec3,
};

use crate::wired::scene::{
    document::{
        Layer,
        script_document,
    },
    properties::{
        Portal,
        Property,
        Text,
    },
};

wired_guest::generate_script!(Script);

/// Text prims, `COLUMNS` to a row.
const LABELS: usize = 120;
const COLUMNS: usize = 12;
const SPACING_X: f32 = 0.28;
const SPACING_Y: f32 = 0.16;
const TOP_Y: f32 = 1.4;
const LABEL_Z: f32 = -5.0;
/// Em height in meters, so the labels are legible from the camera origin.
const LABEL_SIZE: f32 = 0.1;
const INK: Color = Color::rgb(0.95, 0.95, 0.95);

/// Authored prims that carry a portal. A destination needs a space id this
/// space does not have, so both open onto nothing.
const PORTALS: [&str; 2] = ["portal-left", "portal-right"];
const PORTAL_WIDTH: f32 = 1.4;
const PORTAL_HEIGHT: f32 = 2.0;

struct Script;

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        let doc = script_document()?;
        let root = doc
            .roots()
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("perf-stress has no root prim"))?;

        let mut batch = doc.local();
        for index in 0..LABELS {
            let label = format!("label {index:03}");
            let column = (index % COLUMNS) as f32;
            let row = (index / COLUMNS) as f32;
            let x = (column - (COLUMNS as f32 - 1.0) / 2.0) * SPACING_X;
            let y = row.mul_add(-SPACING_Y, TOP_Y);

            let prim = doc.create_prim(Layer::Local, Some(root))?;
            batch = batch
                .set(prim, Property::Name(format!("label-{index:03}")))
                .set(
                    prim,
                    Property::Transform(Transform::from_translation(Vec3::new(x, y, LABEL_Z))),
                )
                .set(
                    prim,
                    Property::Text(Text {
                        size: Some(LABEL_SIZE),
                        color: Some(INK),
                        emissive: Some(0.0),
                        ..Text::new(label)
                    }),
                );
        }

        for name in PORTALS {
            let portal = find_one(&doc, name)?;
            batch = batch.set(
                portal,
                Property::Portal(Portal {
                    width:       PORTAL_WIDTH,
                    height:      PORTAL_HEIGHT,
                    destination: None,
                }),
            );
        }
        batch.flush()?;

        println!(
            "perf-stress ready: {LABELS} labels, {} portals",
            PORTALS.len()
        );

        Ok(Self)
    }
}
