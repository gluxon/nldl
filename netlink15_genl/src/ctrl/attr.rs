use netlink15_core::attr::{
    Nested, NetlinkAttributeDeserializable, NetlinkAttributeSerializable, UnknownAttribute,
};
use netlink15_core::utils::nla_get_string;
use netlink15_core::utils::nla_get_u16;
use netlink15_core::utils::nla_get_u32;
use netlink15_core::utils::NlaGetStringError;
use netlink15_core::utils::ParseNlaIntError;
use netlink15_derive::NetlinkAttributeSerializable;

// https://www.infradead.org/~tgr/libnl/doc/api/ctrl_8c_source.html#l00043
#[derive(Debug, PartialEq)]
pub enum ControllerAttribute {
    Unspec,
    FamilyId(u16),
    FamilyName(String),
    Version(u32),
    HeaderSize(u32),
    MaxAttr(u32),
    Operations(Nested<ControllerAttributeOperation>),
    MulticastGroups(Nested<ControllerAttributeMulticastGroup>),
    Unknown { ty: u16, payload: Vec<u8> },
}

#[derive(Debug, PartialEq, NetlinkAttributeSerializable)]
pub enum ControllerAttributeOperation {
    #[netlink15(nla_type = "libc::CTRL_ATTR_OP_UNSPEC as u16")]
    Unspec,
    #[netlink15(nla_type = "libc::CTRL_ATTR_OP_ID as u16")]
    Id(u32),
    #[netlink15(nla_type = "libc::CTRL_ATTR_OP_FLAGS as u16")]
    Flags(u32),
    #[netlink15(nla_type_unknown)]
    Unknown(UnknownAttribute),
}

#[derive(Debug, PartialEq)]
pub enum ControllerAttributeMulticastGroup {
    Unspec,
    Name(String),
    Id(u32),
    Unknown { ty: u16, payload: Vec<u8> },
}

impl NetlinkAttributeSerializable for ControllerAttribute {
    fn get_type(&self) -> u16 {
        match self {
            ControllerAttribute::Unspec => libc::CTRL_ATTR_UNSPEC as u16,
            ControllerAttribute::FamilyId(_) => libc::CTRL_ATTR_FAMILY_ID as u16,
            ControllerAttribute::FamilyName(_) => libc::CTRL_ATTR_FAMILY_NAME as u16,
            ControllerAttribute::Version(_) => libc::CTRL_ATTR_VERSION as u16,
            ControllerAttribute::HeaderSize(_) => libc::CTRL_ATTR_HDRSIZE as u16,
            ControllerAttribute::MaxAttr(_) => libc::CTRL_ATTR_MAXATTR as u16,
            ControllerAttribute::Operations(_) => libc::CTRL_ATTR_OPS as u16,
            ControllerAttribute::MulticastGroups(_) => libc::CTRL_ATTR_MCAST_GROUPS as u16,
            ControllerAttribute::Unknown { ty, payload: _ } => *ty,
        }
    }

    fn serialize_payload(&self, buf: &mut Vec<u8>) {
        match self {
            ControllerAttribute::Unspec => {}
            ControllerAttribute::FamilyId(family_id) => {
                buf.extend_from_slice(&family_id.to_ne_bytes()[..]);
            }
            ControllerAttribute::FamilyName(family_name) => {
                buf.extend_from_slice(family_name.as_bytes());
                buf.push(0);
            }
            ControllerAttribute::Version(_) => todo!(),
            ControllerAttribute::HeaderSize(_) => todo!(),
            ControllerAttribute::MaxAttr(_) => todo!(),
            ControllerAttribute::Operations(_) => todo!(),
            ControllerAttribute::MulticastGroups(_) => todo!(),
            ControllerAttribute::Unknown { ty: _, payload } => {
                buf.extend_from_slice(payload);
            }
        };
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ControllerAttributeDeserializeError {
    #[error(transparent)]
    ParseNlaIntError(#[from] ParseNlaIntError),
    #[error(transparent)]
    NlaGetStringError(#[from] NlaGetStringError),
}

impl NetlinkAttributeDeserializable for ControllerAttribute {
    type Error = ControllerAttributeDeserializeError;

    fn deserialize(ty: u16, payload: &[u8]) -> Result<Self, Self::Error> {
        let attr = match ty.into() {
            libc::CTRL_ATTR_UNSPEC => ControllerAttribute::Unspec,
            libc::CTRL_ATTR_FAMILY_ID => ControllerAttribute::FamilyId(nla_get_u16(payload)?),
            libc::CTRL_ATTR_FAMILY_NAME => {
                ControllerAttribute::FamilyName(nla_get_string(payload)?)
            }
            libc::CTRL_ATTR_VERSION => ControllerAttribute::Version(nla_get_u32(payload)?),
            libc::CTRL_ATTR_HDRSIZE => ControllerAttribute::HeaderSize(nla_get_u32(payload)?),
            libc::CTRL_ATTR_MAXATTR => ControllerAttribute::MaxAttr(nla_get_u32(payload)?),
            libc::CTRL_ATTR_OPS => ControllerAttribute::Operations(
                NetlinkAttributeDeserializable::deserialize(0, &payload[4..]).unwrap(),
            ),
            libc::CTRL_ATTR_MCAST_GROUPS => ControllerAttribute::MulticastGroups(
                NetlinkAttributeDeserializable::deserialize(0, &payload[4..]).unwrap(),
            ),
            _ => ControllerAttribute::Unknown {
                ty,
                payload: payload.into(),
            },
        };

        Ok(attr)
    }
}

impl NetlinkAttributeDeserializable for ControllerAttributeOperation {
    type Error = ParseNlaIntError;

    fn deserialize(ty: u16, payload: &[u8]) -> Result<Self, Self::Error> {
        let attr = match ty.into() {
            libc::CTRL_ATTR_OP_UNSPEC => ControllerAttributeOperation::Unspec,
            libc::CTRL_ATTR_OP_ID => ControllerAttributeOperation::Id(nla_get_u32(payload)?),
            libc::CTRL_ATTR_OP_FLAGS => ControllerAttributeOperation::Flags(nla_get_u32(payload)?),
            _ => ControllerAttributeOperation::Unknown(UnknownAttribute {
                ty,
                payload: payload.into(),
            }),
        };

        Ok(attr)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ControllerAttributeOperationDeserializeError {
    #[error(transparent)]
    ParseNlaIntError(#[from] ParseNlaIntError),
    #[error(transparent)]
    NlaGetStringError(#[from] NlaGetStringError),
}

impl NetlinkAttributeDeserializable for ControllerAttributeMulticastGroup {
    type Error = ControllerAttributeOperationDeserializeError;

    fn deserialize(ty: u16, payload: &[u8]) -> Result<Self, Self::Error> {
        let attr = match ty.into() {
            libc::CTRL_ATTR_MCAST_GRP_UNSPEC => ControllerAttributeMulticastGroup::Unspec,
            libc::CTRL_ATTR_MCAST_GRP_NAME => {
                ControllerAttributeMulticastGroup::Name(nla_get_string(payload)?)
            }
            libc::CTRL_ATTR_MCAST_GRP_ID => {
                ControllerAttributeMulticastGroup::Id(nla_get_u32(payload)?)
            }
            _ => ControllerAttributeMulticastGroup::Unknown {
                ty,
                payload: payload.into(),
            },
        };

        Ok(attr)
    }
}
