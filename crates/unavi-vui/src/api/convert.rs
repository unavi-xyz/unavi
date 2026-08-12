use crate::{
    api::mote::handle,
    exports::unavi::vui::api::{
        Event,
        Landing,
        Mount,
        Page,
    },
    scene::{
        event as scene,
        mount,
    },
};

pub const fn mount(mount: Mount) -> mount::Mount {
    mount::Mount::ahead(mount.distance, mount.height).beside(mount.offset)
}

pub fn event(event: scene::Event) -> Event {
    match event {
        scene::Event::Opened(mote) => Event::Opened(handle(mote)),
        scene::Event::Closed(mote) => Event::Closed(handle(mote)),
        scene::Event::Activated(mote) => Event::Activated(handle(mote)),
        scene::Event::Casting(mote) => Event::Casting(handle(mote)),
        scene::Event::Cast(mote) => Event::Cast(handle(mote)),
        scene::Event::Aborted(mote) => Event::Aborted(handle(mote)),
        scene::Event::Planted(mote, landing) => Event::Planted((
            handle(mote),
            Landing {
                at:       landing.at,
                velocity: landing.velocity,
            },
        )),
        scene::Event::Filed(mote) => Event::Filed(handle(mote)),
        scene::Event::Paged {
            index,
            count,
            total,
        } => Event::Paged(Page {
            index: index as u32,
            count: count as u32,
            total: total as u32,
        }),
    }
}
