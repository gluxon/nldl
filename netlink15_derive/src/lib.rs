extern crate proc_macro;

use parsing::nla_type::PartitionedAttributeKinds;
use proc_macro::TokenStream;
use quote::quote;

mod parsing;

// Created following pattern from:
// https://doc.rust-lang.org/1.46.0/book/ch19-06-macros.html?highlight=procedural,macros#how-to-write-a-custom-derive-macro
#[proc_macro_derive(NetlinkAttributeSerializable, attributes(nla_type))]
pub fn netlink_attribute_serializable_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("Unable to parse DeriveInput from TokenStream");
    impl_netlink_attribute_serializable(&ast).into()
}

fn impl_netlink_attribute_serializable(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let data_enum = match &ast.data {
        syn::Data::Enum(data_enum) => data_enum,
        _ => panic!("NetlinkAttributeSerializable derive may only be used on enums."),
    };
    let partitioned_variants = PartitionedAttributeKinds::from(data_enum)
        .expect("Failed to parse enum variants in NetlinkAttributeSerializable derive.");
    if let Some(unmarked_variant) = partitioned_variants.unmarked.first() {
        panic!(
            "Please annotate all enum variants with #[nla_type(..)]. Saw \"{}\" unannotated.",
            unmarked_variant.ident
        );
    }
    let unknown_ident = match &partitioned_variants.unknown[..] {
        [variant] => vec![variant.ident],
        [] => vec![],
        [..] => panic!(
            "Only 1 variant may be marked with #[nla_type(_)]. Saw {}",
            partitioned_variants.unknown.len()
        ),
    };

    let name = &ast.ident;
    let (none_idents, none_nla_types) =
        partitioned_variants
            .none
            .into_iter()
            .fold((vec![], vec![]), |mut acc, attr| {
                acc.0.push(attr.ident);
                acc.1.push(attr.ty);
                acc
            });
    let (some_idents, some_nla_types) =
        partitioned_variants
            .some
            .into_iter()
            .fold((vec![], vec![]), |mut acc, attr| {
                acc.0.push(attr.ident);
                acc.1.push(attr.ty);
                acc
            });

    quote! {
        impl netlink15_core::attr::NetlinkAttributeSerializable for #name {

            fn get_type(&self) -> u16 {
                use netlink15_core::attr::NetlinkAttributeSerializable;

                match self {
                    #( Self::#none_idents => #none_nla_types, )*
                    #( Self::#some_idents(_) => #some_nla_types, )*
                    #( Self::#unknown_ident(a) => NetlinkAttributeSerializable::get_type(a), )*
                }
            }

            fn serialize_payload(&self, buf: &mut Vec<u8>) {
                use netlink15_core::message::NetlinkPayloadRequest;
                use netlink15_core::attr::NetlinkAttributeSerializable;

                match self {
                    #( Self::#none_idents => {}, )*
                    #( Self::#some_idents(val) => NetlinkPayloadRequest::serialize(val, buf), )*
                    #( Self::#unknown_ident(a) => NetlinkAttributeSerializable::serialize_payload(a, buf), )*
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::impl_netlink_attribute_serializable;
    use syn::parse_quote;
    use syn::DeriveInput;

    #[test]
    fn deriving_simple_enum_does_not_panic() {
        let test_enum: DeriveInput = parse_quote! {
            enum ControllerAttributeOperation {
                #[nla_type(0)]
                Unspec,
                #[nla_type(1)]
                Id(u32),
                #[nla_type(2)]
                Flags(u32),
                #[nla_type(_)]
                Unknown(UnknownPayload)
            }
        };

        impl_netlink_attribute_serializable(&test_enum);
    }

    #[test]
    #[should_panic(expected = "NetlinkAttributeSerializable derive may only be used on enums.")]
    fn derive_struct() {
        let test_enum: DeriveInput = parse_quote! {
            struct TestStruct {}
        };

        impl_netlink_attribute_serializable(&test_enum);
    }

    #[test]
    #[should_panic(
        expected = "Please annotate all enum variants with #[nla_type(..)]. Saw \"Unknown\" unannotated."
    )]
    fn unannotated_variant() {
        let test_enum: DeriveInput = parse_quote! {
            enum ControllerAttributeOperation {
                #[nla_type(0)]
                Unspec,
                #[nla_type(1)]
                Id(u32),
                #[nla_type(2)]
                Flags(u32),
                Unknown(UnknownPayload)
            }
        };

        impl_netlink_attribute_serializable(&test_enum);
    }

    #[test]
    #[should_panic(expected = "Only 1 variant may be marked with #[nla_type(_)]. Saw 2")]
    fn multiple_unknowns() {
        let test_enum: DeriveInput = parse_quote! {
            enum TestEnum {
                #[nla_type(_)]
                Unknown(UnknownPayload),
                #[nla_type(_)]
                Unknown2(UnknownPayload)
            }
        };

        impl_netlink_attribute_serializable(&test_enum);
    }

    #[test]
    #[should_panic(
        // TODO: Print the more specific multiple associated values error. At
        // the moment it prints a pretty version of the error struct.
        expected = "Failed to parse enum variants in NetlinkAttributeSerializable derive"
    )]
    fn multiple_associated_values() {
        let test_enum: DeriveInput = parse_quote! {
            enum TestEnum {
                #[nla_type(1)]
                Flags(u32, u32)
            }
        };

        impl_netlink_attribute_serializable(&test_enum);
    }
}
