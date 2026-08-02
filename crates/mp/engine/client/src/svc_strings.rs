//! `svc_strings` — the `svc_ops_e` debug-name table `CL_ParseServerMessage`
//! indexes when `cl_shownet` is on.
//!
//! Source: `oracle/codemp/client/cl_parse.cpp:22-39`

#![allow(non_upper_case_globals)]

/// Raven `char *svc_strings[256]`. The ten named commands fill indices 0-9;
/// the rest stay Raven's zero-initialized `NULL`, which `CL_ParseServerMessage`
/// reads as the "BAD CMD" branch (`svc_strings[cmd].is_empty()` here).
///
/// Source: `oracle/codemp/client/cl_parse.cpp:22-39`
pub const svc_strings: [&str; 256] = {
    let mut arr = [""; 256];
    arr[0] = "svc_bad";
    arr[1] = "svc_nop";
    arr[2] = "svc_gamestate";
    arr[3] = "svc_configstring";
    arr[4] = "svc_baseline";
    arr[5] = "svc_serverCommand";
    arr[6] = "svc_download";
    arr[7] = "svc_snapshot";
    arr[8] = "svc_setgame";
    arr[9] = "svc_mapchange";
    arr
};
