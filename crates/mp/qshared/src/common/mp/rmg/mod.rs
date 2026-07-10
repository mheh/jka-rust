//! MP RMG (random mission generator) common types.
//!
//! Mirrors `oracle/codemp/RMG/` ownership (RMG-D2(b)). Currently the single
//! relocated ABI type `rmAutomapSymbol_t` (RMG-D4d) shared by `mp_engine_rmg`
//! and `mp_engine_client`.

pub mod rm_automap_symbol_t;

pub use rm_automap_symbol_t::RmAutomapSymbol;
