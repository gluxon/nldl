#[derive(Debug, PartialEq, nldl::attr::Deserialize)]
#[nldl(deserialize(error = "ParseNlaIntError"))]
enum ControllerAttributeOperation {
    #[nla_type(0)]
    Unspec,
}

fn main() {}
