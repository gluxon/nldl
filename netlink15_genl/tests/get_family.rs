use netlink15_genl::ctrl::attr::ControllerAttribute;
use netlink15_genl::ctrl::NetlinkGenericController;
use netlink15_genl::socket::GenlSocket;

#[test]
fn get_nlctrl_id() -> anyhow::Result<()> {
    let genl_controller = GenlSocket::connect()?;
    let attrs = genl_controller.get_family("nlctrl".to_string())?;
    let family_id_attr = attrs
        .iter()
        .find(|attr| matches!(attr, ControllerAttribute::FamilyId(_)));

    assert_eq!(family_id_attr, Some(&ControllerAttribute::FamilyId(0x10)));
    Ok(())
}
