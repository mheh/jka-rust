//! MP vehicle data/info shared types.
//!
//! //TODO: Port module mp_bg::vehicles — subsystem dir only; porters add flat
//! `<type>.rs` files here as types are ported.
//! Source: `oracle/oracle/codemp/game/bg_vehicles.h`

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

pub mod e_weapon_pose;
pub mod min_landing_slope;
pub mod turret_stats_t;
pub mod veh_flags_t;
pub mod veh_turret_status_t;
pub mod veh_weapon_info_t;
pub mod veh_weapon_stats_t;
pub mod veh_weapon_status_t;
pub mod vehicle_info_t;
pub mod vehicle_s;
pub mod vehicle_type_t;

pub use e_weapon_pose::EWeaponPose;
pub use min_landing_slope::MIN_LANDING_SLOPE;
pub use turret_stats_t::{turretStats_t, MAX_VEHICLE_TURRET_MUZZLES};
pub use veh_flags_t::vehFlags_t;
pub use veh_turret_status_t::vehTurretStatus_t;
pub use veh_weapon_info_t::vehWeaponInfo_t;
pub use veh_weapon_stats_t::vehWeaponStats_t;
pub use veh_weapon_status_t::vehWeaponStatus_t;
pub use vehicle_info_t::vehicleInfo_t;
pub use vehicle_s::Vehicle_t;
pub use vehicle_type_t::vehicleType_t;
