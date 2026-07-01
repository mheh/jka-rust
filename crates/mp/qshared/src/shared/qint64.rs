#![allow(non_camel_case_types)]

use native_types::byte;

/// Raven `qint64` — 64-bit integer for the global rankings interface,
/// implemented as a byte struct for qvm compatibility.
///
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:1726-1736`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct qint64 {
    pub b0: byte,
    pub b1: byte,
    pub b2: byte,
    pub b3: byte,
    pub b4: byte,
    pub b5: byte,
    pub b6: byte,
    pub b7: byte,
}

const _: () = {
    assert!(core::mem::size_of::<qint64>() == 8);
};
