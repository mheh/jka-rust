//! SP cgame ABI-local payload marker types.

#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int};

/// Raven `clipHandle_t` collision model handle.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:188`
pub type clipHandle_t = c_int;

/// Raven `memtag_t` zone tag wire value.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:2688`
///
/// Kept as a transparent `char`-sized value matching Raven's typedef. Syscalls
/// still carry it in an integer word before the engine casts to `memtag_t`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct memtag_t(c_char);

impl memtag_t {
    pub const fn from_wire(value: c_char) -> Self {
        Self(value)
    }

    pub const fn as_wire(self) -> c_char {
        self.0
    }
}

/// Raven cinematic status wire value.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:2670-2679`
///
/// Kept as a transparent integer rather than a Rust enum so decoding syscall
/// return words remains ABI-safe even if an engine sends an out-of-range value.
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

/// `markFragment_t` ABI record returned through `fragmentBuffer`.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:1402-1405`
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct markFragment_t {
    pub firstPoint: c_int,
    pub numPoints: c_int,
}

/// Raven `stereoFrame_t` wire value.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_types.h:183-187`
///
/// Kept as a transparent integer rather than a Rust enum so decoding vmMain
/// words remains ABI-safe even if an engine sends an out-of-range value.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct stereoFrame_t(c_int);

impl stereoFrame_t {
    /// Raven `STEREO_CENTER`.
    pub const STEREO_CENTER: Self = Self(0);
    /// Raven `STEREO_LEFT`.
    pub const STEREO_LEFT: Self = Self(1);
    /// Raven `STEREO_RIGHT`.
    pub const STEREO_RIGHT: Self = Self(2);

    pub const fn from_wire(value: c_int) -> Self {
        Self(value)
    }

    pub const fn as_wire(self) -> c_int {
        self.0
    }
}

/// Opaque Raven `CGhoul2Info_v` C++ class.
///
/// Type definition source: `oracle/oracle/code/game/ghoul2_shared.h:311`
#[repr(C)]
pub struct CGhoul2Info_v {
    _private: [u8; 0],
}

/// Opaque Raven `surfaceInfo_v` C++ vector alias.
///
/// Type definition source: `oracle/oracle/code/game/ghoul2_shared.h:201`
#[repr(C)]
pub struct surfaceInfo_v {
    _private: [u8; 0],
}

/// Opaque Raven `boneInfo_v` C++ vector alias.
///
/// Type definition source: `oracle/oracle/code/game/ghoul2_shared.h:202`
#[repr(C)]
pub struct boneInfo_v {
    _private: [u8; 0],
}

/// Opaque Raven `boltInfo_v` C++ vector alias.
///
/// Type definition source: `oracle/oracle/code/game/ghoul2_shared.h:203`
#[repr(C)]
pub struct boltInfo_v {
    _private: [u8; 0],
}

/// Opaque Raven `mdxaBone_v` C++ vector alias.
///
/// Type definition source: `oracle/oracle/code/game/ghoul2_shared.h:204`
#[repr(C)]
pub struct mdxaBone_v {
    _private: [u8; 0],
}
