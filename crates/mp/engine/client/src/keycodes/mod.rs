//! Client-engine key code tables (`oracle/codemp/client/keycodes.h`).
//!
//! The `ui` module carries its own copy of the same header
//! (`crates/mp/ui/src/keycodes/`). Both stay, per the duplicate-don't-unify rule.

pub mod fake_ascii_t;
pub mod k_char_flag;

pub use fake_ascii_t::fakeAscii_t;
pub use k_char_flag::K_CHAR_FLAG;
