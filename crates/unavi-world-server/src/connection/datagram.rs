use anyhow::Result;
use xwt_wtransport::{Connection, Datagram};

pub async fn handle_datagram(dgram: Datagram, connection: &Connection) -> Result<()> {
    let str_data = std::str::from_utf8(&dgram.0)?;
    tracing::info!("Datagram: {}", str_data);

    connection.0.send_datagram(b"ACK")?;

    Ok(())
}
