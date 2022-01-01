use nldl::attr::UnknownAttribute;

const ONE: i32 = 1;

#[derive(Debug, PartialEq, nldl::attr::Serialize, nldl::attr::Deserialize)]
enum ControllerAttributeOperation {
    #[nla_type(0i32 as u16)]
    Unspec,
    #[nla_type(crate::ONE as u16)]
    Id(u32),
    #[nla_type(_)]
    Unknown(UnknownAttribute),
}

fn main() {}
