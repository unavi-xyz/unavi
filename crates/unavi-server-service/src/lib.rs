use serde::{Deserialize, Serialize};
use xdid::core::did::Did;

pub type RpcResult<T> = Result<T, String>;

#[tarpc::service]
pub trait ControlService {
    async fn join_space(id: String) -> RpcResult<()>;
    async fn leave_space(id: String) -> RpcResult<()>;
    async fn spaces() -> Vec<String>;

    async fn players() -> Vec<Player>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    pub did: Did,
}

pub mod from_client {
    use bincode::{Decode, Encode};
    use serde::{Deserialize, Serialize};
    use xdid::core::did::Did;

    #[derive(Encode, Decode)]
    pub enum StreamHeader {
        Transform,
        Voice,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct TransformMeta {
        pub player: Did,
    }
}

pub mod from_server {
    use bincode::{Decode, Encode};
    use serde::{Deserialize, Serialize};

    #[derive(Encode, Decode)]
    pub enum StreamHeader {
        Transform,
        Voice,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct TransformMeta {}
}
