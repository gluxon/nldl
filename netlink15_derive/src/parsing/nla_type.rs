use proc_macro2::TokenStream;
use std::convert::{TryFrom, TryInto};
use syn::{Attribute, DataEnum, Variant};

const NLA_TYPE_ATTR: &'static str = "nla_type";

/// Describes Rust enum variants from the perspective of the
/// `#[derive(NetlinkSerializable)]` and `#[derive(NetlinkDeserializable)]
/// macros.
enum NetlinkAttributeKind<'a> {
    /// Enum variants marked with `nla_type` but with no associated value.
    /// Typically "Unspec" attributes.
    NoPayload(NetlinkAttributeKindNoPayload<'a>),
    /// Enum variants marked with `nla_type` having some serializable and/or
    /// deserializable associated value.
    Some(NetlinkAttributeKindSome<'a>),
    /// Enum variants marked with `nla_type(_)`. Typically for netlink attributes
    /// with unmatched type ids during deserialization.
    ///
    ///  - There should only ever be one variant marked with `nla_type(_)`.
    ///  - The payload value should implement the `NetlinkSerializable` trait.
    Wildcard(NetlinkAttributeKindWildcard<'a>),
    /// Enum variants without `nla_type`.
    Unmarked(NetlinkAttributeKindUnmarked<'a>),
}

#[derive(thiserror::Error, Debug)]
pub enum NetlinkAttributeKindFromVariantError {
    #[error(transparent)]
    SynError(#[from] syn::Error),
    #[error("Variant \"{ident:?}\" contains more than one associated value.")]
    MultipleAssociatedValues { ident: syn::Ident },
}

impl<'a> TryFrom<&'a Variant> for NetlinkAttributeKind<'a> {
    type Error = NetlinkAttributeKindFromVariantError;

    fn try_from(variant: &'a Variant) -> Result<Self, Self::Error> {
        let ident = &variant.ident;
        let nla_type_attr = (variant.attrs)
            .iter()
            .find(|attr| is_nla_type_attr(attr))
            // .find(|attr| attr.path.is_ident(NLA_TYPE_ATTR))
            .map(|attr| attr.parse_args::<TokenStream>())
            .transpose()?;

        let ty = match nla_type_attr {
            None => return Ok(Self::Unmarked(NetlinkAttributeKindUnmarked { ident })),
            Some(ty) => ty,
        };

        if is_underscore(ty.clone()) {
            return Ok(Self::Wildcard(NetlinkAttributeKindWildcard { ident }));
        }

        match variant.fields.len() {
            0 => Ok(Self::NoPayload(NetlinkAttributeKindNoPayload { ident, ty })),
            1 => Ok(Self::Some(NetlinkAttributeKindSome { ident, ty })),
            _ => Err(Self::Error::MultipleAssociatedValues {
                ident: ident.clone(),
            }),
        }
    }
}

pub struct NetlinkAttributeKindNoPayload<'a> {
    pub ident: &'a syn::Ident,
    pub ty: TokenStream,
}

pub struct NetlinkAttributeKindSome<'a> {
    pub ident: &'a syn::Ident,
    pub ty: TokenStream,
}

pub struct NetlinkAttributeKindWildcard<'a> {
    pub ident: &'a syn::Ident,
}

pub struct NetlinkAttributeKindUnmarked<'a> {
    pub ident: &'a syn::Ident,
}

pub struct PartitionedAttributeKinds<'a> {
    pub no_payload: Vec<NetlinkAttributeKindNoPayload<'a>>,
    pub some: Vec<NetlinkAttributeKindSome<'a>>,
    pub wildcard: Vec<NetlinkAttributeKindWildcard<'a>>,
    pub unmarked: Vec<NetlinkAttributeKindUnmarked<'a>>,
}

impl<'a> PartitionedAttributeKinds<'a> {
    pub fn from(data_enum: &'a DataEnum) -> Result<Self, NetlinkAttributeKindFromVariantError> {
        let mut partitioned_variants = Self {
            no_payload: vec![],
            some: vec![],
            wildcard: vec![],
            unmarked: vec![],
        };

        for variant in &data_enum.variants {
            let variant: NetlinkAttributeKind = variant.try_into()?;
            match variant {
                NetlinkAttributeKind::NoPayload(val) => partitioned_variants.no_payload.push(val),
                NetlinkAttributeKind::Some(val) => partitioned_variants.some.push(val),
                NetlinkAttributeKind::Wildcard(val) => partitioned_variants.wildcard.push(val),
                NetlinkAttributeKind::Unmarked(val) => partitioned_variants.unmarked.push(val),
            }
        }

        Ok(partitioned_variants)
    }
}

fn is_nla_type_attr(attribute: &Attribute) -> bool {
    attribute.path.is_ident(NLA_TYPE_ATTR)
}

fn is_underscore(tokens: TokenStream) -> bool {
    let parsed = syn::parse2::<syn::token::Underscore>(tokens);
    parsed.map(|_| true).unwrap_or(false)
}
