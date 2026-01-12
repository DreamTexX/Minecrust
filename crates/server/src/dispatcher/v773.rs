use minecrust_codec::packet::RawPacket;
use minecrust_protocol::{
    datatype::{GameProfile, TextComponent, VarInt},
    packet::v773::{
        client::{
            self,
            status::{PongResponse, StatusResponse},
        },
        server::{self, status::PingRequest},
    },
};
use rand::Rng;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey, pkcs8::EncodePublicKey};
use uuid::Uuid;

use crate::{
    connection::{Action, ConnectionError, ProtocolState},
    dispatcher::Dispatcher,
};

pub(crate) struct StatusDispatcher;

impl Dispatcher for StatusDispatcher {
    fn dispatch(&mut self, raw_packet: RawPacket) -> Result<Vec<Action>, ConnectionError> {
        let mut actions = vec![];
        match *raw_packet.id {
            // Status Request
            0x00 => {
                actions.push(Action::SendPacket(
                    (
                        0x00,
                        StatusResponse(format!(
                            // 773
                            r#"{{ "version": {{ "name": "Maintenance", "protocol": 0 }}, "description": {{ "text": "{}" }}, "enforcesSecureChat": false }}"#,
                            //SERVER_STATE.description.load()
                            "Todo: Reimplement description"
                        )),
                    )
                        .into(),
                ));
            }
            // Ping Request
            0x01 => {
                let ping_request: PingRequest = raw_packet.try_into()?;
                actions.push(Action::SendPacket(
                    (0x01, PongResponse(ping_request.0)).into(),
                ));
            }
            _ => {}
        }

        Ok(actions)
    }
}

pub(crate) struct LoginDispatcher {
    verification_token: [u8; 32],
    private_key: RsaPrivateKey,
    public_key: Vec<u8>,
    username: Option<String>,
    uuid: Option<Uuid>,
}

impl LoginDispatcher {
    pub fn new() -> Self {
        let rng = &mut rand::thread_rng();
        let mut verification_token = [0u8; 32];
        rng.fill(&mut verification_token);
        let private_key = RsaPrivateKey::new(rng, 1024).unwrap();
        let public_key = RsaPublicKey::from(&private_key)
            .to_public_key_der()
            .unwrap()
            .to_vec();

        Self {
            verification_token,
            private_key,
            public_key,
            username: None,
            uuid: None,
        }
    }
}

impl Dispatcher for LoginDispatcher {
    fn dispatch(&mut self, raw_packet: RawPacket) -> Result<Vec<Action>, ConnectionError> {
        let mut actions = vec![];
        match *raw_packet.id {
            0x00 => {
                let server::login::Hello { name, player_uuid } = raw_packet.try_into()?;
                tracing::trace!(name, ?player_uuid, "hello");
                self.username = Some(name);
                self.uuid = Some(player_uuid);

                actions.push(Action::SendPacket(
                    (
                        0x01,
                        client::login::Hello {
                            server_id: String::new(),
                            public_key: self.public_key.clone(),
                            should_authenticate: true,
                            verify_token: self.verification_token,
                        },
                    )
                        .into(),
                ));
            }
            0x01 => {
                let server::login::Key {
                    shared_secret,
                    verify_token,
                } = raw_packet.try_into()?;
                let verification_token = self
                    .private_key
                    .decrypt(Pkcs1v15Encrypt, &verify_token)
                    .unwrap();
                if verification_token != self.verification_token {
                    actions.push(Action::SendPacket(
                        (
                            0x00,
                            client::login::LoginDisconnect(TextComponent(
                                r#"{"type":"text","text":"Unsecure connection."}"#.to_string(),
                            )),
                        )
                            .into(),
                    ));
                    // Actions::Disconnect
                }
                let shared_secret = self
                    .private_key
                    .decrypt(Pkcs1v15Encrypt, &shared_secret)
                    .unwrap();
                actions.push(Action::EnableEncryption(
                    shared_secret.as_slice().try_into().unwrap(),
                ));
                actions.push(Action::SendPacket(
                    (0x03, client::login::LoginCompression(VarInt::from(256))).into(),
                ));
                actions.push(Action::EnableCompression(256));
                actions.push(Action::SendPacket(
                    (
                        0x02,
                        client::login::LoginFinished(GameProfile {
                            username: self.username.clone().unwrap(),
                            uuid: self.uuid.unwrap(),
                            properties: vec![],
                        }),
                    )
                        .into(),
                ));
            }
            0x03 => {
                actions.push(Action::ProtocolState(ProtocolState::Configuration));
            }
            _ => {}
        }
        Ok(actions)
    }
}
