use std::mem::size_of;
use std::string::FromUtf8Error;

#[derive(thiserror::Error, Clone, Debug, PartialEq, Eq)]
pub enum ParseNlaIntError {
    #[error(
        "Invalid buffer length for integer. Expected {} found {}.",
        expected,
        found
    )]
    InvalidLength { expected: usize, found: usize },
}

macro_rules! create_nla_get_int {
    ($func_name: ident, $int_type: ident, $int_type_len: expr) => {
        pub fn $func_name(buf: &[u8]) -> Result<$int_type, ParseNlaIntError> {
            if (buf.len() != $int_type_len) {
                return Err(ParseNlaIntError::InvalidLength {
                    expected: $int_type_len,
                    found: buf.len(),
                });
            }

            let mut arr = [0u8; $int_type_len];
            arr.copy_from_slice(&buf);
            Ok($int_type::from_ne_bytes(arr))
        }
    };
}

create_nla_get_int!(nla_get_u8, u8, size_of::<u8>());
create_nla_get_int!(nla_get_u16, u16, size_of::<u16>());
create_nla_get_int!(nla_get_u32, u32, size_of::<u32>());
create_nla_get_int!(nla_get_u64, u64, size_of::<u64>());
create_nla_get_int!(nla_get_i64, i64, size_of::<i64>());

macro_rules! create_nla_put_int {
    ($func_name: ident, $int_type: ident) => {
        pub fn $func_name(buf: &mut Vec<u8>, value: $int_type) {
            buf.extend_from_slice(&value.to_ne_bytes()[..]);
        }
    };
}

create_nla_put_int!(nla_put_u8, u8);
create_nla_put_int!(nla_put_u16, u16);
create_nla_put_int!(nla_put_u32, u32);
create_nla_put_int!(nla_put_u64, u64);

#[derive(thiserror::Error, Clone, Debug)]
pub enum NlaGetStringError {
    #[error("Cannot parse empty buffer as String")]
    NullBuffer,
    #[error("Expected a null-terminated string. Found byte {:#04x} instead", .0)]
    NotNullTerminated(u8),
    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),
}

pub fn nla_get_string(buf: &[u8]) -> Result<String, NlaGetStringError> {
    // Although payload is a known length, a null-terminated C string is still
    // sent over netlink. We should check that this was the case before dropping
    // the last character (which should be null).
    let mut payload = buf.to_vec();
    let last_byte = payload.pop();

    match last_byte {
        None => Err(NlaGetStringError::NullBuffer),
        Some(0) => Ok(String::from_utf8(payload)?),
        Some(last_byte) => Err(NlaGetStringError::NotNullTerminated(last_byte)),
    }
}

pub fn nla_put_string(buf: &mut Vec<u8>, val: &str) {
    buf.extend_from_slice(val.as_bytes());
    // netlink strings are C strings which are null-terminated.
    buf.push(0);
}
