use parry3d::{
    math::Real,
    na::{Point3, Vector3},
};

use crate::bindings::wired::scene::mesh::Primitive;

pub fn set_attributes(
    primitive: &Primitive,
    (positions, indices): (Vec<Point3<Real>>, Vec<[u32; 3]>),
) {
    let normals = calculate_normals(&positions, &indices)
        .into_iter()
        .flat_map(|x| [x[0], x[1], x[2]])
        .collect::<Vec<_>>();
    primitive.set_normals(&normals);

    let positions = positions
        .into_iter()
        .flat_map(|x| [x[0], x[1], x[2]])
        .collect::<Vec<_>>();
    primitive.set_positions(&positions);

    let indices = indices.into_iter().flatten().collect::<Vec<_>>();
    primitive.set_indices(&indices);
}

fn calculate_normals(positions: &[Point3<Real>], indices: &[[u32; 3]]) -> Vec<Vector3<Real>> {
    let mut normals = vec![Vector3::zeros(); positions.len()];
    let mut counts = vec![0; positions.len()];

    for index in indices {
        let i0 = index[0] as usize;
        let i1 = index[1] as usize;
        let i2 = index[2] as usize;

        let v0 = positions[i0];
        let v1 = positions[i1];
        let v2 = positions[i2];

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;

        let normal = edge1.cross(&edge2).normalize();

        normals[i0] += normal;
        normals[i1] += normal;
        normals[i2] += normal;

        counts[i0] += 1;
        counts[i1] += 1;
        counts[i2] += 1;
    }

    for i in 0..normals.len() {
        normals[i] /= counts[i] as f32;
        normals[i] = normals[i].normalize();
    }

    normals
}
