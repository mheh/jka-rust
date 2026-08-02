//! `client.h` and `cg_public.h` constants the client engine reads.

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// Raven `RETRANSMIT_TIMEOUT` — milliseconds between connection-packet retries.
///
/// Source: `oracle/codemp/client/client.h:19`
pub const RETRANSMIT_TIMEOUT: c_int = 3000;

/// Raven `MAX_PARSE_ENTITIES` — entries in `cl.parseEntities`, the ring that
/// holds `PACKET_BACKUP` frames of delta-decoded entities. This is the
/// non-Xbox value (`_XBOX` gives `1024`, and this project builds only the
/// non-Xbox branch).
///
/// Source: `oracle/codemp/client/client.h:68-71`
pub const MAX_PARSE_ENTITIES: usize = 2048;

/// Raven `CMD_BACKUP` — usercmd ring length — and `CMD_MASK`, its index mask.
///
/// Source: `oracle/codemp/cgame/cg_public.h:6-7`
pub const CMD_BACKUP: c_int = 64;
pub const CMD_MASK: c_int = CMD_BACKUP - 1;

/// Raven `RESET_TIME` — milliseconds `CL_CGameSystemCalls` holds the automap
/// reset flag after a view change.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:1868`
pub const RESET_TIME: c_int = 500;
