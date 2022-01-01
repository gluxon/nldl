use super::nla_type::PartitionedAttributeKinds;
use proc_macro2::Ident;
use proc_macro2::TokenStream;
use syn::Data;
use syn::DeriveInput;

pub struct ParseOrPanicReturn<'a> {
    pub name: &'a Ident,
    pub no_payload_idents: Vec<&'a Ident>,
    pub no_payload_nla_types: Vec<TokenStream>,
    pub simple_idents: Vec<&'a Ident>,
    pub simple_nla_types: Vec<TokenStream>,
    pub wildcard_ident: Option<&'a Ident>,
}

pub fn parse_or_panic(ast: &DeriveInput) -> ParseOrPanicReturn {
    let data_enum = match &ast.data {
        Data::Enum(data_enum) => data_enum,
        _ => panic!("nldl_derive may only be used on enums."),
    };

    let partitioned_variants = PartitionedAttributeKinds::from(data_enum)
        .unwrap_or_else(|err| panic!("Failed to parse enum variants in nldl_derive: {}", err));
    if let Some(unmarked_variant) = partitioned_variants.unmarked.first() {
        panic!(
            "Please annotate all enum variants with #[nla_type(..)]. Saw \"{}\" unannotated.",
            unmarked_variant.ident
        );
    }
    let wildcard_ident = match &partitioned_variants.wildcard[..] {
        [] => None,
        [variant] => Some(variant.ident),
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

    ParseOrPanicReturn {
        name,
        no_payload_idents,
        no_payload_nla_types,
        simple_idents,
        simple_nla_types,
        wildcard_ident,
    }
}
