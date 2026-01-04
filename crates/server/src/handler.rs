use aes::Aes128;
use cipher::KeyIvInit;
use minecrust_protocol::{
    datatype::{GameProfile, TextComponent},
    packets::{
        inbound::{login::LoginPacket, status::StatusPacket},
        outbound::{
            login::{Hello, LoginDisconnect, LoginFinished},
            status::{PongResponse, StatusResponse},
        },
    },
};
use rand::Rng;
use rsa::Pkcs1v15Encrypt;

use crate::{
    codec::Codec,
    connection::{Connection, start_encryption},
};

pub enum Decision {
    End,
    Continue(Box<dyn Handler>),
}

pub trait Handler {
    fn handle(self) -> impl Future<Output = Result<Decision, minecrust_protocol::Error>> + Send
    where
        Self: Sized;
}

pub struct StatusHandler<C: Codec> {
    connection: Connection<C>,
}

impl<C: Codec> StatusHandler<C> {
    pub fn new(connection: Connection<C>) -> Self {
        Self { connection }
    }
}

impl<C: Codec + Send> Handler for StatusHandler<C> {
    async fn handle(mut self) -> Result<Decision, minecrust_protocol::Error> {
        let StatusPacket::StatusRequest(_) = self.connection.read().await? else {
            return Err(minecrust_protocol::Error::UnexpectedPacket);
        };
        tracing::debug!("received status request");

        let response = StatusResponse(format!(
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
            //SERVER_STATE.description.load()
            "Todo: Reimplement description"
        ));
        self.connection.write(response).await?;

        let StatusPacket::PingRequest(ping_request) = self.connection.read().await? else {
            return Err(minecrust_protocol::Error::UnexpectedPacket);
        };
        tracing::debug!("received ping request");

        let response = PongResponse(ping_request.0);
        self.connection.write(response).await;

        Ok(Decision::End)
    }
}

pub struct LoginHandler<C: Codec> {
    connection: Connection<C>,
    verification_token: [u8; 32],
}

impl<C: Codec> LoginHandler<C> {
    pub fn new(connection: Connection<C>) -> Self {
        let mut verification_token = [0u8; 32];
        rand::thread_rng().fill(&mut verification_token);

        Self {
            connection,
            verification_token,
        }
    }
}

impl<C: Codec + Send> Handler for LoginHandler<C> {
    type Value = ();

    async fn handle(mut self) -> Result<(), minecrust_protocol::Error> {
        let LoginPacket::Hello(hello_packet) = self.connection.read().await? else {
            return Err(minecrust_protocol::Error::UnexpectedPacket);
        };
        tracing::trace!(?hello_packet, "hello packed received");

        let response = Hello {
            server_id: String::new(),
            public_key: SERVER_STATE.encryption.2.clone(),
            verify_token: self.verification_token,
            should_authenticate: true,
        };
        self.connection.write(response).await?;

        let LoginPacket::Key(key_packet) = self.connection.read().await? else {
            return Err(minecrust_protocol::Error::UnexpectedPacket);
        };

        let private_key = &SERVER_STATE.encryption.0;
        let decrypted_verify_key = private_key
            .decrypt(Pkcs1v15Encrypt, &key_packet.verify_token)
            .unwrap();
        let decrypted_shared_secret = private_key
            .decrypt(Pkcs1v15Encrypt, &key_packet.shared_secret)
            .unwrap();
        let shared_secret = decrypted_shared_secret.as_slice();

        if decrypted_verify_key != self.verification_token {
            self.connection
                .write(LoginDisconnect(TextComponent(
                    r#"{"type":"text","text":"Unsecure connection."}"#.to_string(),
                )))
                .await?;
            return Ok(());
        }

        self.connection = start_encryption(self.connection, shared_secret);

        self.connection
            .write(LoginFinished(GameProfile {
                username: hello_packet.name,
                uuid: hello_packet.player_uuid,
                properties: vec![],
            }))
            .await?;

        let LoginPacket::LoginAcknowledged(login_acknowledged_packet) =
            self.connection.read::<LoginPacket>().await?
        else {
            return Err(minecrust_protocol::Error::UnexpectedPacket);
        };
        tracing::trace!(?login_acknowledged_packet, "login successful");

        Ok(())
    }
}
