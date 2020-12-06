use get_family::GetFamilyResult;
use netlink15_genl::socket::GenlSocket;

pub mod attr;
pub mod err;
mod get_family;

pub trait NetlinkGenericController {
    fn get_family(&self, family_name: String) -> GetFamilyResult;
}

impl NetlinkGenericController for GenlSocket {
    fn get_family(&self, family_name: String) -> GetFamilyResult {
        get_family::get_family(&self, family_name)
    }
}
