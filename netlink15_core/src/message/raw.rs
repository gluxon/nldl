use super::RawNetlinkMessageHeader;
use arrayref::array_ref;
use std::convert::TryFrom;
use std::mem::size_of;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RawNetlinkMessage<'a> {
    pub header: RawNetlinkMessageHeader,
    pub payload: &'a [u8],
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ReadRawNetlinkMessageError {
    #[error("Found a netlink message with an incomplete header. Saw {len} bytes, but at least {} were expected", size_of::<libc::nlmsghdr>())]
    IncompleteHeader { len: usize },
    #[error("Found a netlink message with an insufficiently sized payload buffer. Expected a payload with {expected} bytes (determined from the header) but saw {actual} bytes.")]
    UnexpectedEndOfPayloadBuffer { actual: usize, expected: usize },
}

impl<'a> TryFrom<&'a [u8]> for RawNetlinkMessage<'a> {
    type Error = ReadRawNetlinkMessageError;

    fn try_from(buf: &'a [u8]) -> Result<Self, Self::Error> {
        if buf.len() < size_of::<libc::nlattr>() {
            return Err(Self::Error::IncompleteHeader { len: buf.len() });
        }

        let header_bytes = array_ref![buf, 0, size_of::<libc::nlmsghdr>()];
        let header = RawNetlinkMessageHeader::deserialize(&header_bytes);

        let remaining = &buf[size_of::<libc::nlmsghdr>()..];

        let payload = {
            let payload_len = (header.len as usize) - size_of::<libc::nlmsghdr>();
            if remaining.len() < payload_len {
                return Err(Self::Error::UnexpectedEndOfPayloadBuffer {
                    actual: remaining.len(),
                    expected: payload_len,
                });
            }

            &remaining[..payload_len]
        };

        Ok(Self { header, payload })
    }
}
