use proc_macro2::TokenStream;
use std::convert::{TryFrom, TryInto};
use syn::{Attribute, DataEnum, Variant};

const NLA_TYPE_ATTR: &'static str = "nla_type";
const NLA_TYPE_UNKNOWN_ATTR: &'static str = "nla_type_unknown";

/// Describes Rust enum variants from the perspective of the
/// `#[derive(NetlinkSerializable)]` and `#[derive(NetlinkDeserializable)]
/// macros.
enum NetlinkAttributeKind<'a> {
    /// Enum variants marked with `nla_type` but with no associated value.
    /// Typically "Unspec" attributes.
    None(NetlinkAttributeKindNone<'a>),
    /// Enum variants marked with `nla_type` having some serializable and/or
    /// deserializable associated value.
    Some(NetlinkAttributeKindSome<'a>),
    /// Enum variants marked with `nla_type_unknown`. Typically for netlink
    /// attributes with unmatched type ids during deserialization.
    ///
    ///  - There should only ever be one variant marked with `nla_type_unknown`.
    ///  - The payload value should always be
    ///    [UnknownPayload](netlink15_core::attr::UnknownAttribute).
    Unknown(NetlinkAttributeKindUnknown<'a>),
    /// Enum variants without `nla_type` or `nla_type_unknown`.
    Unmarked(NetlinkAttributeKindUnmarked<'a>),
}

#[derive(thiserror::Error, Debug)]
pub enum NetlinkAttributeKindFromVariantError {
    #[error(
        "Variant \"{ident:?}\" cannot contain both {} and {}",
        NLA_TYPE_ATTR,
        NLA_TYPE_UNKNOWN_ATTR
    )]
    ContainsNlaTypeAndUnknown { ident: syn::Ident },
    #[error("Variant \"{ident:?}\" contains more than one associated value.")]
    MultipleAssociatedValues { ident: syn::Ident },
}

impl<'a> TryFrom<&'a Variant> for NetlinkAttributeKind<'a> {
    type Error = NetlinkAttributeKindFromVariantError;

    fn try_from(variant: &'a Variant) -> Result<Self, Self::Error> {
        let ident = &variant.ident;

        let contains_unknown_attr = variant.attrs.iter().any(is_nla_type_unknown_attr);
        let nla_type_attr = variant.attrs.iter().find_map(find_nla_type_attr);

        match (nla_type_attr, contains_unknown_attr) {
            (None, false) => Ok(Self::Unmarked(NetlinkAttributeKindUnmarked { ident })),
            (None, true) => Ok(Self::Unknown(NetlinkAttributeKindUnknown { ident })),
            (Some(ty), false) => match variant.fields.len() {
                0 => Ok(Self::None(NetlinkAttributeKindNone { ident, ty })),
                1 => Ok(Self::Some(NetlinkAttributeKindSome { ident, ty })),
                _ => Err(Self::Error::MultipleAssociatedValues {
                    ident: ident.clone(),
                }),
            },
            (Some(_), true) => Err(Self::Error::ContainsNlaTypeAndUnknown {
                ident: ident.clone(),
            }),
        }
    }
}

pub struct NetlinkAttributeKindNone<'a> {
    pub ident: &'a syn::Ident,
    pub ty: &'a TokenStream,
}

pub struct NetlinkAttributeKindSome<'a> {
    pub ident: &'a syn::Ident,
    pub ty: &'a TokenStream,
}

pub struct NetlinkAttributeKindUnknown<'a> {
    pub ident: &'a syn::Ident,
}

pub struct NetlinkAttributeKindUnmarked<'a> {
    pub ident: &'a syn::Ident,
}

pub struct PartitionedAttributeKinds<'a> {
    pub none: Vec<NetlinkAttributeKindNone<'a>>,
    pub some: Vec<NetlinkAttributeKindSome<'a>>,
    pub unknown: Vec<NetlinkAttributeKindUnknown<'a>>,
    pub unmarked: Vec<NetlinkAttributeKindUnmarked<'a>>,
}

impl<'a> PartitionedAttributeKinds<'a> {
    pub fn from(data_enum: &'a DataEnum) -> Result<Self, NetlinkAttributeKindFromVariantError> {
        let mut partitioned_variants = Self {
            none: vec![],
            some: vec![],
            unknown: vec![],
            unmarked: vec![],
        };

        for variant in &data_enum.variants {
            let variant: NetlinkAttributeKind = variant.try_into()?;
            match variant {
                NetlinkAttributeKind::None(val) => partitioned_variants.none.push(val),
                NetlinkAttributeKind::Some(val) => partitioned_variants.some.push(val),
                NetlinkAttributeKind::Unknown(val) => partitioned_variants.unknown.push(val),
                NetlinkAttributeKind::Unmarked(val) => partitioned_variants.unmarked.push(val),
            }
        }

        Ok(partitioned_variants)
    }
}

fn find_nla_type_attr(attribute: &Attribute) -> Option<&TokenStream> {
    if attribute.path.is_ident(NLA_TYPE_ATTR) {
        return Some(&attribute.tokens);
    }
    None
}

fn is_nla_type_unknown_attr(attribute: &Attribute) -> bool {
    if attribute.path.is_ident(NLA_TYPE_UNKNOWN_ATTR) {
        return true;
    }
    false
}
