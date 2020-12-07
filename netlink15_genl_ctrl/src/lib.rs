use get_family::GetFamilyResult;
use list::ListResult;
use netlink15_genl::socket::GenlSocket;

pub mod attr;
pub mod err;
mod get_family;
mod list;

pub trait NetlinkGenericController {
    fn get_family(&self, family_name: String) -> GetFamilyResult;
    fn list(&self) -> ListResult;
}

impl NetlinkGenericController for GenlSocket {
    fn get_family(&self, family_name: String) -> GetFamilyResult {
        get_family::get_family(&self, family_name)
    }

    fn list(&self) -> ListResult {
        list::list(&self)
    }
}
