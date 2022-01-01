use proc_macro2::TokenStream;
use quote::quote;
use syn::Data;
use syn::DeriveInput;

use crate::parsing::nla_type::PartitionedAttributeKinds;

pub fn impl_netlink_attribute_deserializable(ast: &DeriveInput) -> TokenStream {
    let data_enum = match &ast.data {
        Data::Enum(data_enum) => data_enum,
        _ => panic!("nldl::attr::Deserialize derive may only be used on enums."),
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
            #( pub const #no_payload_idents: ::std::primitive::u16 = #no_payload_nla_types; )*
            #( pub const #simple_idents: ::std::primitive::u16 = #simple_nla_types; )*
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

    let name_str = name.to_string();

    quote! {
        impl ::nldl::attr::Deserialize for #name {
            fn deserialize(ty: ::std::primitive::u16, payload: &[::std::primitive::u8]) -> ::std::result::Result<Self, ::nldl::attr::DeserializeError> {
                #type_ids_enum_checker_quote

                #type_id_consts_quote

                match ty {
                    #( #type_ids_mod_name::#no_payload_idents => Ok(Self::#no_payload_idents), )*
                    #( #type_ids_mod_name::#simple_idents =>
                        match ::nldl::message::NetlinkPayloadResponse::deserialize(payload) {
                            Ok(val) => Ok(Self::#simple_idents(val)),
                            Err(err) => Err(::nldl::attr::DeserializeError {
                                attribute_struct_name: #name_str,
                                nla_type_id: ty,
                                source: Box::new(err)
                            })
                        },
                    )*
                    _ => Ok(Self::#wildcard_ident(::nldl::attr::UnknownAttribute { ty, payload: ::std::vec::Vec::from(payload) })),
                }
            }
        }
    }
}
