//! A beacon: a small glowing cube planted by halo's `Nav`, standing for one
//! space. It announces that space on `BEACON_CHANNEL`, and a gate whose
//! socket it sits in opens to it. Let go near a socket, it is drawn in.
//!
//! The authored template prim does nothing. A real beacon is a document of
//! its own, minted by copying the template and naming a prim for its space.

use std::str::FromStr;

use blake3::Hash;
use wired_guest::{
    beacon::{
        ANNOUNCE_TICKS,
        Announce,
        BEACON_CHANNEL,
        BEACON_SIZE,
        DocumentId,
        PLANTED_KEY,
        PULL_RADIUS,
        PrimId,
        age_secs,
        now_ms,
    },
    color::generate_color,
    math::{
        Transform,
        Vec3,
    },
};

use crate::{
    deploy::Deploy,
    float::Float,
    shell::{
        Mood,
        Shell,
        make_body,
        prim_name,
    },
    sockets::{
        Nearby,
        Sockets,
    },
    wired::{
        core::{
            error::Error,
            ids::PrimRef,
        },
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
        peer::authority::{
            is_author,
            is_holder,
        },
        physics::simulation::{
            BodyVelocity,
            set_velocity,
            velocity,
        },
        scene::{
            document::{
                Document,
                script_document,
            },
            properties::{
                Property,
                PropertyKey,
            },
        },
    },
};

mod deploy;
mod float;
mod pull;
mod shell;
mod sockets;

wired_guest::generate_script!(Script);

/// Ticks between looks for the space-named prim while none is found.
const FIND_INTERVAL: u32 = 30;

/// No space-named prim means the authored template, which does nothing, or a
/// minted beacon whose committed prim has not synced to this peer yet, which
/// is looked for again every [`FIND_INTERVAL`] ticks.
struct Script {
    beacon: Option<Beacon>,
    waited: u32,
}

struct Beacon {
    doc:           Document,
    /// The space-named prim, which carries the collider and rigid body.
    body:          (u64, u64),
    space:         Hash,
    shell:         Shell,
    input:         InputSubscription,
    sockets:       Sockets,
    float:         Float,
    /// The body's pose when it was let go, which the hover moves from.
    rest:          Option<Transform>,
    /// Whether this peer's grip is on it.
    held:          bool,
    /// Seconds since this peer's grip last let it go.
    since_release: Option<f32>,
    /// Whether a socket is drawing it in, with gravity off.
    pulling:       bool,
    /// The nearest a pull has brought it to its seat, and seconds since it
    /// last came closer.
    progress:      (f32, f32),
    /// Whether a stalled pull has given up, until the next release.
    blocked:       bool,
    /// The socket this beacon sits in, as last measured.
    seated_in:     Option<(DocumentId, PrimId)>,
    /// The socket it was taken out of, which does not draw it back until it
    /// has been carried out of that socket's reach.
    left:          Option<(DocumentId, PrimId)>,
    /// What the core shows, decided on fixed ticks and drawn every frame.
    mood:          Mood,
    /// Its fold-out gate, when its document carries one.
    deploy:        Option<Deploy>,
    announce_in:   u32,
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        Ok(Self {
            beacon: Beacon::find()?,
            waited: 0,
        })
    }

    fn fixed_update(&mut self, tick: Tick) -> anyhow::Result<()> {
        if self.beacon.is_none() {
            self.waited += 1;
            if self.waited < FIND_INTERVAL {
                return Ok(());
            }
            self.waited = 0;
            self.beacon = Beacon::find()?;
        }
        self.beacon
            .as_mut()
            .map_or_else(|| Ok(()), |beacon| beacon.fixed_update(tick.dt))
    }

    fn update(&mut self, tick: Tick) -> anyhow::Result<()> {
        self.beacon
            .as_mut()
            .map_or_else(|| Ok(()), |beacon| beacon.animate(tick.dt))
    }
}

/// The space a registry hex names, as a `document-id`'s words.
fn space_id(space: Hash) -> (u64, u64, u64, u64) {
    let bytes = space.as_bytes();
    let word = |at: usize| {
        let mut word = [0; 8];
        word.copy_from_slice(&bytes[at..at + 8]);
        u64::from_le_bytes(word)
    };
    (word(0), word(8), word(16), word(24))
}

