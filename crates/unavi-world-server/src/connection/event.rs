use std::collections::{btree_map::Entry, BTreeMap};

use anyhow::Result;

use wired_world::datagram_capnp;
use xwt_core::base::Session;

use crate::update_loop::OutgoingEvent;

#[derive(Default)]
pub struct EventContext {
    local_ids: BTreeMap<usize, u16>,
    local_ids_rev: BTreeMap<u16, usize>,
    next_id: u16,
}

pub async fn handle_event(
    event: OutgoingEvent,
    ctx: &mut EventContext,
    session: &impl Session,
) -> Result<()> {
    match event {
        OutgoingEvent::PlayerJoined { id } => {
            if let Entry::Vacant(e) = ctx.local_ids.entry(id) {
                e.insert(ctx.next_id);
                ctx.local_ids_rev.insert(ctx.next_id, id);

                if ctx.next_id == u16::MAX {
                    ctx.next_id = 0;
                } else {
                    ctx.next_id += 1;
                }

                while ctx.local_ids_rev.contains_key(&ctx.next_id) {
                    ctx.next_id += 1;
                }
            }
        }
        OutgoingEvent::PlayerLeft { id } => {
            if let Some(local_id) = ctx.local_ids.remove(&id) {
                ctx.local_ids_rev.remove(&local_id);
            }
        }
        OutgoingEvent::Transforms(transforms) => {
            let mut buf = Vec::new();

            for (transform, player_id) in transforms.into_iter().zip(ctx.local_ids.values()) {
                let mut msg = capnp::message::Builder::new_default();
                let mut root = msg.init_root::<datagram_capnp::receive_transform::Builder>();

                root.set_player_id(*player_id);

                let mut translation = root.reborrow().init_translation();
                translation.set_x(transform.translation[0]);
                translation.set_y(transform.translation[1]);
                translation.set_z(transform.translation[2]);

                let mut rotation = root.init_rotation();
                rotation.set_x(transform.rotation[0]);
                rotation.set_y(transform.rotation[1]);
                rotation.set_z(transform.rotation[2]);
                rotation.set_w(transform.rotation[3]);

                capnp::serialize_packed::write_message(&mut buf, &msg)?;

                session.send_datagram(&buf).await?;
            }
        }
    };

    Ok(())
}
