use parry3d::na::Point3;

pub fn cuboid_uvs(positions: &[Point3<f32>]) -> Vec<f32> {
    let mut uvs = Vec::new();

    for position in positions {
        let uv = match (position.x, position.y, position.z) {
            (x, y, 0.0) => [x, y],          // Front face
            (x, y, z) if z > 0.0 => [x, y], // Back face
            (x, 0.0, z) => [x, z],          // Bottom face
            (x, y, z) if y > 0.0 => [x, z], // Top face
            (0.0, y, z) => [y, z],          // Left face
            (x, y, z) if x > 0.0 => [y, z], // Right face
            _ => [0.0, 0.0],                // Default (shouldn't occur)
        };

        uvs.push(uv[0]);
        uvs.push(uv[1]);
    }

    uvs
}
