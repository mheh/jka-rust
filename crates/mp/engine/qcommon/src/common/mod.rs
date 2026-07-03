//! `common` — the `common.cpp`-derived qcommon state + primitives.
//!
//! Mechanical file tree (LIFE-D2): `Common` + `com_printf` (common.rs),
//! `ErrorState`/`ComError`/`com_error` (error.rs), `SysEventQueue`
//! (sys_event_queue.rs), `Journal` (journal.rs).

pub mod common;
pub mod error;
pub mod journal;
pub mod sys_event_queue;

pub use common::{com_printf, Common};
pub use error::{com_error, ComError, ErrorLevel, ErrorState};
pub use journal::Journal;
pub use sys_event_queue::{SysEventQueue, MAX_QUED_EVENTS};
