extern crate proc_macro;

use crate::parsing::parse_or_panic::parse_or_panic;
use crate::parsing::parse_or_panic::ParseOrPanicReturn;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn impl_netlink_attribute_serializable(ast: &DeriveInput) -> TokenStream {
    let ParseOrPanicReturn {
        name,
        no_payload_idents,
        no_payload_nla_types,
        simple_idents,
        simple_nla_types,
        wildcard_ident,
    } = parse_or_panic(ast);

    // Option structs don't seem to be iterable in the quote! context below. Converting to a Vec as
    // a workaround.
    let wildcard_ident = match wildcard_ident {
        None => vec![],
        Some(ident) => vec![ident],
    };

    quote! {
        impl ::nldl::attr::Serialize for #name {

            fn get_type(&self) -> ::std::primitive::u16 {
                match self {
                    #( Self::#no_payload_idents => #no_payload_nla_types, )*
                    #( Self::#simple_idents(_) => #simple_nla_types, )*
                    #( Self::#wildcard_ident(a) => ::nldl::attr::Serialize::get_type(a), )*
                }
            }

            fn serialize_payload(&self, buf: &mut ::std::vec::Vec<::std::primitive::u8>) {
                match self {
                    #( Self::#no_payload_idents => {}, )*
                    #( Self::#simple_idents(val) => ::nldl::message::NetlinkPayloadRequest::serialize(val, buf), )*
                    #( Self::#wildcard_ident(a) => ::nldl::attr::Serialize::serialize_payload(a, buf), )*
                }
            }
        }
    }
}
