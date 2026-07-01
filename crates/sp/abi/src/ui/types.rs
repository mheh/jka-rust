//! SP UI ABI-local payload marker types.

#![allow(non_camel_case_types)]

use core::ffi::c_int;

/// Raven `e_status` cinematic state wire value.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:2670-2679`
///
/// Kept as a transparent integer rather than a Rust enum so decoding syscall
/// words remains ABI-safe even if an engine sends an out-of-range value.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct e_status(c_int);

impl e_status {
    /// Raven `FMV_IDLE`.
    pub const FMV_IDLE: Self = Self(0);
    /// Raven `FMV_PLAY`.
    pub const FMV_PLAY: Self = Self(1);
    /// Raven `FMV_EOF`.
    pub const FMV_EOF: Self = Self(2);
    /// Raven `FMV_ID_BLT`.
    pub const FMV_ID_BLT: Self = Self(3);
    /// Raven `FMV_ID_IDLE`.
    pub const FMV_ID_IDLE: Self = Self(4);
    /// Raven `FMV_LOOPED`.
    pub const FMV_LOOPED: Self = Self(5);
    /// Raven `FMV_ID_WAIT`.
    pub const FMV_ID_WAIT: Self = Self(6);

    pub const fn from_wire(value: c_int) -> Self {
        Self(value)
    }

    pub const fn as_wire(self) -> c_int {
        self.0
    }
}

/// Opaque Raven `uiClientState_t` payload.
///
/// Type definition source: `oracle/oracle/codemp/ui/ui_public.h:7-15`
///
/// SP Raven comments out the `trap_GetClientState` syscall wrapper in
/// `oracle/oracle/code/ui/ui_syscalls.cpp:148-153`; keeping the payload opaque
/// avoids claiming an SP layout that is not active in the SP UI transport.
#[repr(C)]
pub struct uiClientState_t {
    _private: [u8; 0],
}

/// Raven `refdef_t` renderer scene description — the full layout now lives in
/// `sp_qshared` (`common/sp/renderer/refdef_t.rs`); re-exported here for the
/// syscall wrappers that transport it by pointer.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_types.h:159-176`
pub use sp_qshared::common::sp::renderer::refdef_t::refdef_t;