impl Beacon {
    /// The beacon this script's document is, once its space-named prim is
    /// present.
    fn find() -> anyhow::Result<Option<Self>> {
        let doc = script_document()?;

        let Some((space, body)) = doc
            .prims()
            .into_iter()
            .find_map(|p| Hash::from_str(&prim_name(&doc, p)?).ok().map(|id| (id, p)))
        else {
            return Ok(None);
        };

        let rest = match doc.get(body, &PropertyKey::Transform) {
            Some(Property::Transform(transform)) => Some(transform),
            _ => None,
        };
        // The age comes from a stamp every peer reads, so a late joiner
        // finds the hover where it is, and a beacon that has fallen does not
        // float again. A beacon with no stamp has nothing to hover from.
        let float = match doc.get(body, &PropertyKey::Custom(PLANTED_KEY.to_owned())) {
            Some(Property::Custom((_, stamp))) => {
                age_secs(&stamp, now_ms()).map_or_else(Float::over, Float::at_age)
            }
            _ => Float::over(),
        };
        make_body(&doc, body, float.is_floating())?;
        let shell = Shell::under(&doc, body, space)?;
        let input = targeted::listen(&doc, body)?;
        let sockets = Sockets::listen(PrimRef {
            document: doc.id(),
            prim:     body,
        })?;
        let deploy = Deploy::find(&doc, space_id(space), generate_color(space));
        println!("Beacon initialized: space={space}");
        Ok(Some(Self {
            doc,
            body,
            space,
            shell,
            input,
            sockets,
            float,
            rest,
            held: false,
            since_release: None,
            pulling: false,
            progress: (f32::INFINITY, 0.0),
            blocked: false,
            seated_in: None,
            left: None,
            mood: Mood::Resting,
            deploy,
            announce_in: 0,
        }))
    }

    fn fixed_update(&mut self, dt: f32) -> anyhow::Result<()> {
        for event in self.input.drain(8) {
            match event.action {
                Action::Pressed(Button::Grip) => {
                    self.held = true;
                    self.left = self.seated_in;
                }
                Action::Released(Button::Grip) => {
                    self.held = false;
                    self.since_release = Some(0.0);
                    self.blocked = false;
                    self.face_thrower()?;
                }
                Action::Pressed(Button::Trigger) if self.held => self.toggle_armed()?,
                _ => {}
            }
        }
        if let Some(since) = &mut self.since_release {
            *since += dt;
        }

        let Ok(now) = self.doc.world_transform(self.body) else {
            return Ok(());
        };
        let me = self.doc.id();
        let sockets_near = self.sockets.measure(now.translation)?;
        self.seated_in = sockets_near
            .iter()
            .find(|socket| socket.occupant == Some(me) && socket.distance < BEACON_SIZE)
            .map(|socket| socket.key);
        let seated = self.seated_in.is_some();
        if let Some(left) = self.left
            && !sockets_near
                .iter()
                .any(|socket| socket.key == left && socket.distance < PULL_RADIUS)
        {
            self.left = None;
        }
        let drawing = self.drawing_socket(&sockets_near, now.translation)?;

        if self.float.step(dt, self.held) || (drawing.is_some() && self.float.is_floating()) {
            self.float.end();
            self.set_gravity(drawing.is_none())?;
        }

        if let Some((socket, moving)) = drawing {
            self.pull(now, socket, moving, dt)?;
        } else if self.pulling {
            self.pulling = false;
            self.set_gravity(true)?;
        } else if self.float.is_floating() {
            self.hover(now)?;
        }

        let armed = deploy::armed(&self.doc, self.body);
        self.shell.set_armed(&self.doc, armed.is_some())?;
        let socket_close = sockets_near
            .iter()
            .any(|socket| socket.distance < pull::NEAR_RADIUS);
        let unfolded = match &mut self.deploy {
            Some(deploy) => {
                if deploy.update(&self.doc, now.translation, armed, socket_close)?
                    && is_author(&self.doc)?
                {
                    deploy::arm(&self.doc, self.body, None)?;
                }
                deploy.is_unfolded()
            }
            None => false,
        };

        let near_free = sockets_near
            .iter()
            .any(|socket| socket.takes(me) && socket.distance < PULL_RADIUS);
        self.mood = if seated || unfolded {
            Mood::Seated
        } else if self.held && near_free {
            Mood::Eager
        } else {
            Mood::Resting
        };
        self.announce()
    }

    /// Draws the core and any unfolded gate by `dt` seconds.
    fn animate(&mut self, dt: f32) -> anyhow::Result<()> {
        self.shell.update(&self.doc, self.mood, dt)?;
        if let Some(deploy) = &mut self.deploy {
            deploy.animate(&self.doc, dt)?;
        }
        Ok(())
    }

