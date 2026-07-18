// PORT-COMPLETE: q_math.c
//! Re-export shim over `native_math` (2026-07-17 centralization ruling).
//!
//! Every `q_math.c` function body now lives in `crates/native/math`:
//! the SP/MP-identical set in `native_math::qmath`, the per-mode divergent
//! pair in `native_math::deviations` (this MP tree re-exports the `MP`
//! variants under their Raven names). The `cplane_t` consumers stay defined
//! in `mp_qshared::shared::q_math` (ABI-tier type). Call sites are unchanged.
#![allow(non_snake_case, unused, clippy::all)]

pub use native_math::deviations::{
    ClearBoundsMP as ClearBounds, PerpendicularVectorMP as PerpendicularVector,
};
pub use native_math::qmath::*;

pub use mp_qshared::shared::q_math::{BoxOnPlaneSide, SetPlaneSignbits};
