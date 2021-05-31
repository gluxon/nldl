use crate::linux::nlmsg_align;
use crate::message::raw::RawNetlinkMessage;
use crate::message::DeserializeNetlinkMessageResult;
use crate::message::NetlinkMessageResponse;
use crate::message::NetlinkMessageResponseDeserializeError;
use crate::message::NetlinkPayloadResponse;
use std::convert::TryFrom;

/// Takes a byte buffer and returns an Iterator over deserialized Netlink messages. The iterator
/// continues until there are no remaining bytes in the buffer.
///
/// This is similar to the process described in libnl here:
/// https://www.infradead.org/~tgr/libnl/doc/core.html#_parsing_a_message
///
/// This iterator intentionally does not free messages that have already been iterated over to avoid
/// re-malloc'ing remaining bytes. Memory is not released until the entire iterator is dropped.
pub fn create_message_iterator<T>(
    buf: Vec<u8>,
) -> impl Iterator<Item = DeserializeNetlinkMessageResult<T>>
where
    T: NetlinkPayloadResponse,
{
    let mut offset: usize = 0;
    let mut did_last_read_error = false;
    std::iter::from_fn(move || {
        if did_last_read_error || offset >= buf.len() {
            return None;
        }

        let read_result = next(&buf[offset..]);
        match read_result {
            Ok(success) => {
                offset += success.consumed;
                Some(Ok(success.message))
            }
            Err(err) => {
                did_last_read_error = true;
                Some(Err(err))
            }
        }
    })
}

struct NextMessageRead<T: NetlinkPayloadResponse> {
    pub message: NetlinkMessageResponse<T>,
    pub consumed: usize,
}

/// Similar to the nlmsg_next function in linux/netlink.h
fn next<T>(buf: &[u8]) -> Result<NextMessageRead<T>, NetlinkMessageResponseDeserializeError<T>>
where
    T: NetlinkPayloadResponse,
{
    let raw = RawNetlinkMessage::try_from(buf)?;
    let consumed = nlmsg_align(raw.header.len as usize);
    let message = NetlinkMessageResponse::<T>::try_from(raw)?;

    Ok(NextMessageRead { message, consumed })
}

#[cfg(test)]
mod tests {
    use super::create_message_iterator;

    #[test]
    fn stops_after_error() {
        // Start with a valid nlctrl message.
        let mut buf = vec![
            0x88, 0x00, 0x00, 0x00, 0x10, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x3A, 0xC6,
            0x00, 0x00, 0x01, 0x02, 0x00, 0x00, 0x0B, 0x00, 0x02, 0x00, 0x6E, 0x6C, 0x63, 0x74,
            0x72, 0x6C, 0x00, 0x00, 0x06, 0x00, 0x01, 0x00, 0x10, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x03, 0x00, 0x02, 0x00, 0x00, 0x00, 0x08, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x05, 0x00, 0x08, 0x00, 0x00, 0x00, 0x2C, 0x00, 0x06, 0x00, 0x14, 0x00,
            0x01, 0x00, 0x08, 0x00, 0x01, 0x00, 0x03, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00,
            0x0E, 0x00, 0x00, 0x00, 0x14, 0x00, 0x02, 0x00, 0x08, 0x00, 0x01, 0x00, 0x0A, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0C, 0x00, 0x00, 0x00, 0x1C, 0x00, 0x07, 0x00,
            0x18, 0x00, 0x01, 0x00, 0x08, 0x00, 0x02, 0x00, 0x10, 0x00, 0x00, 0x00, 0x0B, 0x00,
            0x01, 0x00, 0x6E, 0x6F, 0x74, 0x69, 0x66, 0x79, 0x00, 0x00,
        ];

        // Add the start of a new message (len = 30) but nothing else to make it invalid.
        buf.extend(&[30, 0, 0, 0]);

        let mut messages = create_message_iterator::<Vec<u8>>(buf);

        assert!(matches![messages.next(), Some(Ok(_))]);
        assert!(matches![messages.next(), Some(Err(_))]);
        assert!(matches![messages.next(), None]);
    }
}
