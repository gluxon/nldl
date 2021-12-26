use nldl_derive::NetlinkAttributeDeserializable;
use nldl_derive::NetlinkAttributeSerializable;
use nldl::attr::UnknownAttribute;
use nldl::utils::ParseNlaIntError;

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
