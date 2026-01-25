use loro::LoroMapValue;
use loro_surgeon::Hydrate;

#[derive(Debug, Clone, Hydrate)]
pub struct StageData {
    pub layers: Vec<LayerData>,
}

#[derive(Debug, Clone, Hydrate)]
pub struct LayerData {
    pub enabled: bool,
    pub opinions: Vec<OpinionData>,
}

#[derive(Debug, Clone, Hydrate)]
pub struct OpinionData {
    pub attrs: LoroMapValue,
    pub node: i64,
}