    /// The socket drawing this beacon in this tick, and how the body moves now,
    /// on the peer deciding its motion. A grip on it draws nothing, so a hand
    /// is never fought. A peer that cannot steer the body draws nothing, and
    /// the beacon seats only by falling into the cradle.
    fn drawing_socket(
        &self,
        sockets_near: &[Nearby],
        at: Vec3,
    ) -> anyhow::Result<Option<(Nearby, BodyVelocity)>> {
        if self.held || self.blocked {
            return Ok(None);
        }
        let thrown = self
            .since_release
            .is_some_and(|since| since < pull::THROW_WINDOW);
        let reach = if thrown || self.pulling {
            PULL_RADIUS
        } else {
            pull::NEAR_RADIUS
        };
        // A beacon its socket already counts as seated rests in it, and is
        // not lifted to be squared up again.
        let Some(socket) = sockets_near
            .iter()
            .filter(|socket| socket.occupant.is_none() && Some(socket.key) != self.left)
            .find(|socket| {
                socket.distance < reach && !pull::at_rest(at, socket.seat.translation, 0.0)
            })
            .copied()
        else {
            return Ok(None);
        };
        if !is_holder(&self.doc)? {
            return Ok(None);
        }
        match velocity(&self.doc, self.body) {
            Ok(moving) => Ok(Some((socket, moving))),
            Err(Error::NotReady | Error::Permission(_)) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    /// Steers the body into the socket's seat, and gives gravity back once it
    /// rests there, so the cradle holds it on every peer.
    fn pull(
        &mut self,
        now: Transform,
        socket: Nearby,
        moving: BodyVelocity,
        dt: f32,
    ) -> anyhow::Result<()> {
        let seat = socket.seat;
        if !self.pulling {
            self.pulling = true;
            self.progress = (f32::INFINITY, 0.0);
            self.set_gravity(false)?;
        }
        let distance = now.translation.distance(seat.translation);
        if distance < self.progress.0 - pull::PROGRESS {
            self.progress = (distance, 0.0);
        } else {
            self.progress.1 += dt;
        }
        if self.progress.1 > pull::STALL_SECS {
            // Caught on something, such as the cradle's underside: let it fall.
            self.pulling = false;
            self.blocked = true;
            return self.set_gravity(true);
        }
        if pull::at_rest(now.translation, seat.translation, moving.linear.length()) {
            self.pulling = false;
            self.since_release = None;
            self.steer(Vec3::ZERO, Vec3::ZERO)?;
            return self.set_gravity(true);
        }
        let drive = pull::drive(now, seat, socket.backing, moving.linear, dt);
        self.steer(drive.linear, drive.angular)
    }

    /// Sets the body's velocities. The body enters the simulation a tick
    /// after it is made, and a peer whose trust in the author grants no
    /// physics leaves the motion to the holder.
    fn steer(&self, linear: Vec3, angular: Vec3) -> anyhow::Result<()> {
        match set_velocity(&self.doc, self.body, Some(linear), Some(angular)) {
            Ok(()) | Err(Error::NotReady | Error::Permission(_)) => Ok(()),
            Err(err) => Err(err.into()),
        }
    }

    /// Holds the body on its hover path. Every peer does the same, since
    /// every peer's simulation steps the body; the holder's motion is the
    /// one that wins, and it is this same path.
    fn hover(&self, now: Transform) -> anyhow::Result<()> {
        let Some(rest) = self.rest else {
            return Ok(());
        };
        let drive = self.float.drive(rest, now);
        self.steer(drive.linear, drive.angular)
    }

    fn set_gravity(&self, on: bool) -> anyhow::Result<()> {
        self.doc
            .local()
            .set(
                self.body,
                Property::GravityScale(if on { 1.0 } else { 0.0 }),
            )
            .flush()?;
        Ok(())
    }

    /// Arms or disarms a beacon held by its author, facing its gate toward
    /// them. Anyone else's trigger does nothing.
    fn toggle_armed(&self) -> anyhow::Result<()> {
        if self.deploy.is_none() || !is_author(&self.doc)? {
            return Ok(());
        }
        let yaw = match deploy::armed(&self.doc, self.body) {
            Some(_) => None,
            None => Some(camera_pose().map_or(0.0, |eye| deploy::facing(eye.forward()))),
        };
        deploy::arm(&self.doc, self.body, yaw)
    }

    /// Turns an armed beacon's gate toward its author as they let it go.
    fn face_thrower(&self) -> anyhow::Result<()> {
        if deploy::armed(&self.doc, self.body).is_none() || !is_author(&self.doc)? {
            return Ok(());
        }
        let Some(eye) = camera_pose() else {
            return Ok(());
        };
        deploy::arm(&self.doc, self.body, Some(deploy::facing(eye.forward())))
    }

    /// Every peer's copy announces to the scripts beside it, since messages
    /// stay on the peer that sent them.
    fn announce(&mut self) -> anyhow::Result<()> {
        if self.announce_in > 0 {
            self.announce_in -= 1;
            return Ok(());
        }
        self.announce_in = ANNOUNCE_TICKS;
        let announce = Announce {
            space: space_id(self.space),
            body:  self.body,
            held:  self.held,
        };
        messaging::emit(
            BEACON_CHANNEL,
            &announce.encode()?,
            None,
            Scope::Spatial(Spatial {
                origin: PrimRef {
                    document: self.doc.id(),
                    prim:     self.body,
                },
                radius: BEACON_SIZE,
            }),
        )?;
        Ok(())
    }
}
