use std::path::Path;

use async_channel::Sender;
use bevy::asset::io::{
    AssetReader,
    AssetReaderError,
    PathStream,
    Reader,
    VecReader,
};
use blake3::Hash;
use bytes::Bytes;
use tokio::sync::oneshot;
use unavi_assets::manifest_entry;

/// One asset fetch, handed from a reader to the world.
pub struct FetchRequest {
    pub rel_path: &'static str,
    pub hash:     Hash,
    pub tx:       oneshot::Sender<Result<Bytes, String>>,
}

/// Serves manifest assets from the iroh blob store.
///
/// A path resolves by exact match against the manifest, so a path from a script
/// or a document reaches manifest content or nothing.
pub struct IrohAssetReader(pub Sender<FetchRequest>);

impl IrohAssetReader {
    async fn fetch(&self, path: &Path) -> Result<Bytes, AssetReaderError> {
        let not_found = || AssetReaderError::NotFound(path.to_path_buf());

        let asset = path
            .to_str()
            .and_then(manifest_entry)
            .ok_or_else(not_found)?;

        let hash = Hash::from_hex(asset.hash).map_err(io_error)?;

        let (tx, rx) = oneshot::channel();
        self.0
            .send(FetchRequest {
                rel_path: asset.rel_path,
                hash,
                tx,
            })
            .await
            .map_err(io_error)?;

        rx.await.map_err(io_error)?.map_err(io_error)
    }
}

fn io_error(err: impl ToString) -> AssetReaderError {
    AssetReaderError::Io(std::io::Error::other(err.to_string()).into())
}

impl AssetReader for IrohAssetReader {
    async fn read<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        let bytes = self.fetch(path).await?;
        Ok(VecReader::new(bytes.to_vec()))
    }

    /// Content-addressed assets carry no meta, so the loader falls back to its
    /// defaults.
    async fn read_meta<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        Err::<VecReader, _>(AssetReaderError::NotFound(path.to_path_buf()))
    }

    async fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> Result<Box<PathStream>, AssetReaderError> {
        Err(AssetReaderError::NotFound(path.to_path_buf()))
    }

    async fn is_directory<'a>(&'a self, _path: &'a Path) -> Result<bool, AssetReaderError> {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use bevy::tasks::{
        block_on,
        futures_lite::future::zip,
    };
    use unavi_assets::DEFAULT_AVATAR;

    use super::*;

    #[test]
    fn an_unmanifested_path_reaches_nothing() {
        let (tx, rx) = async_channel::unbounded();
        let reader = IrohAssetReader(tx);

        for path in ["model/unknown.vrm", "../../etc/passwd", "/etc/passwd"] {
            let err = block_on(reader.fetch(Path::new(path))).expect_err("no such asset");
            assert!(
                matches!(err, AssetReaderError::NotFound(_)),
                "{path} resolves to nothing"
            );
        }

        assert!(rx.is_empty(), "no fetch is dispatched for an unknown path");
    }

    #[test]
    fn a_manifest_path_fetches_by_hash() {
        let (tx, rx) = async_channel::unbounded();
        let reader = IrohAssetReader(tx);

        let served = Bytes::from_static(b"a vrm");
        let (fetched, ()) = block_on(zip(reader.fetch(Path::new(DEFAULT_AVATAR)), async {
            let request = rx.recv().await.expect("dispatched fetch");
            assert_eq!(request.rel_path, DEFAULT_AVATAR);
            request.tx.send(Ok(served.clone())).ok();
        }));

        assert_eq!(fetched.expect("bytes"), served);
    }

    #[test]
    fn a_failed_fetch_reads_as_an_io_error() {
        let (tx, rx) = async_channel::unbounded();
        let reader = IrohAssetReader(tx);

        let (fetched, ()) = block_on(zip(reader.fetch(Path::new(DEFAULT_AVATAR)), async {
            let request = rx.recv().await.expect("dispatched fetch");
            request.tx.send(Err("no provider".to_string())).ok();
        }));

        assert!(matches!(
            fetched.expect_err("failure"),
            AssetReaderError::Io(_)
        ));
    }
}
