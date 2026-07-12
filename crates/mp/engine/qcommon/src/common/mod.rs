//! `common` — the `common.cpp`-derived qcommon state + primitives.
//!
//! Mechanical file tree (LIFE-D2): `Common` + `com_printf` (common.rs),
//! `ErrorState`/`ComError`/`com_error` (error.rs), `SysEventQueue`
//! (sys_event_queue.rs), `Journal` (journal.rs).

pub mod common;
pub mod common_consts;
pub mod engine_hooks;
pub mod engine_host_view;
pub mod error;
pub mod journal;
pub mod opaque_slots;
pub mod qrand;
pub mod sys_event_queue;

pub use common::{com_printf, Common};
pub use engine_hooks::EngineHooks;
pub use engine_host_view::EngineHostView;
pub use error::{com_error, ComError, ErrorLevel, ErrorState};
pub use journal::Journal;
pub use qrand::QRand;
pub use sys_event_queue::{SysEventQueue, MASK_QUED_EVENTS, MAX_QUED_EVENTS};
