use syn::Attribute;
use syn::DeriveInput;
use syn::Lit;
use syn::LitStr;
use syn::Meta;
use syn::MetaList;
use syn::MetaNameValue;
use syn::NestedMeta;

pub const CRATE_ATTR: &str = "netlink15";

pub struct DeriveOptions {
    pub deserialize: DeriveDeserializeOptions,
}

pub struct DeriveDeserializeOptions {
    pub error_type_name: LitStr,
}

#[derive(thiserror::Error, Debug)]
pub enum DeriveOptionsTryFromError {
    #[error(transparent)]
    SynError(#[from] syn::Error),
    #[error("Failed to find NetlinkAttributeDeserializable error type. Please annotate this enum. Ex: #[netlink15(deserialize(error = \"ParseNlaIntError\"))]")]
    DeserializeErrorNotFound,
}

impl DeriveOptions {
    pub fn try_from(ast: &DeriveInput) -> Result<DeriveOptions, DeriveOptionsTryFromError> {
        let optional_crate_attrs = ast
            .attrs
            .iter()
            .find(|attr| is_crate_attr(attr))
            .map(Attribute::parse_meta)
            .transpose()?
            .and_then(|meta| match meta {
                syn::Meta::List(meta_list) => Some(meta_list.nested),
                syn::Meta::Path(_) | syn::Meta::NameValue(_) => None,
            });

        let optional_deserialize_attrs = optional_crate_attrs.and_then(|meta_list| {
            meta_list
                .into_iter()
                .find_map(|nested_meta| match nested_meta {
                    NestedMeta::Meta(Meta::List(meta_list)) if is_deserialize_attr(&meta_list) => {
                        Some(meta_list.nested)
                    }
                    _ => None,
                })
        });

        let optional_deserialize_error_type_name =
            optional_deserialize_attrs.and_then(|meta_list| {
                meta_list
                    .into_iter()
                    .find_map(|nested_meta| match nested_meta {
                        NestedMeta::Meta(Meta::NameValue(meta_name_value))
                            if is_deserialize_error_type_attr(&meta_name_value) =>
                        {
                            Some(meta_name_value.lit)
                        }
                        _ => None,
                    })
            });

        optional_deserialize_error_type_name
            .and_then(|lit| match lit {
                Lit::Str(lit_str) => Some(lit_str),
                _ => None,
            })
            .map(|error_type_name| {
                Ok(DeriveOptions {
                    deserialize: DeriveDeserializeOptions { error_type_name },
                })
            })
            .unwrap_or_else(|| Err(DeriveOptionsTryFromError::DeserializeErrorNotFound))
    }
}

fn is_crate_attr(attr: &Attribute) -> bool {
    attr.path.is_ident(CRATE_ATTR)
}

fn is_deserialize_attr(meta_list: &MetaList) -> bool {
    meta_list.path.is_ident("deserialize")
}

fn is_deserialize_error_type_attr(meta_name_value: &MetaNameValue) -> bool {
    meta_name_value.path.is_ident("error")
}
