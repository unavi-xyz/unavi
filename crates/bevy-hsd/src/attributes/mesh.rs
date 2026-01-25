use bevy::mesh::PrimitiveTopology;
use loro_surgeon::Hydrate;
use wired_schemas::HydratedHash;

#[derive(Debug, Clone, Hydrate)]
pub struct MeshAttr {
    #[loro(with = "topology")]
    pub topology: PrimitiveTopology,
    pub colors: Option<HydratedHash>,
    pub indices: HydratedHash,
    pub normals: Option<HydratedHash>,
    pub points: HydratedHash,
    pub tangents: Option<HydratedHash>,
    pub uvs: Option<HydratedHash>,
}

mod topology {
    use bevy::mesh::PrimitiveTopology;
    use loro::LoroValue;
    use loro_surgeon::HydrateError;
    use smol_str::SmolStr;

    pub fn hydrate(value: &LoroValue) -> Result<PrimitiveTopology, HydrateError> {
        let LoroValue::I64(num) = value else {
            return Err(HydrateError::TypeMismatch {
                path: SmolStr::default(),
                expected: "i64".into(),
                actual: format!("{value:?}").into(),
            });
        };

        match num {
            0 => Ok(PrimitiveTopology::PointList),
            1 => Ok(PrimitiveTopology::LineList),
            2 => Ok(PrimitiveTopology::LineStrip),
            3 => Ok(PrimitiveTopology::TriangleList),
            4 => Ok(PrimitiveTopology::TriangleStrip),
            other => Err(HydrateError::TypeMismatch {
                path: SmolStr::default(),
                expected: "valid topology".into(),
                actual: format!("{other}").into(),
            }),
        }
    }
}
