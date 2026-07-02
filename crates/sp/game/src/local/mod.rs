//! SP game-local definitions (`g_local.h`).

pub mod alert_event_level_e;
pub mod alert_event_s;
pub mod alert_event_type_e;
pub mod anim_file_set_t;
pub mod combat_point_t;
pub mod interest_point_t;
pub mod level_locals_t;
pub mod reference_tag_s;
pub mod spawn;
pub mod waypoint_data_t;

pub use spawn::{MAX_SPAWN_VARS, MAX_SPAWN_VARS_CHARS};
