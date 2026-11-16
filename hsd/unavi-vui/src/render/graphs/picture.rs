//! A picture drawn in cells, from an image with one texel per cell.
//!
//! The image bound at texture slot 0 is the picture to show and the one at
//! slot 1 the picture it replaces. A change travels: each cell turns from the
//! old texel to the new as the front reaches it, and flashes while it does.

use wired_guest::{
    math::Vec2,
    style,
};

use super::{
    Net,
    binary,
    cell::{
        CELL_OWN,
        Look,
        look_defaults,
    },
    cf,
};
use crate::wired::{
    scene::document::{
        Document,
        Layer,
    },
    shading::graph::{
        BlendMode,
        CullMode,
        ExtractOp,
        GraphValue,
        LerpOp,
        Node,
        Port,
        ShaderGraph,
        StepOp,
        SurfaceGraph,
        SurfaceOutput,
        TextureSampleOp,
        UnlitOutput,
        set_graph,
    },
};

/// Cells across and down.
pub const PICTURE_SIZE: u16 = CELL_OWN;
/// Where the change began, in cells from the top left.
pub const PICTURE_ORIGIN: u16 = CELL_OWN + 1;
/// Seconds since the change began.
pub const PICTURE_AGE: u16 = CELL_OWN + 2;

/// The age of a change that every front has long crossed.
pub const SETTLED: f32 = 1.0e4;

/// A texel at or above this reads as ember. Between the off byte and the
/// ember one in both encodings: raw, ember is 0.50, and decoded from sRGB it
/// is 0.22.
const READ_EMBER: f32 = 0.1;
/// A texel at or above this reads as lit, past the ember byte in both
/// encodings.
const READ_LIT: f32 = 0.85;

impl Net {
    /// The first channel of the texel the cell at `uv` holds, at `slot`.
    fn texel(&mut self, uv: Port, slot: u8) -> Port {
        let colour = self.push(Node::TextureSample(TextureSampleOp { uv, slot }));
        self.push(Node::Extract(ExtractOp {
            v:       colour,
            channel: 0,
        }))
    }
}

/// A picture of one-texel cells: three levels, square, wearing the look
/// inputs. Blended, so the cells darken the glass behind them.
pub fn picture() -> ShaderGraph {
    let mut net = Net::default();
    let size = Port::Input(PICTURE_SIZE);

    let uv = net.push(Node::Uv);
    let scaled = net.push(Node::Mul(binary(uv, size)));
    let at = net.push(Node::Floor(scaled));
    let within = net.push(Node::Sub(binary(scaled, at)));
    let centre = net.push(Node::Add(binary(at, cf(0.5))));
    // Sampling at a cell's centre reads its own texel whatever the filter.
    let texel_uv = net.push(Node::Div(binary(centre, size)));
    let new = net.texel(texel_uv, 0);
    let old = net.texel(texel_uv, 1);

    // The front reaches a cell at its distance from the origin over the
    // travel speed. Before it, the old texel shows.
    let apart = net.push(Node::Distance(binary(centre, Port::Input(PICTURE_ORIGIN))));
    let reach = net.push(Node::Div(binary(apart, cf(style::CELLS_TRAVEL))));
    let since = net.push(Node::Sub(binary(Port::Input(PICTURE_AGE), reach)));
    let arrived = net.push(Node::Step(StepOp {
        edge: cf(0.0),
        x:    since,
    }));
    let texel = net.push(Node::Lerp(LerpOp {
        a: old,
        b: new,
        t: arrived,
    }));

    let ember = net.push(Node::Step(StepOp {
        edge: cf(READ_EMBER),
        x:    texel,
    }));
    let lit = net.push(Node::Step(StepOp {
        edge: cf(READ_LIT),
        x:    texel,
    }));
    let level = net.level(ember, lit);
    let off = net.push(Node::OneMinus(ember));
    let level = net.breathe(level, at, off);
    let level = net.wake(level, centre);

    // A cell flashes as the front reaches it and settles over the front
    // time.
    let ageing = net.push(Node::Div(binary(since, cf(style::CELLS_FRONT))));
    let ageing = net.push(Node::Saturate(ageing));
    let fresh = net.push(Node::OneMinus(ageing));
    let flash = net.push(Node::Mul(binary(fresh, arrived)));
    let flash = net.push(Node::Mul(binary(flash, cf(style::CELLS_FRONT_FLASH))));
    let level = net.push(Node::Max(binary(level, flash)));

    let shape = net.square(within);
    let color = net.paint(level, shape);

    let mut public_inputs = look_defaults();
    public_inputs.extend([
        GraphValue::Vec2(Vec2::new(
            style::CELLS_GLYPH_WIDTH as f32,
            style::CELLS_GLYPH_WIDTH as f32,
        )),
        GraphValue::Vec2(Vec2::ZERO),
        GraphValue::Float(SETTLED),
    ]);

    ShaderGraph {
        public_inputs,
        surface: SurfaceGraph {
            nodes:        net.nodes,
            output:       SurfaceOutput::Unlit(UnlitOutput {
                color,
                alpha_clip_threshold: None,
            }),
            blend:        BlendMode::Blend,
            cull:         CullMode::None,
            cast_shadows: false,
        },
        displacement: None,
    }
}

