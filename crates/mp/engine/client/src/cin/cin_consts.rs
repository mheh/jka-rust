//! `cl_cin.cpp` file-scope `#define`s.

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_long, c_uint};

/// Raven `MAXSIZE` / `MINSIZE` — the largest and smallest RoQ quad edge, in pixels.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:22-23`
pub const MAXSIZE: c_long = 8;
pub const MINSIZE: c_long = 4;

/// Raven `DEFAULT_CIN_WIDTH` / `DEFAULT_CIN_HEIGHT` — the `linbuf` sizing bound.
/// A RoQ stream never decodes wider or taller than this.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:25-26`
pub const DEFAULT_CIN_WIDTH: c_long = 512;
pub const DEFAULT_CIN_HEIGHT: c_long = 512;

/// Raven RoQ chunk ids, read from the two-byte chunk header.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:28-36`
pub const ROQ_QUAD: c_uint = 0x1000;
pub const ROQ_QUAD_INFO: c_uint = 0x1001;
pub const ROQ_CODEBOOK: c_uint = 0x1002;
pub const ROQ_QUAD_VQ: c_uint = 0x1011;
pub const ROQ_QUAD_JPEG: c_uint = 0x1012;
pub const ROQ_QUAD_HANG: c_uint = 0x1013;
pub const ROQ_PACKET: c_uint = 0x1030;
pub const ZA_SOUND_MONO: c_uint = 0x1020;
pub const ZA_SOUND_STEREO: c_uint = 0x1021;

/// Raven `MAX_VIDEO_HANDLES` — slots in `cinTable`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:38`
pub const MAX_VIDEO_HANDLES: usize = 16;
