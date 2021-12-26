use netlink15_derive::NetlinkAttributeDeserializable;
use netlink15_core::attr::UnknownAttribute;

#[derive(Debug, PartialEq, NetlinkAttributeDeserializable)]
enum ControllerAttributeOperation {
    #[nla_type(_)]
    Unknown(UnknownAttribute),
}

fn main() {}
