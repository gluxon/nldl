use nldl::attr::UnknownAttribute;
use nldl::utils::ParseNlaIntError;

const ZERO: u16 = 0;

#[derive(Debug, PartialEq, nldl::attr::Serialize, nldl::attr::Deserialize)]
#[nldl(deserialize(error = "ParseNlaIntError"))]
enum ControllerAttributeOperation {
    #[nla_type(0)]
    Unspec,
    #[nla_type(crate::ZERO)]
    Id(u32),
    #[nla_type(_)]
    Unknown(UnknownAttribute),
}

fn main() {}
