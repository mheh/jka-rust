//! SP saber definitions from `q_shared.h`: color, type, style, trail, blade, saber.
//!
//! SP diverges from MP: smaller `saberTrail_t`/`bladeInfo_t`, pointer-bearing
//! `saberInfo_t`, and a named (non-`typedef int`) `saber_colors_t`.

pub mod blade_info;
pub mod saber_colors;
pub mod saber_info;
pub mod saber_styles;
pub mod saber_trail;
pub mod saber_type;

pub use blade_info::{bladeInfo_t, MAX_BLADES};
pub use saber_colors::saber_colors_t;
pub use saber_info::{saberInfo_t, MAX_SABERS};
pub use saber_styles::saber_styles_t;
pub use saber_trail::saberTrail_t;
pub use saber_type::saberType_t;
