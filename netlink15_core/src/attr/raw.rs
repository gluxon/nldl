use crate::utils::nla_get_u16;
use crate::utils::ParseNlaIntError;
use std::{convert::TryFrom, mem::size_of};

/// Assists parsing within this crate. Not meant to be used outside the
/// netlink15_core. Instead implementations of
/// `NetlinkAttributeSerializable`/`NetlinkAttributeDeserializable` should be
/// used.
pub(crate) struct RawNetlinkAttribute<'a> {
    pub len: u16,
    pub ty: u16,
    pub payload: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for RawNetlinkAttribute<'a> {
    type Error = ParseNlaIntError;

    /// Reads the first 2 bytes from a buffer to determine the Netlink
    /// Attribute's length. The length bytes are interpreted with the host's
    /// natural endianness. This method will panic if the buffer is shorter than
    /// expected.
    ///
    /// It's acceptable to pass a longer buffer than necessary. The remaining
    /// bytes beyond the retrieved length will be ignored.
    // TODO: Make this not panic if the buffer is shorter than the expected
    // payload.
    fn try_from(buf: &'a [u8]) -> Result<Self, Self::Error> {
        let (header_bytes, remaining) = buf.split_at(size_of::<libc::nlattr>());

        let len = nla_get_u16(&header_bytes[0..size_of::<u16>()])?;
        let ty = nla_get_u16(&header_bytes[size_of::<u16>()..2 * size_of::<u16>()])?;
        let payload = {
            let payload_len = usize::from(len) - size_of::<libc::nlattr>();
            &remaining[..payload_len]
        };

        Ok(Self { len, ty, payload })
    }
}
