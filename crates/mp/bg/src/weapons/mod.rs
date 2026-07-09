//! MP weapon data/info shared types.
//!
//! //TODO: Port module mp_bg::weapons — subsystem dir only; porters add flat
//! `<type>.rs` files here as types are ported.
//! Source: `oracle/codemp/game/bg_weapons.h`

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

pub mod ammo_data;
pub mod ammo_data_t;
pub mod ammo_t;
pub mod weapon_data;
pub mod weapon_data_t;
pub mod weapon_t;
pub mod wp_muzzle_point;

pub use ammo_data::ammoData;
pub use weapon_data::weaponData;
pub use wp_muzzle_point::WP_MuzzlePoint;
