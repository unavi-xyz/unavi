use serde::Deserialize;

#[derive(Deserialize)]
struct Schema {
    #[serde(rename = "$id")]
    id: String,
}

pub fn get_schema_id(slice: &[u8]) -> Option<String> {
    serde_json::from_slice::<Schema>(slice).map(|s| s.id).ok()
}

#[derive(Deserialize)]
struct Protocol {
    protocol: String,
}

pub fn get_protocol_url(slice: &[u8]) -> Option<String> {
    serde_json::from_slice::<Protocol>(slice)
        .map(|s| s.protocol)
        .ok()
}
