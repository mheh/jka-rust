#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::c_int;

/// Raven presence types (`PRESENCE_*`) — bot body-presence flags used for AAS area queries.
///
/// Source: `oracle/codemp/botlib/aasfile.h:11-13`
pub const PRESENCE_NONE: c_int = 1;
pub const PRESENCE_NORMAL: c_int = 2;
pub const PRESENCE_CROUCH: c_int = 4;
