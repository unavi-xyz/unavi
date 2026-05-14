use bevy::{
    asset::{AssetLoader, AsyncReadExt, LoadContext, io::Reader},
    platform::collections::HashMap,
    prelude::*,
    reflect::TypePath,
    tasks::ConditionalSendFuture,
};
use blake3::Hash;
use hsd::Hsd;

#[derive(Asset, Debug, TypePath)]
pub struct HsdAsset {
    pub doc: Hsd,
    pub deps: HashMap<Hash, Handle<BlobAsset>>,
}

#[derive(Default, TypePath)]
pub struct HsdLoader;

impl AssetLoader for HsdLoader {
    type Asset = HsdAsset;
    type Settings = ();
    type Error = anyhow::Error;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut value = String::new();
            reader.read_to_string(&mut value).await?;

            let doc = Hsd::parse(&value)?;
            let mut deps = HashMap::new();

            for hash in doc
                .assets
                .values()
                .chain(doc.images.values().filter_map(|i| i.data.as_ref()))
                .chain(
                    doc.nodes
                        .iter()
                        .filter_map(|t| t.data.as_ref())
                        .flat_map(|n| n.scripts.iter()),
                )
            {
                let dep_path = load_context
                    .path()
                    .path()
                    .parent()
                    .expect("asset parent dir")
                    .join(hash.to_string());
                deps.insert(hash.0, load_context.load(dep_path));
            }

            Ok(HsdAsset { doc, deps })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["hsd"]
    }
}

#[derive(Asset, Debug, Deref, DerefMut, TypePath)]
pub struct BlobAsset(pub Vec<u8>);

#[derive(Default, TypePath)]
pub struct BlobLoader;

impl AssetLoader for BlobLoader {
    type Asset = BlobAsset;
    type Settings = ();
    type Error = anyhow::Error;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            Ok(BlobAsset(bytes))
        })
    }

    fn extensions(&self) -> &[&str] {
        &[]
    }
}
