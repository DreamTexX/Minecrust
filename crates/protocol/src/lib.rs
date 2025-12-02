mod datatype;
mod deserialize;
mod error;
pub mod packets;
mod serialize;

pub use deserialize::Deserialize;
pub use error::{Error, Result};
pub use serialize::Serialize;
use tokio::io::AsyncRead;

use crate::datatype::VarInt;

pub async fn read_packet_length<R: AsyncRead + Unpin>(reader: &mut R) -> Result<usize> {
    tracing::trace!("reading packet length");
    let length = *VarInt::async_deserialize(reader).await?;
    tracing::trace!(length, "packet length");
    Ok(length as usize)
}
