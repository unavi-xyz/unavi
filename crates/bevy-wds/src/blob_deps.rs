use bevy::prelude::*;

use crate::{BlobDep, BlobResponse};

pub const fn load_blob_deps(_deps: Query<&BlobDep, With<BlobResponse>>) {}
