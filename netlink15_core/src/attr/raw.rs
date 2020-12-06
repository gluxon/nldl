use crate::utils::nla_get_u16;
use crate::utils::ParseNlaIntError;
use std::convert::TryFrom;
use std::mem::size_of;

/// Assists parsing within this crate. Not meant to be used outside the
/// netlink15_core. Instead implementations of
/// `NetlinkAttributeSerializable`/`NetlinkAttributeDeserializable` should be
/// used.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RawNetlinkAttribute<'a> {
    pub len: u16,
    pub ty: u16,
    pub payload: &'a [u8],
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ParseRawNetlinkAttributeError {
    #[error("Found a netlink attribute with an incomplete header. Saw {len} bytes, but at least {} were expected", size_of::<libc::nlattr>())]
    IncompleteHeader { len: usize },
    #[error("Found a netlink attribute with an insufficiently sized payload buffer. Expected a payload with {expected} bytes (determined from the header) but saw {actual} bytes.")]
    UnexpectedEndOfPayloadBuffer { actual: usize, expected: usize },
    #[error(transparent)]
    ParseNlaIntError(#[from] ParseNlaIntError),
}

impl<'a> TryFrom<&'a [u8]> for RawNetlinkAttribute<'a> {
    type Error = ParseRawNetlinkAttributeError;

    /// Reads the first 2 bytes from a buffer to determine the Netlink
    /// Attribute's length. The length bytes are interpreted with the host's
    /// natural endianness.
    ///
    /// It's acceptable to pass a longer buffer than necessary. The remaining
    /// bytes beyond the retrieved length will be ignored.
    fn try_from(buf: &'a [u8]) -> Result<Self, Self::Error> {
        if buf.len() < size_of::<libc::nlattr>() {
            return Err(Self::Error::IncompleteHeader { len: buf.len() });
        }

        let (header_bytes, remaining) = buf.split_at(size_of::<libc::nlattr>());

        let len = nla_get_u16(&header_bytes[0..size_of::<u16>()])?;
        let ty = nla_get_u16(&header_bytes[size_of::<u16>()..2 * size_of::<u16>()])?;
        let payload = {
            let payload_len = usize::from(len) - size_of::<libc::nlattr>();
            if remaining.len() < payload_len {
                return Err(Self::Error::UnexpectedEndOfPayloadBuffer {
                    actual: remaining.len(),
                    expected: payload_len,
                });
            }

            &remaining[..payload_len]
        };

        Ok(Self { len, ty, payload })
    }
}

#[cfg(test)]
mod tests {
    use super::{ParseRawNetlinkAttributeError, RawNetlinkAttribute};
    use std::{convert::TryFrom, mem::size_of};

    #[test]
    fn padded_buffer() {
        assert_eq!(
            RawNetlinkAttribute::try_from(&[7, 0, 0, 0, 1, 1, 1, 0][..]),
            Ok(RawNetlinkAttribute {
                len: 7,
                ty: 0,
                payload: &[1, 1, 1]
            })
        );
    }

    #[test]
    fn multiple_attributes() {
        assert_eq!(
            RawNetlinkAttribute::try_from(&[8, 0, 1, 0, 1, 1, 1, 1, 8, 0, 2, 0, 2, 2, 2, 2][..]),
            Ok(RawNetlinkAttribute {
                len: 8,
                ty: 1,
                payload: &[1, 1, 1, 1]
            }),
            "Only first attribute should be read from the buffer, safely ignoring extra data. This is the stated behavior in the method's documentation."
        );
    }

    #[test]
    fn incomplete_header_detection() {
        assert_eq!(
            size_of::<libc::nlattr>(),
            4,
            "This test relies on nlattr being 4 bytes. This shouldn't change unless a breaking Linux kernel release happens."
        );

        assert_eq!(
            RawNetlinkAttribute::try_from(&[3, 0, 0][..]),
            Err(ParseRawNetlinkAttributeError::IncompleteHeader { len: 3 })
        );

        assert!(!matches!(
            RawNetlinkAttribute::try_from(&[4, 0, 0, 0][..]),
            Err(ParseRawNetlinkAttributeError::IncompleteHeader { len: _ })
        ));

        assert!(!matches!(
            RawNetlinkAttribute::try_from(&[5, 0, 0, 0, 1][..]),
            Err(ParseRawNetlinkAttributeError::IncompleteHeader { len: _ })
        ));
    }

    #[test]
    fn incomplete_payload_detection() {
        assert_eq!(
            RawNetlinkAttribute::try_from(&[8, 0, 0, 0, 1, 1, 1, 1][..]),
            Ok(RawNetlinkAttribute {
                len: 8,
                ty: 0,
                payload: &[1, 1, 1, 1]
            })
        );

        assert_eq!(
            RawNetlinkAttribute::try_from(&[9, 0, 0, 0, 1, 1, 1, 1][..]),
            Err(
                ParseRawNetlinkAttributeError::UnexpectedEndOfPayloadBuffer {
                    actual: 4,
                    expected: 5
                }
            )
        );

        assert_eq!(
            RawNetlinkAttribute::try_from(&[10, 0, 0, 0, 1, 1, 1, 1][..]),
            Err(
                ParseRawNetlinkAttributeError::UnexpectedEndOfPayloadBuffer {
                    actual: 4,
                    expected: 6
                }
            )
        );
    }
}
