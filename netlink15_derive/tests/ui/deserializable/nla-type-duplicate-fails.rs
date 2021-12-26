use netlink15_derive::NetlinkAttributeDeserializable;
use netlink15_derive::NetlinkAttributeSerializable;
use nldl::utils::ParseNlaIntError;
use nldl::attr::UnknownAttribute;

const ZERO: u16 = 0;

#[derive(Debug, PartialEq, NetlinkAttributeSerializable, NetlinkAttributeDeserializable)]
#[netlink15(deserialize(error = "ParseNlaIntError"))]
enum ControllerAttributeOperation {
    #[nla_type(0)]
    Unspec,
    #[nla_type(crate::ZERO)]
    Id(u32),
    #[nla_type(_)]
    Unknown(UnknownAttribute),
}

fn main() {}
