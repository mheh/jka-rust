//! Error state + the receiverless `com_error` leaf throw (STATE-D7 / STATE-Q4).

use mp_qshared::shared::errorParm_t;

/// state-ownership names the `ComError` level-field type `ErrorLevel`;
/// lifecycle.md freezes that name here (LIFE-D3) as the per-mode `errorParm_t`
/// (MP 5-variant / SP 4-variant). The alias makes `com_error`'s `errorParm_t`
/// and `ComError.level`'s `ErrorLevel` one type.
pub type ErrorLevel = errorParm_t;

/// The `Common.error` field group (state-ownership § qcommon — Common).
/// `last_error_time`/`error_count` are the MP-only rapid-error statics
/// (`common.cpp:251-252`, hoisted off a `static` per §B3).
///
/// Source: `oracle/codemp/qcommon/common.cpp:83,86,251-252`
pub struct ErrorState {
    /// `com_errorEntered` (`common.cpp:83`) — the re-entry guard.
    pub entered: bool,
    /// `com_errorMessage[MAXPRINTMSG]` (`common.cpp:86`; `MAXPRINTMSG=4096`).
    pub message: [u8; 4096],
    /// MP-only rapid-error escalation static `lastErrorTime` (`common.cpp:251`).
    pub last_error_time: i32,
    /// MP-only rapid-error escalation static `errorCount` (`common.cpp:252`).
    pub error_count: i32,
}

/// `Com_Error`'s typed panic payload (STATE-Q4). `{level, msg}` is exhaustive
/// (the whole payload the catch-side recovery reads); needs no derive — a
/// `panic_any`/`downcast` payload only requires `Any + Send + 'static`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:249`
pub struct ComError {
    pub level: ErrorLevel,
    pub msg: String,
}

/// Raven `Com_Error` (MP `common.cpp:249` / SP `:245`). RECEIVERLESS and pure
/// (STATE-Q4, LIFE-D3 amended): formats the message into the payload and
/// `panic_any(ComError { level, msg })` — NO recovery, NO `Engine`. The
/// per-level recovery Raven ran before its throw is relocated CATCH-SIDE into
/// `com_frame`/`com_init` (in `mp_engine_core`). Lives in `mp_engine_qcommon`
/// so leaf throw sites (e.g. `mp_engine_server`) can raise it. `-> !`.
///
/// Source: `oracle/codemp/qcommon/common.cpp:249`
pub fn com_error(level: errorParm_t, msg: String) -> ! {
    // Pure format + diverge (STATE-D7): the varargs formatting is caller-side
    // (msg arrives formatted, mirroring vsprintf into com_errorMessage,
    // common.cpp:293-295); ALL recovery runs catch-side in mp_engine_core.
    std::panic::panic_any(ComError { level, msg })
}
