use netlink15_derive::NetlinkAttributeDeserializable;
use netlink15_derive::NetlinkAttributeSerializable;
use netlink15_core::attr::UnknownAttribute;
use netlink15_core::utils::ParseNlaIntError;

#[derive(Debug, PartialEq, NetlinkAttributeSerializable, NetlinkAttributeDeserializable)]
#[netlink15(deserialize(error = "ParseNlaIntError"))]
enum ControllerAttributeOperation {
    #[nla_type(0)]
    Unspec,
    #[nla_type(1)]
    Id(u32),
    #[nla_type(2)]
    Flags(u32),
    #[nla_type(_)]
    Unknown(UnknownAttribute),
}

fn main() {}
