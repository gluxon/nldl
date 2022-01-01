# nldl

[![Build Status](https://github.com/gluxon/nldl/workflows/primary/badge.svg?branch=main)](https://github.com/gluxon/nldl/actions?query=workflow%3Aprimary)

nldl is a library for declaratively defining serializable/deserializable [Netlink](https://en.wikipedia.org/wiki/Netlink) data structures in Rust.

**This library is in active development and does not yet follow semantic versioning.** API changes will be actively made until a 1.0 release.

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
