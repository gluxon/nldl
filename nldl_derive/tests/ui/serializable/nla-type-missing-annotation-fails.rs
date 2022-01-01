use nldl::attr::UnknownAttribute;

#[derive(Debug, PartialEq, nldl::attr::Serialize)]
enum ControllerAttributeOperation {
    #[nla_type(0)]
    Unspec,
    #[nla_type(1)]
    Id(u32),
    #[nla_type(2)]
    Flags(u32),
    Unknown(UnknownAttribute)
}

fn main() {}
