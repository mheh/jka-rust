//! MP `DAMAGE_*` flags (`means-of-damage` modifiers).
//!
//! This is the canonical (game-tier) home. `DAMAGE_NO_ARMOR` — the one flag bg
//! consumes (`bg_slidemove`) — is mirrored as a bit-identical twin in
//! `mp_bg::public::damage_flags` (keep the value in sync).
//!
//! Source: `oracle/codemp/game/g_local.h:1170-1190`

use core::ffi::c_int;

pub const DAMAGE_NORMAL: c_int = 0x00000000; // No flags set.
pub const DAMAGE_RADIUS: c_int = 0x00000001; // damage was indirect
pub const DAMAGE_NO_ARMOR: c_int = 0x00000002; // armour does not protect from this damage
pub const DAMAGE_NO_KNOCKBACK: c_int = 0x00000004; // do not affect velocity, just view angles
pub const DAMAGE_NO_PROTECTION: c_int = 0x00000008; // armor/shields/invuln/godmode have no effect
pub const DAMAGE_NO_TEAM_PROTECTION: c_int = 0x00000010; // armor/shields/invuln/godmode have no effect
pub const DAMAGE_EXTRA_KNOCKBACK: c_int = 0x00000040; // add extra knockback to this damage
pub const DAMAGE_DEATH_KNOCKBACK: c_int = 0x00000080; // only does knockback on death of target
pub const DAMAGE_IGNORE_TEAM: c_int = 0x00000100; // damage is always done, regardless of teams
pub const DAMAGE_NO_DAMAGE: c_int = 0x00000200; // no actual damage but react as if damage was taken
pub const DAMAGE_HALF_ABSORB: c_int = 0x00000400; // half shields, half health
pub const DAMAGE_HALF_ARMOR_REDUCTION: c_int = 0x00000800; // doesn't whittle down armor as efficiently
pub const DAMAGE_HEAVY_WEAP_CLASS: c_int = 0x00001000; // Heavy damage
pub const DAMAGE_NO_HIT_LOC: c_int = 0x00002000; // No hit location
pub const DAMAGE_NO_SELF_PROTECTION: c_int = 0x00004000; // Don't apply half damage to self attacks
pub const DAMAGE_NO_DISMEMBER: c_int = 0x00008000; // Don't do dismemberment
pub const DAMAGE_SABER_KNOCKBACK1: c_int = 0x00010000; // Check attacker's 1st saber for knockbackScale
pub const DAMAGE_SABER_KNOCKBACK2: c_int = 0x00020000; // Check attacker's 2nd saber for knockbackScale
pub const DAMAGE_SABER_KNOCKBACK1_B2: c_int = 0x00040000; // Check attacker's 1st saber for knockbackScale2
pub const DAMAGE_SABER_KNOCKBACK2_B2: c_int = 0x00080000; // Check attacker's 2nd saber for knockbackScale2
