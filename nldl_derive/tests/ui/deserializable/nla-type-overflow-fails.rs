use nldl::attr::UnknownAttribute;
use nldl::utils::ParseNlaIntError;

#[derive(Debug, PartialEq, nldl::attr::Serialize, nldl::attr::Deserialize)]
#[nldl(deserialize(error = "ParseNlaIntError"))]
enum ControllerAttributeOperation {
    #[nla_type(0)]
    Unspec,
    #[nla_type(u16::MAX + 1)]
    Id(u32),
    #[nla_type(_)]
    Unknown(UnknownAttribute),
}

fn main() {}
