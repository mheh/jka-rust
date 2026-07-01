#![allow(non_camel_case_types)]

/// Raven `fsOrigin_t` — seek origin for filesystem operations.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:1193-1197`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum fsOrigin_t {
    FS_SEEK_CUR,
    FS_SEEK_END,
    FS_SEEK_SET,
}
