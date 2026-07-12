#![allow(non_camel_case_types, non_snake_case)]

//! `TROFFHeader`/`TROFFEntry`/`TROFF2Header`/`TROFF2Entry` — the `#[repr(C)]`
//! on-disk binary layouts for the two `.rof` file formats (v1 and v2).
//!
//! These are private nested `typedef struct` members of `CROFFSystem`
//! (`RoffSystem.h:54-89`) with **no member functions** — pure on-disk data,
//! read only by `IsROFF`/`InitROFF`/`InitROFF2` (`roff_system.rs`) to decode a
//! `.rof` file into a [`crate::roff::croff::Croff`]. Not a persisted Rust type
//! (ROFF-D4): it exists solely so the parse is bit-exact against the retail
//! 32-bit-Windows exporter's fixed layout.
//!
//! The on-disk `long mVersion` fields are the fixed 4-byte on-disk width
//! (32-bit Windows `long`), so they are Rust `i32`, **never** `c_long`
//! (`c_long` is 8 bytes under LP64 and would shift every following field's
//! offset, breaking bit-exactness — see `docs/subsystems/roff.md` ROFF-D4 and
//! the sibling SP `g_roff` port's `c_long` mistake it warns against).
//!
//! Type definition source: `oracle/codemp/qcommon/RoffSystem.h:54-89`

use core::ffi::c_char;

/// Raven `tROFFHeader` / `TROFFHeader` — the version-1 `.rof` file header.
///
/// Raven: "should match roff_string defined above" (`mHeader`); `mCount`: "I
/// think this is a float because of a limitation of the roff exporter".
///
/// Type definition source: `oracle/codemp/qcommon/RoffSystem.h:54-61`
#[repr(C)]
pub struct TROFFHeader {
    /// Should match [`crate::roff::ROFF_STRING`] — validated (with a
    /// famously off-by-one-byte compare, ROFF-V1) by `IsROFF`.
    pub mHeader: [c_char; 4],
    /// Version num; supported versions are [`crate::roff::ROFF_VERSION`] (1)
    /// and [`crate::roff::ROFF_NEW_VERSION`] (2).
    pub mVersion: i32,
    /// Entry count. A `float` "because of a limitation of the roff exporter".
    pub mCount: f32,
}

const _: () = assert!(core::mem::size_of::<TROFFHeader>() == 12);
const _: () = assert!(core::mem::offset_of!(TROFFHeader, mHeader) == 0);
const _: () = assert!(core::mem::offset_of!(TROFFHeader, mVersion) == 4);
const _: () = assert!(core::mem::offset_of!(TROFFHeader, mCount) == 8);

/// Raven `tROFFEntry` / `TROFFEntry` — one version-1 move/rotate entry.
///
/// Type definition source: `oracle/codemp/qcommon/RoffSystem.h:64-69`
#[repr(C)]
pub struct TROFFEntry {
    pub mOriginOffset: [f32; 3],
    pub mRotateOffset: [f32; 3],
}

const _: () = assert!(core::mem::size_of::<TROFFEntry>() == 24);
const _: () = assert!(core::mem::offset_of!(TROFFEntry, mOriginOffset) == 0);
const _: () = assert!(core::mem::offset_of!(TROFFEntry, mRotateOffset) == 12);

/// Raven `tROFF2Header` / `TROFF2Header` — the version-2 `.rof` file header.
///
/// Adds `mFrameRate` (playback frame rate) and `mNumNotes` (count of packed
/// NUL-terminated note-track strings following the roff data) over
/// [`TROFFHeader`].
///
/// Type definition source: `oracle/codemp/qcommon/RoffSystem.h:71-80`
#[repr(C)]
pub struct TROFF2Header {
    /// Should match [`crate::roff::ROFF_STRING`].
    pub mHeader: [c_char; 4],
    /// Version num; must equal [`crate::roff::ROFF_NEW_VERSION`] to route
    /// here.
    pub mVersion: i32,
    /// Entry count.
    pub mCount: i32,
    /// Frame rate the roff should be played at (`mLerp = 1000/mFrameRate`).
    pub mFrameRate: i32,
    /// Number of notes (NUL-terminated strings) after the roff data.
    pub mNumNotes: i32,
}

const _: () = assert!(core::mem::size_of::<TROFF2Header>() == 20);
const _: () = assert!(core::mem::offset_of!(TROFF2Header, mHeader) == 0);
const _: () = assert!(core::mem::offset_of!(TROFF2Header, mVersion) == 4);
const _: () = assert!(core::mem::offset_of!(TROFF2Header, mCount) == 8);
const _: () = assert!(core::mem::offset_of!(TROFF2Header, mFrameRate) == 12);
const _: () = assert!(core::mem::offset_of!(TROFF2Header, mNumNotes) == 16);

/// Raven `tROFF2Entry` / `TROFF2Entry` — one version-2 move/rotate entry.
///
/// Adds `mStartNote`/`mNumNotes` (note-track info for this frame) over
/// [`TROFFEntry`].
///
/// Type definition source: `oracle/codemp/qcommon/RoffSystem.h:83-89`
#[repr(C)]
pub struct TROFF2Entry {
    pub mOriginOffset: [f32; 3],
    pub mRotateOffset: [f32; 3],
    pub mStartNote: i32,
    pub mNumNotes: i32,
}

const _: () = assert!(core::mem::size_of::<TROFF2Entry>() == 32);
const _: () = assert!(core::mem::offset_of!(TROFF2Entry, mOriginOffset) == 0);
const _: () = assert!(core::mem::offset_of!(TROFF2Entry, mRotateOffset) == 12);
const _: () = assert!(core::mem::offset_of!(TROFF2Entry, mStartNote) == 24);
const _: () = assert!(core::mem::offset_of!(TROFF2Entry, mNumNotes) == 28);
