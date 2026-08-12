use bevy::prelude::*;
use bevy_hsd::attributes::text::TextData;
use bevy_msdf::{
    billboard::Billboard,
    mesh::Anchor,
    text::{
        MissingGlyphs,
        MsdfStyle,
        MsdfText,
    },
};
use hsd::attributes::{
    material::ColorVec,
    text::TextAttr,
};
use msdf::layout::Align;
use rstest::rstest;
use tracing_test::traced_test;

use crate::common::*;

mod common;

fn label(value: &str) -> TextAttr {
    TextAttr {
        value: value.to_string(),
        ..Default::default()
    }
}

#[traced_test]
#[rstest]
fn test_text_lifecycle(mut ctx: TestContext) {
    let root = ctx.create_prim();
    ctx.set_attr(root, &label("Workshop"));
    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut query = world.query::<&MsdfText>();
    let found = query.query(world).into_iter().collect::<Vec<_>>();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].value, "Workshop");

    ctx.remove_attr::<TextAttr>(root);
    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut query = world.query::<(Option<&MsdfText>, Option<&TextData>)>();
    let remaining = query
        .query(world)
        .into_iter()
        .filter(|(text, data)| text.is_some() || data.is_some())
        .count();
    assert_eq!(
        remaining, 0,
        "clearing the attribute takes the text with it"
    );
}

#[traced_test]
#[rstest]
fn test_text_becomes_a_mesh(mut ctx: TestContext) {
    let root = ctx.create_prim();
    ctx.set_attr(root, &label("Hello"));
    ctx.app.update();
    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut query = world.query::<&MissingGlyphs>();
    let found = query.query(world).into_iter().collect::<Vec<_>>();
    assert_eq!(found.len(), 1, "a text prim reports its missing glyphs");
    assert_eq!(found[0].0, 0, "the shipped font covers plain Latin");
    drop(query);

    let mut parents = world.query::<(&MsdfText, &Children)>();
    let children = parents
        .query(world)
        .into_iter()
        .next()
        .map(|(_, children)| children.iter().collect::<Vec<_>>())
        .expect("text");
    drop(parents);
    assert!(!children.is_empty(), "each page becomes a child mesh");
    let meshes = world.resource::<Assets<Mesh>>();
    let vertices = children
        .iter()
        .filter_map(|child| world.get::<Mesh3d>(*child))
        .map(|mesh| meshes.get(&mesh.0).expect("mesh").count_vertices())
        .sum::<usize>();
    assert_eq!(vertices, "Hello".len() * 4, "one quad per glyph");
}

#[traced_test]
#[rstest]
fn test_text_settings_reach_the_renderer(mut ctx: TestContext) {
    let root = ctx.create_prim();
    ctx.set_attr(
        root,
        &TextAttr {
            value:         "Atrium".to_string(),
            size:          Some(0.05),
            align:         Some("center".to_string()),
            anchor:        Some("middle".to_string()),
            wrap:          Some(0.4),
            line_height:   Some(1.5),
            color:         Some(ColorVec(vec![1.0, 0.0, 0.0, 1.0])),
            outline:       Some(ColorVec(vec![0.0, 0.0, 0.0, 1.0])),
            outline_width: Some(0.3),
            emissive:      Some(2.0),
            billboard:     Some("yaw".to_string()),
        },
    );
    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut query = world.query::<(&MsdfText, &MsdfStyle, &Billboard)>();
    let found = query.query(world).into_iter().collect::<Vec<_>>();
    assert_eq!(found.len(), 1);

    let (text, style, billboard) = found[0];
    assert!((text.size - 0.05).abs() < 1.0e-6);
    assert_eq!(text.align, Align::Center);
    assert_eq!(text.anchor, Anchor::Middle);
    assert_eq!(text.wrap, Some(0.4));
    assert!((text.line_height - 1.5).abs() < 1.0e-6);
    assert_eq!(style.color, Color::linear_rgba(1.0, 0.0, 0.0, 1.0));
    assert!((style.emissive - 2.0).abs() < 1.0e-6);
    assert!(style.outline.is_some());
    assert_eq!(*billboard, Billboard::Yaw);
}

/// A newer client's variant must not cost the reader its text.
#[traced_test]
#[rstest]
fn test_an_unknown_variant_still_draws(mut ctx: TestContext) {
    let root = ctx.create_prim();
    ctx.set_attr(
        root,
        &TextAttr {
            value: "Garden".to_string(),
            align: Some("justify".to_string()),
            billboard: Some("spherical".to_string()),
            ..Default::default()
        },
    );
    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut query = world.query::<(&MsdfText, Option<&Billboard>)>();
    let found = query.query(world).into_iter().collect::<Vec<_>>();
    assert_eq!(found.len(), 1);
    assert_eq!(
        found[0].0.align,
        Align::Left,
        "falls back rather than fails"
    );
    assert!(found[0].1.is_none());
}

/// Characters the shipped font has no glyph for are reported rather than
/// silently dropped.
#[traced_test]
#[rstest]
fn test_uncovered_characters_are_reported(mut ctx: TestContext) {
    let root = ctx.create_prim();
    ctx.set_attr(root, &label("hi 漢字"));
    ctx.app.update();
    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut query = world.query::<&MissingGlyphs>();
    let found = query.query(world).into_iter().collect::<Vec<_>>();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].0, 2, "the Latin field covers no CJK");
}
