use nldl::attr::UnknownAttribute;

#[derive(Debug, PartialEq, nldl::attr::Serialize)]
enum TestEnum {
    #[nla_type(_)]
    Unknown(UnknownAttribute),
    #[nla_type(_)]
    Unknown2(UnknownAttribute)
}

fn main() {}
