use std::collections::BTreeMap;

use bevy::{
    asset::RenderAssetUsages,
    mesh::{
        Indices,
        PrimitiveTopology,
    },
    prelude::*,
};
use msdf::layout::{
    Laid,
    Quad,
};

/// Where the text block sits relative to its transform. Horizontal placement
/// is [`msdf::layout::Align`]'s job, so this only answers the vertical.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Anchor {
    /// The origin is the first baseline — where a caller positioning text
    /// against other text wants it.
    #[default]
    Baseline,
    Top,
    Middle,
    Bottom,
}

impl Anchor {
    /// How far to lift the block so the anchor lands on the origin.
    #[must_use]
    pub fn offset(self, laid: &Laid) -> f32 {
        match self {
            Self::Baseline => 0.0,
            Self::Top => -laid.bounds.max[1],
            Self::Middle => -(laid.bounds.min[1] + laid.bounds.max[1]) / 2.0,
            Self::Bottom => -laid.bounds.min[1],
        }
    }
}

/// Builds one quad per glyph in the XY plane, facing +Z.
#[must_use]
pub fn build(laid: &Laid, anchor: Anchor) -> Mesh {
    build_quads(&laid.quads, anchor.offset(laid))
}

/// One mesh per font and page, so each page is drawn with the image that holds
/// it.
///
/// Quads from different pages — or different fonts in a fallback stack — share
/// a plane but sample different textures, so they cannot share a mesh. The key
/// is `(font, page)`.
#[must_use]
pub fn page_meshes(laid: &Laid, anchor: Anchor) -> Vec<((u32, u32), Mesh)> {
    let raise = anchor.offset(laid);
    let mut grouped: BTreeMap<(u32, u32), Vec<Quad>> = BTreeMap::new();
    for quad in &laid.quads {
        grouped
            .entry((quad.font, quad.page))
            .or_default()
            .push(*quad);
    }
    grouped
        .into_iter()
        .map(|(key, quads)| (key, build_quads(&quads, raise)))
        .collect()
}

