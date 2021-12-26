use nldl_derive::NetlinkAttributeDeserializable;
use nldl::attr::UnknownAttribute;

#[derive(Debug, PartialEq, NetlinkAttributeDeserializable)]
enum ControllerAttributeOperation {
    #[nla_type(_)]
    Unknown(UnknownAttribute),
}

fn main() {}
