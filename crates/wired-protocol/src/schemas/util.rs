use serde::Deserialize;

#[derive(Deserialize)]
struct Schema {
    #[serde(rename = "$id")]
    id: String,
}

pub fn schema_id(slice: &[u8]) -> Option<String> {
    serde_json::from_slice::<Schema>(slice).map(|s| s.id).ok()
}
