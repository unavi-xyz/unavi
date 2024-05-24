//! Rust types for [The Wired](https://github.com/unavi-xyz/wired-protocol)'s world protocol.

pub mod world_server_capnp {
    include!(concat!(env!("OUT_DIR"), "/world_server_capnp.rs"));
}

pub mod datagram_capnp {
    include!(concat!(env!("OUT_DIR"), "/datagram_capnp.rs"));
}
