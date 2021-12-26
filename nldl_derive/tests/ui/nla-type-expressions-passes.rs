use nldl_derive::NetlinkAttributeDeserializable;
use nldl_derive::NetlinkAttributeSerializable;
use nldl::attr::UnknownAttribute;
use nldl::utils::ParseNlaIntError;

const ONE: i32 = 1;

#[derive(Debug, PartialEq, NetlinkAttributeSerializable, NetlinkAttributeDeserializable)]
#[netlink15(deserialize(error = "ParseNlaIntError"))]
enum ControllerAttributeOperation {
    #[nla_type(0i32 as u16)]
    Unspec,
    #[nla_type(crate::ONE as u16)]
    Id(u32),
    #[nla_type(_)]
    Unknown(UnknownAttribute),
}

fn main() {}
