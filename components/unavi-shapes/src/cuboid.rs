use crate::bindings::wired::math::types::Vec3;

pub struct Vertices {
    pub positions: Vec<f32>,
    pub normals: Vec<f32>,
    pub uvs: Vec<f32>,
    pub indices: Vec<u32>,
}

pub fn vertices(half_size: Vec3) -> Vertices {
    let min = Vec3 {
        x: -half_size.x,
        y: -half_size.y,
        z: -half_size.z,
    };

    let max = half_size;

    let vertices = &[
        // Front
        ([min.x, min.y, max.z], [0.0, 0.0, 1.0], [0.0, 0.0]),
        ([max.x, min.y, max.z], [0.0, 0.0, 1.0], [1.0, 0.0]),
        ([max.x, max.y, max.z], [0.0, 0.0, 1.0], [1.0, 1.0]),
        ([min.x, max.y, max.z], [0.0, 0.0, 1.0], [0.0, 1.0]),
        // Back
        ([min.x, max.y, min.z], [0.0, 0.0, -1.0], [1.0, 0.0]),
        ([max.x, max.y, min.z], [0.0, 0.0, -1.0], [0.0, 0.0]),
        ([max.x, min.y, min.z], [0.0, 0.0, -1.0], [0.0, 1.0]),
        ([min.x, min.y, min.z], [0.0, 0.0, -1.0], [1.0, 1.0]),
        // Right
        ([max.x, min.y, min.z], [1.0, 0.0, 0.0], [0.0, 0.0]),
        ([max.x, max.y, min.z], [1.0, 0.0, 0.0], [1.0, 0.0]),
        ([max.x, max.y, max.z], [1.0, 0.0, 0.0], [1.0, 1.0]),
        ([max.x, min.y, max.z], [1.0, 0.0, 0.0], [0.0, 1.0]),
        // Left
        ([min.x, min.y, max.z], [-1.0, 0.0, 0.0], [1.0, 0.0]),
        ([min.x, max.y, max.z], [-1.0, 0.0, 0.0], [0.0, 0.0]),
        ([min.x, max.y, min.z], [-1.0, 0.0, 0.0], [0.0, 1.0]),
        ([min.x, min.y, min.z], [-1.0, 0.0, 0.0], [1.0, 1.0]),
        // Top
        ([max.x, max.y, min.z], [0.0, 1.0, 0.0], [1.0, 0.0]),
        ([min.x, max.y, min.z], [0.0, 1.0, 0.0], [0.0, 0.0]),
        ([min.x, max.y, max.z], [0.0, 1.0, 0.0], [0.0, 1.0]),
        ([max.x, max.y, max.z], [0.0, 1.0, 0.0], [1.0, 1.0]),
        // Bottom
        ([max.x, min.y, max.z], [0.0, -1.0, 0.0], [0.0, 0.0]),
        ([min.x, min.y, max.z], [0.0, -1.0, 0.0], [1.0, 0.0]),
        ([min.x, min.y, min.z], [0.0, -1.0, 0.0], [1.0, 1.0]),
        ([max.x, min.y, min.z], [0.0, -1.0, 0.0], [0.0, 1.0]),
    ];

    let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).flatten().collect();
    let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).flatten().collect();
    let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).flatten().collect();

    let indices = vec![
        0, 1, 2, 2, 3, 0, // front
        4, 5, 6, 6, 7, 4, // back
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // top
        20, 21, 22, 22, 23, 20, // bottom
    ];

    Vertices {
        positions,
        normals,
        uvs,
        indices,
    }
}
