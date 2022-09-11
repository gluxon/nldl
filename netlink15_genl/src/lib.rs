use nldl::message::NetlinkPayloadRequest;
use nldl::message::NetlinkPayloadResponse;
use std::mem::size_of;

pub mod socket;

/// See [genlmsghdr](libc::genlmsghdr)
#[derive(Debug, PartialEq, Eq)]
pub struct GenericNetlinkHeader {
    pub cmd: u8,
    pub version: u8,
}

impl GenericNetlinkHeader {
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(self.cmd);
        buf.push(self.version);

        // Add 2 reserved bytes per genlmsghdr's C definition.
        buf.push(0);
        buf.push(0);
    }

    fn deserialize(buf: [u8; size_of::<libc::genlmsghdr>()]) -> Self {
        Self {
            cmd: *buf.first().unwrap(),
            version: *buf.get(1).unwrap(),
        }
    }
}

pub struct GenericNetlinkRequest<T: NetlinkPayloadRequest> {
    pub header: GenericNetlinkHeader,
    pub payload: T,
}

impl<T: NetlinkPayloadRequest> NetlinkPayloadRequest for GenericNetlinkRequest<T> {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.header.serialize(buf);
        self.payload.serialize(buf);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct GenericNetlinkResponse<T: NetlinkPayloadResponse> {
    pub header: GenericNetlinkHeader,
    pub payload: T,
}

impl<T: NetlinkPayloadResponse> NetlinkPayloadResponse for GenericNetlinkResponse<T> {
    type Error = T::Error;

    fn deserialize(buf: &[u8]) -> Result<Self, Self::Error> {
        let (header_bytes, payload_bytes) = buf.split_at(size_of::<libc::genlmsghdr>());

        let header_bytes = {
            let mut arr = [0u8; size_of::<libc::genlmsghdr>()];
            arr.clone_from_slice(header_bytes);
            arr
        };

        let header = GenericNetlinkHeader::deserialize(header_bytes);
        let payload = T::deserialize(payload_bytes)?;

        Ok(Self { header, payload })
    }
}
