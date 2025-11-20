use serde::{Deserialize, Serialize};
use serde_vrm::vrm0::BoneName;
use smallvec::SmallVec;
use xdid::core::did::Did;

pub type RpcResult<T> = Result<T, String>;

#[tarpc::service]
pub trait ControlService {
    async fn tickrate_ms() -> u64;

    async fn join_space(id: String) -> RpcResult<()>;
    async fn leave_space(id: String) -> RpcResult<()>;
    async fn spaces() -> Vec<String>;

    async fn players() -> Vec<Player>;

    async fn set_player_tickrate(player_id: u64, tickrate_ms: u64) -> RpcResult<()>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    pub did: Did,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TrackingUpdate {
    IFrame(TrackingIFrame),
    PFrame(TrackingPFrame),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrackingIFrame {
    pub iframe_id: u8,
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub joints: SmallVec<[JointIFrame; 8]>,
}

impl Default for TrackingIFrame {
    fn default() -> Self {
        Self {
            iframe_id: 0,
            translation: Default::default(),
            rotation: [0.0, 0.0, 0.0, 1.0],
            joints: SmallVec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrackingPFrame {
    pub iframe_id: u8,
    pub translation: [i16; 3],
    pub rotation: [i16; 4],
    pub joints: SmallVec<[JointPFrame; 8]>,
}

impl Default for TrackingPFrame {
    fn default() -> Self {
        Self {
            iframe_id: 0,
            translation: Default::default(),
            rotation: [0, 0, 0, i16::MAX],
            joints: SmallVec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JointIFrame {
    pub id: BoneName,
    pub rotation: [f32; 4],
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JointPFrame {
    pub id: BoneName,
    pub rotation: [i16; 4],
}

pub const TRANSFORM_LENGTH_FIELD_LENGTH: usize = 2;
pub const TRANSFORM_MAX_FRAME_LENGTH: usize = 4 * 1024;

pub mod from_client {
    use bincode::{Decode, Encode};

    #[derive(Encode, Decode, Debug)]
    pub enum StreamHeader {
        TransformIFrame,
        TransformPFrame,
        Voice,
    }
}

pub mod from_server {
    use bincode::{Decode, Encode};

    #[derive(Encode, Decode, Debug)]
    pub enum StreamHeader {
        TransformIFrame,
        TransformPFrame,
        Voice,
        Control,
    }

    #[derive(Encode, Decode, Debug)]
    pub struct TransformMeta {
        pub player: u64,
    }

    #[derive(Encode, Decode, Debug)]
    pub struct VoiceMeta {
        pub player: u64,
    }

    #[derive(Encode, Decode, Debug, Clone)]
    pub enum ControlMessage {
        PlayerLeft { player_id: u64 },
    }
}
