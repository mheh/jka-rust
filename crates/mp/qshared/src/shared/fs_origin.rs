#![allow(non_camel_case_types)]

/// Raven `fsOrigin_t` filesystem seek origins.
///
/// Type definition source: `oracle/codemp/game/q_shared.h:1692-1696`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum fsOrigin_t {
    FS_SEEK_CUR,
    FS_SEEK_END,
    FS_SEEK_SET,
}
