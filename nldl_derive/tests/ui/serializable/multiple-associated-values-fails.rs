#[derive(nldl::attr::Serialize)]
enum TestEnum {
    #[nla_type(1)]
    Flags(u32, u32)
}

fn main() {}
