//! Pre-allocated serialization buffers for zero-allocation networking.

use std::mem::size_of;

use postcard::experimental::max_size::MaxSize;

use super::types::{
    object_id::ObjectId,
    physics_state::{PhysicsIFrame, PhysicsPFrame},
    pose::{IFrameTransform, PFrameRootTransform, PFrameTransform},
};

/// Primitive sizes from [`MaxSize`] derives on transform types.
const IFRAME_TRANSFORM_SIZE: usize = IFrameTransform::POSTCARD_MAX_SIZE;
const PFRAME_ROOT_SIZE: usize = PFrameRootTransform::POSTCARD_MAX_SIZE;
const PFRAME_TRANSFORM_SIZE: usize = PFrameTransform::POSTCARD_MAX_SIZE;

/// Enum discriminant for [`bevy_vrm::BoneName`] (55 variants < 128).
const BONE_NAME_SIZE: usize = 1;

/// Bone pose sizes (discriminant + transform).
const IFRAME_BONE_SIZE: usize = BONE_NAME_SIZE + IFRAME_TRANSFORM_SIZE;
const PFRAME_BONE_SIZE: usize = BONE_NAME_SIZE + PFRAME_TRANSFORM_SIZE;

/// Maximum number of bones in a pose message.
/// VRM humanoid has 55 bones; we add headroom.
pub const MAX_BONES: usize = 64;

/// Maximum varint size for bone vector length (counts < 16384 fit in 2 bytes).
const VEC_LEN_VARINT_MAX: usize = 2;

/// Maximum serialized size of an [`IFrameMsg`].
pub const IFRAME_MSG_MAX_SIZE: usize = size_of::<u16>()  // id
    + IFRAME_TRANSFORM_SIZE                              // root
    + VEC_LEN_VARINT_MAX                                 // bones vec length
    + (IFRAME_BONE_SIZE * MAX_BONES)                     // bones
    ;

/// Maximum serialized size of a [`PFrameDatagram`].
pub const PFRAME_MSG_MAX_SIZE: usize = size_of::<u16>()  // iframe_id
    + size_of::<u16>()                                   // seq
    + PFRAME_ROOT_SIZE                                   // root (absolute position)
    + VEC_LEN_VARINT_MAX                                 // bones vec length
    + (PFRAME_BONE_SIZE * MAX_BONES)                     // bones
    ;

/// Maximum serialized size of a [`ControlMsg`].
pub const CONTROL_MSG_MAX_SIZE: usize = 8;

/// Pre-allocated buffers for outbound serialization.
pub struct SendBuffer {
    pub iframe: [u8; IFRAME_MSG_MAX_SIZE],
    pub pframe: [u8; PFRAME_MSG_MAX_SIZE],
}

impl SendBuffer {
    pub const fn new() -> Self {
        Self {
            iframe: [0u8; IFRAME_MSG_MAX_SIZE],
            pframe: [0u8; PFRAME_MSG_MAX_SIZE],
        }
    }
}

impl Default for SendBuffer {
    fn default() -> Self {
        Self::new()
    }
}

// Object buffer sizing.

/// Object ID size.
const OBJECT_ID_SIZE: usize = ObjectId::POSTCARD_MAX_SIZE;

/// Object physics state sizes.
const OBJECT_IFRAME_SIZE: usize = PhysicsIFrame::POSTCARD_MAX_SIZE;
const OBJECT_PFRAME_SIZE: usize = PhysicsPFrame::POSTCARD_MAX_SIZE;

/// Maximum serialized size of an [`ObjectIFrameMsg`] (single object per stream).
/// Object ID is known from `StreamInit::Object`, not included in each message.
pub const OBJECT_IFRAME_MSG_MAX_SIZE: usize = size_of::<u16>()  // id
    + size_of::<u64>()                                          // timestamp
    + OBJECT_IFRAME_SIZE                                        // state
    ;

/// Maximum serialized size of an [`ObjectPFrameDatagram`] (single object per datagram).
pub const OBJECT_PFRAME_MSG_MAX_SIZE: usize = size_of::<u16>()  // iframe_id
    + size_of::<u16>()                                          // seq
    + size_of::<u64>()                                          // timestamp
    + OBJECT_ID_SIZE                                            // object_id
    + OBJECT_PFRAME_SIZE                                        // state
    ;

/// Enum discriminant for [`Datagram`] (postcard varint, < 128 variants).
const DATAGRAM_DISCRIMINANT_SIZE: usize = 1;

/// Maximum serialized size of a [`Datagram`].
/// Largest variant is the one with more data.
pub const DATAGRAM_MAX_SIZE: usize =
    DATAGRAM_DISCRIMINANT_SIZE + const_max(PFRAME_MSG_MAX_SIZE, OBJECT_PFRAME_MSG_MAX_SIZE);

const fn const_max(a: usize, b: usize) -> usize {
    if a > b { a } else { b }
}

/// Pre-allocated buffers for object outbound serialization.
pub struct ObjectSendBuffer {
    pub iframe: [u8; OBJECT_IFRAME_MSG_MAX_SIZE],
    pub pframe: [u8; OBJECT_PFRAME_MSG_MAX_SIZE],
}

impl ObjectSendBuffer {
    pub const fn new() -> Self {
        Self {
            iframe: [0u8; OBJECT_IFRAME_MSG_MAX_SIZE],
            pframe: [0u8; OBJECT_PFRAME_MSG_MAX_SIZE],
        }
    }
}

impl Default for ObjectSendBuffer {
    fn default() -> Self {
        Self::new()
    }
}
