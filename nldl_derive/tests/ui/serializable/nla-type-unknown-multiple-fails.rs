use nldl::attr::UnknownAttribute;
use nldl_derive::NetlinkAttributeSerializable;

#[derive(Debug, PartialEq, NetlinkAttributeSerializable)]
enum TestEnum {
    #[nla_type(_)]
    Unknown(UnknownAttribute),
    #[nla_type(_)]
    Unknown2(UnknownAttribute)
}

fn main() {}
