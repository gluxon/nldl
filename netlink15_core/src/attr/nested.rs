use super::NestedAttributesDeserializeError;
use super::NetlinkAttributeDeserializable;
use super::NetlinkAttributeSerializable;
use crate::message::NetlinkPayloadRequest;
use crate::message::NetlinkPayloadResponse;

/// Wraps zero or more child netlink attributes. The Netlink attribute type ID
/// (`nla_type`) for this container will always be `0`.
#[derive(Debug, PartialEq)]
pub struct Nested<T>(pub Vec<T>);

pub const NESTED_ATTR_NLA_TYPE: u16 = 0;

impl<T: NetlinkAttributeSerializable> NetlinkAttributeSerializable for Nested<T> {
    fn get_type(&self) -> u16 {
        NESTED_ATTR_NLA_TYPE
    }

    fn serialize_payload(&self, buf: &mut Vec<u8>) {
        NetlinkPayloadRequest::serialize(&self.0, buf);
    }
}

impl<T: NetlinkAttributeDeserializable> NetlinkAttributeDeserializable for Nested<T> {
    type Error = NestedAttributesDeserializeError<T>;

    fn deserialize(_ty: u16, payload: &[u8]) -> Result<Self, Self::Error> {
        let attributes: Vec<T> = NetlinkPayloadResponse::deserialize(payload)?;
        Ok(Self(attributes))
    }
}
