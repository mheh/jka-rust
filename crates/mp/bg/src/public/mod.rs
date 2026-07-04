//! MP `bg_public.h` shared game definitions.

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

pub mod anim_event_type;
pub mod anim_number;
pub mod animation;
pub mod animevent;
pub mod bg_entity;
pub mod bg_field;
pub mod bg_loaded_anim;
pub mod bg_loaded_events;
pub mod broken_limb;
pub mod ctf_msg;
pub mod dm_flags;
pub mod duel_team;
pub mod effect_types;
pub mod entity_effects;
pub mod entity_event;
pub mod entity_type;
pub mod fieldtype;
pub mod footstep_type;
pub mod force_hand_anims;
pub mod g2_model_parts;
pub mod gametype;
pub mod gender;
pub mod global_team_sound;
pub mod holdable;
pub mod item_type;
pub mod jump_velocity;
pub mod means_of_death;
pub mod parry_debounce_table;
pub mod pd_sounds;
pub mod pers_enum;
pub mod pmove_t;
pub mod set_anim;
pub mod pmtype;
pub mod powerup;
pub mod saber_move_data;
pub mod saber_move_data_table;
pub mod saber_move_name;
pub mod saber_move_transition_angle_table;
pub mod saber_quadrant;
pub mod saberlock;
pub mod spawn;
pub mod transition_move_table;
pub mod stat_index;
pub mod team;
pub mod teamtask;
pub mod weaponstate;

pub use jump_velocity::JUMP_VELOCITY;
pub use spawn::{MAX_SPAWN_VARS, MAX_SPAWN_VARS_CHARS};
pub use team::{team_t, TEAM_BLUE, TEAM_FREE, TEAM_NUM_TEAMS, TEAM_RED, TEAM_SPECTATOR};
