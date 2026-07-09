#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use mp_qshared::shared::vec3_t;

use super::weapon_t::WP_NUM_WEAPONS;

/// Raven `WP_MuzzlePoint[WP_NUM_WEAPONS]` — per-weapon muzzle offset table
/// (forward, right, up).
///
/// Raven only initializes entries up to `WP_BRYAR_OLD`; the trailing
/// `WP_EMPLACED_GUN`/`WP_TURRET` slots are zero-filled by C's partial
/// array-initializer rule.
/// Source: `oracle/codemp/game/bg_weapons.c:10-29`
pub static WP_MuzzlePoint: [vec3_t; WP_NUM_WEAPONS as usize] = [
	[0.0,	0.0,	0.0],	// WP_NONE,
	[0.0,	8.0,	0.0],	// WP_STUN_BATON,
	[0.0,	8.0,	0.0],	// WP_MELEE,
	[8.0,	16.0,	0.0],	// WP_SABER,
	[12.0,	6.0,	-6.0],	// WP_BRYAR_PISTOL,
	[12.0,	6.0,	-6.0],	// WP_BLASTER,
	[12.0,	6.0,	-6.0],	// WP_DISRUPTOR,
	[12.0,	2.0,	-6.0],	// WP_BOWCASTER,
	[12.0,	4.5,	-6.0],	// WP_REPEATER,
	[12.0,	6.0,	-6.0],	// WP_DEMP2,
	[12.0,	6.0,	-6.0],	// WP_FLECHETTE,
	[12.0,	8.0,	-4.0],	// WP_ROCKET_LAUNCHER,
	[12.0,	0.0,	-4.0],	// WP_THERMAL,
	[12.0,	0.0,	-10.0],	// WP_TRIP_MINE,
	[12.0,	0.0,	-4.0],	// WP_DET_PACK,
	[12.0,	6.0,	-6.0],	// WP_CONCUSSION
	[12.0,	6.0,	-6.0],	// WP_BRYAR_OLD,
	[0.0,	0.0,	0.0],	// WP_EMPLACED_GUN (zero-filled: not in Raven's initializer)
	[0.0,	0.0,	0.0],	// WP_TURRET (zero-filled: not in Raven's initializer)
];
