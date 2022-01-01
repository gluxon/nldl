use nldl::attr::Nested;
use nldl::attr::UnknownAttribute;

// https://www.infradead.org/~tgr/libnl/doc/api/ctrl_8c_source.html#l00043
#[derive(Debug, PartialEq, nldl::attr::Serialize, nldl::attr::Deserialize)]
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
