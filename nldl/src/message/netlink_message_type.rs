use super::netlink_message_error::NetlinkErrorMessagePayload;
use super::netlink_message_error::ReadNetlinkErrorMessageError;
use super::NetlinkPayloadRequest;
use super::NetlinkPayloadResponse;

/// The NetlinkMessageType variant is determined from a Netlink message's "type" field.
///
/// <https://www.infradead.org/~tgr/libnl/doc/core.html#core_msg_types>
#[derive(Debug, PartialEq)]
pub enum NetlinkMessageType<T> {
    /// No operation, message must be discarded
    Noop,

    /// Error message or ACK
    Error(NetlinkErrorMessagePayload),

    /// End of multipart sequence
    Done,

    /// Overrun notification (Error)
    Overrun,

    /// Protocols communicating over Netlink define their own message types by associating message
    /// type values with an integer >= NLMSG_MIN_TYPE (0x10).
    ProtocolMessage(T),
}

const NLMSG_NOOP: u16 = libc::NLMSG_NOOP as u16;
const NLMSG_ERROR: u16 = libc::NLMSG_ERROR as u16;
const NLMSG_DONE: u16 = libc::NLMSG_DONE as u16;
const NLMSG_OVERRUN: u16 = libc::NLMSG_OVERRUN as u16;

impl<T> NetlinkMessageType<T>
where
    T: NetlinkPayloadRequest,
{
    pub fn serialize(&self, buf: &mut Vec<u8>) {
        match self {
            // These message types don't seem to have defined bodies.
            NetlinkMessageType::Noop | NetlinkMessageType::Done | NetlinkMessageType::Overrun => {}

            NetlinkMessageType::Error(err_payload) => err_payload.serialize(buf),
            NetlinkMessageType::ProtocolMessage(payload) => payload.serialize(buf),
        }
    }
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum NetlinkMessageTypeDeserializeError<T: NetlinkPayloadResponse> {
    #[error(transparent)]
    ParseErrorMessageFail(#[from] ReadNetlinkErrorMessageError),
    #[error("{0}")]
    ParseProtocolMessageFail(T::Error),
}

impl<T> NetlinkMessageType<T>
where
    T: NetlinkPayloadResponse,
{
    pub fn deserialize(
        message_type: u16,
        buf: &[u8],
    ) -> Result<Self, NetlinkMessageTypeDeserializeError<T>> {
        match message_type {
            NLMSG_NOOP => Ok(Self::Noop),
            NLMSG_ERROR => Ok(Self::Error(NetlinkErrorMessagePayload::deserialize(buf)?)),
            NLMSG_DONE => Ok(Self::Done),
            NLMSG_OVERRUN => Ok(Self::Overrun),

            // TODO: Pass in the message_type to generic deserialization implementations. A new
            // trait will likely need to be created.
            _ => T::deserialize(buf)
                .map_err(NetlinkMessageTypeDeserializeError::ParseProtocolMessageFail)
                .map(Self::ProtocolMessage),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NetlinkMessageType;
    use crate::message::netlink_message_type::NLMSG_DONE;
    use crate::message::netlink_message_type::NLMSG_NOOP;
    use crate::message::netlink_message_type::NLMSG_OVERRUN;

    #[test]
    fn test_noop_serialization() -> anyhow::Result<()> {
        let original = NetlinkMessageType::<()>::Noop;
        let mut buf = vec![];
        original.serialize(&mut buf);
        assert_eq!(original, NetlinkMessageType::deserialize(NLMSG_NOOP, &buf)?);
        Ok(())
    }

    #[test]
    fn test_done_serialization() -> anyhow::Result<()> {
        let original = NetlinkMessageType::<()>::Done;
        let mut buf = vec![];
        original.serialize(&mut buf);
        assert_eq!(original, NetlinkMessageType::deserialize(NLMSG_DONE, &buf)?);
        Ok(())
    }

    #[test]
    fn test_overrun_serialization() -> anyhow::Result<()> {
        let original = NetlinkMessageType::<()>::Overrun;
        let mut buf = vec![];
        original.serialize(&mut buf);
        assert_eq!(
            original,
            NetlinkMessageType::deserialize(NLMSG_OVERRUN, &buf)?
        );
        Ok(())
    }
}
