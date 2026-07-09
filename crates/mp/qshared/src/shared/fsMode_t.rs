use core::ffi::c_int;

/// Raven `fsMode_t`.
///
/// Raven comment: `mode parm for FS_FOpenFile`
///
/// Type definition source: `oracle/code/game/q_shared.h:1185`
/// Type definition source: `oracle/codemp/game/q_shared.h:1684`
pub type fsMode_t = c_int;

pub const FS_READ: fsMode_t = 0;
pub const FS_WRITE: fsMode_t = 1;
pub const FS_APPEND: fsMode_t = 2;
pub const FS_APPEND_SYNC: fsMode_t = 3;
