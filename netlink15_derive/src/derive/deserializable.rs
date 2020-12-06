use proc_macro2::TokenStream;
use quote::quote;
use syn::Data;
use syn::DeriveInput;

use crate::parsing::crate_attr::DeriveOptions;
use crate::parsing::nla_type::PartitionedAttributeKinds;

pub fn impl_netlink_attribute_deserializable(ast: &DeriveInput) -> TokenStream {
    let data_enum = match &ast.data {
        Data::Enum(data_enum) => data_enum,
        _ => panic!("NetlinkAttributeDeserializable derive may only be used on enums."),
    };

    let deserialize_error_type: TokenStream = match DeriveOptions::try_from(ast) {
        Ok(val) => val.deserialize.error_type_name.parse().unwrap(),
        Err(msg) => panic!("{}", msg),
    };

    let partitioned_variants = PartitionedAttributeKinds::from(data_enum)
        .expect("Failed to parse enum variants in NetlinkAttributeDeserializable derive.");
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
        impl netlink15_core::attr::NetlinkAttributeDeserializable for #name {
            type Error = #deserialize_error_type;

            fn deserialize(ty: u16, payload: &[u8]) -> Result<Self, Self::Error> {
                use netlink15_core::message::NetlinkPayloadResponse;

                Ok(match ty {
                    #( #no_payload_nla_types => Self::#no_payload_idents, )*
                    #( #simple_nla_types => Self::#simple_idents(NetlinkPayloadResponse::deserialize(payload)?), )*
                    #( _ => Self::#wildcard_ident(UnknownAttribute { ty, payload: Vec::from(payload) }), )*
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::impl_netlink_attribute_deserializable;
    use syn::parse_quote;
    use syn::DeriveInput;

    #[test]
    #[should_panic(
        expected = "Failed to find NetlinkAttributeDeserializable error type. Please annotate this enum. Ex: #[netlink15(deserialize(error = \"ParseNlaIntError\"))]"
    )]
    fn unannotated_crate_attr_panics() {
        let test_enum: DeriveInput = parse_quote! {
            enum ControllerAttributeOperation {}
        };

        impl_netlink_attribute_deserializable(&test_enum);
    }

    #[test]
    #[should_panic(
        expected = "Failed to find NetlinkAttributeDeserializable error type. Please annotate this enum. Ex: #[netlink15(deserialize(error = \"ParseNlaIntError\"))]"
    )]
    fn missing_deserialize_options_panics() {
        let test_enum: DeriveInput = parse_quote! {
            #[netlink15]
            enum ControllerAttributeOperation {}
        };

        impl_netlink_attribute_deserializable(&test_enum);
    }

    #[test]
    fn deriving_simple_enum_succeeds() {
        let test_enum: DeriveInput = parse_quote! {
            #[netlink15(deserialize(error = "ParseNlaIntError"))]
            enum ControllerAttributeOperation {
                #[nla_type(0)]
                Unspec,
                #[nla_type(1)]
                Id(u32),
                #[nla_type(2)]
                Flags(u32),
            }
        };

        impl_netlink_attribute_deserializable(&test_enum);
    }
}
