#[derive(Debug, thiserror::Error)]
#[error("netlink attribute deserialization failed")]
pub struct DeserializeError {
    pub attribute_struct_name: &'static str,

    /// The Netlink attribute type ID deserialization failed on.
    pub nla_type_id: u16,

    #[source]
    pub source: Box<dyn std::error::Error + Send + Sync>,
}
