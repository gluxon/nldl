use nldl::attr::UnknownAttribute;

#[derive(Debug, PartialEq, nldl::attr::Deserialize)]
enum ControllerAttributeOperation {
    #[nla_type(_)]
    Unknown(UnknownAttribute),
}

fn main() {}