/// Submits [`picture`] to `prim`.
pub fn set_picture(doc: &Document, prim: (u64, u64)) -> anyhow::Result<()> {
    set_graph(doc, prim, Layer::Local, Some(&picture()))?;
    Ok(())
}

/// Everything [`picture`] is handed beyond its two images.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Inputs {
    pub look:   Look,
    /// Cells across and down.
    pub size:   (usize, usize),
    pub origin: Vec2,
    /// Seconds since the change began, [`SETTLED`] once it is over.
    pub age:    f32,
}

impl Inputs {
    /// The override list for `set_overrides`, by input index.
    #[must_use]
    pub fn overrides(&self) -> Vec<(u16, GraphValue)> {
        let mut values = self.look.overrides();
        values.extend([
            (
                PICTURE_SIZE,
                GraphValue::Vec2(Vec2::new(self.size.0 as f32, self.size.1 as f32)),
            ),
            (PICTURE_ORIGIN, GraphValue::Vec2(self.origin)),
            (PICTURE_AGE, GraphValue::Float(self.age)),
        ]);
        values
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{
            cell::{
                CELL_ATTEND,
                CELL_FLAT,
                CELL_POINTER,
            },
            interp::{
                self,
                Context,
            },
        },
        *,
    };
    use crate::cells::{
        Grid,
        Level,
    };

    /// How the host might hand back a stored byte: as it is, or decoded from
    /// sRGB.
    #[derive(Clone, Copy)]
    enum Encoding {
        Raw,
        Srgb,
    }

    fn decode(byte: u8, encoding: Encoding) -> f32 {
        let value = f32::from(byte) / 255.0;
        match encoding {
            Encoding::Raw => value,
            Encoding::Srgb if value <= 0.04045 => value / 12.92,
            Encoding::Srgb => ((value + 0.055) / 1.055).powf(2.4),
        }
    }

    fn grid(width: usize, height: usize, level: impl Fn(usize, usize) -> Level) -> Grid {
        let levels = (0..width * height)
            .map(|cell| level(cell % width, cell / width))
            .collect();
        Grid::new(width, height, levels).expect("a grid")
    }

    fn flat(level: Level) -> Grid {
        grid(12, 12, |_, _| level)
    }

