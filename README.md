# nldl

[![Build Status](https://github.com/gluxon/nldl/workflows/primary/badge.svg?branch=main)](https://github.com/gluxon/nldl/actions?query=workflow%3Aprimary)
[![codecov](https://codecov.io/gh/gluxon/wireguard-uapi-rs/branch/develop/graph/badge.svg)](https://codecov.io/gh/gluxon/nldl)
![MIT](https://img.shields.io/github/license/gluxon/nldl)

nldl is a library for declaratively defining serializable/deserializable [Netlink](https://en.wikipedia.org/wiki/Netlink) data structures in Rust.

The declarative nature of this library is expected to reduce the amount of effort needed to implement Rust support for new Netlink protocols. **This library is experimental and not comprehensively documented.** Until it reaches a stable release, we recommend existing alternatives such as [neli](https://github.com/jbaublitz/neli) and [little-dude/netlink](https://github.com/little-dude/netlink).

## Example

Here's how the [`family_op_policy` struct from libnl](https://www.infradead.org/~tgr/libnl/doc/api/ctrl_8c_source.html#l00054) might be defined with nldl.

```c
static struct nla_policy family_op_policy[CTRL_ATTR_OP_MAX+1] = {
        [CTRL_ATTR_OP_ID]       = { .type = NLA_U32 },
        [CTRL_ATTR_OP_FLAGS]    = { .type = NLA_U32 },
};
```

```rust
use nldl::attr::UnknownAttribute;

#[derive(Debug, PartialEq, nldl::attr::Serialize, nldl::attr::Deserialize)]
pub enum ControllerAttributeOperation {
    #[nla_type(libc::CTRL_ATTR_OP_UNSPEC as u16)]
    Unspec,
    #[nla_type(libc::CTRL_ATTR_OP_ID as u16)]
    Id(u32),
    #[nla_type(libc::CTRL_ATTR_OP_FLAGS as u16)]
    Flags(u32),
    #[nla_type(_)]
    Unknown(UnknownAttribute),
}
```

## FAQ

### Why another Rust netlink library?

`nldl` provides a derive trait with the goal of making it easier to describe how Rust data structures map to Netlink messages. At the time of writing, existing Rust netlink libraries require more manual (de)serialization implementation for each new message/attribute type.

### Why not use serde?

`nldl` is heavily inspired by [`serde`](https://serde.rs/), but it's concerned with a bit more than `serde` in some areas, and a bit less in others. This makes the functionality gap significant enough to experiment.

- `nldl` may also provide a common runtime library for sending/receiving Netlink messages derived from the library.
- `serde` allows data structures to be serialized/deserialized into multiple different formats (e.g. json, toml). This flexibility may not be //! valuable for data structures specific to the Netlink protocol.

License: MIT
