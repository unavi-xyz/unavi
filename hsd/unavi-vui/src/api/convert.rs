//! Converts between `unavi:vui`'s WIT-generated types and this crate's own.

use crate::{
    api::mote::handle,
    exports::unavi::vui::api::{
        Bearing,
        Event,
        Landing,
        Mount,
        Page,
        Planted,
    },
    render::{
        event,
        mount,
    },
};

pub const fn mount(mount: Mount) -> mount::Mount {
    mount::Mount {
        bearing: match mount.bearing {
            Bearing::Level => mount::Bearing::Level,
            Bearing::Sight => mount::Bearing::Sight,
        },
        ..mount::Mount::ahead(mount.distance, mount.height).beside(mount.offset)
    }
}

pub fn event(event: event::Event) -> Event {
    match event {
        event::Event::Opened(mote) => Event::Opened(handle(mote)),
        event::Event::Closed(mote) => Event::Closed(handle(mote)),
        event::Event::Activated(mote) => Event::Activated(handle(mote)),
        event::Event::Casting(mote) => Event::Casting(handle(mote)),
        event::Event::Cast(mote) => Event::Cast(handle(mote)),
        event::Event::Aborted(mote) => Event::Aborted(handle(mote)),
        event::Event::Planted(mote, landing) => Event::Planted(Planted {
            mote:    handle(mote),
            landing: Landing {
                at:       landing.at,
                velocity: landing.velocity,
            },
        }),
        event::Event::Filed(mote) => Event::Filed(handle(mote)),
        event::Event::Paged {
            index,
            count,
            total,
        } => Event::Paged(Page {
            index: index as u32,
            pages: count as u32,
            motes: total as u32,
        }),
    }
}
