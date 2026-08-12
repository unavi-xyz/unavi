//! The VUI gallery: a showcase consumer of `unavi:vui`.
//!
//! Everything here is what a script actually writes — a tree of motes, one
//! orbit over it, and a loop that reads back what happened. The drawing,
//! pointing, carrying and paging all happen behind the interface.
//!
//! Nothing here is handed to a space. A fruit put down is a document standing
//! on its own in the room, which is all a standalone harness can be: making it
//! everybody's would need a space to hand it to, and there is none here.

use unavi::vui::api::{
    self,
    Arrange,
    Event,
    Kind,
    Landing,
    Mote,
    Mount,
    Orbit,
};
use wired_prelude::prelude::*;

use crate::fruit::{
    Fruit,
    Shape,
    Throw,
    Variety,
    rgb,
};

mod fruit;

wired_prelude::generate_script!(Script);

/// Motes the orbit draws at once; anything past this paginates.
const CAPACITY: u32 = 16;
const MOUNT: Mount = Mount {
    distance: 1.1,
    height:   -0.15,
    offset:   Vec2::ZERO,
};

struct Script {
    orbit:  Orbit,
    /// Every fruit, so the one that was just let go can be grown.
    fruit:  Vec<Fruit>,
    /// Throws waiting on the bodies they belong to.
    thrown: Vec<Throw>,
}

fn mote(kind: Kind, label: &str, description: &str, children: Vec<Mote>) -> Mote {
    let mote = Mote::new(kind, label);
    if !description.is_empty() {
        mote.describe(description);
    }
    for child in &children {
        mote.add_child(child);
    }
    mote
}

/// A crate of them: dropping one leaves a new fruit and the crate is no
/// emptier.
const fn source(
    label: &'static str,
    description: &'static str,
    shape: Shape,
    color: Color,
    size: f32,
) -> Variety {
    Variety {
        label,
        description,
        shape,
        color,
        size,
        unique: false,
    }
}

/// The only one of it in the shop: dropping it puts *that* fruit down, and
/// picking it up again moves the same one.
const fn one_of(
    label: &'static str,
    description: &'static str,
    shape: Shape,
    color: Color,
    size: f32,
) -> Variety {
    Variety {
        unique: true,
        ..source(label, description, shape, color, size)
    }
}

const CITRUS: [Variety; 4] = [
    source(
        "Lemon",
        "Sharp and thin-skinned.",
        Shape::Round,
        rgb(0.95, 0.85, 0.25),
        0.055,
    ),
    source(
        "Lime",
        "Smaller, and greener.",
        Shape::Round,
        rgb(0.55, 0.80, 0.30),
        0.045,
    ),
    source(
        "Orange",
        "The one everybody pictures.",
        Shape::Round,
        rgb(0.95, 0.55, 0.15),
        0.060,
    ),
    one_of(
        "Grapefruit",
        "The last one, and it is enormous.",
        Shape::Round,
        rgb(0.95, 0.50, 0.45),
        0.075,
    ),
];

const BERRIES: [Variety; 3] = [
    source(
        "Strawberry",
        "Not botanically a berry.",
        Shape::Round,
        rgb(0.85, 0.20, 0.25),
        0.030,
    ),
    source(
        "Blueberry",
        "Actually a berry.",
        Shape::Round,
        rgb(0.30, 0.35, 0.70),
        0.020,
    ),
    one_of(
        "Raspberry",
        "Somebody has already taken the rest.",
        Shape::Round,
        rgb(0.75, 0.20, 0.40),
        0.025,
    ),
];

const TREE_FRUIT: [Variety; 3] = [
    source(
        "Pear",
        "Ripe for about an hour.",
        Shape::Long,
        rgb(0.75, 0.80, 0.40),
        0.050,
    ),
    source(
        "Quince",
        "Inedible raw, excellent cooked.",
        Shape::Cube,
        rgb(0.85, 0.75, 0.35),
        0.055,
    ),
    one_of(
        "Medlar",
        "There is one of these, and this is it.",
        Shape::Round,
        rgb(0.55, 0.42, 0.32),
        0.045,
    ),
];

fn grove(label: &str, description: &str, varieties: &[Variety], fruit: &mut Vec<Fruit>) -> Mote {
    let group = mote(Kind::Group, label, description, Vec::new());
    for variety in varieties {
        let grown = Fruit::grow(variety);
        group.add_child(&grown.mote);
        fruit.push(grown);
    }
    group
}

