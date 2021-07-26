use crate::attr::ControllerAttribute;
use netlink15_core::message::NetlinkMessageResponseDeserializeError;
use netlink15_genl::socket::RecvUntilDoneError;
use netlink15_genl::GenericNetlinkResponse;

#[derive(thiserror::Error, Debug)]
pub enum GenlCtrlCommandError {
    #[error(transparent)]
    NixError(#[from] nix::Error),
    #[error(transparent)]
    DeserializeError(
        #[from]
        NetlinkMessageResponseDeserializeError<GenericNetlinkResponse<Vec<ControllerAttribute>>>,
    ),
    #[error(transparent)]
    RecvUntilDoneError(#[from] RecvUntilDoneError<Vec<ControllerAttribute>>),
    #[error("Recieved an unexpected NLMSG type: {0}")]
    UnexpectedMessageType(u16),
}
