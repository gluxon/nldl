use netlink15_genl::ctrl::attr::ControllerAttribute;
use netlink15_genl::ctrl::NetlinkGenericController;
use netlink15_genl::socket::GenlSocket;

#[test]
fn get_acpi_event_id() -> anyhow::Result<()> {
    let genl_controller = GenlSocket::connect()?;
    let attrs = genl_controller.get_family("acpi_event".to_string())?;
    let family_id_attr = attrs.iter().find(|attr| match attr {
        ControllerAttribute::FamilyId(_) => true,
        _ => false,
    });

    assert_eq!(family_id_attr, Some(&ControllerAttribute::FamilyId(0x18)));
    Ok(())
}
