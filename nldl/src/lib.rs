//! [![Build Status](https://github.com/gluxon/nldl/workflows/primary/badge.svg?branch=main)](https://github.com/gluxon/nldl/actions?query=workflow%3Aprimary)
//!
//! nldl is a library for declaratively defining serializable/deserializable [Netlink](https://en.wikipedia.org/wiki/Netlink) data structures in Rust.
//!
//! **This library is in active development and does not yet follow semantic versioning.** API changes will be actively made until a 1.0 release.
//!
//! # Example
//!
//! Here's how the [`family_op_policy` struct from libnl](https://www.infradead.org/~tgr/libnl/doc/api/ctrl_8c_source.html#l00054) might be defined with nldl.
//!
//! ```c
//! static struct nla_policy family_op_policy[CTRL_ATTR_OP_MAX+1] = {
//!         [CTRL_ATTR_OP_ID]       = { .type = NLA_U32 },
//!         [CTRL_ATTR_OP_FLAGS]    = { .type = NLA_U32 },
//! };
//! ```
//!
//! ```
//! use nldl::attr::UnknownAttribute;
//! use nldl::utils::ParseNlaIntError;
//!
//! #[derive(Debug, PartialEq, nldl::attr::Serialize, nldl::attr::Deserialize)]
//! #[nldl(deserialize(error = "ParseNlaIntError"))]
//! pub enum ControllerAttributeOperation {
//!     #[nla_type(libc::CTRL_ATTR_OP_UNSPEC as u16)]
//!     Unspec,
//!     #[nla_type(libc::CTRL_ATTR_OP_ID as u16)]
//!     Id(u32),
//!     #[nla_type(libc::CTRL_ATTR_OP_FLAGS as u16)]
//!     Flags(u32),
//!     #[nla_type(_)]
//!     Unknown(UnknownAttribute),
//! }
//! ```

use std::mem::size_of;

pub mod attr;
mod linux;
pub mod message;
pub mod utils;

use message::*;

pub fn serialize<T: NetlinkPayloadRequest>(message: &NetlinkMessageRequest<T>) -> Vec<u8> {
    let mut bytes = vec![];
    message.serialize(&mut bytes);
    bytes
}

fn write_to_buf_with_prefixed_u32_len<F>(buf: &mut Vec<u8>, write: F)
where
    F: FnOnce(&mut Vec<u8>),
{
    let num_bytes_before = buf.len();
    buf.extend_from_slice(&[0u8; size_of::<u32>()]);
    let len_bytes_range = num_bytes_before..buf.len();

    write(buf);

    let num_bytes_after = buf.len();
    // TODO: Propagate this error properly
    let message_len = u32::try_from(num_bytes_after - num_bytes_before).unwrap();

    buf.splice(len_bytes_range, message_len.to_ne_bytes().iter().cloned());
}

fn write_to_buf_with_prefixed_u16_len<F>(buf: &mut Vec<u8>, write: F)
where
    F: FnOnce(&mut Vec<u8>),
{
    let num_bytes_before = buf.len();
    buf.extend_from_slice(&[0u8; size_of::<u16>()]);
    let len_bytes_range = num_bytes_before..buf.len();

    write(buf);

    let num_bytes_after = buf.len();
    // TODO: Propagate this error properly
    let message_len = u16::try_from(num_bytes_after - num_bytes_before).unwrap();

    buf.splice(len_bytes_range, message_len.to_ne_bytes().iter().cloned());
}
