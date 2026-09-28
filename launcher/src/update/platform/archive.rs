use std::{
    fs::File,
    io::BufReader,
    path::Path,
};

use anyhow::Context;
use tracing::info;

pub fn unpack_tar_xz(archive: &Path, dest: &Path) -> anyhow::Result<()> {
    info!("Extracting {} to {}", archive.display(), dest.display());

    let file = File::open(archive).context("failed to open archive")?;
    let decoder = xz2::read::XzDecoder::new(BufReader::new(file));

    tar::Archive::new(decoder)
        .unpack(dest)
        .context("failed to extract archive")
}
