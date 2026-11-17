//! A row of cells lit from the left to a level, with no texture.
//!
//! The row is [`style::CELLS_CELL_ROW`] cells long and one tall, so a quad
//! drawn at that proportion has square cells.

use wired_guest::style;

use super::{
    Net,
    binary,
    cell::{
        CELL_OWN,
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
        Combine2Op,
        CullMode,
        ExtractOp,
        GraphValue,
        Node,
        Port,
        ShaderGraph,
        StepOp,
        SurfaceGraph,
        SurfaceOutput,
        UnlitOutput,
        set_graph,
    },
};

/// How much of the row is lit, `0` to `1`. The cell the level ends inside is
/// ember.
pub const METER_LEVEL: u16 = CELL_OWN;

/// What a cell must hold before it counts as touched, so a level that ends
/// exactly on a cell boundary does not leave the next cell ember.
const TOUCHED: f32 = 1.0e-4;

/// A meter: the first `level × row` cells lit, the cell the level ends in
/// ember, the rest off.
pub fn meter() -> ShaderGraph {
    let mut net = Net::default();
    let row = style::CELLS_CELL_ROW as f32;

    let uv = net.push(Node::Uv);
    let along = net.push(Node::Extract(ExtractOp {
        v:       uv,
        channel: 0,
    }));
    let up = net.push(Node::Extract(ExtractOp {
        v:       uv,
        channel: 1,
    }));
    let cells_along = net.push(Node::Mul(binary(along, cf(row))));
    let column = net.push(Node::Floor(cells_along));
    let across = net.push(Node::Sub(binary(cells_along, column)));
    let within = net.push(Node::Combine2(Combine2Op { x: across, y: up }));

    // How much of this cell the level fills: the lit cells before it count
    // whole, the one it ends in by its fraction.
    let reached = net.push(Node::Mul(binary(Port::Input(METER_LEVEL), cf(row))));
    let filled = net.push(Node::Sub(binary(reached, column)));
    let filled = net.push(Node::Saturate(filled));
    let touched = net.push(Node::Step(StepOp {
        edge: cf(TOUCHED),
        x:    filled,
    }));
    let whole = net.push(Node::Step(StepOp {
        edge: cf(1.0),
        x:    filled,
    }));
    let level = net.level(touched, whole);

    let off = net.push(Node::OneMinus(touched));
    let at = net.push(Node::Combine2(Combine2Op {
        x: column,
        y: cf(0.0),
    }));
    let level = net.breathe(level, at, off);
    let middle = net.push(Node::Add(binary(column, cf(0.5))));
    let centre = net.push(Node::Combine2(Combine2Op {
        x: middle,
        y: cf(0.5),
    }));
    let level = net.wake(level, centre);

    let shape = net.square(within);
    let color = net.paint(level, shape);

    let mut public_inputs = look_defaults();
    public_inputs.push(GraphValue::Float(0.0));

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

/// Submits [`meter`] to `prim`.
pub fn set_meter(doc: &Document, prim: (u64, u64)) -> anyhow::Result<()> {
    set_graph(doc, prim, Layer::Local, Some(&meter()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        super::interp::{
            self,
            Context,
        },
        *,
    };

    /// The alpha of cell `column` of the row, with the meter at `level`.
    fn cell(level: f32, column: usize) -> f32 {
        let graph = meter();
        let mut values = graph.public_inputs.clone();
        values[usize::from(METER_LEVEL)] = GraphValue::Float(level);
        let row = style::CELLS_CELL_ROW as f32;
        let at = Context::at([(column as f32 + 0.5) / row, 0.5]);
        interp::shade(&graph, &values, &at).color.c[3]
    }

    fn near(seen: f32, want: f32) -> bool {
        (seen - want).abs() <= style::CELLS_BREATH + 1.0e-4
    }

    #[test]
    fn a_meter_lights_the_first_cells_of_its_row() {
        let row = style::CELLS_CELL_ROW as usize;
        for lit in [0, 1, 5, row] {
            // A hair past the boundary, so float rounding cannot leave the
            // last cell a hair short of whole.
            let level = if lit == 0 {
                0.0
            } else {
                (lit as f32 + 5.0e-5) / row as f32
            };
            for column in 0..row {
                let want = if column < lit {
                    style::CELLS_LEVEL_LIT
                } else {
                    style::CELLS_LEVEL_OFF
                };
                let seen = cell(level, column);
                assert!(near(seen, want), "{lit} lit, cell {column} drew {seen}");
            }
        }
    }

    #[test]
    fn the_cell_a_level_ends_inside_is_ember() {
        let level = 4.5 / style::CELLS_CELL_ROW as f32;
        assert!(near(cell(level, 3), style::CELLS_LEVEL_LIT));
        assert!(near(cell(level, 4), style::CELLS_LEVEL_EMBER));
        assert!(near(cell(level, 5), style::CELLS_LEVEL_OFF));
    }

    #[test]
    fn a_level_on_a_cell_boundary_leaves_the_next_cell_off() {
        let level = 6.0 / style::CELLS_CELL_ROW as f32;
        assert!(near(cell(level, 5), style::CELLS_LEVEL_LIT));
        assert!(near(cell(level, 6), style::CELLS_LEVEL_OFF));
    }

    #[test]
    fn a_meter_blends_unclipped_over_the_glass() {
        let graph = meter();
        assert!(matches!(graph.surface.blend, BlendMode::Blend));
        let SurfaceOutput::Unlit(unlit) = graph.surface.output else {
            panic!("cells are unlit");
        };
        assert!(unlit.alpha_clip_threshold.is_none());
    }
}