fn build_quads(quads: &[Quad], raise: f32) -> Mesh {
    let count = quads.len();

    let mut positions = Vec::with_capacity(count * 4);
    let mut uvs = Vec::with_capacity(count * 4);
    let mut indices = Vec::with_capacity(count * 6);

    for (index, quad) in quads.iter().enumerate() {
        let (left, right) = (quad.plane.min[0], quad.plane.max[0]);
        let (bottom, top) = (quad.plane.min[1] + raise, quad.plane.max[1] + raise);
        positions.extend([
            [left, top, 0.0],
            [right, top, 0.0],
            [left, bottom, 0.0],
            [right, bottom, 0.0],
        ]);
        // The plane is y-up and the field is y-down, so the top corners take
        // the lower texture coordinate.
        uvs.extend([
            [quad.uv.min[0], quad.uv.min[1]],
            [quad.uv.max[0], quad.uv.min[1]],
            [quad.uv.min[0], quad.uv.max[1]],
            [quad.uv.max[0], quad.uv.max[1]],
        ]);

        let base = (index * 4) as u32;
        indices.extend([base, base + 2, base + 1, base + 1, base + 2, base + 3]);
    }

    let normals = vec![[0.0, 0.0, 1.0]; positions.len()];

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

#[cfg(test)]
mod tests {
    use bevy::mesh::VertexAttributeValues;
    use msdf::{
        atlas::{
            Glyph,
            GlyphSource,
            Rect,
            VerticalMetrics,
        },
        layout::{
            LayoutOpts,
            layout,
        },
    };

    use super::*;

    /// Every glyph one em wide and half an em tall, so a width is a character
    /// count.
    struct Stub(BTreeMap<char, Glyph>);

    impl GlyphSource for Stub {
        fn vertical(&self) -> VerticalMetrics {
            VerticalMetrics {
                ascender:  0.75,
                descender: -0.25,
                line_gap:  0.0,
            }
        }

        fn glyph(&self, ch: char) -> Option<Glyph> {
            self.0.get(&ch).copied()
        }

        fn kern(&self, _left: char, _right: char) -> f32 {
            0.0
        }
    }

    fn atlas() -> Stub {
        let glyph = Glyph {
            plane:   Rect {
                min: [0.0, 0.0],
                max: [1.0, 0.5],
            },
            uv:      Rect {
                min: [0.25, 0.25],
                max: [0.5, 0.5],
            },
            advance: 1.0,
            page:    0,
            font:    0,
        };
        Stub(('a'..='z').map(|ch| (ch, glyph)).collect())
    }

    fn opts() -> LayoutOpts {
        LayoutOpts {
            size: 1.0,
            ..Default::default()
        }
    }

    fn mesh(text: &str, anchor: Anchor) -> Mesh {
        build(&layout(text, &atlas(), &opts()).expect("layout"), anchor)
    }

    fn positions(mesh: &Mesh) -> Vec<[f32; 3]> {
        match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(values)) => values.clone(),
            _ => panic!("positions"),
        }
    }

    #[test]
    fn a_glyph_becomes_one_quad() {
        let mesh = mesh("abc", Anchor::Baseline);
        assert_eq!(positions(&mesh).len(), 12);
        assert_eq!(mesh.indices().expect("indices").len(), 18);
    }

    #[test]
    fn quads_split_into_one_mesh_per_page() {
        let quad = |font, page| Quad {
            plane: Rect {
                min: [0.0, 0.0],
                max: [1.0, 0.5],
            },
            uv: Rect {
                min: [0.0, 0.0],
                max: [0.1, 0.1],
            },
            page,
            font,
        };
        let laid = Laid {
            quads:   vec![quad(0, 0), quad(0, 1), quad(1, 0), quad(0, 1)],
            bounds:  Rect::ZERO,
            ink:     Rect::ZERO,
            lines:   1,
            missing: Vec::new(),
        };
        let meshes = page_meshes(&laid, Anchor::Baseline);
        assert_eq!(meshes.len(), 3, "font and page both split meshes");
        let keys = meshes.iter().map(|(key, _)| *key).collect::<Vec<_>>();
        assert_eq!(keys, vec![(0, 0), (0, 1), (1, 0)]);
        assert_eq!(positions(&meshes[0].1).len(), 4, "(0,0) holds one quad");
        assert_eq!(positions(&meshes[1].1).len(), 8, "(0,1) holds two quads");
        assert_eq!(positions(&meshes[2].1).len(), 4, "(1,0) holds one quad");
    }

    #[test]
    fn an_empty_string_builds_an_empty_mesh() {
        let mesh = mesh("", Anchor::Baseline);
        assert_eq!(positions(&mesh), [] as [[f32; 3]; 0]);
        assert_eq!(mesh.indices().expect("indices").len(), 0);
    }

    #[test]
    fn the_top_corners_take_the_lower_texture_coordinate() {
        let mesh = mesh("a", Anchor::Baseline);
        let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0)
        else {
            panic!("uvs")
        };
        let positions = positions(&mesh);
        assert!(
            positions[0][1] > positions[2][1],
            "vertex 0 is the top-left"
        );
        assert!(uvs[0][1] < uvs[2][1], "and the field runs the other way");
    }

    #[test]
    fn every_triangle_faces_the_camera() {
        let mesh = mesh("ab", Anchor::Baseline);
        let positions = positions(&mesh);
        let Some(indices) = mesh.indices() else {
            panic!("indices")
        };
        for triangle in indices.iter().collect::<Vec<_>>().chunks(3) {
            let [a, b, c] = [
                positions[triangle[0]],
                positions[triangle[1]],
                positions[triangle[2]],
            ];
            let edge = |from: [f32; 3], to: [f32; 3]| [to[0] - from[0], to[1] - from[1]];
            let (first, second) = (edge(a, b), edge(b, c));
            assert!(
                second[0].mul_add(-first[1], first[0] * second[1]) > 0.0,
                "a back-facing quad is an invisible one"
            );
        }
    }

    #[test]
    fn anchoring_moves_the_block_and_not_its_shape() {
        let baseline = positions(&mesh("ab", Anchor::Baseline));
        let top = positions(&mesh("ab", Anchor::Top));
        let lift = top[0][1] - baseline[0][1];
        assert!(lift < 0.0, "anchoring to the top hangs the text below");
        for (before, after) in baseline.iter().zip(&top) {
            assert!((after[1] - before[1] - lift).abs() < 1.0e-5);
            assert!((after[0] - before[0]).abs() < 1.0e-5);
        }
    }

    #[test]
    fn a_middle_anchored_block_straddles_the_origin() {
        let laid = layout("ab\ncd", &atlas(), &opts()).expect("layout");
        let lift = Anchor::Middle.offset(&laid);
        assert!(
            (laid.bounds.min[1] + lift + (laid.bounds.max[1] + lift)).abs() < 1.0e-5,
            "the metric box centres, so a two-line block does not drift"
        );
    }
}
