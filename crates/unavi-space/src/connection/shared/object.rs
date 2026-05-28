use iroh::endpoint::{
    RecvStream,
    SendStream,
};

pub async fn recv_object_stream(_tx: SendStream, _rx: RecvStream) -> anyhow::Result<()> {
    todo!()
}
