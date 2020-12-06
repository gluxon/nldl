use crate::linux::nlmsg_align;
use crate::message::raw::RawNetlinkMessage;
use crate::message::DeserializeNetlinkMessageResult;
use crate::message::NetlinkMessageResponse;
use crate::message::NetlinkMessageResponseDeserializeError;
use crate::message::NetlinkPayloadResponse;
use std::convert::TryFrom;

/// This iterator intentionally does not free messages that have already been
/// iterated over. Memory is not released until the entire iterator is dropped.
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

        // TODO: Check for NLM_F_MULTI and NLMSG_DONE.
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
