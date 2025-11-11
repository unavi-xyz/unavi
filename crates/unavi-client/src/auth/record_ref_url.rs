use std::str::FromStr;

use xdid::core::{did::Did, did_url::DidUrl};

/// Parse a record reference URL to extract the DID and record ID.
///
/// Expected format: `{did}?service=dwn&relativeRef=/records/{record-id}`
pub fn parse_record_ref_url(url: &str) -> anyhow::Result<(Did, String)> {
    let did_url = DidUrl::from_str(url)?;

    // Extract record ID from relativeRef query parameter.
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
        .ok_or_else(|| anyhow::anyhow!("invalid relativeRef format"))?
        .to_string();

    Ok((did_url.did, record_id))
}
