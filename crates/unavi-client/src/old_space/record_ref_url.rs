use xdid::core::{did::Did, did_url::DidUrl};

pub fn new_record_ref_url(did: Did, record_id: &str) -> DidUrl {
    DidUrl {
        did,
        query: Some(format!("service=dwn&relativeRef=/records/{record_id}").into()),
        fragment: None,
        path_abempty: None,
    }
}

/// Parse a record reference URL to extract the record ID.
///
/// Expected format: `{did}?service=dwn&relativeRef=/records/{record-id}`
pub fn parse_record_ref_url(did_url: &DidUrl) -> anyhow::Result<&str> {
    let relative_ref = did_url
        .query
        .as_ref()
        .and_then(|q| {
            q.split('&')
                .find(|param| param.starts_with("relativeRef="))
                .and_then(|param| param.strip_prefix("relativeRef="))
        })
        .ok_or_else(|| anyhow::anyhow!("missing relativeRef in record ref URL"))?;

    let record_id = relative_ref
        .strip_prefix("/records/")
        .ok_or_else(|| anyhow::anyhow!("invalid relativeRef format"))?;

    Ok(record_id)
}
