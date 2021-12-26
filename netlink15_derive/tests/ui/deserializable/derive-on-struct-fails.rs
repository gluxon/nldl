use netlink15_derive::NetlinkAttributeDeserializable;

#[derive(Debug, PartialEq, NetlinkAttributeDeserializable)]
struct TestStruct {}

fn main() {}
