use netlink15_derive::NetlinkAttributeDeserializable;
use netlink15_derive::NetlinkAttributeSerializable;
use netlink15_core::utils::ParseNlaIntError;
use netlink15_core::attr::UnknownAttribute;

#[derive(Debug, PartialEq, NetlinkAttributeSerializable, NetlinkAttributeDeserializable)]
#[netlink15(deserialize(error = "ParseNlaIntError"))]
enum ControllerAttributeOperation {
    #[nla_type(0)]
    Unspec,
    #[nla_type(u16::MAX + 1)]
    Id(u32),
    #[nla_type(_)]
    Unknown(UnknownAttribute),
}

fn main() {}
