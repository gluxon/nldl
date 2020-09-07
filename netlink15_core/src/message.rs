use super::utils::nla_put_string;
use super::utils::nla_put_u16;
use super::utils::nla_put_u32;
use super::write_to_buf_with_prefixed_u32_len;
use std::{convert::TryInto, mem::size_of};

/// Similar to [nlmsghdr][libc::nlmsghdr] and
/// [RawNetlinkMessageHeader](RawNetlinkMessageHeader) but omits the `len` field.
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

pub struct RawNetlinkMessageHeader {
    pub len: u32,
    pub ty: u16,
    pub flags: u16,
    pub seq: u32,
    pub pid: u32,
}

impl RawNetlinkMessageHeader {
    fn deserialize(buf: [u8; size_of::<libc::nlmsghdr>()]) -> Self {
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
    pub payload: T,
}

impl<T: NetlinkPayloadRequest> NetlinkMessageRequest<T> {
    pub fn serialize(&self, buf: &mut Vec<u8>) {
        write_to_buf_with_prefixed_u32_len(buf, |buf| {
            self.header.serialize(buf);
            self.payload.serialize(buf);
        });
    }
}

pub struct NetlinkMessageResponse<T: NetlinkPayloadResponse> {
    pub header: NetlinkMessageHeader,
    pub payload: T,
}

impl<T: NetlinkPayloadResponse> NetlinkMessageResponse<T> {
    pub fn deserialize(buf: &[u8]) -> Result<Self, T::Error> {
        let (header_bytes, payload_bytes) = buf.split_at(size_of::<libc::nlmsghdr>());

        let header_bytes = {
            let mut arr: [u8; size_of::<libc::nlmsghdr>()] = Default::default();
            arr.clone_from_slice(&header_bytes);
            arr
        };

        let raw_header = RawNetlinkMessageHeader::deserialize(header_bytes);
        let len = raw_header.len;
        let header: NetlinkMessageHeader = raw_header.into();

        let payload_len = (len as usize) - size_of::<libc::nlmsghdr>();
        let payload = T::deserialize(&payload_bytes[..payload_len])?;

        Ok(Self { header, payload })
    }
}

pub trait NetlinkPayloadRequest {
    fn serialize(&self, buf: &mut Vec<u8>);
}

pub trait NetlinkPayloadResponse: PartialEq + Sized {
    type Error: std::fmt::Debug;
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
