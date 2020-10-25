use netlink15_core::attr::{Nested, ParseNetlinkAttributeFromBufferError, UnknownAttribute};
use netlink15_core::utils::NlaGetStringError;
use netlink15_core::utils::ParseNlaIntError;
use netlink15_derive::NetlinkAttributeDeserializable;
use netlink15_derive::NetlinkAttributeSerializable;

const CTRL_ATTR_UNSPEC: u16 = libc::CTRL_ATTR_UNSPEC as u16;
const CTRL_ATTR_FAMILY_ID: u16 = libc::CTRL_ATTR_FAMILY_ID as u16;
const CTRL_ATTR_FAMILY_NAME: u16 = libc::CTRL_ATTR_FAMILY_NAME as u16;
const CTRL_ATTR_VERSION: u16 = libc::CTRL_ATTR_VERSION as u16;
const CTRL_ATTR_HDRSIZE: u16 = libc::CTRL_ATTR_HDRSIZE as u16;
const CTRL_ATTR_MAXATTR: u16 = libc::CTRL_ATTR_MAXATTR as u16;
const CTRL_ATTR_OPS: u16 = libc::CTRL_ATTR_OPS as u16;
const CTRL_ATTR_MCAST_GROUPS: u16 = libc::CTRL_ATTR_MCAST_GROUPS as u16;

// https://www.infradead.org/~tgr/libnl/doc/api/ctrl_8c_source.html#l00043
#[derive(Debug, PartialEq, NetlinkAttributeSerializable, NetlinkAttributeDeserializable)]
#[netlink15(deserialize(error = "ControllerAttributeDeserializeError"))]
pub enum ControllerAttribute {
    #[nla_type(CTRL_ATTR_UNSPEC)]
    Unspec,
    #[nla_type(CTRL_ATTR_FAMILY_ID)]
    FamilyId(u16),
    #[nla_type(CTRL_ATTR_FAMILY_NAME)]
    FamilyName(String),
    #[nla_type(CTRL_ATTR_VERSION)]
    Version(u32),
    #[nla_type(CTRL_ATTR_HDRSIZE)]
    HeaderSize(u32),
    #[nla_type(CTRL_ATTR_MAXATTR)]
    MaxAttr(u32),
    #[nla_type(CTRL_ATTR_OPS)]
    Operations(Nested<ControllerAttributeOperation>),
    #[nla_type(CTRL_ATTR_MCAST_GROUPS)]
    MulticastGroups(Nested<ControllerAttributeMulticastGroup>),
    #[nla_type(_)]
    Unknown(UnknownAttribute),
}

const CTRL_ATTR_OP_UNSPEC: u16 = libc::CTRL_ATTR_OP_UNSPEC as u16;
const CTRL_ATTR_OP_ID: u16 = libc::CTRL_ATTR_OP_ID as u16;
const CTRL_ATTR_OP_FLAGS: u16 = libc::CTRL_ATTR_OP_FLAGS as u16;

#[derive(Debug, PartialEq, NetlinkAttributeSerializable, NetlinkAttributeDeserializable)]
#[netlink15(deserialize(error = "ParseNlaIntError"))]
pub enum ControllerAttributeOperation {
    #[nla_type(CTRL_ATTR_OP_UNSPEC)]
    Unspec,
    #[nla_type(CTRL_ATTR_OP_ID)]
    Id(u32),
    #[nla_type(CTRL_ATTR_OP_FLAGS)]
    Flags(u32),
    #[nla_type(_)]
    Unknown(UnknownAttribute),
}

const CTRL_ATTR_MCAST_GRP_UNSPEC: u16 = libc::CTRL_ATTR_MCAST_GRP_UNSPEC as u16;
const CTRL_ATTR_MCAST_GRP_NAME: u16 = libc::CTRL_ATTR_MCAST_GRP_NAME as u16;
const CTRL_ATTR_MCAST_GRP_ID: u16 = libc::CTRL_ATTR_MCAST_GRP_ID as u16;

#[derive(Debug, PartialEq, NetlinkAttributeSerializable, NetlinkAttributeDeserializable)]
#[netlink15(deserialize(error = "ControllerAttributeMulticastGroupDeserializeError"))]
pub enum ControllerAttributeMulticastGroup {
    #[nla_type(CTRL_ATTR_MCAST_GRP_UNSPEC)]
    Unspec,
    #[nla_type(CTRL_ATTR_MCAST_GRP_NAME)]
    Name(String),
    #[nla_type(CTRL_ATTR_MCAST_GRP_ID)]
    Id(u32),
    #[nla_type(_)]
    Unknown(UnknownAttribute),
}

#[derive(thiserror::Error, Debug)]
pub enum ControllerAttributeDeserializeError {
    #[error(transparent)]
    ParseNlaIntError(#[from] ParseNlaIntError),
    #[error(transparent)]
    NlaGetStringError(#[from] NlaGetStringError),
    #[error(transparent)]
    DeserializeOperationError(
        #[from] ParseNetlinkAttributeFromBufferError<Nested<ControllerAttributeOperation>>,
    ),
    #[error(transparent)]
    DeserializeMulticastGroupError(
        #[from] ParseNetlinkAttributeFromBufferError<Nested<ControllerAttributeMulticastGroup>>,
    ),
}

#[derive(thiserror::Error, Debug)]
pub enum ControllerAttributeMulticastGroupDeserializeError {
    #[error(transparent)]
    ParseNlaIntError(#[from] ParseNlaIntError),
    #[error(transparent)]
    NlaGetStringError(#[from] NlaGetStringError),
}

#[derive(thiserror::Error, Debug)]
pub enum ControllerAttributeOperationDeserializeError {
    #[error(transparent)]
    ParseNlaIntError(#[from] ParseNlaIntError),
    #[error(transparent)]
    NlaGetStringError(#[from] NlaGetStringError),
}
