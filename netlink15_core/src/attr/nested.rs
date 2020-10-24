use super::NestedAttributesDeserializeError;
use super::NetlinkAttributeDeserializable;
use crate::message::NetlinkPayloadResponse;

#[derive(Debug, PartialEq)]
pub struct Nested<T>(pub Vec<T>);

impl<T: NetlinkAttributeDeserializable> NetlinkAttributeDeserializable for Nested<T> {
    type Error = NestedAttributesDeserializeError<T>;

    fn deserialize(_ty: u16, payload: &[u8]) -> Result<Self, Self::Error> {
        let attributes: Vec<T> = NetlinkPayloadResponse::deserialize(payload)?;
        Ok(Self(attributes))
    }
}
