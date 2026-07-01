//! MP world container and level-scope types (`g_local.h`). Ported fresh from oracle.

pub mod alert_event;
pub mod bot_settings;
pub mod combat_point;
pub mod damage_flags;
pub mod interest_point;
pub mod level_locals;
pub mod reference_tag;
pub mod spawn_flags;
pub mod waypoint_data;

pub use alert_event::{alertEvent_t, alertEventLevel_e, alertEventType_e, MAX_ALERT_EVENTS};
pub use bot_settings::{bot_settings_t, MAX_FILEPATH};
pub use combat_point::{combatPoint_t, MAX_COMBAT_POINTS};
pub use interest_point::{interestPoint_t, MAX_INTEREST_POINTS};
pub use level_locals::{level_locals_t, BODY_QUEUE_SIZE};
pub use reference_tag::{reference_tag_t, MAX_REFNAME, RTF_NAVGOAL, RTF_NONE};
pub use waypoint_data::waypointData_t;
