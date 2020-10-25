use super::socket::GenlSocket;
use attr::ControllerAttribute;

pub mod attr;
mod get_family;

pub trait NetlinkGenericController {
    fn get_family(&self, family_name: String) -> nix::Result<Vec<ControllerAttribute>>;
}

impl NetlinkGenericController for GenlSocket {
    fn get_family(&self, family_name: String) -> nix::Result<Vec<ControllerAttribute>> {
        get_family::get_family(&self, family_name)
    }
}
