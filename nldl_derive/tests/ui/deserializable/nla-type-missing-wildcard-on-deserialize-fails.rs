use nldl_derive::NetlinkAttributeDeserializable;

#[derive(Debug, PartialEq, NetlinkAttributeDeserializable)]
#[netlink15(deserialize(error = "ParseNlaIntError"))]
enum ControllerAttributeOperation {
    #[nla_type(0)]
    Unspec,
}

fn main() {}
