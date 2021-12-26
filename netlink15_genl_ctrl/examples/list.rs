use netlink15_genl::socket::GenlSocket;
use netlink15_genl_ctrl::attr::ControllerAttribute;
use netlink15_genl_ctrl::family::Family;
use netlink15_genl_ctrl::NetlinkGenericController;

fn main() -> anyhow::Result<()> {
    let genl_controller = GenlSocket::connect()?;
    let families = genl_controller.list()?;
    for family in families {
        print_family(family)?;
    }
    Ok(())
}

fn print_family(attrs: Vec<ControllerAttribute>) -> anyhow::Result<()> {
    let family: Family = attrs.try_into()?;
    println!("{:#?}", family);

    Ok(())
}
