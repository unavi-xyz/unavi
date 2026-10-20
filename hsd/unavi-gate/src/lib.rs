//! A gate: a portal between white pillars, with a socket sunk into the black
//! stone beside it.
//!
//! A beacon seated in the socket opens the portal to the beacon's space, and
//! taking it out closes it. Every peer decides the seat from where the beacon
//! is, so every peer opens and closes together. A gate the far side paired
//! shows a ghost beacon in its socket, and pulling the ghost out unplugs the
//! pairing.

use wired_guest::{
    beacon::{
        ANNOUNCE_TICKS,
        BEACON_SIZE,
        DocumentId,
        PULL_RADIUS,
        SOCKET_CHANNEL,
        Socket,
    },
    color::{
        document_seed,
        generate_color,
    },
    math::{
        Color,
        Vec3,
    },
};

use crate::{
    beacons::Beacons,
    link::Link,
    look::{
        Look,
        Mood,
    },
    seat::{
        Occupant,
        Seat,
    },
    unavi::shapes::api::Cuboid,
    wired::{
        core::ids::PrimRef,
        event::messaging::{
            self,
            Scope,
            Spatial,
        },
        input::{
            targeted,
            types::{
                Action,
                Button,
                InputSubscription,
            },
        },
        scene::{
            document::{
                Document,
                script_document,
            },
            properties::{
                Portal,
                Property,
                PropertyKey,
                RigidBody,
                RigidBodyKind,
            },
        },
    },
};

mod beacons;
mod frame;
mod link;
mod look;
mod seat;

wired_guest::generate_script!(Script);

const PORTAL_PRIM_NAME: &str = "portal";
const SOCKET_PRIM_NAME: &str = "socket";
/// Metres from the seat within which a beacon makes the ghost untouchable,
/// so its collider never stands in a beacon's way.
const GHOST_CLEAR: f32 = 0.4;
/// Kilograms the ghost weighs while it can be taken hold of. Light, so a hand
/// carrying it does not shove the gate's stone aside.
const GHOST_MASS: f32 = 0.05;
/// Per second, how hard a let-go ghost is slowed, so it hangs where it was
/// left instead of drifting.
const GHOST_DAMPING: f32 = 12.0;
/// Seconds a taken ghost stays whole in a hand before it starts to dissolve.
/// Long enough to carry it clear of the socket and feel the pull let go.
const GHOST_HOLD_SECS: f32 = 1.0;

struct Script {
    doc:         Document,
    socket:      (u64, u64),
    ghost:       (u64, u64),
    ghost_rx:    InputSubscription,
    /// Whether the ghost carries a collider and a body now.
    bodied:      bool,
    /// Whether a hand is carrying the ghost.
    held:        bool,
    /// Seconds the ghost has been in a hand since it was pulled.
    held_for:    f32,
    beacons:     Beacons,
    seat:        Seat,
    link:        Link,
    look:        Look,
    announce_in: u32,
    /// What the socket shows, decided on fixed ticks and drawn every frame.
    mood:        Mood,
}

fn space_color(space: DocumentId) -> Color {
    generate_color(document_seed(space))
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        let doc = script_document()?;
        let root = doc
            .roots()
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("gate has no root prim"))?;

        // Authored prims have ids baked at build time, identical on every
        // peer; a prim minted at runtime gets a per-peer id.
        let portal = find_one(&doc, PORTAL_PRIM_NAME)?;
        let socket = find_one(&doc, SOCKET_PRIM_NAME)?;

        doc.local()
            .set(
                portal,
                Property::Portal(Portal {
                    width:       frame::PORTAL_WIDTH,
                    height:      frame::PORTAL_HEIGHT,
                    destination: None,
                }),
            )
            .flush()?;
        let seat = match doc.get(socket, &PropertyKey::Transform) {
            Some(Property::Transform(transform)) => transform.translation,
            _ => Vec3::ZERO,
        };
        let parts = frame::build(&doc, root, socket, seat)?;
        let ghost = parts.ghost;
        let ghost_rx = targeted::listen(&doc, ghost)?;

        let beacons = Beacons::listen(PrimRef {
            document: doc.id(),
            prim:     socket,
        })?;

        Ok(Self {
            doc,
            socket,
            ghost,
            ghost_rx,
            bodied: false,
            held: false,
            held_for: 0.0,
            beacons,
            seat: Seat::default(),
            link: Link::new(portal)?,
            look: Look::new(parts),
            announce_in: 0,
            mood: Mood::Idle,
        })
    }

    fn fixed_update(&mut self, tick: Tick) -> anyhow::Result<()> {
        let Ok(seat_at) = self.doc.world_transform(self.socket) else {
            return Ok(());
        };
        let near = self.beacons.measure(seat_at.translation)?;
        let occupant = self.seat.update(&near);
        self.link.update(&self.doc, occupant);
        self.take_ghost(&near, tick.dt)?;
        self.announce(occupant)?;

        self.mood = if let Some(occupant) = occupant {
            Mood::Seated(space_color(occupant.space))
        } else if let Some(leads) = self.link.leads() {
            Mood::Ghost {
                space: leads,
                color: space_color(leads),
            }
        } else if let Some(nearest) = beacons::nearest(&near, None) {
            Mood::Near {
                color:     space_color(nearest.space),
                closeness: 1.0 - nearest.distance / PULL_RADIUS,
            }
        } else {
            Mood::Idle
        };
        Ok(())
    }

    fn update(&mut self, tick: Tick) -> anyhow::Result<()> {
        self.look.update(&self.doc, self.mood, tick.dt)
    }
}

