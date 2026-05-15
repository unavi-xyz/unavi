use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::Context;
use bevy::prelude::*;
use hsd::HSD_CONTAINER_ID;
use loro::{ExportMode, LoroDoc, Subscription, event::ContainerDiff};

use crate::{
    Hsd,
    attributes::{AttributeParser, PARSERS, xform::XformParser},
    diff::{DiffQueue, DiffSender, HsdDiffEvent},
};

#[derive(Component)]
pub struct HsdSubscription {
    _handle: Subscription,
}

#[derive(Clone)]
struct DocContext {
    doc: Arc<LoroDoc>,
    tx: DiffSender,
}

pub fn subscribe_to_docs(trigger: On<Add, Hsd>, docs: Query<&Hsd>, mut commands: Commands) {
    let doc = docs.get(trigger.entity).expect("get doc");

    let (tx, rx) = std::sync::mpsc::channel();

    let mut parsers = HashMap::<_, Box<dyn AttributeParser>>::new();
    parsers.insert("xform", Box::new(XformParser));

    let ctx = DocContext {
        doc: Arc::clone(&doc.0),
        tx: Arc::new(tx),
    };
    let ctx_sub = ctx.clone();

    // Subscribe to future diffs.
    let handle = doc.0.subscribe_root(Arc::new(move |diff| {
        for event in diff.events {
            if let Err(err) = process_diff_event(&ctx_sub, event) {
                warn!(?err, "Failed to process HSD diff");
            }
        }
    }));

    // Process initial state.
    if let Err(err) = process_initial_state(&ctx) {
        error!(?err, "Failed to process initial HSD state");
    }

    commands.entity(trigger.entity).insert((
        HsdSubscription { _handle: handle },
        DiffQueue(Arc::new(Mutex::new(rx))),
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
                warn!(?err, "Failed to process initial HSD diff");
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
        // Only process events under the HSD tree.
        return Ok(());
    }

    // info!("path: {:#?}", event.path);
    // info!("diff: {:#?}", event.diff);

    match event.path.len() {
        1 => {
            // Prim diff.
            let diff = event
                .diff
                .into_tree()
                .map_err(|_| anyhow::anyhow!("invalid root diff type"))?;

            for item in diff.diff.clone() {
                ctx.tx
                    .send(HsdDiffEvent::Prim(item))
                    .map_err(|_| anyhow::anyhow!("failed to send diff event"))?;
            }
        }
        2 => {
            // Attribute diff.
            let prim = *event.path[1]
                .1
                .as_node()
                .ok_or_else(|| anyhow::anyhow!("invalid index type"))?;

            let diff = event
                .diff
                .as_map()
                .ok_or_else(|| anyhow::anyhow!("invalid attr diff type"))?;

            for (key, value) in &diff.updated {
                ctx.tx
                    .send(HsdDiffEvent::Attr {
                        prim,
                        attr: key.to_string(),
                        value: value.clone(),
                    })
                    .map_err(|_| anyhow::anyhow!("failed to send diff event"))?;
            }
        }
        _ => {
            // Attribute data diff.
            let prim = *event.path[1]
                .1
                .as_node()
                .ok_or_else(|| anyhow::anyhow!("invalid index type"))?;

            let attr = event.path[2]
                .1
                .as_key()
                .ok_or_else(|| anyhow::anyhow!("invalid index type"))?
                .to_string();

            let path = if event.path.len() >= 4 {
                &event.path[3..]
            } else {
                &[]
            };

            if let Some(p) = PARSERS.get(attr.as_str())
                && let Some(data) = p
                    .parse(&ctx.doc, prim, path, event.diff)
                    .with_context(|| format!("{} parser", attr.as_str()))?
            {
                ctx.tx
                    .send(HsdDiffEvent::AttrData { prim, data })
                    .map_err(|_| anyhow::anyhow!("failed to send diff event"))?;
            }
        }
    }

    Ok(())
}
