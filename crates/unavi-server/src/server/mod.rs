use futures::StreamExt;
use tarpc::{
    server::{BaseChannel, Channel},
    tokio_util::codec::{Framed, LengthDelimitedCodec},
};
use tokio_serde::formats::Bincode;
use unavi_server_service::ControlService;
use xwt_wtransport::wtransport::{endpoint::IncomingSession, stream::BiStream};

mod control;

pub async fn handle(incoming: IncomingSession) -> anyhow::Result<()> {
    let req = incoming.await?;
    let con = req.accept().await?;

    let ctrl_task = {
        let stream = con.accept_bi().await?;
        let bi_stream = BiStream::join(stream);
        let framed = Framed::new(bi_stream, LengthDelimitedCodec::default());
        let transport = tarpc::serde_transport::new(framed, Bincode::default());
        let channel = BaseChannel::with_defaults(transport).max_concurrent_requests(2);

        let server = control::ControlServer;

        tokio::spawn(channel.execute(server.serve()).for_each(|res| async move {
            tokio::spawn(res);
        }))
    };

    ctrl_task.await?;

    Ok(())
}