    /// The picture graph run over two grids: the one shown and the one it
    /// replaces.
    struct Probe<'a> {
        graph:    ShaderGraph,
        now:      &'a Grid,
        before:   &'a Grid,
        encoding: Encoding,
        values:   Vec<GraphValue>,
        time:     f32,
    }

    impl<'a> Probe<'a> {
        fn new(now: &'a Grid, before: &'a Grid) -> Self {
            let graph = picture();
            Self {
                values: graph.public_inputs.clone(),
                graph,
                now,
                before,
                encoding: Encoding::Raw,
                time: 0.0,
            }
        }

        fn set(&mut self, overrides: Vec<(u16, GraphValue)>) {
            for (input, value) in overrides {
                self.values[usize::from(input)] = value;
            }
        }

        /// The fragment's colour and alpha at `across` cells right and `down`
        /// cells below the centre of `column`, `row`.
        fn read(&self, column: usize, row: usize, across: f32, down: f32) -> [f32; 4] {
            let size = self.now.width() as f32;
            let at = [
                (column as f32 + 0.5 + across) / size,
                (row as f32 + 0.5 + down) / size,
            ];
            let texel = |slot: u8, uv: [f32; 2]| {
                let grid = if slot == 0 { self.now } else { self.before };
                let column = (uv[0] * size).floor() as usize;
                let row = (uv[1] * size).floor() as usize;
                let value = decode(grid.level(column, row).texel(), self.encoding);
                [value, value, value, 1.0]
            };
            let context = Context {
                uv:     at,
                time:   self.time,
                scale:  [1.0; 3],
                screen: [0.5; 2],
                scene:  [0.0; 4],
                texel:  &texel,
            };
            interp::shade(&self.graph, &self.values, &context).color.c
        }

        fn alpha(&self, column: usize, row: usize) -> f32 {
            self.read(column, row, 0.0, 0.0)[3]
        }
    }

    #[test]
    fn the_picture_is_well_formed_and_blends_over_the_glass() {
        let graph = picture();
        assert!(matches!(graph.surface.blend, BlendMode::Blend));
        assert!(matches!(graph.surface.cull, CullMode::None));
        let SurfaceOutput::Unlit(unlit) = graph.surface.output else {
            panic!("cells are unlit");
        };
        assert!(unlit.alpha_clip_threshold.is_none());
    }

    #[test]
    fn a_cell_reads_off_ember_or_lit_in_either_texture_encoding() {
        let picture = grid(12, 12, |column, _| match column % 3 {
            0 => Level::Off,
            1 => Level::Ember,
            _ => Level::Lit,
        });
        for encoding in [Encoding::Raw, Encoding::Srgb] {
            let mut probe = Probe::new(&picture, &picture);
            probe.encoding = encoding;
            for column in 0..12 {
                let (level, slack) = match column % 3 {
                    0 => (style::CELLS_LEVEL_OFF, style::CELLS_BREATH),
                    1 => (style::CELLS_LEVEL_EMBER, style::CELLS_BREATH),
                    _ => (style::CELLS_LEVEL_LIT, 0.0),
                };
                let seen = probe.alpha(column, 4);
                assert!(
                    (seen - level).abs() <= slack + 1.0e-4,
                    "column {column} drew {seen}, wanted {level}"
                );
            }
        }
    }

    #[test]
    fn a_cell_is_a_square_that_ends_before_its_pitch_does() {
        let lit = flat(Level::Lit);
        let probe = Probe::new(&lit, &lit);
        let inside = style::CELLS_FILL / 2.0 - style::CELLS_SOFT;
        assert!((probe.read(3, 3, inside, inside)[3] - 1.0).abs() < 1.0e-4);
        assert!(
            probe.read(3, 3, 0.499, 0.0)[3] < 1.0e-4,
            "nothing is drawn between cells"
        );
        assert!(probe.read(3, 3, 0.0, 0.499)[3] < 1.0e-4);
    }

    #[test]
    fn a_flat_cell_keeps_the_average_coverage_of_a_square_one() {
        let lit = flat(Level::Lit);
        let mut probe = Probe::new(&lit, &lit);
        let steps = 64;
        let mean = |probe: &Probe| {
            let mut sum = 0.0;
            for y in 0..steps {
                for x in 0..steps {
                    let (across, down) = (
                        (x as f32 + 0.5) / steps as f32 - 0.5,
                        (y as f32 + 0.5) / steps as f32 - 0.5,
                    );
                    sum += probe.read(5, 5, across, down)[3];
                }
            }
            sum / (steps * steps) as f32
        };
        let squares = mean(&probe);
        probe.set(vec![(CELL_FLAT, GraphValue::Float(1.0))]);
        let flattened = mean(&probe);
        assert!(
            (squares - flattened).abs() < 0.01,
            "squares cover {squares}, the flat cell {flattened}"
        );
        let average = style::CELLS_FILL * style::CELLS_FILL;
        assert!((flattened - average).abs() < 1.0e-4);
    }

    #[test]
    fn the_attended_face_is_blue_and_the_rest_are_the_cell_colour() {
        let lit = flat(Level::Lit);
        let mut probe = Probe::new(&lit, &lit);
        let rgb = |c: [f32; 4]| [c[0], c[1], c[2]];
        let near = |seen: [f32; 3], want: [f32; 3]| {
            seen.iter().zip(want).all(|(a, b)| (a - b).abs() < 1.0e-5)
        };
        let rest = rgb(probe.read(2, 2, 0.0, 0.0));
        let ink = style::CELLS_COLOUR;
        assert!(near(rest, [ink.r, ink.g, ink.b]), "{rest:?}");
        probe.set(vec![(CELL_ATTEND, GraphValue::Float(1.0))]);
        let attended = rgb(probe.read(2, 2, 0.0, 0.0));
        let blue = style::CELLS_ATTENDED;
        assert!(near(attended, [blue.r, blue.g, blue.b]), "{attended:?}");
    }

    #[test]
    fn a_change_crosses_at_the_travel_speed() {
        let (new, old) = (flat(Level::Lit), flat(Level::Off));
        let mut probe = Probe::new(&new, &old);
        let origin = Vec2::new(0.5, 0.5);
        probe.set(vec![(PICTURE_ORIGIN, GraphValue::Vec2(origin))]);
        for column in [2_usize, 5, 9, 11] {
            let distance = column as f32;
            let arrival = distance / style::CELLS_TRAVEL;
            probe.set(vec![(PICTURE_AGE, GraphValue::Float(arrival - 0.002))]);
            assert!(
                probe.alpha(column, 0) < 0.2,
                "column {column} is still the old picture"
            );
            probe.set(vec![(PICTURE_AGE, GraphValue::Float(arrival + 0.002))]);
            assert!(
                probe.alpha(column, 0) > 0.9,
                "column {column} has turned to the new one"
            );
        }
    }

    #[test]
    fn a_cell_flashes_as_the_front_reaches_it_and_settles() {
        let off = flat(Level::Off);
        let mut probe = Probe::new(&off, &off);
        probe.set(vec![(
            PICTURE_ORIGIN,
            GraphValue::Vec2(Vec2::new(0.5, 0.5)),
        )]);
        let arrival = 6.0 / style::CELLS_TRAVEL;
        let at = |probe: &mut Probe, age: f32| {
            probe.set(vec![(PICTURE_AGE, GraphValue::Float(age))]);
            probe.alpha(6, 0)
        };
        assert!(
            (at(&mut probe, arrival) - style::CELLS_FRONT_FLASH).abs() < 0.01,
            "at the front"
        );
        let halfway = at(&mut probe, arrival + style::CELLS_FRONT / 2.0);
        assert!(halfway > style::CELLS_LEVEL_OFF + 0.05 && halfway < style::CELLS_FRONT_FLASH);
        let after = at(&mut probe, arrival + style::CELLS_FRONT + 0.01);
        assert!(
            (after - style::CELLS_LEVEL_OFF).abs() <= style::CELLS_BREATH + 1.0e-4,
            "settled to off, drew {after}"
        );
        let settled = at(&mut probe, SETTLED);
        assert!((settled - style::CELLS_LEVEL_OFF).abs() <= style::CELLS_BREATH + 1.0e-4);
    }

    #[test]
    fn cells_rise_near_the_pointer_and_not_at_its_radius() {
        let off = flat(Level::Off);
        let mut probe = Probe::new(&off, &off);
        let rest = probe.alpha(2, 6);
        let under = Vec2::new(2.5, 6.5);
        probe.set(vec![(CELL_POINTER, GraphValue::Vec2(under))]);
        let near = probe.alpha(2, 6);
        assert!((near - rest - style::CELLS_WAKE_BOOST).abs() < 1.0e-4);
        let radius = style::CELLS_WAKE_RADIUS as usize;
        let edge = probe.alpha(2 + radius, 6);
        let alone = Probe::new(&off, &off).alpha(2 + radius, 6);
        assert!((edge - alone).abs() < 1.0e-4, "no boost at the radius");
    }

    #[test]
    fn off_cells_breathe_within_their_amplitude_and_lit_cells_hold() {
        let off = flat(Level::Off);
        let lit = flat(Level::Lit);
        let (mut low, mut high) = (f32::MAX, f32::MIN);
        for step in 0..120 {
            let mut probe = Probe::new(&off, &off);
            probe.time = step as f32 * 0.1;
            let seen = probe.alpha(4, 4);
            low = low.min(seen);
            high = high.max(seen);
            let mut steady = Probe::new(&lit, &lit);
            steady.time = probe.time;
            assert!((steady.alpha(4, 4) - 1.0).abs() < 1.0e-4);
        }
        assert!(high - low > style::CELLS_BREATH, "it moves");
        assert!(
            high - low <= style::CELLS_BREATH.mul_add(2.0, 1.0e-4),
            "by ±breath"
        );
    }

    #[test]
    fn the_override_list_names_every_input_once() {
        let inputs = Inputs {
            look:   Look {
                attend:  0.0,
                pointer: None,
                flat:    0.0,
            },
            size:   (12, 12),
            origin: Vec2::ZERO,
            age:    SETTLED,
        };
        let mut named = inputs
            .overrides()
            .iter()
            .map(|(input, _)| *input)
            .collect::<Vec<_>>();
        named.sort_unstable();
        let all = (0..picture().public_inputs.len() as u16).collect::<Vec<_>>();
        assert_eq!(named, all);
    }

    #[test]
    fn the_ember_byte_reads_as_ember_in_both_encodings() {
        for encoding in [Encoding::Raw, Encoding::Srgb] {
            let off = decode(Level::Off.texel(), encoding);
            let ember = decode(Level::Ember.texel(), encoding);
            let lit = decode(Level::Lit.texel(), encoding);
            assert!(off < READ_EMBER);
            assert!((READ_EMBER..READ_LIT).contains(&ember), "{ember}");
            assert!(lit >= READ_LIT);
        }
    }
}
