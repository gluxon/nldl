use super::GenericNetlinkRequest;
use super::GenericNetlinkResponse;
use netlink15_core::message::utils::create_message_iterator;
use netlink15_core::message::DeserializeNetlinkMessageResult;
use netlink15_core::message::NetlinkErrorMessagePayload;
use netlink15_core::message::NetlinkMessageHeader;
use netlink15_core::message::NetlinkMessageRequest;
use netlink15_core::message::NetlinkMessageResponse;
use netlink15_core::message::NetlinkMessageResponseDeserializeError;
use netlink15_core::message::NetlinkMessageType;
use netlink15_core::message::NetlinkPayloadRequest;
use netlink15_core::message::NetlinkPayloadResponse;
use nix::sys::socket::bind;
use nix::sys::socket::socket;
use nix::sys::socket::AddressFamily;
use nix::sys::socket::MsgFlags;
use nix::sys::socket::NetlinkAddr;
use nix::sys::socket::SockAddr;
use nix::sys::socket::SockFlag;
use nix::sys::socket::SockProtocol;
use nix::sys::socket::SockType;
use std::os::unix::io::RawFd;

pub struct GenlSocket {
    fd: RawFd,
}

impl GenlSocket {
    pub fn connect() -> nix::Result<GenlSocket> {
        let fd = {
            let protocol = transmute_to_sock_protocol(libc::NETLINK_GENERIC);
            socket(
                AddressFamily::Netlink,
                SockType::Raw,
                SockFlag::empty(),
                protocol,
            )?
        };

        let addr = SockAddr::Netlink(NetlinkAddr::new(0, 0));
        bind(fd, &addr)?;

        Ok(Self { fd })
    }

    pub fn send<T: NetlinkPayloadRequest>(
        &self,
        genl_request: GenericNetlinkRequest<T>,
        flags: u16,
    ) -> nix::Result<()> {
        let message = NetlinkMessageRequest {
            header: NetlinkMessageHeader {
                ty: libc::GENL_ID_CTRL as u16,
                flags,
                seq: 1,
                pid: 0,
            },
            payload: NetlinkMessageType::ProtocolMessage(genl_request),
        };

        let message_bytes = netlink15_core::serialize(&message);

        nix::sys::socket::send(self.fd, &message_bytes, MsgFlags::empty())?;
        Ok(())
    }

    pub fn recv<T: NetlinkPayloadResponse>(
        &self,
    ) -> nix::Result<DeserializeNetlinkMessageResult<GenericNetlinkResponse<T>>> {
        let mut resp_bytes = vec![0; 32768];
        let bytes_read = nix::sys::socket::recv(self.fd, &mut resp_bytes, MsgFlags::empty())?;
        resp_bytes.truncate(bytes_read);
        Ok(NetlinkMessageResponse::<GenericNetlinkResponse<T>>::deserialize(&resp_bytes))
    }

    pub fn recv_multipart<T: NetlinkPayloadResponse>(
        &self,
    ) -> nix::Result<impl Iterator<Item = DeserializeNetlinkMessageResult<GenericNetlinkResponse<T>>>>
    {
        let mut resp_bytes = vec![0; 32768];
        let bytes_read = nix::sys::socket::recv(self.fd, &mut resp_bytes, MsgFlags::empty())?;
        resp_bytes.truncate(bytes_read);
        Ok(create_message_iterator(resp_bytes))
    }

    /// Return all messages from a multipart response buffered into a Vec. Stops after encountering
    /// Error, Done, or Overrun message types.
    pub fn recv_until_done_buffered<T: NetlinkPayloadResponse>(
        &self,
    ) -> Result<Vec<GenericNetlinkResponse<T>>, RecvUntilDoneError<T>> {
        let mut messages: Vec<GenericNetlinkResponse<T>> = vec![];

        loop {
            let batch = self.recv_multipart()?;
            for deser_result in batch {
                let next_message = deser_result.map_err(RecvUntilDoneError::DeserializeError)?;

                match next_message.payload {
                    NetlinkMessageType::Noop => (),
                    NetlinkMessageType::Error(err) => {
                        return Err(RecvUntilDoneError::NetlinkError(err))
                    }
                    NetlinkMessageType::Done => return Ok(messages),
                    NetlinkMessageType::Overrun => {
                        return Err(RecvUntilDoneError::UnexpectedOverrun)
                    }
                    NetlinkMessageType::ProtocolMessage(message) => messages.push(message),
                };
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum RecvUntilDoneError<T: NetlinkPayloadResponse> {
    #[error(transparent)]
    NixError(#[from] nix::Error),
    #[error("{0}")]
    DeserializeError(NetlinkMessageResponseDeserializeError<GenericNetlinkResponse<T>>),
    #[error("Received NLMSG_OVERRUN message instead of proper response.")]
    UnexpectedOverrun,
    #[error("Received NLMSG_ERROR message with code: {:?}", .0.error_code)]
    NetlinkError(NetlinkErrorMessagePayload),
}

/// Some SockProtocol values aren't bound by nix yet.
fn transmute_to_sock_protocol(value: libc::c_int) -> SockProtocol {
    unsafe { std::mem::transmute::<libc::c_int, SockProtocol>(value) }
}
