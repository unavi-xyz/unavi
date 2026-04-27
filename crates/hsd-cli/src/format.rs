use std::path::Path;

use anyhow::{Context, Result};
use hsd::Hsd;

pub fn format_file(input: &Path) -> Result<()> {
    let src =
        std::fs::read_to_string(input).with_context(|| format!("reading {}", input.display()))?;
    let hsd = Hsd::parse(&src).with_context(|| format!("parsing {}", input.display()))?;
    std::fs::write(input, hsd.to_ron()?).with_context(|| format!("writing {}", input.display()))?;
    Ok(())
}
