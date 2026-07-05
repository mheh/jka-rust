//! MP `q_shared.h` saber flags (`saberInfo_t::saberFlags` bits).
//!
//! Source: `oracle/oracle/codemp/game/q_shared.h:687-712`

use core::ffi::c_int;

/// Raven `SFL_NOT_LOCKABLE` — can't get into a saberlock.
/// Source: `oracle/oracle/codemp/game/q_shared.h:687`
pub const SFL_NOT_LOCKABLE: c_int = 1 << 0;

/// Raven `SFL_NOT_THROWABLE` — can't be thrown.
/// Source: `oracle/oracle/codemp/game/q_shared.h:688`
pub const SFL_NOT_THROWABLE: c_int = 1 << 1;

/// Raven `SFL_NOT_DISARMABLE` — can't be dropped.
/// Source: `oracle/oracle/codemp/game/q_shared.h:689`
pub const SFL_NOT_DISARMABLE: c_int = 1 << 2;

/// Raven `SFL_NOT_ACTIVE_BLOCKING` — don't try to block incoming shots with this saber.
/// Source: `oracle/oracle/codemp/game/q_shared.h:690`
pub const SFL_NOT_ACTIVE_BLOCKING: c_int = 1 << 3;

/// Raven `SFL_TWO_HANDED` — uses both hands.
/// Source: `oracle/oracle/codemp/game/q_shared.h:691`
pub const SFL_TWO_HANDED: c_int = 1 << 4;

/// Raven `SFL_SINGLE_BLADE_THROWABLE` — can throw this saber if only the first blade is on.
/// Source: `oracle/oracle/codemp/game/q_shared.h:692`
pub const SFL_SINGLE_BLADE_THROWABLE: c_int = 1 << 5;

/// Raven `SFL_RETURN_DAMAGE` — when returning from a saber throw, it keeps spinning and doing damage.
/// Source: `oracle/oracle/codemp/game/q_shared.h:693`
pub const SFL_RETURN_DAMAGE: c_int = 1 << 6;

/// Raven `SFL_ON_IN_WATER` — if set, weapon stays active even in water.
/// Source: `oracle/oracle/codemp/game/q_shared.h:695`
pub const SFL_ON_IN_WATER: c_int = 1 << 7;

/// Raven `SFL_BOUNCE_ON_WALLS` — if set, the saber will bounce back when it hits solid architecture.
/// Source: `oracle/oracle/codemp/game/q_shared.h:696`
pub const SFL_BOUNCE_ON_WALLS: c_int = 1 << 8;

/// Raven `SFL_BOLT_TO_WRIST` — if set, saber model is bolted to wrist, not in hand.
/// Source: `oracle/oracle/codemp/game/q_shared.h:697`
pub const SFL_BOLT_TO_WRIST: c_int = 1 << 9;

/// Raven `SFL_NO_PULL_ATTACK` — if set, cannot do pull+attack move.
/// Source: `oracle/oracle/codemp/game/q_shared.h:701`
pub const SFL_NO_PULL_ATTACK: c_int = 1 << 10;

/// Raven `SFL_NO_BACK_ATTACK` — if set, cannot do back-stab moves.
/// Source: `oracle/oracle/codemp/game/q_shared.h:702`
pub const SFL_NO_BACK_ATTACK: c_int = 1 << 11;

/// Raven `SFL_NO_STABDOWN` — if set, cannot do stabdown move when enemy is on ground.
/// Source: `oracle/oracle/codemp/game/q_shared.h:703`
pub const SFL_NO_STABDOWN: c_int = 1 << 12;

/// Raven `SFL_NO_WALL_RUNS` — if set, cannot side-run or forward-run on walls.
/// Source: `oracle/oracle/codemp/game/q_shared.h:704`
pub const SFL_NO_WALL_RUNS: c_int = 1 << 13;

/// Raven `SFL_NO_WALL_FLIPS` — if set, cannot do backflip off wall or side-flips off walls.
/// Source: `oracle/oracle/codemp/game/q_shared.h:705`
pub const SFL_NO_WALL_FLIPS: c_int = 1 << 14;

/// Raven `SFL_NO_WALL_GRAB` — if set, cannot grab wall & jump off.
/// Source: `oracle/oracle/codemp/game/q_shared.h:706`
pub const SFL_NO_WALL_GRAB: c_int = 1 << 15;

/// Raven `SFL_NO_ROLLS` — if set, cannot roll.
/// Source: `oracle/oracle/codemp/game/q_shared.h:707`
pub const SFL_NO_ROLLS: c_int = 1 << 16;

/// Raven `SFL_NO_FLIPS` — if set, cannot do flips.
/// Source: `oracle/oracle/codemp/game/q_shared.h:708`
pub const SFL_NO_FLIPS: c_int = 1 << 17;

/// Raven `SFL_NO_CARTWHEELS` — if set, cannot do cartwheels.
/// Source: `oracle/oracle/codemp/game/q_shared.h:709`
pub const SFL_NO_CARTWHEELS: c_int = 1 << 18;

/// Raven `SFL_NO_KICKS` — if set, cannot do kicks.
/// Source: `oracle/oracle/codemp/game/q_shared.h:710`
pub const SFL_NO_KICKS: c_int = 1 << 19;

/// Raven `SFL_NO_MIRROR_ATTACKS` — if set, cannot do simultaneous attack left/right moves.
/// Source: `oracle/oracle/codemp/game/q_shared.h:711`
pub const SFL_NO_MIRROR_ATTACKS: c_int = 1 << 20;

/// Raven `SFL_NO_ROLL_STAB` — if set, cannot do roll-stab move at end of roll.
/// Source: `oracle/oracle/codemp/game/q_shared.h:712`
pub const SFL_NO_ROLL_STAB: c_int = 1 << 21;
