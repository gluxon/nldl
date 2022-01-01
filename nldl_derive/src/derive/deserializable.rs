use proc_macro2::TokenStream;
use quote::quote;
use syn::Data;
use syn::DeriveInput;

use crate::parsing::crate_attr::DeriveOptions;
use crate::parsing::nla_type::PartitionedAttributeKinds;

pub fn impl_netlink_attribute_deserializable(ast: &DeriveInput) -> TokenStream {
    let data_enum = match &ast.data {
        Data::Enum(data_enum) => data_enum,
        _ => panic!("nldl::attr::Deserialize derive may only be used on enums."),
    };

    let deserialize_error_type: TokenStream = match DeriveOptions::try_from(ast) {
        Ok(val) => val.deserialize.error_type_name.parse().unwrap(),
        Err(msg) => panic!("{}", msg),
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
        [variant] => variant.ident,
        [] => panic!(
            "One variant must be marked with #[nla_type(_)] for wildcard handling. None found."
        ),
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

    // Expressions can't be inlined on the left side of a match arm, so we'll assign them to a
    // namespaced constant first.
    //
    // Example:
    //
    //     mod InternalTypeIdsNamespace {
    //         pub const Unspec = EXAMPLE_ZERO_CONST as u16
    //     }
    //
    //     // ...
    //
    //     Ok(match ty {
    //         InternalTypeIdsNamespace::Unspec => ...
    //     })
    let type_ids_mod_name = quote! { InternalTypeIdsNamespace };
    let type_id_consts_quote = quote! {
        #[allow(non_upper_case_globals)]
        mod #type_ids_mod_name {
            #( pub const #no_payload_idents: u16 = #no_payload_nla_types; )*
            #( pub const #simple_idents: u16 = #simple_nla_types; )*
        }
    };

    // Generate an unused enum to check that the same u16 type id value doesn't appear more than
    // once. The compiler errors are relatively appropriate.
    let type_ids_enum_checker_quote = quote! {
        #[repr(u16)]
        enum InternalTypeIdsEnum {
            #( #no_payload_idents = #no_payload_nla_types, )*
            #( #simple_idents = #simple_nla_types, )*
        }
    };

    quote! {
        impl nldl::attr::Deserialize for #name {
            type Error = #deserialize_error_type;

            fn deserialize(ty: u16, payload: &[u8]) -> Result<Self, Self::Error> {
                use nldl::message::NetlinkPayloadResponse;

                #type_ids_enum_checker_quote

                #type_id_consts_quote

                Ok(match ty {
                    #( #type_ids_mod_name::#no_payload_idents => Self::#no_payload_idents, )*
                    #( #type_ids_mod_name::#simple_idents => Self::#simple_idents(NetlinkPayloadResponse::deserialize(payload)?), )*
                    _ => Self::#wildcard_ident(UnknownAttribute { ty, payload: Vec::from(payload) }),
                })
            }
        }
    }
}
