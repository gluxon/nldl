extern crate proc_macro;

use crate::parsing::nla_type::PartitionedAttributeKinds;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Data;
use syn::DeriveInput;

pub fn impl_netlink_attribute_serializable(ast: &DeriveInput) -> TokenStream {
    let data_enum = match &ast.data {
        Data::Enum(data_enum) => data_enum,
        _ => panic!("nldl::attr::Serialize derive may only be used on enums."),
    };
    let partitioned_variants = PartitionedAttributeKinds::from(data_enum).unwrap_or_else(|err| {
        panic!(
            "Failed to parse enum variants in NetlinkAttributeSerializable derive: {}",
            err
        )
    });
    if let Some(unmarked_variant) = partitioned_variants.unmarked.first() {
        panic!(
            "Please annotate all enum variants with #[nla_type(..)]. Saw \"{}\" unannotated.",
            unmarked_variant.ident
        );
    }
    let wildcard_ident = match &partitioned_variants.wildcard[..] {
        [variant] => vec![variant.ident],
        [] => vec![],
        [..] => panic!(
            "Only 1 variant may be marked with #[nla_type(_)]. Saw {}",
            partitioned_variants.wildcard.len()
        ),
    };

    let name = &ast.ident;
    let (no_payload_idents, no_payload_nla_types) = partitioned_variants
        .no_payload
        .into_iter()
        .fold((vec![], vec![]), |mut acc, attr| {
            acc.0.push(attr.ident);
            acc.1.push(attr.ty);
            acc
        });
    let (simple_idents, simple_nla_types) =
        partitioned_variants
            .simple
            .into_iter()
            .fold((vec![], vec![]), |mut acc, attr| {
                acc.0.push(attr.ident);
                acc.1.push(attr.ty);
                acc
            });

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
