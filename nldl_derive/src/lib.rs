use derive::deserializable::impl_netlink_attribute_deserializable;
use derive::serializable::impl_netlink_attribute_serializable;
use proc_macro::TokenStream;

mod derive;
mod parsing;

// Created following pattern from:
// https://doc.rust-lang.org/1.46.0/book/ch19-06-macros.html?highlight=procedural,macros#how-to-write-a-custom-derive-macro
#[proc_macro_derive(NetlinkAttributeSerializable, attributes(nla_type))]
pub fn netlink_attribute_serializable_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("Unable to parse DeriveInput from TokenStream");
    impl_netlink_attribute_serializable(&ast).into()
}

#[proc_macro_derive(NetlinkAttributeDeserializable, attributes(netlink15, nla_type))]
pub fn netlink_attribute_deserializable_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("Unable to parse DeriveInput from TokenStream");
    impl_netlink_attribute_deserializable(&ast).into()
}
