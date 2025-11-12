use xdid::core::did_url::DidUrl;

/// Parse a record reference URL to extract the DID and record ID.
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
