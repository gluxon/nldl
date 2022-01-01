use super::Deserialize;
use super::Serialize;

#[derive(Debug, PartialEq)]
pub struct UnknownAttribute {
    pub ty: u16,
    pub payload: Vec<u8>,
}

impl Serialize for UnknownAttribute {
    fn get_type(&self) -> u16 {
        self.ty
    }

    fn serialize_payload(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.payload[..])
    }
}

impl Deserialize for UnknownAttribute {
    type Error = std::convert::Infallible;

    fn deserialize(ty: u16, payload: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self {
            ty,
            payload: Vec::from(payload),
        })
    }
}
