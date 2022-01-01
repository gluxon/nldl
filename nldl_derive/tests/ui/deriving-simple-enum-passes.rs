use nldl::attr::UnknownAttribute;

#[derive(Debug, PartialEq, nldl::attr::Serialize, nldl::attr::Deserialize)]
enum ControllerAttributeOperation {
    #[nla_type(0)]
    Unspec,
    #[nla_type(1)]
    Id(u32),
    #[nla_type(2)]
    Flags(u32),
    #[nla_type(_)]
    Unknown(UnknownAttribute),
}

fn main() {}
