//! `mp_qshared` — MP `q_shared.h`/`qcommon` substrate (Tier 0), the umbrella that
//! the MP module and engine crates share. Re-exports the `native` primitives.
//!
//! Internal module paths (`shared`, `common::mp`) are preserved from the pre-crate
//! layout so migrated files keep their `crate::shared::*` / `crate::common::mp::*`
//! references; the umbrella presents them to dependents as `mp_qshared::shared::*`
//! and `mp_qshared::common::mp::*`.

#![allow(non_camel_case_types, non_snake_case)]

pub mod common;
pub mod shared;

// `rmAutomapSymbol_t` relocated here from `mp_engine_client` (RMG-D4d/RMG-D2(b),
// ruling 21) and surfaced at the crate root as `mp_qshared::RmAutomapSymbol` so
// `mp_engine_rmg` names it directly with no `rmg → mp_engine_client` edge.
pub use common::mp::rmg::RmAutomapSymbol;
