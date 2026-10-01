use std::mem::size_of;

use bytemuck::Pod;
use hsd::bounds::MAX_MESH_STREAM_BYTES;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BufferError {
    #[error("buffer is {len} bytes, over the cap of {MAX_MESH_STREAM_BYTES}")]
    TooLarge { len: usize },
    #[error("buffer is not a whole number of elements")]
    Ragged,
}

/// Casts a peer-written buffer, refusing one over
/// [`MAX_MESH_STREAM_BYTES`] or not a whole number of `T`.
pub fn cast_buffer<T: Pod>(bytes: &[u8]) -> Result<Vec<T>, BufferError> {
    if bytes.len() > MAX_MESH_STREAM_BYTES {
        return Err(BufferError::TooLarge { len: bytes.len() });
    }
    if !bytes.len().is_multiple_of(size_of::<T>()) {
        return Err(BufferError::Ragged);
    }
    Ok(bytemuck::pod_collect_to_vec(bytes))
}
