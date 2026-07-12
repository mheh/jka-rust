#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::qboolean;

/// Raven `objectives_t` — a displayable mission objective.
///
/// Type definition source: `oracle/code/game/g_shared.h:292-296`
#[repr(C)]
pub struct objectives_t {
    /// A displayable objective?
    pub display: qboolean,
    /// Succeed or fail or pending.
    pub status: core::ffi::c_int,
}

const _: () = assert!(core::mem::size_of::<objectives_t>() == 8);
const _: () = assert!(core::mem::offset_of!(objectives_t, display) == 0);
const _: () = assert!(core::mem::offset_of!(objectives_t, status) == 4);
