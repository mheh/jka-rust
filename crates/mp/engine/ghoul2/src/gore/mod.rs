//! MP Ghoul2 gore-system types.

pub mod crag_doll_params;
pub mod gore_set;
pub mod gore_texture_coordinates;
pub mod sgore_surface;
pub mod srag_doll_effector_collision;

/// `SSkinGoreData` crosses the module seam (`trap_G2API_AddSkinGore`), so its
/// canonical home is `mp_qshared`, matching Raven's `q_shared.h` placement.
pub use mp_qshared::common::mp::ghoul2::sskin_gore_data;
