use loro_surgeon::Hydrate;

#[derive(Debug, Clone, Hydrate)]
pub struct MaterialAttr {
    #[loro(with = "wired_schemas::conv::float_slice::optional")]
    pub base_color: Option<[f64; 4]>,
    pub metallic: Option<f64>,
    pub roughness: Option<f64>,
}
