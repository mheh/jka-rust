#![allow(non_camel_case_types, non_snake_case)]

/// Raven `serverAddress_t` — a saved server address (favorites list entry).
///
/// Type definition source: `oracle/codemp/client/client.h:290-293`
#[repr(C)]
pub struct serverAddress_t {
    pub ip: [u8; 4],
    pub port: u16,
}

const _: () = assert!(core::mem::size_of::<serverAddress_t>() == 6);
const _: () = assert!(core::mem::offset_of!(serverAddress_t, ip) == 0);
const _: () = assert!(core::mem::offset_of!(serverAddress_t, port) == 4);
