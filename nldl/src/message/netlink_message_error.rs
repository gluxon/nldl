use super::NetlinkPayloadRequest;
use super::NetlinkPayloadResponse;
use super::RawNetlinkMessageHeader;
use arrayref::array_ref;
use std::mem::size_of;

/// https://www.infradead.org/~tgr/libnl/doc/core.html#core_errmsg
#[derive(Debug, PartialEq)]
pub struct NetlinkErrorMessagePayload {
    pub error_code: u32,
    pub original_header: RawNetlinkMessageHeader,
}

impl NetlinkPayloadRequest for NetlinkErrorMessagePayload {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.error_code.serialize(buf);
        self.original_header.serialize(buf);
    }
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ReadNetlinkErrorMessageError {
    #[error("Found a netlink message with an insufficiently sized payload buffer. Expected a payload with {expected} bytes (determined from the header) but saw {actual} bytes.")]
    UnexpectedLen { actual: usize, expected: usize },
}

impl NetlinkPayloadResponse for NetlinkErrorMessagePayload {
    type Error = ReadNetlinkErrorMessageError;

    fn deserialize(buf: &[u8]) -> Result<Self, Self::Error> {
        let expected_len = size_of::<u32>() + size_of::<libc::nlmsghdr>();
        if buf.len() < expected_len {
            return Err(Self::Error::UnexpectedLen {
                actual: buf.len(),
                expected: expected_len,
            });
        }

        let error_code_bytes = array_ref![buf, 0, size_of::<u32>()];
        let error_code = u32::from_ne_bytes(*error_code_bytes);

        let original_header_bytes = array_ref![buf, size_of::<u32>(), size_of::<libc::nlmsghdr>()];
        let original_header = RawNetlinkMessageHeader::deserialize(original_header_bytes);

        Ok(Self {
            error_code,
            original_header,
        })
    }
}
