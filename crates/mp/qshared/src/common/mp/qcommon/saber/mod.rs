//! MP saber definitions from `q_shared.h`: color, type, style, trail, blade, saber.

pub mod blade_info;
pub mod saber_colors;
pub mod saber_flags;
pub mod saber_info;
pub mod saber_styles;
pub mod saber_trail;
pub mod saber_type;
pub mod w_saber_consts;

pub use blade_info::{bladeInfo_t, MAX_BLADES};
pub use saber_colors::{
    saber_colors_t, NUM_SABER_COLORS, SABER_BLUE, SABER_GREEN, SABER_ORANGE, SABER_PURPLE,
    SABER_RED, SABER_YELLOW,
};
pub use saber_info::{saberInfo_t, MAX_SABERS};
pub use saber_styles::saber_styles_t;
pub use saber_trail::saberTrail_t;
pub use saber_type::saberType_t;
