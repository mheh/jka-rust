#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// Raven `MAX_SERVERSPERPACKET` — max server entries parsed from a single
/// master-server response packet in `CL_ServersResponsePacket`.
///
/// Source: `oracle/codemp/client/cl_main.cpp:1827`
pub const MAX_SERVERSPERPACKET: c_int = 256;

/// Raven `MAX_STRINGED_SV_STRING` — max buffer size for
/// `CL_CheckSVStringEdRef`'s string-editor-reference scratch buffer.
///
/// Source: `oracle/codemp/client/cl_main.cpp:1949`
pub const MAX_STRINGED_SV_STRING: c_int = 1024;

/// Raven `MAXPRINTMSG` — max size of `CL_RefPrintf`'s formatted-message
/// buffer (a client-local copy distinct from `Com_Printf`'s of the same name
/// and value in `common.cpp`).
///
/// Source: `oracle/codemp/client/cl_main.cpp:2386`
pub const MAXPRINTMSG: c_int = 4096;

/// Raven `MODEL_CHANGE_DELAY` — milliseconds between allowed `model` cvar
/// changes (enforcement is dead/commented out in `CL_SetModel_f`, but the
/// constant itself is live).
///
/// Source: `oracle/codemp/client/cl_main.cpp:2504`
pub const MODEL_CHANGE_DELAY: c_int = 5000;

/// Raven `G2_VERT_SPACE_CLIENT_SIZE`.
///
/// Source: `oracle/codemp/client/cl_main.cpp:2542`
pub const G2_VERT_SPACE_CLIENT_SIZE: c_int = 256;
