use self::raw::RawNetlinkMessage;
use self::raw::ReadRawNetlinkMessageError;
use super::utils::nla_get_string;
use super::utils::nla_get_u16;
use super::utils::nla_get_u32;
use super::utils::nla_put_string;
use super::utils::nla_put_u16;
use super::utils::nla_put_u32;
use super::utils::NlaGetStringError;
use super::utils::ParseNlaIntError;
use super::write_to_buf_with_prefixed_u32_len;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt::Debug;
use std::mem::size_of;

mod raw;
pub mod utils;

/// Similar to [nlmsghdr][libc::nlmsghdr] and
/// [RawNetlinkMessageHeader](RawNetlinkMessageHeader) but omits the `len` field.
#[derive(Debug)]
pub struct NetlinkMessageHeader {
    pub ty: u16,
    pub flags: u16,
    pub seq: u32,
    pub pid: u32,
}

impl NetlinkMessageHeader {
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.ty.to_ne_bytes()[..]);
        buf.extend_from_slice(&self.flags.to_ne_bytes()[..]);
        buf.extend_from_slice(&self.seq.to_ne_bytes()[..]);
        buf.extend_from_slice(&self.pid.to_ne_bytes()[..]);
    }
}

impl From<RawNetlinkMessageHeader> for NetlinkMessageHeader {
    fn from(raw: RawNetlinkMessageHeader) -> Self {
        Self {
            ty: raw.ty,
            flags: raw.flags,
            seq: raw.seq,
            pid: raw.pid,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RawNetlinkMessageHeader {
    pub len: u32,
    pub ty: u16,
    pub flags: u16,
    pub seq: u32,
    pub pid: u32,
}

impl RawNetlinkMessageHeader {
    fn deserialize(buf: &[u8; size_of::<libc::nlmsghdr>()]) -> Self {
        Self {
            len: u32::from_ne_bytes((&buf[0..4]).try_into().unwrap()),
            ty: u16::from_ne_bytes((&buf[4..6]).try_into().unwrap()),
            flags: u16::from_ne_bytes((&buf[6..8]).try_into().unwrap()),
            seq: u32::from_ne_bytes((&buf[8..12]).try_into().unwrap()),
            pid: u32::from_ne_bytes((&buf[12..16]).try_into().unwrap()),
        }
    }
}

pub struct NetlinkMessageRequest<T: NetlinkPayloadRequest> {
    pub header: NetlinkMessageHeader,
    pub payload: NetlinkMessageType<T>,
}

impl<T: NetlinkPayloadRequest> NetlinkMessageRequest<T> {
    pub fn serialize(&self, buf: &mut Vec<u8>) {
        write_to_buf_with_prefixed_u32_len(buf, |buf| {
            self.header.serialize(buf);
            self.payload.serialize(buf);
        });
    }
}

#[derive(Debug)]
pub struct NetlinkMessageResponse<T: NetlinkPayloadResponse> {
    pub header: NetlinkMessageHeader,
    pub payload: NetlinkMessageType<T>,
}

pub type DeserializeNetlinkMessageResult<T> =
    Result<NetlinkMessageResponse<T>, NetlinkMessageResponseDeserializeError<T>>;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum NetlinkMessageResponseDeserializeError<T: NetlinkPayloadResponse> {
    #[error(transparent)]
    RawReadError(#[from] ReadRawNetlinkMessageError),

    // There's a cryptic compiler error message when #[error(transparent)] is
    // set on the generic below.
    #[error("{0}")]
    PayloadDeserialize(T::Error),
}

impl<T: NetlinkPayloadResponse> NetlinkMessageResponse<T> {
    pub fn deserialize(buf: &[u8]) -> Result<Self, NetlinkMessageResponseDeserializeError<T>> {
        let raw = RawNetlinkMessage::try_from(buf)?;
        raw.try_into()
    }
}

impl<T: NetlinkPayloadResponse> TryFrom<RawNetlinkMessage<'_>> for NetlinkMessageResponse<T> {
    type Error = NetlinkMessageResponseDeserializeError<T>;

    fn try_from(raw: RawNetlinkMessage<'_>) -> Result<Self, Self::Error> {
        let header: NetlinkMessageHeader = raw.header.into();
        let payload = NetlinkMessageType::deserialize(header.ty, &raw.payload)
            .map_err(NetlinkMessageResponseDeserializeError::PayloadDeserialize)?;

        Ok(Self { header, payload })
    }
}

/// https://www.infradead.org/~tgr/libnl/doc/core.html#core_msg_types
#[derive(Debug)]
pub enum NetlinkMessageType<T> {
    Noop,
    Error,
    Done,
    Overrun,
    Other(T),
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
            NetlinkMessageType::Noop => {}
            NetlinkMessageType::Error => {}
            NetlinkMessageType::Done => {}
            NetlinkMessageType::Overrun => {}
            NetlinkMessageType::Other(payload) => payload.serialize(buf),
        }
    }
}

impl<T> NetlinkMessageType<T>
where
    T: NetlinkPayloadResponse,
{
    pub fn deserialize(message_type: u16, buf: &[u8]) -> Result<Self, T::Error> {
        match message_type {
            NLMSG_NOOP => Ok(Self::Noop),
            NLMSG_ERROR => Ok(Self::Error),
            NLMSG_DONE => Ok(Self::Done),
            NLMSG_OVERRUN => Ok(Self::Overrun),
            _ => T::deserialize(buf).map(Self::Other),
        }
    }
}

pub trait NetlinkPayloadRequest {
    fn serialize(&self, buf: &mut Vec<u8>);
}

pub trait NetlinkPayloadResponse: Debug + PartialEq + Sized {
    type Error: Debug + std::error::Error;
    fn deserialize(buf: &[u8]) -> Result<Self, Self::Error>;
}

impl NetlinkPayloadRequest for u16 {
    fn serialize(&self, buf: &mut Vec<u8>) {
        nla_put_u16(buf, *self);
    }
}

impl NetlinkPayloadRequest for u32 {
    fn serialize(&self, buf: &mut Vec<u8>) {
        nla_put_u32(buf, *self);
    }
}

impl NetlinkPayloadRequest for String {
    fn serialize(&self, buf: &mut Vec<u8>) {
        nla_put_string(buf, self);
    }
}

impl<T: NetlinkPayloadRequest> NetlinkPayloadRequest for Option<T> {
    fn serialize(&self, buf: &mut Vec<u8>) {
        match self {
            Some(val) => val.serialize(buf),
            None => {}
        }
    }
}

impl NetlinkPayloadResponse for u16 {
    type Error = ParseNlaIntError;

    fn deserialize(buf: &[u8]) -> Result<Self, Self::Error> {
        nla_get_u16(buf)
    }
}

impl NetlinkPayloadResponse for u32 {
    type Error = ParseNlaIntError;

    fn deserialize(buf: &[u8]) -> Result<Self, Self::Error> {
        nla_get_u32(buf)
    }
}

impl NetlinkPayloadResponse for Vec<u8> {
    type Error = std::convert::Infallible;

    fn deserialize(buf: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self::from(buf))
    }
}

impl NetlinkPayloadResponse for String {
    type Error = NlaGetStringError;

    fn deserialize(buf: &[u8]) -> Result<Self, Self::Error> {
        nla_get_string(buf)
    }
}
