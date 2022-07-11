use crate::attr::ControllerAttribute;
use netlink15_core::message::NetlinkMessageResponseDeserializeError;
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
    // TODO: Expand up on this description.
    #[error("Unexpected message type")]
    UnexpectedMessageType,
}
