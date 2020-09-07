use std::convert::{TryFrom, TryInto};
use syn::{
    punctuated::Punctuated, Attribute, DataEnum, Expr, ExprLit, Lit, Meta, MetaList, NestedMeta,
    Variant,
};

const CRATE_META_NAME: &'static str = "netlink15";
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
    #[error(transparent)]
    SynError(#[from] syn::Error),
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
        let crate_attrs = find_crate_attrs_from_attr_slice(&variant.attrs)?
            .unwrap_or_else(syn::punctuated::Punctuated::new);

        let contains_unknown_attr = crate_attrs.iter().any(is_nla_type_unknown_attr);
        let nla_type_attr = crate_attrs
            .into_iter()
            .find_map(find_nla_type_attr)
            .map(parse_nla_type_attr_value)
            .transpose()?;

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
    pub ty: Expr,
}

pub struct NetlinkAttributeKindSome<'a> {
    pub ident: &'a syn::Ident,
    pub ty: Expr,
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

/// Attributes in this crate are namespaced by "netlink15". Iterate over
/// attributes to find the nested attribute list starting with that namespace.
/// Ex: #[netlink15(nla_type = ...)]
fn find_crate_attrs_from_attr_slice(
    attrs: &[Attribute],
) -> syn::Result<Option<Punctuated<NestedMeta, syn::Token![,]>>> {
    attrs
        .iter()
        .map(|attr| attr.parse_meta())
        .map(|result| result.map(find_crate_attrs_from_meta))
        .find(|result| match result {
            // Return the first error or successful find. This can be more
            // cleanly written with .try_find, but that method isn't in stable
            // Rust yet.
            Err(_) => true,
            Ok(Some(_)) => true,
            Ok(None) => false,
        })
        .unwrap_or(Ok(None))
}

fn find_crate_attrs_from_meta(meta: Meta) -> Option<Punctuated<NestedMeta, syn::Token![,]>> {
    is_meta_list(meta)
        .filter(|list| list.path.is_ident(CRATE_META_NAME))
        .map(|list| list.nested)
}

fn is_meta_list(meta: Meta) -> Option<MetaList> {
    match meta {
        Meta::List(list) => Some(list),
        _ => None,
    }
}

/// Assumes `nested` is the meta list from the "netlink15" attribute.
fn find_nla_type_attr(nested_meta: NestedMeta) -> Option<Lit> {
    if let NestedMeta::Meta(Meta::NameValue(name_value)) = nested_meta {
        if name_value.path.is_ident(NLA_TYPE_ATTR) {
            return Some(name_value.lit);
        }
    }
    None
}

fn is_nla_type_unknown_attr(nested_meta: &NestedMeta) -> bool {
    if let NestedMeta::Meta(Meta::Path(path)) = nested_meta {
        if path.is_ident(NLA_TYPE_UNKNOWN_ATTR) {
            return true;
        }
    }
    false
}

/// Attempts to parse the argument if it's a string. Otherwise the argument is
/// transparently converted in to a syn::Expr.
///
/// There are some syn::Lit variants that don't make sense as an nla_type value.
/// The quote! result will show the proper error later on if that's the case.
/// This is on purpose — the rust compiler actually highlights the attribute
/// value with a "cannot convert blah to u16" message, which is really sweet.
fn parse_nla_type_attr_value(literal: Lit) -> syn::Result<Expr> {
    match literal {
        Lit::Str(string) => string.parse::<Expr>(),
        _ => Ok(Expr::Lit(ExprLit {
            attrs: vec![],
            lit: literal,
        })),
    }
}
