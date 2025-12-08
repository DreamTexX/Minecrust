use minecrust_protocol::packets::v773::{
    StatusIncoming, StatusOutgoing,
    outgoing::{PongResponse, StatusResponse},
};

use crate::{SERVER_STATE, codec::Codec, connection::Connection};

pub trait Handler {
    fn handle(&mut self) -> impl Future<Output = Result<(), minecrust_protocol::Error>> + Send;
}

#[derive(Debug)]
pub struct StatusHandler<C: Codec> {
    connection: Connection<C>,
}

impl<C: Codec> StatusHandler<C> {
    pub fn new(connection: Connection<C>) -> Self {
        Self { connection }
    }
}

impl<C: Codec + Send> Handler for StatusHandler<C> {
    async fn handle(&mut self) -> Result<(), minecrust_protocol::Error> {
        loop {
            match self.connection.read().await? {
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
                    self.connection.write(response).await?;
                }
                StatusIncoming::PingRequest(packet) => {
                    tracing::debug!("received ping request");

                    let response = StatusOutgoing::PongResponse(PongResponse {
                        timestamp: packet.timestamp,
                    });
                    self.connection.write(response).await?;
                    return Ok(());
                }
            }
        }
    }
}
