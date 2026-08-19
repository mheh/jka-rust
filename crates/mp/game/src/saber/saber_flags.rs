//! MP `q_shared.h` saber flags (`saberInfo_t::saberFlags` bits).
//!
//! Relocated to the qshared tier (`q_shared.h` is a shared header), so the bg crate can reach the `SFL_*` bits.
//! Re-exported here, so game importers and the prelude keep resolving `crate::saber::saber_flags::*` unchanged.
//! Canonical home: `mp_qshared::common::mp::qcommon::saber::saber_flags`.
//!
//! Source: `oracle/codemp/game/q_shared.h:687-712`

pub use mp_qshared::common::mp::qcommon::saber::saber_flags::*;
