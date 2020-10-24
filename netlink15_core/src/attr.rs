use super::linux::nlmsg_align;
use super::message::{NetlinkPayloadRequest, NetlinkPayloadResponse};
use super::utils::{nla_get_u16, ParseNlaIntError};
use super::write_to_buf_with_prefixed_u16_len;
use std::fmt::Debug;
use std::mem::size_of;

pub trait NetlinkAttributeSerializable {
    fn get_type(&self) -> u16;
    fn serialize_payload(&self, buf: &mut Vec<u8>);
}

pub trait NetlinkAttributeDeserializable: Debug + Sized + PartialEq {
    type Error: Debug + std::error::Error;
    fn deserialize(ty: u16, payload: &[u8]) -> Result<Self, Self::Error>;
}

impl<T: NetlinkAttributeSerializable> NetlinkPayloadRequest for T {
    fn serialize(&self, buf: &mut Vec<u8>) {
        write_to_buf_with_prefixed_u16_len(buf, |buf| {
            buf.extend_from_slice(&self.get_type().to_ne_bytes()[..]);
            self.serialize_payload(buf);
        });
    }
}

impl<T: NetlinkAttributeSerializable> NetlinkPayloadRequest for Vec<T> {
    fn serialize(&self, buf: &mut Vec<u8>) {
        for attr in self {
            attr.serialize(buf);
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum NestedAttributesDeserializeError<T: NetlinkAttributeDeserializable> {
    #[error(transparent)]
    ParseNlaIntError(#[from] ParseNlaIntError),

    // There's a cryptic compiler error message when #[error(transparent)] is
    // set on the generic below.
    #[error("{0}")]
    ChildAttributeDeserializeError(T::Error),
}

impl<T: NetlinkAttributeDeserializable> NetlinkPayloadResponse for Vec<T> {
    type Error = NestedAttributesDeserializeError<T>;

    fn deserialize(buf: &[u8]) -> Result<Self, Self::Error> {
        let mut attrs = vec![];
        let mut view = &buf[..];

        while !view.is_empty() {
            let (header_bytes, remaining) = view.split_at(size_of::<libc::nlattr>());

            let len = nla_get_u16(&header_bytes[0..size_of::<u16>()]).map(usize::from)?;
            let ty = nla_get_u16(&header_bytes[size_of::<u16>()..2 * size_of::<u16>()])?;
            let payload = {
                let payload_len = len - size_of::<libc::nlattr>();
                &remaining[..payload_len]
            };

            view = &view[nlmsg_align(len)..];

            let attr = T::deserialize(ty, payload)
                .map_err(NestedAttributesDeserializeError::ChildAttributeDeserializeError)?;
            attrs.push(attr);
        }

        Ok(attrs)
    }
}

#[derive(Debug, PartialEq)]
pub struct Nested<T>(pub Vec<T>);

impl<T: NetlinkAttributeDeserializable> NetlinkAttributeDeserializable for Nested<T> {
    type Error = NestedAttributesDeserializeError<T>;

    fn deserialize(_ty: u16, payload: &[u8]) -> Result<Self, Self::Error> {
        let attributes: Vec<T> = NetlinkPayloadResponse::deserialize(payload)?;
        Ok(Self(attributes))
    }
}

#[derive(Debug, PartialEq)]
pub struct UnknownAttribute {
    pub ty: u16,
    pub payload: Vec<u8>,
}

impl NetlinkAttributeSerializable for UnknownAttribute {
    fn get_type(&self) -> u16 {
        self.ty
    }

    fn serialize_payload(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.payload[..])
    }
}
