use std::sync::{Arc, Mutex};

use anyhow::Context;
use bevy::prelude::*;
use hsd::{
    HSD_CONTAINER_ID,
    attributes::{ATTRIBUTES_KEY, RELATIONSHIPS_KEY},
};
use loro::{ExportMode, LoroValue, Subscription, TreeID, ValueOrContainer, event::ContainerDiff};

use crate::{
    Hsd, HsdPrimIndex,
    attributes::{DocContext, PARSERS},
    diff::{DiffQueue, HsdDiffEvent},
};

#[derive(Component)]
pub struct HsdSubscription {
    _handle: Subscription,
}

pub fn subscribe_to_docs(trigger: On<Add, Hsd>, docs: Query<&Hsd>, mut commands: Commands) {
    let doc = docs.get(trigger.entity).expect("get doc");

    let (tx, rx) = std::sync::mpsc::channel();

    let ctx = DocContext {
        doc: Arc::clone(&doc.0),
        tx: Arc::new(tx),
    };
    let ctx_sub = ctx.clone();

    let handle = doc.0.subscribe_root(Arc::new(move |diff| {
        for event in diff.events {
            if let Err(err) = process_diff_event(&ctx_sub, event) {
                warn!(?err, "failed to process hsd diff");
            }
        }
    }));

    if let Err(err) = process_initial_state(&ctx) {
        error!(?err, "failed to process initial hsd state");
    }

    commands.entity(trigger.entity).insert((
        HsdSubscription { _handle: handle },
        DiffQueue(Arc::new(Mutex::new(rx))),
        HsdPrimIndex::default(),
    ));
}

fn process_initial_state(ctx: &DocContext) -> anyhow::Result<()> {
    // Export then re-import on a fresh document to force diff events.
    let mut fresh_ctx = ctx.clone();
    fresh_ctx.doc = Arc::default();

    let fresh_ctx_sub = fresh_ctx.clone();
    let handle = fresh_ctx.doc.subscribe_root(Arc::new(move |diff| {
        for event in diff.events {
            if let Err(err) = process_diff_event(&fresh_ctx_sub, event) {
                warn!(?err, "failed to process initial hsd diff");
            }
        }
    }));

    let snapshot = ctx.doc.export(ExportMode::Snapshot)?;
    fresh_ctx.doc.import(&snapshot)?;

    handle.unsubscribe();

    Ok(())
}

fn process_diff_event(ctx: &DocContext, event: ContainerDiff) -> anyhow::Result<()> {
    if event.path.is_empty() {
        return Ok(());
    }
    if event.path[0].0 != *HSD_CONTAINER_ID {
        return Ok(());
    }

    match event.path.len() {
        1 => dispatch_prim(ctx, event),
        2 => Ok(()),
        3 => dispatch_section(ctx, event),
        _ => dispatch_attribute_data(ctx, event),
    }
}

fn dispatch_prim(ctx: &DocContext, event: ContainerDiff) -> anyhow::Result<()> {
    let diff = event
        .diff
        .into_tree()
        .map_err(|_| anyhow::anyhow!("invalid root diff type"))?;

    for item in diff.diff.clone() {
        ctx.tx
            .send(HsdDiffEvent::Prim(item))
            .map_err(|_| anyhow::anyhow!("failed to send diff event"))?;
    }
    Ok(())
}

fn dispatch_section(ctx: &DocContext, event: ContainerDiff) -> anyhow::Result<()> {
    let prim = prim_from_path(event.path)?;
    let section = event.path[2]
        .1
        .as_key()
        .ok_or_else(|| anyhow::anyhow!("invalid section index type"))?
        .to_string();

    let map_diff = event
        .diff
        .as_map()
        .ok_or_else(|| anyhow::anyhow!("invalid section diff type"))?;

    match section.as_str() {
        ATTRIBUTES_KEY => {
            for (key, value) in &map_diff.updated {
                ctx.tx
                    .send(HsdDiffEvent::Attr {
                        prim,
                        attr: key.to_string(),
                        value: normalize_value(value.clone()),
                    })
                    .map_err(|_| anyhow::anyhow!("failed to send attr event"))?;
            }
        }
        RELATIONSHIPS_KEY => {
            for (key, value) in &map_diff.updated {
                let target = relationship_target(value.as_ref())?;
                ctx.tx
                    .send(HsdDiffEvent::Relationship {
                        prim,
                        key: key.to_string(),
                        target,
                    })
                    .map_err(|_| anyhow::anyhow!("failed to send relationship event"))?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn dispatch_attribute_data(ctx: &DocContext, event: ContainerDiff) -> anyhow::Result<()> {
    let prim = prim_from_path(event.path)?;
    let section = event.path[2]
        .1
        .as_key()
        .ok_or_else(|| anyhow::anyhow!("invalid section index type"))?
        .to_string();

    if section != ATTRIBUTES_KEY {
        return Ok(());
    }

    let attr = event.path[3]
        .1
        .as_key()
        .ok_or_else(|| anyhow::anyhow!("invalid attribute index type"))?
        .to_string();

    let inner_path = if event.path.len() >= 5 {
        &event.path[4..]
    } else {
        &[]
    };

    let Some(parser) = PARSERS.get(attr.as_str()) else {
        return Ok(());
    };

    parser
        .parse(ctx, prim, inner_path, event.diff)
        .with_context(|| format!("{attr} parser"))?;
    Ok(())
}

fn prim_from_path(path: &[(loro::ContainerID, loro::Index)]) -> anyhow::Result<TreeID> {
    path[1]
        .1
        .as_node()
        .copied()
        .ok_or_else(|| anyhow::anyhow!("invalid prim index type"))
}

/// Collapse `Some(Value(Null))` to `None`. Lorosurgeon's `MaybeMissing<T>`
/// reconciles `Missing` by writing `Null`, but for our lifecycle dispatch
/// "null in the doc" should mean "attribute absent".
fn normalize_value(value: Option<ValueOrContainer>) -> Option<ValueOrContainer> {
    match value {
        Some(ValueOrContainer::Value(LoroValue::Null)) => None,
        other => other,
    }
}

fn relationship_target(value: Option<&ValueOrContainer>) -> anyhow::Result<Option<TreeID>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let ValueOrContainer::Value(LoroValue::String(s)) = value else {
        anyhow::bail!("relationship target must be a string");
    };
    let target = TreeID::try_from(s.as_str())
        .map_err(|err| anyhow::anyhow!("invalid relationship TreeID {s:?}: {err}"))?;
    Ok(Some(target))
}