impl Script {
    /// Gives the ghost a body while it can be taken hold of, and unplugs the
    /// pairing when someone takes it. A taken ghost stays whole in the hand
    /// for [`GHOST_HOLD_SECS`] before it starts to dissolve, so pulling it out
    /// is a thing done rather than a thing that happens.
    fn take_ghost(&mut self, near: &[seat::Candidate], dt: f32) -> anyhow::Result<()> {
        for event in self.ghost_rx.drain(8) {
            match event.action {
                Action::Pressed(Button::Grip) if self.bodied => {
                    if !self.held {
                        self.held = true;
                        self.held_for = 0.0;
                        self.unplug(true)?;
                    }
                }
                Action::Pressed(Button::Trigger) if self.bodied => self.unplug(false)?,
                Action::Released(Button::Grip) if self.held => {
                    self.held = false;
                    self.look.release();
                }
                _ => {}
            }
        }
        if self.held {
            self.held_for += dt;
            if self.held_for >= GHOST_HOLD_SECS {
                self.held = false;
                self.look.release();
            }
        }

        let clear = near.iter().all(|c| c.distance > GHOST_CLEAR);
        let bodied =
            (clear && self.look.ghost_touchable()) || (self.held && self.look.ghost_visible());
        if !bodied {
            self.held = false;
        }
        if bodied != self.bodied {
            self.bodied = bodied;
            let batch = self.doc.local();
            let batch = if bodied {
                let shape = Cuboid::new(Vec3::splat(BEACON_SIZE));
                batch
                    .set(self.ghost, Property::Collider(shape.collider()))
                    .set(
                        self.ghost,
                        Property::RigidBody(RigidBody {
                            kind:            RigidBodyKind::Dynamic,
                            mass:            Some(GHOST_MASS),
                            friction:        Some(0.0),
                            restitution:     Some(0.0),
                            linear_damping:  Some(GHOST_DAMPING),
                            angular_damping: Some(GHOST_DAMPING),
                        }),
                    )
                    .set(self.ghost, Property::GravityScale(0.0))
            } else {
                batch
                    .clear(self.ghost, PropertyKey::RigidBody)
                    .clear(self.ghost, PropertyKey::Collider)
                    .clear(self.ghost, PropertyKey::GravityScale)
            };
            batch.flush()?;
        }
        Ok(())
    }

    /// States the pairing as unplugged and starts its ghost on its way out.
    /// `grabbed` keeps the ghost whole until the hand lets go.
    fn unplug(&mut self, grabbed: bool) -> anyhow::Result<()> {
        if self.look.is_unplugged() {
            return Ok(());
        }
        let space = self.link.leads();
        self.link.unplug(&self.doc)?;
        if let Some(space) = space {
            self.look.unplug(space, grabbed);
        }
        Ok(())
    }

    fn announce(&mut self, occupant: Option<Occupant>) -> anyhow::Result<()> {
        if self.announce_in > 0 {
            self.announce_in -= 1;
            return Ok(());
        }
        self.announce_in = ANNOUNCE_TICKS;
        let socket = Socket {
            seat:     self.socket,
            backing:  frame::backing(),
            occupant: occupant.map(|o| o.beacon),
            leads:    self.link.leads(),
        };
        messaging::emit(
            SOCKET_CHANNEL,
            &socket.encode()?,
            None,
            Scope::Spatial(Spatial {
                origin: PrimRef {
                    document: self.doc.id(),
                    prim:     self.socket,
                },
                radius: PULL_RADIUS,
            }),
        )?;
        Ok(())
    }
}
