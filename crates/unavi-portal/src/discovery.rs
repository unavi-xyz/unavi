use std::sync::Arc;

use bevy::prelude::*;
use bevy_hsd::{
    Hsd,
    HsdChild,
    HsdPrimIndex,
    HsdRecordId,
    Prim,
    attributes::portal::PortalConfig,
};
use hsd::{
    HSD_CONTAINER_ID,
    attributes::{
        Attribute,
        portal::{
            PortalAttr,
            PortalDestination as HsdPortalDestination,
            PortalReceptor,
        },
    },
};
use loro::{
    Container,
    LoroDoc,
    LoroMap,
    ValueOrContainer,
};
use loro_surgeon::bytes::ByteArray;

use crate::{
    PortalAllowIncoming,
    PortalTargetReceptor,
    PortalTargetSpace,
};

pub fn on_hsd_ready(
    trigger: On<Insert, Hsd>,
    dest_docs: Query<(&HsdRecordId, &HsdPrimIndex)>,
    sources: Query<(
        &PortalTargetSpace,
        Option<&PortalTargetReceptor>,
        &HsdChild,
    )>,
    source_docs: Query<&Hsd>,
    candidate_portals: Query<(&Prim, &PortalConfig, &PortalAllowIncoming)>,
) {
    let dest_ent = trigger.entity;
    let Ok((dest_record_id, dest_index)) = dest_docs.get(dest_ent) else {
        return;
    };

    let Some((receptor_prim, receptor_prim_id)) =
        find_open_receptor(dest_index, &candidate_portals)
    else {
        return;
    };

    for (target_space, existing, hsd_child) in &sources {
        if target_space.0 != dest_record_id.0 {
            continue;
        }
        if existing.is_some() {
            continue;
        }
        let Ok(source_doc) = source_docs.get(hsd_child.0) else {
            continue;
        };

        write_receptor_into_source(
            &source_doc.0,
            target_space.0,
            dest_record_id.0,
            receptor_prim_id.clone(),
        );
    }

    let _ = receptor_prim;
}

fn find_open_receptor(
    dest_index: &HsdPrimIndex,
    candidate_portals: &Query<(&Prim, &PortalConfig, &PortalAllowIncoming)>,
) -> Option<(Entity, String)> {
    for (&tree_id, &entity) in &dest_index.0 {
        let Ok((_, cfg, incoming)) = candidate_portals.get(entity) else {
            continue;
        };
        if incoming.0 && cfg.0.destination.is_none() {
            return Some((entity, tree_id.to_string()));
        }
    }
    None
}

fn write_receptor_into_source(
    source_doc: &Arc<LoroDoc>,
    target_space: blake3::Hash,
    receptor_document: blake3::Hash,
    receptor_prim: String,
) {
    let tree = source_doc.get_tree(&*HSD_CONTAINER_ID);
    for node in tree.nodes() {
        let Ok(meta) = tree.get_meta(node) else {
            continue;
        };
        let Some(attrs) = attributes_map(&meta) else {
            continue;
        };
        let Ok(mut attr) = PortalAttr::attr_hydrate(&attrs) else {
            continue;
        };
        let Some(dest) = attr.destination.as_ref() else {
            continue;
        };
        if dest.space.0 != *target_space.as_bytes() {
            continue;
        }
        attr.destination = Some(HsdPortalDestination {
            receptor: Some(PortalReceptor {
                document: ByteArray(*receptor_document.as_bytes()),
                prim:     receptor_prim.clone(),
            }),
            space:    ByteArray(*target_space.as_bytes()),
        });
        let _ = attr.attr_reconcile(attrs);
    }
    source_doc.commit();
}

fn attributes_map(meta: &LoroMap) -> Option<LoroMap> {
    match meta.get("attributes")? {
        ValueOrContainer::Container(Container::Map(m)) => Some(m),
        _ => None,
    }
}
