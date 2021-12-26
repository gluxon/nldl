use netlink15_derive::NetlinkAttributeSerializable;

// TODO: Print the more specific multiple associated values error. At the moment it prints a pretty
// version of the error struct.

#[derive(NetlinkAttributeSerializable)]
enum TestEnum {
    #[nla_type(1)]
    Flags(u32, u32)
}

fn main() {}
