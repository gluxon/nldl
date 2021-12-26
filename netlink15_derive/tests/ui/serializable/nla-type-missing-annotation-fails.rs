use nldl::attr::UnknownAttribute;
use netlink15_derive::NetlinkAttributeSerializable;

#[derive(Debug, PartialEq, NetlinkAttributeSerializable)]
enum ControllerAttributeOperation {
    #[nla_type(0)]
    Unspec,
    #[nla_type(1)]
    Id(u32),
    #[nla_type(2)]
    Flags(u32),
    Unknown(UnknownAttribute)
}

fn main() {}
