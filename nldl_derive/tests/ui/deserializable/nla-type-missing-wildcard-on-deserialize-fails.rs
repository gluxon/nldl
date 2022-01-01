#[derive(Debug, PartialEq, nldl::attr::Deserialize)]
#[netlink15(deserialize(error = "ParseNlaIntError"))]
enum ControllerAttributeOperation {
    #[nla_type(0)]
    Unspec,
}

fn main() {}