/// Deliberately uneven: group sizes differ, one group overflows the pip cap,
/// one outruns its grid and has to paginate, and the depth is unbounded.
fn produce(fruit: &mut Vec<Fruit>) -> Mote {
    let root = Mote::new(Kind::Group, "Produce");

    let orchard = grove(
        "Orchard",
        "Tree fruit, and the deepest level here.",
        &TREE_FRUIT,
        fruit,
    );

    // Stock, every one of it a source: the shop is not down to its last Gala.
    let apples = grove("Apples", "The same group, opened as a grid.", &[], fruit);
    apples.set_arrange(Arrange::Grid);
    for (variety, [r, g, b]) in VARIETIES {
        let grown = Fruit::grow(&source(variety, "", Shape::Cube, rgb(r, g, b), 0.055));
        apples.add_child(&grown.mote);
        fruit.push(grown);
    }
    orchard.add_child(&apples);

    for group in [
        grove("Citrus", "Sharp fruit with a thick rind.", &CITRUS, fruit),
        grove(
            "Berries",
            "Small, soft, and quick to spoil.",
            &BERRIES,
            fruit,
        ),
        orchard,
    ] {
        root.add_child(&group);
    }
    root
}

/// More than the grid's twelve cells, so it pages.
const VARIETIES: [(&str, [f32; 3]); 14] = [
    ("Gala", [0.86, 0.28, 0.22]),
    ("Fuji", [0.90, 0.42, 0.30]),
    ("Bramley", [0.55, 0.68, 0.28]),
    ("Pink Lady", [0.92, 0.45, 0.52]),
    ("Granny Smith", [0.48, 0.75, 0.32]),
    ("Braeburn", [0.78, 0.30, 0.20]),
    ("Cox", [0.82, 0.52, 0.24]),
    ("Discovery", [0.88, 0.34, 0.30]),
    ("Egremont", [0.72, 0.55, 0.30]),
    ("Worcester", [0.85, 0.26, 0.28]),
    ("Katy", [0.90, 0.36, 0.34]),
    ("Spartan", [0.62, 0.20, 0.26]),
    ("Russet", [0.66, 0.50, 0.32]),
    ("Ambrosia", [0.92, 0.55, 0.40]),
];

fn report(event: &Event) {
    let what = match event {
        Event::Opened(mote) => format!("opened {}", mote.label()),
        Event::Closed(mote) => format!("back to {}", mote.label()),
        Event::Activated(mote) => format!("activated {}", mote.label()),
        Event::Casting(mote) => format!("casting {}", mote.label()),
        Event::Cast(mote) => format!("cast {}", mote.label()),
        Event::Aborted(mote) => format!("aborted {}", mote.label()),
        Event::Planted((mote, _)) => format!("dropped {}", mote.label()),
        Event::Filed(mote) => format!("filed {}", mote.label()),
        Event::Paged(page) => format!("page {} of {}", page.index + 1, page.count),
    };
    println!("{what}");
}

impl Script {
    /// Puts a fruit where its mote was let go. The mote stays where it is
    /// either way; whether that is the same fruit moving or another one is
    /// the fruit's own business.
    fn deliver(&mut self, mote: &Mote, landing: Landing) {
        let Some(fruit) = self.fruit.iter_mut().find(|fruit| fruit.mote.is(mote)) else {
            return;
        };
        match fruit.deliver(landing) {
            Ok(throw) => self.thrown.push(throw),
            Err(err) => eprintln!("could not put down '{}': {err}", mote.label()),
        }
    }
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        println!("vui gallery — click to open, drag an item out to drop a fruit");
        println!("  warm motes are sources: every drop is another of that fruit");
        println!("  cool ones there is only one of, and dropping one moves it");

        let mut fruit = Vec::new();
        let orbit = Orbit::new(&produce(&mut fruit), MOUNT, CAPACITY)?;

        Ok(Self {
            orbit,
            fruit,
            thrown: Vec::new(),
        })
    }

    fn fixed_update(&mut self) -> anyhow::Result<()> {
        api::fixed_update()?;
        Ok(())
    }

    fn update(&mut self) -> anyhow::Result<()> {
        api::update()?;
        self.thrown.retain_mut(Throw::apply);

        for event in self.orbit.events() {
            report(&event);
            if let Event::Planted((mote, landing)) = &event {
                self.deliver(mote, *landing);
            }
        }
        Ok(())
    }
}
