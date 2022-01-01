use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::parsing::parse_or_panic::parse_or_panic;
use crate::parsing::parse_or_panic::ParseOrPanicReturn;

pub fn impl_netlink_attribute_deserializable(ast: &DeriveInput) -> TokenStream {
    let ParseOrPanicReturn {
        name,
        no_payload_idents,
        no_payload_nla_types,
        simple_idents,
        simple_nla_types,
        wildcard_ident,
    } = parse_or_panic(ast);

    let wildcard_ident = match wildcard_ident {
        None => panic!(
            "One variant must be marked with #[nla_type(_)] for wildcard handling. None found."
        ),
        Some(ident) => ident,
    };

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
