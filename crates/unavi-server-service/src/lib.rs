use serde::{Deserialize, Serialize};
use serde_vrm::vrm0::BoneName;
use xdid::core::did::Did;

pub type RpcResult<T> = Result<T, String>;

#[tarpc::service]
pub trait ControlService {
    async fn tickrate_ms() -> u64;

    async fn join_space(id: String) -> RpcResult<()>;
    async fn leave_space(id: String) -> RpcResult<()>;
    async fn spaces() -> Vec<String>;

    async fn players() -> Vec<Player>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    pub did: Did,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TrackingUpdate {
    IFrame(TrackingIFrame),
    PFrame(TrackingPFrame),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TrackingIFrame {
    pub pos: [f32; 3],
    pub joints: Vec<JointIFrame>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TrackingPFrame {
    pub pos: [i16; 3],
    pub joints: Vec<JointPFrame>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JointIFrame {
    pub id: u8,
    pub rot: [f32; 4],
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JointPFrame {
    pub id: BoneName,
    pub rot: [i16; 4],
}

pub mod from_client {
    use bincode::{Decode, Encode};

    #[derive(Encode, Decode)]
    pub enum StreamHeader {
        Transform,
        Voice,
    }

    #[derive(Encode, Decode, Debug)]
    pub struct TransformMeta {}
}

pub mod from_server {
    use bincode::{Decode, Encode};

    #[derive(Encode, Decode)]
    pub enum StreamHeader {
        Transform,
        Voice,
    }

    #[derive(Encode, Decode, Debug)]
    pub struct TransformMeta {
        pub player: u64,
    }

    #[derive(Encode, Decode, Debug)]
    pub struct VoiceMeta {
        pub player: u64,
    }
}
