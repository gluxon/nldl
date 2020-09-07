// This file should be moved to libc.

pub const NLMSG_ALIGNTO: usize = libc::NLA_ALIGNTO as usize;

// From netlink.h
pub const fn nlmsg_align(len: usize) -> usize {
    (len + NLMSG_ALIGNTO - 1) & !(NLMSG_ALIGNTO - 1)
}
