//! `CgLightState` — `cg_light.c`'s mutable file-scope globals as one `CgWorld`
//! sub-struct.

#![allow(non_snake_case)]

use mp_qshared::shared::MAX_QPATH;

use crate::cg_light::MAX_LIGHT_STYLES;
use crate::lights::clightstyle_t::clightstyle_t;

/// `cg_light.c`'s mutable file-scope globals, grouped by owning `.c` file
/// (§B3: file-scope globals become owned state, they never become Rust
/// globals).
///
/// `cl_lightstyle` is the compiled per-style animation table `CG_RunLightStyles`
/// walks every frame; `lastofs` is the last computed `cg.time / 50` bucket (the
/// early-out that would use it is commented out in Raven, so it's write-only
/// here too - see `// PORT-NOTE:` at the write site).
///
/// Source: `oracle/codemp/cgame/cg_light.c:7-8`
///
/// `clightstyle_t` (`crate::lights::clightstyle_t`) carries no `Clone`/`Debug`
/// derive, so this struct implements `Default` by hand instead of deriving it
/// alongside its sibling `Cg*State`s.
pub struct CgLightState {
    pub cl_lightstyle: [clightstyle_t; MAX_LIGHT_STYLES],
    pub lastofs: i32,
}

impl Default for CgLightState {
    fn default() -> Self {
        Self {
            cl_lightstyle: core::array::from_fn(|_| clightstyle_t {
                length: 0,
                value: [0, 0, 0, 0],
                map: [[0, 0, 0, 0]; MAX_QPATH],
            }),
            lastofs: 0,
        }
    }
}
