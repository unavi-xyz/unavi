use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use xdid::core::did::Did;

pub type RpcResult<T> = Result<T, String>;

#[tarpc::service]
pub trait ControlService {
    async fn join_world(id: String) -> RpcResult<()>;
    async fn leave_world(id: String) -> RpcResult<()>;
    async fn worlds() -> Vec<String>;

    async fn players() -> Vec<Player>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    pub did: Did,
}

#[derive(Encode, Decode)]
pub enum StreamHeader {
    Transform,
    Voice,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransformClientMeta {
    pub player: Did,
}
