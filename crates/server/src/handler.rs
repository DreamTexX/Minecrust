use minecrust_protocol::{
    Deserialize,
    packets::v773::{
        StatusIncoming, StatusOutgoing,
        outgoing::{PongResponse, StatusResponse},
    },
};

use crate::{SERVER_STATE, connection::Connection};

pub trait Handler {
    fn new(connection: Connection) -> Self;
    fn handle(&mut self) -> impl Future<Output = Result<(), minecrust_protocol::Error>> + Send;
}

#[derive(Debug)]
pub struct StatusHandler {
    connection: Connection,
}

impl Handler for StatusHandler {
    fn new(connection: Connection) -> Self {
        Self { connection }
    }

    async fn handle(&mut self) -> Result<(), minecrust_protocol::Error> {
        loop {
            let mut packet = self.connection.next_packet().await?;
            match StatusIncoming::deserialize(&mut packet)? {
                StatusIncoming::StatusRequest(_) => {
                    tracing::debug!("received status request");

                    let response = StatusOutgoing::StatusResponse(StatusResponse(format!(
                        // 773
                        r#"
                        {{
                            "version": {{
                                "name": "Maintenance",
                                "protocol": 0
                            }},
                            "description": {},
                            "enforcesSecureChat": false
                        }}
                        "#,
                        SERVER_STATE.description.load()
                    )));
                    self.connection.send_packet(response).await?;
                }
                StatusIncoming::PingRequest(packet) => {
                    tracing::debug!("received ping request");

                    let response = StatusOutgoing::PongResponse(PongResponse {
                        timestamp: packet.timestamp,
                    });
                    self.connection.send_packet(response).await?;
                    return Ok(());
                }
            }
        }
    }
}
