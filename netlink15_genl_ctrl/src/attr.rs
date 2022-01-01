use nldl::attr::Nested;
use nldl::attr::ParseNetlinkAttributeFromBufferError;
use nldl::attr::UnknownAttribute;
use nldl::utils::NlaGetStringError;
use nldl::utils::ParseNlaIntError;

// https://www.infradead.org/~tgr/libnl/doc/api/ctrl_8c_source.html#l00043
#[derive(Debug, PartialEq, nldl::attr::Serialize, nldl::attr::Deserialize)]
#[nldl(deserialize(error = "ControllerAttributeDeserializeError"))]
pub enum ControllerAttribute {
    #[nla_type(libc::CTRL_ATTR_UNSPEC as u16)]
    Unspec,
    #[nla_type(libc::CTRL_ATTR_FAMILY_ID as u16)]
    FamilyId(u16),
    #[nla_type(libc::CTRL_ATTR_FAMILY_NAME as u16)]
    FamilyName(String),
    #[nla_type(libc::CTRL_ATTR_VERSION as u16)]
    Version(u32),
    #[nla_type(libc::CTRL_ATTR_HDRSIZE as u16)]
    HeaderSize(u32),
    #[nla_type(libc::CTRL_ATTR_MAXATTR as u16)]
    MaxAttr(u32),
    #[nla_type(libc::CTRL_ATTR_OPS as u16)]
    Operations(Vec<Nested<ControllerAttributeOperation>>),
    #[nla_type(libc::CTRL_ATTR_MCAST_GROUPS as u16)]
    MulticastGroups(Vec<Nested<ControllerAttributeMulticastGroup>>),
    #[nla_type(_)]
    Unknown(UnknownAttribute),
}

#[derive(Debug, PartialEq, nldl::attr::Serialize, nldl::attr::Deserialize)]
#[nldl(deserialize(error = "ParseNlaIntError"))]
pub enum ControllerAttributeOperation {
    #[nla_type(libc::CTRL_ATTR_OP_UNSPEC as u16)]
    Unspec,
    #[nla_type(libc::CTRL_ATTR_OP_ID as u16)]
    Id(u32),
    #[nla_type(libc::CTRL_ATTR_OP_FLAGS as u16)]
    Flags(u32),
    #[nla_type(_)]
    Unknown(UnknownAttribute),
}

#[derive(Debug, PartialEq, nldl::attr::Serialize, nldl::attr::Deserialize)]
#[nldl(deserialize(error = "ControllerAttributeMulticastGroupDeserializeError"))]
pub enum ControllerAttributeMulticastGroup {
    #[nla_type(libc::CTRL_ATTR_MCAST_GRP_UNSPEC as u16)]
    Unspec,
    #[nla_type(libc::CTRL_ATTR_MCAST_GRP_NAME as u16)]
    Name(String),
    #[nla_type(libc::CTRL_ATTR_MCAST_GRP_ID as u16)]
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
    DeserializeBufferError(#[from] ParseNetlinkAttributeFromBufferError),
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
