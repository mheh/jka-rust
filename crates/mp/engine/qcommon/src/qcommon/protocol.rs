#![allow(non_camel_case_types, non_snake_case)]

/// Raven `PROTOCOL_VERSION`.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:205`
pub const PROTOCOL_VERSION: i32 = 26;

/// Raven `UPDATE_SERVER_NAME`.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:208`
pub const UPDATE_SERVER_NAME: &str = "updatejk3.ravensoft.com";

/// Raven `MASTER_SERVER_NAME`.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:209`
pub const MASTER_SERVER_NAME: &str = "masterjk3.ravensoft.com";

// Raven's `AUTHORIZE_SERVER_NAME` (`qcommon.h:212`) is compiled only under
// `USE_CD_KEY`, which is `#define`d out (`qcommon.h:10`) — never defined in
// a live build, so it is not ported.

// Non-`_XBOX` branch (`qcommon.h:217-227`); the engine never builds `_XBOX`.

/// Raven `PORT_MASTER`.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:220`
pub const PORT_MASTER: i32 = 29060;

/// Raven `PORT_UPDATE`.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:221`
pub const PORT_UPDATE: i32 = 29061;

/// Raven `PORT_SERVER` — ...+9 more for multiple servers.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:223`
pub const PORT_SERVER: i32 = 29070;

/// Raven `NUM_SERVER_PORTS` — broadcast scan this many ports after
/// `PORT_SERVER` so a single machine can run multiple servers.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:224`
pub const NUM_SERVER_PORTS: i32 = 4;
