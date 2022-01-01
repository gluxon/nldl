use super::linux::nlmsg_align;
use super::message::NetlinkPayloadRequest;
use super::message::NetlinkPayloadResponse;
use super::write_to_buf_with_prefixed_u16_len;
use raw::RawNetlinkAttribute;
use std::fmt::Debug;
pub use unknown::UnknownAttribute;

mod nested;
mod raw;
mod unknown;

pub use nested::Nested;
pub use raw::ParseRawNetlinkAttributeError;

pub trait Serialize {
    fn get_type(&self) -> u16;
    fn serialize_payload(&self, buf: &mut Vec<u8>);
}

pub trait Deserialize: Debug + Sized + PartialEq {
    type Error: Debug + std::error::Error;
    fn deserialize(ty: u16, payload: &[u8]) -> Result<Self, Self::Error>;
}

impl<T: Serialize> NetlinkPayloadRequest for T {
    fn serialize(&self, buf: &mut Vec<u8>) {
        write_to_buf_with_prefixed_u16_len(buf, |buf| {
            buf.extend_from_slice(&self.get_type().to_ne_bytes()[..]);
            self.serialize_payload(buf);
        });
    }
}

impl<T: Deserialize> NetlinkPayloadResponse for T {
    type Error = ParseNetlinkAttributeFromBufferError<T>;

    fn deserialize(buf: &[u8]) -> Result<Self, Self::Error> {
        let raw = RawNetlinkAttribute::try_from(buf)?;
        Deserialize::deserialize(raw.ty, raw.payload)
            .map_err(ParseNetlinkAttributeFromBufferError::AttributeDeserializeError)
    }
}

impl<T: Serialize> NetlinkPayloadRequest for Vec<T> {
    fn serialize(&self, buf: &mut Vec<u8>) {
        for attr in self {
            attr.serialize(buf);
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseNetlinkAttributeFromBufferError<T: Deserialize> {
    #[error(
        "An error occurred partitioning a buffer into netlink attribute
    #(len, type, payload) fields: {0}"
    )]
    PartitionBufferError(#[from] ParseRawNetlinkAttributeError),

    // There's a cryptic compiler error message when #[error(transparent)] is
    // set on the generic below.
    #[error("{0}")]
    AttributeDeserializeError(T::Error),
}

impl<T: Deserialize> NetlinkPayloadResponse for Vec<T> {
    type Error = ParseNetlinkAttributeFromBufferError<T>;

    fn deserialize(buf: &[u8]) -> Result<Self, Self::Error> {
        let mut attrs = vec![];
        let mut view = buf;

        while !view.is_empty() {
            let raw = RawNetlinkAttribute::try_from(view)?;
            let RawNetlinkAttribute { len, ty, payload } = raw;

            view = &view[nlmsg_align(usize::from(len))..];

            let attr = T::deserialize(ty, payload)
                .map_err(ParseNetlinkAttributeFromBufferError::AttributeDeserializeError)?;
            attrs.push(attr);
        }

        Ok(attrs)
    }
}

#[cfg(feature = "nldl_derive")]
pub use nldl_derive::NetlinkAttributeDeserializable as Deserialize;
#[cfg(feature = "nldl_derive")]
pub use nldl_derive::NetlinkAttributeSerializable as Serialize;
