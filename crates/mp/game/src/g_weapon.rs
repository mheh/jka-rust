// PORT-COMPLETE: g_weapon.c
//! FAITHFUL port of `oracle/codemp/game/g_weapon.c`.
//!
//! Filled by the jampgame mega-pass; functions reach file-scope game state
//! (`level`, `g_entities`, cvars) and engine traps through the threaded
//! `GameContext`/`GameWorld` handle.
#![allow(non_snake_case, unused, clippy::all)]

use crate::bg_channel::{GameBgTraps, GameCallbacksImpl};
use crate::g_combat::{G_GetHitLocation, G_HeavyMelee};
use crate::g_missile::{CreateMissile, G_ExplodeMissile, G_MissileImpact};
use crate::g_object::G_RunObject;
use crate::g_utils::{G_BoxInBounds, G_RadiusList};
use crate::prelude::*;
use crate::w_force::Jedi_DodgeEvasion;
use mp_bg::public::entity_type::entityType_t;
use mp_bg::public::stat_index::statIndex_t;
use mp_qshared::common::mp::qcommon::b_set_t::bSet_t;
use mp_qshared::probe;

// Pass-2: entity fn-pointer dispatch as fn-ID enums and the
// `DAMAGE_*` dflag family (`g_local.h:1170-1190`).
use crate::ent_fn_enums::EntThink;
use crate::entity::hit_location::*;
use crate::level::damage_flags::{
    DAMAGE_DEATH_KNOCKBACK, DAMAGE_HEAVY_WEAP_CLASS, DAMAGE_NORMAL, DAMAGE_NO_KNOCKBACK,
};
use mp_bg::bg_misc::snap_vector;

// Raven `qboolean` is `c_int`; keep the source spelling at assignment sites.

// `MASK_SHOT` resolves via the prelude's `surface_flags` re-export (canonical
// `mp_qshared::shared::surface_flags::MASK_SHOT`,
// `oracle/codemp/game/bg_public.h:1177`); no local mirror.

// `DEFAULT_MINS_2` canonical in `mp_bg::public::viewheight` (`c_int`, cast
// here to match the `vec3_t` component it seeds).
// Source: `oracle/codemp/game/bg_public.h:41`
const DEFAULT_MINS_2: f32 = mp_bg::public::viewheight::DEFAULT_MINS_2 as f32;

// Per-weapon damage/velocity/size `#define`s, file-local to `g_weapon.c` in
// Raven (never promoted to a header), ported the same way here.
// Source: `oracle/codemp/game/g_weapon.c:18-155`
const BRYAR_PISTOL_VEL: c_int = 1600;
const BRYAR_PISTOL_DAMAGE: c_int = 10;
const BRYAR_CHARGE_UNIT: f32 = 200.0;
const BRYAR_ALT_SIZE: f32 = 1.0;

const BLASTER_SPREAD: f32 = 1.6;
const BLASTER_VELOCITY: c_int = 2300;
const BLASTER_DAMAGE: c_int = 20;

const DISRUPTOR_MAIN_DAMAGE: c_int = 30;
const DISRUPTOR_MAIN_DAMAGE_SIEGE: c_int = 50;
const DISRUPTOR_NPC_MAIN_DAMAGE_CUT: f32 = 0.25;
const DISRUPTOR_ALT_DAMAGE: c_int = 100;
const DISRUPTOR_NPC_ALT_DAMAGE_CUT: f32 = 0.2;
const DISRUPTOR_ALT_TRACES: c_int = 3;
const DISRUPTOR_CHARGE_UNIT: f32 = 50.0;

const BOWCASTER_DAMAGE: c_int = 50;
const BOWCASTER_VELOCITY: c_int = 1300;
const BOWCASTER_SPLASH_DAMAGE: c_int = 0;
const BOWCASTER_SPLASH_RADIUS: c_int = 0;
const BOWCASTER_SIZE: c_int = 2;
const BOWCASTER_ALT_SPREAD: f32 = 5.0;
const BOWCASTER_VEL_RANGE: f32 = 0.3;
const BOWCASTER_CHARGE_UNIT: f32 = 200.0;

const REPEATER_SPREAD: f32 = 1.4;
const REPEATER_DAMAGE: c_int = 14;
const REPEATER_VELOCITY: c_int = 1600;
const REPEATER_ALT_SIZE: c_int = 3;
const REPEATER_ALT_DAMAGE: c_int = 60;
const REPEATER_ALT_SPLASH_DAMAGE: c_int = 60;
const REPEATER_ALT_SPLASH_RADIUS: c_int = 128;
const REPEATER_ALT_SPLASH_RAD_SIEGE: c_int = 80;
const REPEATER_ALT_VELOCITY: c_int = 1100;

const DEMP2_DAMAGE: c_int = 35;
const DEMP2_VELOCITY: c_int = 1800;
const DEMP2_SIZE: c_int = 2;
const DEMP2_ALT_DAMAGE: c_int = 8;
const DEMP2_CHARGE_UNIT: f32 = 700.0;
const DEMP2_ALT_RANGE: c_int = 4096;
const DEMP2_ALT_SPLASHRADIUS: c_int = 256;

const FLECHETTE_SHOTS: c_int = 5;
const FLECHETTE_SPREAD: f32 = 4.0;
const FLECHETTE_DAMAGE: c_int = 12;
const FLECHETTE_VEL: c_int = 3500;
const FLECHETTE_SIZE: c_int = 1;
const FLECHETTE_MINE_RADIUS_CHECK: c_int = 256;
const FLECHETTE_ALT_DAMAGE: c_int = 60;
const FLECHETTE_ALT_SPLASH_DAM: c_int = 60;
const FLECHETTE_ALT_SPLASH_RAD: c_int = 128;

const ROCKET_VELOCITY: c_int = 900;
const ROCKET_DAMAGE: c_int = 100;
const ROCKET_SPLASH_DAMAGE: c_int = 100;
const ROCKET_SPLASH_RADIUS: c_int = 160;
const ROCKET_SIZE: c_int = 3;
const ROCKET_ALT_THINK_TIME: c_int = 100;

const CONC_VELOCITY: c_int = 3000;
const CONC_DAMAGE: c_int = 75;
const CONC_NPC_DAMAGE_EASY: c_int = 40;
const CONC_NPC_DAMAGE_NORMAL: c_int = 80;
const CONC_NPC_DAMAGE_HARD: c_int = 100;
const CONC_SPLASH_DAMAGE: c_int = 40;
const CONC_SPLASH_RADIUS: c_int = 200;
const CONC_ALT_DAMAGE: c_int = 25;
const CONC_ALT_NPC_DAMAGE_EASY: c_int = 20;
const CONC_ALT_NPC_DAMAGE_MEDIUM: c_int = 35;
const CONC_ALT_NPC_DAMAGE_HARD: c_int = 50;

const STUN_BATON_DAMAGE: c_int = 20;
const STUN_BATON_ALT_DAMAGE: c_int = 20;
const STUN_BATON_RANGE: c_int = 8;

const MELEE_SWING1_DAMAGE: c_int = 10;
const MELEE_SWING2_DAMAGE: c_int = 12;
const MELEE_RANGE: c_int = 8;

const ATST_MAIN_VEL: c_int = 4000;
const ATST_MAIN_DAMAGE: c_int = 25;
const ATST_MAIN_SIZE: c_int = 3;
const ATST_SIDE_MAIN_DAMAGE: c_int = 75;
const ATST_SIDE_MAIN_VELOCITY: c_int = 1300;
const ATST_SIDE_MAIN_NPC_DAMAGE_EASY: c_int = 30;
const ATST_SIDE_MAIN_NPC_DAMAGE_NORMAL: c_int = 40;
const ATST_SIDE_MAIN_NPC_DAMAGE_HARD: c_int = 50;
const ATST_SIDE_MAIN_SIZE: c_int = 4;
const ATST_SIDE_MAIN_SPLASH_DAMAGE: c_int = 10;
const ATST_SIDE_MAIN_SPLASH_RADIUS: c_int = 16;
const ATST_SIDE_ALT_VELOCITY: c_int = 1100;
const ATST_SIDE_ALT_NPC_VELOCITY: c_int = 600;
const ATST_SIDE_ALT_DAMAGE: c_int = 130;
const ATST_SIDE_ROCKET_NPC_DAMAGE_EASY: c_int = 30;
const ATST_SIDE_ROCKET_NPC_DAMAGE_NORMAL: c_int = 50;
const ATST_SIDE_ROCKET_NPC_DAMAGE_HARD: c_int = 90;
const ATST_SIDE_ALT_SPLASH_DAMAGE: c_int = 130;
const ATST_SIDE_ALT_SPLASH_RADIUS: c_int = 200;
const ATST_SIDE_ALT_ROCKET_SIZE: c_int = 5;
const ATST_SIDE_ALT_ROCKET_SPLASH_SCALE: f32 = 0.5;

// Thermal-detonator consts (`touchThermalDetonator`/`ThermalDetonatorExplode`).
// Source: `oracle/codemp/game/g_weapon.c:1918-1931`
const TD_DAMAGE: c_int = 70;
const TD_SPLASH_RAD: c_int = 128;
const TD_SPLASH_DAM: c_int = 90;
const TD_VELOCITY: c_int = 900;
const TD_MIN_CHARGE: f32 = 0.15;
const TD_TIME: c_int = 3000;
const TD_ALT_TIME: c_int = 3000;
const TD_ALT_DAMAGE: c_int = 60;
const TD_ALT_SPLASH_RAD: c_int = 128;
const TD_ALT_SPLASH_DAM: c_int = 50;
const TD_ALT_VELOCITY: c_int = 600;
const TD_ALT_MIN_CHARGE: f32 = 0.15;

// Vehicle homing-missile / emplaced-gun / crosshair consts.
// Source: `oracle/codemp/game/g_weapon.c:3627,4049,4662,4825`
const VEH_HOMING_MISSILE_THINK_TIME: c_int = 100;
pub(crate) const MAX_XHAIR_DIST_ACCURACY: f32 = 20000.0;
const EMPLACED_CANRESPAWN: c_int = 1;
const EMPLACED_GUN_HEALTH: c_int = 800;

// Raven `MAX_STRAFE_TIME` (`bg_vehicles.h:398`, "FIXME: extern?" in the
// original — still a plain `#define` there, never externed).
const MAX_STRAFE_TIME: f32 = 2000.0;

// `CONTENTS_LIGHTSABER`, `CONTENTS_SHOTCLIP`, `MASK_SOLID`, and `SVF_BROADCAST`
// resolve via the prelude re-exports (canonical `mp_qshared::shared::surface_flags`
// / `crate::g_public_consts`); no local mirrors. The former local
// `MASK_SOLID = CONTENTS_SOLID` dropped `CONTENTS_TERRAIN` relative to Raven's
// `#define MASK_SOLID (CONTENTS_SOLID|CONTENTS_TERRAIN)` (`bg_public.h:1171`) — a
// trace-mask parity bug now fixed by deferring to the canonical value.

// Raven `team_t::TEAM_SPECTATOR` (`bg_public.h`); local mirror, same
// in-repo convention as `g_team.rs`.
const TEAM_SPECTATOR: c_int = 3;

/// Raven `touch_NULL`.
///
/// Source: `oracle/codemp/game/g_weapon.c:165-168`
pub fn touch_NULL(ent: EntityId, other: Option<EntityId>, trace: *mut trace_t) {
    // Raven: empty body — deliberate no-op touch callback.
}

/// Raven `WP_SpeedOfMissileForWeapon`.
///
/// Source: `oracle/codemp/game/g_weapon.c:176-179`
pub fn WP_SpeedOfMissileForWeapon(wp: c_int, alt_fire: bool) -> f32 {
    // Raven comment: "We should really organize weapon data into tables or
    // parse from the ext data so we have accurate info for this."
    500.0
}

/// Raven `W_TraceSetStart`.
///
/// Source: `oracle/codemp/game/g_weapon.c:182-218`
// Oracle writes the wall-corrected point back through the `start` out-param
// (`VectorCopy(tr.endpos, start)`); return it so callers pick up the adjustment.
pub fn W_TraceSetStart(
    ctx: &mut GameContext,
    ent: EntityId,
    start: vec3_t,
    mins: vec3_t,
    maxs: vec3_t,
) -> vec3_t {
    let mut start = start;
    let mut entMins: vec3_t = [0.0; 3];
    let mut entMaxs: vec3_t = [0.0; 3];
    {
        let e = ctx.world.entity(ent);
        for i in 0..3 {
            entMins[i] = e.r.currentOrigin[i] + e.r.mins[i];
            entMaxs[i] = e.r.currentOrigin[i] + e.r.maxs[i];
        }
    }

    if G_BoxInBounds(start, mins, maxs, entMins, entMaxs) != qfalse {
        return start;
    }

    // FLAG: firing ent may be an NPC (pool client); read the client pointer value
    // and deref it raw as Raven does.
    let ent_client = ctx.world.entity(ent).client;
    if ent_client.is_null() {
        return start;
    }

    let mut eyePoint = ctx.world.entity(ent).s.pos.trBase;
    eyePoint[2] += unsafe { (*ent_client).ps.viewheight } as f32;

    let mut tr: trace_t = unsafe { std::mem::zeroed() };
    let ent_num = ctx.world.entity(ent).s.number;
    trap::Trace(
        ctx.engine,
        mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
            &mut tr,
            &eyePoint as *const vec3_t,
            &mins as *const vec3_t,
            &maxs as *const vec3_t,
            &start as *const vec3_t,
            ent_num,
            MASK_SOLID | CONTENTS_SHOTCLIP,
        ),
    );

    if tr.startsolid != 0 || tr.allsolid != 0 {
        return start;
    }

    if tr.fraction < 1.0 {
        start = tr.endpos;
    }
    start
}

/// Raven `WP_FireBryarPistol`.
///
/// Source: `oracle/codemp/game/g_weapon.c:236-293`
pub fn WP_FireBryarPistol(ctx: &mut GameContext, ent: EntityId, altFire: bool) {
    let mut damage: c_int = BRYAR_PISTOL_DAMAGE;
    let mut count: c_int;

    let muzzle = ctx.world.globals.muzzle;
    let forward = ctx.world.globals.forward;
    let mid = CreateMissile(
        ctx,
        muzzle,
        forward,
        BRYAR_PISTOL_VEL as f32,
        10000,
        ent,
        altFire,
    );

    {
        let m = ctx.world.entity_mut(mid);
        m.classname = c"bryar_proj".as_ptr() as *mut c_char;
        m.s.weapon = WP_BRYAR_PISTOL;
    }

    if altFire {
        let boxSize: f32;

        // FLAG: firing ent may be an NPC (pool client); deref the client value raw.
        let ent_client = ctx.world.entity(ent).client;
        let now = ctx.world.level.time;
        count = ((now - unsafe { (*ent_client).ps.weaponChargeTime }) as f32 / BRYAR_CHARGE_UNIT)
            as c_int;

        if count < 1 {
            count = 1;
        } else if count > 5 {
            count = 5;
        }

        if count > 1 {
            damage = (damage as f32 * (count as f32 * 1.7)) as c_int;
        } else {
            damage = (damage as f32 * (count as f32 * 1.5)) as c_int;
        }

        boxSize = BRYAR_ALT_SIZE * (count as f32 * 0.5);

        let m = ctx.world.entity_mut(mid);
        m.s.generic1 = count; // The missile will then render according to the charge level.
        m.r.maxs = [boxSize, boxSize, boxSize];
        m.r.mins = [-boxSize, -boxSize, -boxSize];
    }

    let m = ctx.world.entity_mut(mid);
    m.damage = damage;
    m.dflags = DAMAGE_DEATH_KNOCKBACK;
    if altFire {
        m.methodOfDeath = MOD_BRYAR_PISTOL_ALT as c_int;
    } else {
        m.methodOfDeath = MOD_BRYAR_PISTOL as c_int;
    }
    m.clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;

    // we don't want it to bounce forever
    m.bounceCount = 8;
}

/// Raven `WP_FireTurretMissile`.
///
/// Source: `oracle/codemp/game/g_weapon.c:304-326`
pub fn WP_FireTurretMissile(
    ctx: &mut GameContext,
    ent: EntityId,
    start: vec3_t,
    dir: vec3_t,
    altFire: bool,
    damage: c_int,
    velocity: c_int,
    r#mod: c_int,
    ignore: Option<EntityId>,
) {
    let mid = CreateMissile(ctx, start, dir, velocity as f32, 10000, ent, altFire);
    let m = ctx.world.entity_mut(mid);

    m.classname = c"generic_proj".as_ptr() as *mut c_char;
    m.s.weapon = WP_TURRET;

    m.damage = damage;
    m.dflags = DAMAGE_DEATH_KNOCKBACK;
    m.methodOfDeath = r#mod;
    m.clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;

    if let Some(ignore) = ignore {
        let num = ctx.world.entity(ignore).s.number;
        ctx.world.entity_mut(mid).passThroughNum = num + 1;
    }

    // we don't want it to bounce forever
    ctx.world.entity_mut(mid).bounceCount = 8;
}

/// Raven `WP_FireGenericBlasterMissile`.
///
/// Only the seeker drone uses this, but it might be useful for other things
/// as well.
///
/// Source: `oracle/codemp/game/g_weapon.c:331-348`
pub fn WP_FireGenericBlasterMissile(
    ctx: &mut GameContext,
    ent: EntityId,
    start: vec3_t,
    dir: vec3_t,
    altFire: bool,
    damage: c_int,
    velocity: c_int,
    r#mod: c_int,
) {
    let mid = CreateMissile(ctx, start, dir, velocity as f32, 10000, ent, altFire);
    let m = ctx.world.entity_mut(mid);

    m.classname = c"generic_proj".as_ptr() as *mut c_char;
    m.s.weapon = WP_BRYAR_PISTOL;

    m.damage = damage;
    m.dflags = DAMAGE_DEATH_KNOCKBACK;
    m.methodOfDeath = r#mod;
    m.clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;

    // we don't want it to bounce forever
    m.bounceCount = 8;
}

/// Raven `WP_FireBlasterMissile`.
///
/// Source: `oracle/codemp/game/g_weapon.c:359-383`
pub fn WP_FireBlasterMissile(
    ctx: &mut GameContext,
    ent: EntityId,
    start: vec3_t,
    dir: vec3_t,
    altFire: bool,
) {
    let velocity: c_int = BLASTER_VELOCITY;
    let mut damage: c_int = BLASTER_DAMAGE;

    if ctx.world.entity(ent).s.eType == entityType_t::ET_NPC as c_int {
        // animent
        damage = 10;
    }

    let mid = CreateMissile(ctx, start, dir, velocity as f32, 10000, ent, altFire);
    let m = ctx.world.entity_mut(mid);

    m.classname = c"blaster_proj".as_ptr() as *mut c_char;
    m.s.weapon = WP_BLASTER;

    m.damage = damage;
    m.dflags = DAMAGE_DEATH_KNOCKBACK;
    m.methodOfDeath = MOD_BLASTER as c_int;
    m.clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;

    // we don't want it to bounce forever
    m.bounceCount = 8;
}

/// Raven `WP_FireTurboLaserMissile`.
///
/// Source: `oracle/codemp/game/g_weapon.c:386-419`
pub fn WP_FireTurboLaserMissile(ctx: &mut GameContext, ent: EntityId, start: vec3_t, dir: vec3_t) {
    // FIXME (Raven): velocity/damage/splash externalized off the shooter ent.
    let velocity: c_int = ctx.world.entity(ent).mass as c_int;
    let mid = CreateMissile(ctx, start, dir, velocity as f32, 10000, ent, false);

    // Shooter-ent reads hoisted into locals (CreateMissile does not mutate `ent`).
    let e = ctx.world.entity(ent);
    let gv14 = e.genericValue14;
    let gv15 = e.genericValue15;
    let e_damage = e.damage;
    let e_splashDamage = e.splashDamage;
    let e_splashRadius = e.splashRadius;
    let e_number = e.s.number;
    let now = ctx.world.level.time;

    let m = ctx.world.entity_mut(mid);

    // use a custom shot effect / custom impact effect
    m.s.otherEntityNum2 = gv14;
    m.s.emplacedOwner = gv15;

    m.classname = c"turbo_proj".as_ptr() as *mut c_char;
    m.s.weapon = WP_TURRET;

    m.damage = e_damage;
    m.splashDamage = e_splashDamage;
    m.splashRadius = e_splashRadius;
    m.dflags = DAMAGE_DEATH_KNOCKBACK;
    m.methodOfDeath = MOD_TARGET_LASER as c_int; // MOD_TURBLAST; count as a heavy weap
    m.splashMethodOfDeath = MOD_TARGET_LASER as c_int;
    m.clipmask = MASK_SHOT;

    // we don't want it to bounce forever
    m.bounceCount = 8;

    // set veh as cgame side owner for purpose of fx overrides
    m.s.owner = e_number;

    // don't let them last forever (at 20000 speed, more than enough)
    m.think = Some(EntThink::G_FreeEntity).into();
    m.nextthink = now + 5000;
}

/// Raven `WP_FireEmplacedMissile`.
///
/// Source: `oracle/codemp/game/g_weapon.c:422-448`
pub fn WP_FireEmplacedMissile(
    ctx: &mut GameContext,
    ent: EntityId,
    start: vec3_t,
    dir: vec3_t,
    altFire: bool,
    ignore: Option<EntityId>,
) {
    let velocity: c_int = BLASTER_VELOCITY;
    let damage: c_int = BLASTER_DAMAGE;

    let mid = CreateMissile(ctx, start, dir, velocity as f32, 10000, ent, altFire);
    let m = ctx.world.entity_mut(mid);

    m.classname = c"emplaced_gun_proj".as_ptr() as *mut c_char;
    m.s.weapon = WP_TURRET; //WP_EMPLACED_GUN;

    m.activator = ignore;

    m.damage = damage;
    m.dflags = DAMAGE_DEATH_KNOCKBACK | DAMAGE_HEAVY_WEAP_CLASS;
    m.methodOfDeath = MOD_VEHICLE as c_int;
    m.clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;

    if let Some(ignore) = ignore {
        let num = ctx.world.entity(ignore).s.number;
        ctx.world.entity_mut(mid).passThroughNum = num + 1;
    }

    // we don't want it to bounce forever
    ctx.world.entity_mut(mid).bounceCount = 8;
}

/// Raven `WP_FireBlaster`.
///
/// Source: `oracle/codemp/game/g_weapon.c:451-469`
pub fn WP_FireBlaster(ctx: &mut GameContext, ent: EntityId, altFire: bool) {
    let mut dir: vec3_t = [0.0; 3];
    let mut angs: vec3_t = [0.0; 3];

    vectoangles(ctx.world.globals.forward, &mut angs);

    if altFire {
        // add some slop to the alt-fire direction
        // C: `crandom()` is `double`, so each `+=` runs in `double` and
        // narrows back to the `float` angle component.
        angs[PITCH] =
            (angs[PITCH] as f64 + ctx.world.bg_state.rng.crandom() * BLASTER_SPREAD as f64) as f32;
        angs[YAW] =
            (angs[YAW] as f64 + ctx.world.bg_state.rng.crandom() * BLASTER_SPREAD as f64) as f32;
    }

    AngleVectors(angs, Some(&mut dir), None, None);

    let muzzle = ctx.world.globals.muzzle;
    // FIXME: if temp_org does not have clear trace to inside the bbox, don't shoot!
    WP_FireBlasterMissile(ctx, ent, muzzle, dir, altFire);
}

/// Raven `WP_DisruptorMainFire`.
///
/// Source: `oracle/codemp/game/g_weapon.c:483-621`
pub fn WP_DisruptorMainFire(ctx: &mut GameContext, ent: EntityId) {
    unsafe {
        let mut damage: c_int = DISRUPTOR_MAIN_DAMAGE;
        let mut render_impact = true;
        let mut start: vec3_t;
        let mut end: vec3_t = [0.0; 3];
        let mut tr: trace_t = std::mem::zeroed();
        let shotRange: f32 = 8192.0;
        let mut ignore: c_int;
        let mut traces: c_int = 0;

        // FLAG: firing ent may be an NPC (pool client); deref its client raw.
        let ent_client = ctx.world.entity(ent).client;
        let ent_num = ctx.world.entity(ent).s.number;

        if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
            damage = DISRUPTOR_MAIN_DAMAGE_SIEGE;
        }

        start = (*ent_client).ps.origin;
        start[2] += (*ent_client).ps.viewheight as f32; // By eyes

        for i in 0..3 {
            end[i] = start[i] + shotRange * ctx.world.globals.forward[i];
        }

        ignore = ent_num;
        traces = 0;
        loop {
            if traces >= 10 {
                break;
            }
            // need to loop this in case we hit a Jedi who dodges the shot
            if ctx.world.cvars.d_projectileGhoul2Collision.integer != 0 {
                trap::G2Trace(
                    ctx.engine,
                    mp_abi::game::syscalls::G_G2TRACE::GG2TraceArgs::new(
                        &mut tr,
                        &start as *const vec3_t,
                        // Oracle passes NULL mins/maxs here (point trace).
                        core::ptr::null(),
                        core::ptr::null(),
                        &end as *const vec3_t,
                        ignore,
                        MASK_SHOT,
                        G2TRFLAG_DOGHOULTRACE
                            | G2TRFLAG_GETSURFINDEX
                            | G2TRFLAG_THICK
                            | G2TRFLAG_HITCORPSES,
                        ctx.world.cvars.g_g2TraceLod.integer,
                    ),
                );
            } else {
                trap::Trace(
                    ctx.engine,
                    mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                        &mut tr,
                        &start as *const vec3_t,
                        std::ptr::null(),
                        std::ptr::null(),
                        &end as *const vec3_t,
                        ignore,
                        MASK_SHOT,
                    ),
                );
            }

            let traceEnt_id = EntityId(tr.entityNum as u32);
            // FLAG: trace target may be an NPC (pool client); deref its client raw.
            let traceEnt_client = ctx.world.entity(traceEnt_id).client;

            let hit_loc = G_GetHitLocation(ctx, traceEnt_id, tr.endpos);
            if ctx.world.cvars.d_projectileGhoul2Collision.integer != 0
                && ctx.world.entity(traceEnt_id).inuse != 0
                && !traceEnt_client.is_null()
            {
                // g2 collision checks -rww
                if ctx.world.entity(traceEnt_id).inuse != 0
                    && !traceEnt_client.is_null()
                    && !ctx.world.entity(traceEnt_id).ghoul2.is_null()
                {
                    (*traceEnt_client).g2LastSurfaceHit = tr.surfaceFlags;
                    (*traceEnt_client).g2LastSurfaceTime = ctx.world.level.time;
                }

                if !ctx.world.entity(traceEnt_id).ghoul2.is_null() {
                    tr.surfaceFlags = 0;
                }
            }

            if !traceEnt_client.is_null()
                && (*traceEnt_client).ps.duelInProgress != 0
                && (*traceEnt_client).ps.duelIndex != ent_num
            {
                start = tr.endpos;
                ignore = (tr.entityNum) as i32;
                traces += 1;
                continue;
            }

            if Jedi_DodgeEvasion(ctx, Some(traceEnt_id), Some(ent), &mut tr, hit_loc) {
                // act like we didn't even hit him
                start = tr.endpos;
                ignore = (tr.entityNum) as i32;
                traces += 1;
                continue;
            } else if !traceEnt_client.is_null()
                && (*traceEnt_client).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize]
                    >= FORCE_LEVEL_3
            {
                if WP_SaberCanBlock(
                    ctx,
                    Some(traceEnt_id),
                    tr.endpos,
                    0,
                    MOD_DISRUPTOR as c_int,
                    true,
                    0,
                ) != 0
                {
                    // broadcast and stop the shot because it was blocked
                    let tent_id = G_TempEntity(ctx, tr.endpos, (EV_DISRUPTOR_MAIN_SHOT) as i32);
                    let muzzle = ctx.world.globals.muzzle;
                    let tent = ctx.entity_mut(tent_id);
                    tent.s.origin2 = muzzle;
                    tent.s.eventParm = ent_num;

                    let te_eid = G_TempEntity(ctx, tr.endpos, (EV_SABER_BLOCK) as i32);
                    let te = ctx.entity_mut(te_eid);
                    te.s.origin = tr.endpos;
                    te.s.angles = tr.plane.normal;
                    if te.s.angles[0] == 0.0 && te.s.angles[1] == 0.0 && te.s.angles[2] == 0.0 {
                        te.s.angles[1] = 1.0;
                    }
                    te.s.eventParm = 0;
                    te.s.weapon = 0; // saberNum
                    te.s.legsAnim = 0; // bladeNum

                    return;
                }
            } else if (ctx.world.entity(traceEnt_id).flags & FL_SHIELDED) != 0 {
                // stopped cold
                return;
            }
            // a Jedi is not dodging this shot
            break;
        }

        if (tr.surfaceFlags & SURF_NOIMPACT) != 0 {
            render_impact = false;
        }

        // always render a shot beam, doing this the old way because I don't much feel like overriding the effect.
        let tent_id = G_TempEntity(ctx, tr.endpos, (EV_DISRUPTOR_MAIN_SHOT) as i32);
        let muzzle = ctx.world.globals.muzzle;
        let tent = ctx.entity_mut(tent_id);
        tent.s.origin2 = muzzle;
        tent.s.eventParm = ent_num;

        let traceEnt_id = EntityId(tr.entityNum as u32);
        // FLAG: trace target may be an NPC (pool client); deref its client raw.
        let traceEnt_client = ctx.world.entity(traceEnt_id).client;

        if render_impact {
            if tr.entityNum < ENTITYNUM_WORLD as i16
                && ctx.world.entity(traceEnt_id).takedamage != 0
            {
                let dmg_dir = Some(&mut (*ctx.world_raw()).globals.forward); // STAGE-2b: irreducible — &mut world.globals.forward aliases the ctx passed to the same call.
                if !traceEnt_client.is_null() && LogAccuracyHit(ctx, traceEnt_id, Some(ent)) {
                    (*ent_client).accuracy_hits += 1;
                }

                G_Damage(
                    ctx,
                    Some(traceEnt_id),
                    Some(ent),
                    Some(ent),
                    dmg_dir,
                    tr.endpos,
                    damage,
                    DAMAGE_NORMAL,
                    MOD_DISRUPTOR as c_int,
                );

                let tent_id = G_TempEntity(ctx, tr.endpos, (EV_DISRUPTOR_HIT) as i32);
                let tent = ctx.entity_mut(tent_id);
                tent.s.eventParm = DirToByte(tr.plane.normal);
                if !traceEnt_client.is_null() {
                    tent.s.weapon = 1;
                }
            } else {
                // Hmmm, maybe don't make any marks on things that could break
                let tent_id = G_TempEntity(ctx, tr.endpos, (EV_DISRUPTOR_SNIPER_MISS) as i32);
                let tent = ctx.entity_mut(tent_id);
                tent.s.eventParm = DirToByte(tr.plane.normal);
                tent.s.weapon = 1;
            }
        }
    }
}

/// Raven `G_CanDisruptify`.
///
/// Source: `oracle/codemp/game/g_weapon.c:624-639`
pub fn G_CanDisruptify(ent: Option<&gentity_t>) -> qboolean {
    let Some(ent) = ent else {
        // not vehicle (Raven's `ent == NULL` guard)
        return qtrue;
    };
    if ent.inuse == 0
        || ent.client.is_null()
        || ent.s.eType != entityType_t::ET_NPC as c_int
        || ent.s.NPC_class != (CLASS_VEHICLE) as i32
        || ent.m_pVehicle.is_null()
    {
        // not vehicle
        return qtrue;
    }

    let veh = ent.m_pVehicle;
    // FLAG: `Vehicle_t`/`vehicleInfo_t` have no accessor; the deref stays raw
    // (bg subsystem type reached through the gentity `m_pVehicle` pointer).
    if unsafe { (*(*veh).m_pVehicleInfo).r#type } == mp_bg::vehicles::vehicleType_t::VH_ANIMAL {
        // animal is only type that can be disintigeiteigerated
        return qtrue;
    }

    // don't do it to any other veh
    qfalse
}

/// Raven `WP_DisruptorAltFire`.
///
/// Source: `oracle/codemp/game/g_weapon.c:642-886`
pub fn WP_DisruptorAltFire(ctx: &mut GameContext, ent: EntityId) {
    unsafe {
        let mut damage: c_int = DISRUPTOR_ALT_DAMAGE - 30;
        let mut render_impact = true;
        let mut start: vec3_t;
        let mut end: vec3_t = [0.0; 3];
        let mut tr: trace_t = std::mem::zeroed();
        let shotRange: f32 = 8192.0;
        let mut count: c_int;
        let mut maxCount: c_int = 60;
        let mut traces: c_int = DISRUPTOR_ALT_TRACES;
        let mut fullCharge = qfalse;

        // FLAG: firing ent may be an NPC (pool client); deref its client raw.
        let ent_client = ctx.world.entity(ent).client;
        let ent_num = ctx.world.entity(ent).s.number;

        if !ent_client.is_null() {
            start = (*ent_client).ps.origin;
            start[2] += (*ent_client).ps.viewheight as f32;

            count = ((ctx.world.level.time - (*ent_client).ps.weaponChargeTime) as f32
                / DISRUPTOR_CHARGE_UNIT) as c_int;
            if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
                maxCount = 200;
            }
        } else {
            start = ctx.world.entity(ent).r.currentOrigin;
            start[2] += 24.0;
            count = (100 as f32 / DISRUPTOR_CHARGE_UNIT) as c_int;
        }

        count *= 2;

        if count < 1 {
            count = 1;
        } else if count >= maxCount {
            count = maxCount;
            fullCharge = qtrue;
        }

        if count < 10 {
            traces = 1;
        } else if count < 20 {
            traces = 2;
        }

        damage += count;

        let mut skip: c_int = ent_num;

        for _i in 0..traces {
            for k in 0..3 {
                end[k] = start[k] + shotRange * ctx.world.globals.forward[k];
            }

            if ctx.world.cvars.d_projectileGhoul2Collision.integer != 0 {
                trap::G2Trace(
                    ctx.engine,
                    mp_abi::game::syscalls::G_G2TRACE::GG2TraceArgs::new(
                        &mut tr,
                        &start as *const vec3_t,
                        // Oracle passes NULL mins/maxs here (point trace).
                        core::ptr::null(),
                        core::ptr::null(),
                        &end as *const vec3_t,
                        skip,
                        MASK_SHOT,
                        G2TRFLAG_DOGHOULTRACE
                            | G2TRFLAG_GETSURFINDEX
                            | G2TRFLAG_THICK
                            | G2TRFLAG_HITCORPSES,
                        ctx.world.cvars.g_g2TraceLod.integer,
                    ),
                );
            } else {
                trap::Trace(
                    ctx.engine,
                    mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                        &mut tr,
                        &start as *const vec3_t,
                        std::ptr::null(),
                        std::ptr::null(),
                        &end as *const vec3_t,
                        skip,
                        MASK_SHOT,
                    ),
                );
            }

            let traceEnt_id = EntityId(tr.entityNum as u32);
            // FLAG: trace target may be an NPC (pool client); deref its client raw.
            let traceEnt_client = ctx.world.entity(traceEnt_id).client;

            let hit_loc = G_GetHitLocation(ctx, traceEnt_id, tr.endpos);
            if ctx.world.cvars.d_projectileGhoul2Collision.integer != 0
                && ctx.world.entity(traceEnt_id).inuse != 0
                && !traceEnt_client.is_null()
            {
                if ctx.world.entity(traceEnt_id).inuse != 0
                    && !traceEnt_client.is_null()
                    && !ctx.world.entity(traceEnt_id).ghoul2.is_null()
                {
                    (*traceEnt_client).g2LastSurfaceHit = tr.surfaceFlags;
                    (*traceEnt_client).g2LastSurfaceTime = ctx.world.level.time;
                }
                if !ctx.world.entity(traceEnt_id).ghoul2.is_null() {
                    tr.surfaceFlags = 0;
                }
            }

            if (tr.surfaceFlags & SURF_NOIMPACT) != 0 {
                render_impact = false;
            }

            if !traceEnt_client.is_null()
                && (*traceEnt_client).ps.duelInProgress != 0
                && (*traceEnt_client).ps.duelIndex != ent_num
            {
                skip = (tr.entityNum) as i32;
                start = tr.endpos;
                continue;
            }

            if Jedi_DodgeEvasion(ctx, Some(traceEnt_id), Some(ent), &mut tr, hit_loc) {
                skip = (tr.entityNum) as i32;
                start = tr.endpos;
                continue;
            } else if !traceEnt_client.is_null()
                && (*traceEnt_client).ps.fd.forcePowerLevel[FP_SABER_DEFENSE as usize]
                    >= FORCE_LEVEL_3
            {
                if WP_SaberCanBlock(
                    ctx,
                    Some(traceEnt_id),
                    tr.endpos,
                    0,
                    MOD_DISRUPTOR_SNIPER as c_int,
                    true,
                    0,
                ) != 0
                {
                    let tent_id = G_TempEntity(ctx, tr.endpos, (EV_DISRUPTOR_SNIPER_SHOT) as i32);
                    let muzzle = ctx.world.globals.muzzle;
                    let tent = ctx.entity_mut(tent_id);
                    tent.s.origin2 = muzzle;
                    tent.s.shouldtarget = fullCharge;
                    tent.s.eventParm = ent_num;

                    let te_eid = G_TempEntity(ctx, tr.endpos, (EV_SABER_BLOCK) as i32);
                    let te = ctx.entity_mut(te_eid);
                    te.s.origin = tr.endpos;
                    te.s.angles = tr.plane.normal;
                    if te.s.angles[0] == 0.0 && te.s.angles[1] == 0.0 && te.s.angles[2] == 0.0 {
                        te.s.angles[1] = 1.0;
                    }
                    te.s.eventParm = 0;
                    te.s.weapon = 0;
                    te.s.legsAnim = 0;

                    return;
                }
            }

            // always render a shot beam, doing this the old way because I don't much feel like overriding the effect.
            let tent_id = G_TempEntity(ctx, tr.endpos, (EV_DISRUPTOR_SNIPER_SHOT) as i32);
            let muzzle = ctx.world.globals.muzzle;
            let tent = ctx.entity_mut(tent_id);
            tent.s.origin2 = muzzle;
            tent.s.shouldtarget = fullCharge;
            tent.s.eventParm = ent_num;

            // If the beam hits a skybox, etc. it would look foolish to add impact effects
            if render_impact {
                if ctx.world.entity(traceEnt_id).takedamage != 0 && !traceEnt_client.is_null() {
                    let traceEnt_num = ctx.world.entity(traceEnt_id).s.number;
                    ctx.entity_mut(tent_id).s.otherEntityNum = traceEnt_num;

                    let tent_id = G_TempEntity(ctx, tr.endpos, (EV_MISSILE_MISS) as i32);
                    let tent = ctx.entity_mut(tent_id);
                    tent.s.eventParm = DirToByte(tr.plane.normal);
                    tent.s.eFlags |= EF_ALT_FIRING;

                    if LogAccuracyHit(ctx, traceEnt_id, Some(ent)) && !ent_client.is_null() {
                        (*ent_client).accuracy_hits += 1;
                    }
                } else {
                    if ctx.world.entity(traceEnt_id).r.svFlags & SVF_GLASS_BRUSH != 0
                        || ctx.world.entity(traceEnt_id).takedamage != 0
                        || ctx.world.entity(traceEnt_id).s.eType == entityType_t::ET_MOVER as c_int
                    {
                        if ctx.world.entity(traceEnt_id).takedamage != 0 {
                            let dmg_dir = Some(&mut (*ctx.world_raw()).globals.forward); // STAGE-2b: irreducible — &mut world.globals.forward aliases the ctx passed to the same call.
                            G_Damage(
                                ctx,
                                Some(traceEnt_id),
                                Some(ent),
                                Some(ent),
                                dmg_dir,
                                tr.endpos,
                                damage,
                                DAMAGE_NO_KNOCKBACK,
                                MOD_DISRUPTOR_SNIPER as c_int,
                            );

                            let tent_id = G_TempEntity(ctx, tr.endpos, (EV_DISRUPTOR_HIT) as i32);
                            let tent = ctx.entity_mut(tent_id);
                            tent.s.eventParm = DirToByte(tr.plane.normal);
                        }
                    } else {
                        let tent_id =
                            G_TempEntity(ctx, tr.endpos, (EV_DISRUPTOR_SNIPER_MISS) as i32);
                        let tent = ctx.entity_mut(tent_id);
                        tent.s.eventParm = DirToByte(tr.plane.normal);
                    }
                    break; // and don't try any more traces
                }

                if (ctx.world.entity(traceEnt_id).flags & FL_SHIELDED) != 0 {
                    break;
                }

                if ctx.world.entity(traceEnt_id).takedamage != 0 {
                    let mut preAng: vec3_t = [0.0; 3];
                    let preHealth = ctx.world.entity(traceEnt_id).health;
                    let mut preLegs: c_int = 0;
                    let mut preTorso: c_int = 0;

                    let dmg_dir = Some(&mut (*ctx.world_raw()).globals.forward); // STAGE-2b: irreducible — &mut world.globals.forward aliases the ctx passed to the same call.
                    if !traceEnt_client.is_null() {
                        preLegs = (*traceEnt_client).ps.legsAnim;
                        preTorso = (*traceEnt_client).ps.torsoAnim;
                        preAng = (*traceEnt_client).ps.viewangles;
                    }

                    G_Damage(
                        ctx,
                        Some(traceEnt_id),
                        Some(ent),
                        Some(ent),
                        dmg_dir,
                        tr.endpos,
                        damage,
                        DAMAGE_NO_KNOCKBACK,
                        MOD_DISRUPTOR_SNIPER as c_int,
                    );

                    if !traceEnt_client.is_null()
                        && preHealth > 0
                        && ctx.world.entity(traceEnt_id).health <= 0
                        && fullCharge != qfalse
                        && G_CanDisruptify(Some(ctx.world.entity(traceEnt_id))) != qfalse
                    {
                        (*traceEnt_client).ps.viewangles = preAng;
                        (*traceEnt_client).ps.eFlags |= EF_DISINTEGRATION;
                        (*traceEnt_client).ps.lastHitLoc = tr.endpos;
                        (*traceEnt_client).ps.legsAnim = preLegs;
                        (*traceEnt_client).ps.torsoAnim = preTorso;
                        ctx.world.entity_mut(traceEnt_id).r.contents = 0;
                        (*traceEnt_client).ps.velocity = [0.0; 3];
                    }

                    let tent_id = G_TempEntity(ctx, tr.endpos, (EV_DISRUPTOR_HIT) as i32);
                    let tent = ctx.entity_mut(tent_id);
                    tent.s.eventParm = DirToByte(tr.plane.normal);
                    if !traceEnt_client.is_null() {
                        tent.s.weapon = 1;
                    }
                }
            } else {
                break;
            }

            // Oracle updates the file-static `muzzle` so the next penetrating
            // segment's beam origin2 starts at this impact point.
            ctx.world.globals.muzzle = tr.endpos;
            start = tr.endpos;
            skip = (tr.entityNum) as i32;
        }
    }
}

/// Raven `WP_FireDisruptor`.
///
/// Source: `oracle/codemp/game/g_weapon.c:890-912`
pub fn WP_FireDisruptor(ctx: &mut GameContext, ent: Option<EntityId>, altFire: bool) {
    let mut altFire = altFire;
    // FLAG: firing ent may be an NPC (pool client); read the client pointer value
    // and deref it raw as Raven does.
    let client = match ent {
        None => std::ptr::null_mut(),
        Some(id) => ctx.world.entity(id).client,
    };
    if ent.is_none() || client.is_null() || unsafe { (*client).ps.zoomMode } != 1 {
        // do not ever let it do the alt fire when not zoomed
        altFire = false;
    }

    if let Some(id) = ent {
        if ctx.world.entity(id).s.eType == entityType_t::ET_NPC as c_int && client.is_null() {
            // special case for animents
            WP_DisruptorAltFire(ctx, id);
            return;
        }
    }

    if altFire {
        WP_DisruptorAltFire(ctx, ent.unwrap());
    } else {
        WP_DisruptorMainFire(ctx, ent.unwrap());
    }
}

/// Raven `WP_BowcasterAltFire`.
///
/// Source: `oracle/codemp/game/g_weapon.c:923-942`
pub fn WP_BowcasterAltFire(ctx: &mut GameContext, ent: EntityId) {
    let damage: c_int = BOWCASTER_DAMAGE;

    let muzzle = ctx.world.globals.muzzle;
    let forward = ctx.world.globals.forward;
    let mid = CreateMissile(
        ctx,
        muzzle,
        forward,
        BOWCASTER_VELOCITY as f32,
        10000,
        ent,
        false,
    );
    let m = ctx.world.entity_mut(mid);

    m.classname = c"bowcaster_proj".as_ptr() as *mut c_char;
    m.s.weapon = WP_BOWCASTER;

    m.r.maxs = [
        (BOWCASTER_SIZE) as f32,
        (BOWCASTER_SIZE) as f32,
        (BOWCASTER_SIZE) as f32,
    ];
    for i in 0..3 {
        m.r.mins[i] = -m.r.maxs[i];
    }

    m.damage = damage;
    m.dflags = DAMAGE_DEATH_KNOCKBACK;
    m.methodOfDeath = MOD_BOWCASTER as c_int;
    m.clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;

    m.flags |= FL_BOUNCE;
    m.bounceCount = 3;
}

/// Raven `WP_BowcasterMainFire`.
///
/// Source: `oracle/codemp/game/g_weapon.c:945-1029`
pub fn WP_BowcasterMainFire(ctx: &mut GameContext, ent: EntityId) {
    let mut damage: c_int = BOWCASTER_DAMAGE;
    let mut count: c_int;
    let mut vel: f32;
    let mut angs: vec3_t = [0.0; 3];
    let mut dir: vec3_t = [0.0; 3];

    // FLAG: firing ent may be an NPC (pool client); read the client pointer value
    // and deref it raw as Raven does.
    let ent_client = ctx.world.entity(ent).client;
    if ent_client.is_null() {
        count = 1;
    } else {
        let now = ctx.world.level.time;
        count = ((now - unsafe { (*ent_client).ps.weaponChargeTime }) as f32
            / BOWCASTER_CHARGE_UNIT) as c_int;
    }

    if count < 1 {
        count = 1;
    } else if count > 5 {
        count = 5;
    }

    if count & 1 == 0 {
        count -= 1;
    }

    if count <= 1 {
        damage = 50;
    } else if count == 2 {
        damage = 45;
    } else if count == 3 {
        damage = 40;
    } else if count == 4 {
        damage = 35;
    } else {
        damage = 30;
    }

    for i in 0..count {
        // C: `BOWCASTER_VELOCITY * (crandom()*RANGE + 1.0f)` runs in `double`
        // (`crandom()` is `double`) and narrows once to the `float vel`.
        vel = (BOWCASTER_VELOCITY as f64
            * (ctx.world.bg_state.rng.crandom() * BOWCASTER_VEL_RANGE as f64 + 1.0))
            as f32;

        vectoangles(ctx.world.globals.forward, &mut angs);

        // C: `crandom()*BOWCASTER_ALT_SPREAD*0.2f` runs in `double`; `0.2f`
        // promotes as `(double)0.2f`, not the `double` `0.2` literal.
        angs[PITCH] = (angs[PITCH] as f64
            + ctx.world.bg_state.rng.crandom() * BOWCASTER_ALT_SPREAD as f64 * 0.2f32 as f64)
            as f32;
        angs[YAW] +=
            (i as f32 + 0.5) * BOWCASTER_ALT_SPREAD - count as f32 * 0.5 * BOWCASTER_ALT_SPREAD;

        AngleVectors(angs, Some(&mut dir), None, None);

        let muzzle = ctx.world.globals.muzzle;
        let mid = CreateMissile(ctx, muzzle, dir, vel, 10000, ent, true);
        let m = ctx.world.entity_mut(mid);

        m.classname = c"bowcaster_alt_proj".as_ptr() as *mut c_char;
        m.s.weapon = WP_BOWCASTER;

        m.r.maxs = [
            (BOWCASTER_SIZE) as f32,
            (BOWCASTER_SIZE) as f32,
            (BOWCASTER_SIZE) as f32,
        ];
        for k in 0..3 {
            m.r.mins[k] = -m.r.maxs[k];
        }

        m.damage = damage;
        m.dflags = DAMAGE_DEATH_KNOCKBACK;
        m.methodOfDeath = MOD_BOWCASTER as c_int;
        m.clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;

        // we don't want it to bounce
        m.bounceCount = 0;
    }
}

/// Raven `WP_FireBowcaster`.
///
/// Source: `oracle/codemp/game/g_weapon.c:1032-1043`
pub fn WP_FireBowcaster(ctx: &mut GameContext, ent: EntityId, altFire: bool) {
    if altFire {
        WP_BowcasterAltFire(ctx, ent);
    } else {
        WP_BowcasterMainFire(ctx, ent);
    }
}

/// Raven `WP_RepeaterMainFire`.
///
/// Source: `oracle/codemp/game/g_weapon.c:1056-1073`
pub fn WP_RepeaterMainFire(ctx: &mut GameContext, ent: EntityId, dir: vec3_t) {
    let damage: c_int = REPEATER_DAMAGE;

    let muzzle = ctx.world.globals.muzzle;
    let mid = CreateMissile(
        ctx,
        muzzle,
        dir,
        REPEATER_VELOCITY as f32,
        10000,
        ent,
        false,
    );
    let m = ctx.world.entity_mut(mid);

    m.classname = c"repeater_proj".as_ptr() as *mut c_char;
    m.s.weapon = WP_REPEATER;

    m.damage = damage;
    m.dflags = DAMAGE_DEATH_KNOCKBACK;
    m.methodOfDeath = MOD_REPEATER as c_int;
    m.clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;

    // we don't want it to bounce forever
    m.bounceCount = 8;
}

/// Raven `WP_RepeaterAltFire`.
///
/// Source: `oracle/codemp/game/g_weapon.c:1076-1107`
pub fn WP_RepeaterAltFire(ctx: &mut GameContext, ent: EntityId) {
    let damage: c_int = REPEATER_ALT_DAMAGE;

    let muzzle = ctx.world.globals.muzzle;
    let forward = ctx.world.globals.forward;
    let mid = CreateMissile(
        ctx,
        muzzle,
        forward,
        REPEATER_ALT_VELOCITY as f32,
        10000,
        ent,
        true,
    );
    // Read the gametype cvar before taking the missile borrow.
    let siege = ctx.world.cvars.g_gametype.integer == GT_SIEGE;
    let m = ctx.world.entity_mut(mid);

    m.classname = c"repeater_alt_proj".as_ptr() as *mut c_char;
    m.s.weapon = WP_REPEATER;

    m.r.maxs = [
        (REPEATER_ALT_SIZE) as f32,
        (REPEATER_ALT_SIZE) as f32,
        (REPEATER_ALT_SIZE) as f32,
    ];
    for i in 0..3 {
        m.r.mins[i] = -m.r.maxs[i];
    }
    m.s.pos.trType = TR_GRAVITY;
    m.s.pos.trDelta[2] += 40.0; // give a slight boost in the upward direction
    m.damage = damage;
    m.dflags = DAMAGE_DEATH_KNOCKBACK;
    m.methodOfDeath = MOD_REPEATER_ALT as c_int;
    m.splashMethodOfDeath = MOD_REPEATER_ALT_SPLASH as c_int;
    m.clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
    m.splashDamage = REPEATER_ALT_SPLASH_DAMAGE;
    if siege {
        // we've been having problems with this being too hyper-potent because of it's radius
        m.splashRadius = REPEATER_ALT_SPLASH_RAD_SIEGE;
    } else {
        m.splashRadius = REPEATER_ALT_SPLASH_RADIUS;
    }

    // we don't want it to bounce forever
    m.bounceCount = 8;
}

/// Raven `WP_FireRepeater`.
///
/// Source: `oracle/codemp/game/g_weapon.c:1110-1131`
pub fn WP_FireRepeater(ctx: &mut GameContext, ent: EntityId, altFire: bool) {
    let mut dir: vec3_t = [0.0; 3];
    let mut angs: vec3_t = [0.0; 3];

    vectoangles(ctx.world.globals.forward, &mut angs);

    if altFire {
        WP_RepeaterAltFire(ctx, ent);
    } else {
        // add some slop to the alt-fire direction
        // C: `crandom()` is `double`; each `+=` runs in `double`, narrows to float.
        angs[PITCH] =
            (angs[PITCH] as f64 + ctx.world.bg_state.rng.crandom() * REPEATER_SPREAD as f64) as f32;
        angs[YAW] =
            (angs[YAW] as f64 + ctx.world.bg_state.rng.crandom() * REPEATER_SPREAD as f64) as f32;

        AngleVectors(angs, Some(&mut dir), None, None);

        WP_RepeaterMainFire(ctx, ent, dir);
    }
}

/// Raven `WP_DEMP2_MainFire`.
///
/// Source: `oracle/codemp/game/g_weapon.c:1142-1160`
pub fn WP_DEMP2_MainFire(ctx: &mut GameContext, ent: EntityId) {
    let damage: c_int = DEMP2_DAMAGE;

    let muzzle = ctx.world.globals.muzzle;
    let forward = ctx.world.globals.forward;
    let mid = CreateMissile(
        ctx,
        muzzle,
        forward,
        DEMP2_VELOCITY as f32,
        10000,
        ent,
        false,
    );
    let m = ctx.world.entity_mut(mid);

    m.classname = c"demp2_proj".as_ptr() as *mut c_char;
    m.s.weapon = WP_DEMP2;

    m.r.maxs = [
        (DEMP2_SIZE) as f32,
        (DEMP2_SIZE) as f32,
        (DEMP2_SIZE) as f32,
    ];
    for i in 0..3 {
        m.r.mins[i] = -m.r.maxs[i];
    }
    m.damage = damage;
    m.dflags = DAMAGE_DEATH_KNOCKBACK;
    m.methodOfDeath = MOD_DEMP2 as c_int;
    m.clipmask = MASK_SHOT;

    // we don't want it to ever bounce
    m.bounceCount = 0;
}

/// Raven `DEMP2_AltRadiusDamage`.
///
/// Source: `oracle/codemp/game/g_weapon.c:1164-1307`
pub fn DEMP2_AltRadiusDamage(ctx: &mut GameContext, ent: EntityId) {
    unsafe {
        // `ent` is the DEMP2 shell projectile; read its constant fields once.
        let ent_origin = ctx.world.entity(ent).r.currentOrigin;
        let ent_count = ctx.world.entity(ent).count;
        let ent_damage = ctx.world.entity(ent).damage;
        let ent_splash_mod = ctx.world.entity(ent).splashMethodOfDeath;
        let ent_generic_value6 = ctx.world.entity(ent).genericValue6;
        let mut frac: f32 =
            (ctx.world.level.time - ctx.world.entity(ent).genericValue5) as f32 / 800.0;
        let mut dist: f32;
        let mut radius: f32;
        let mut fact: f32;
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];
        let mut v: vec3_t = [0.0; 3];
        let mut dir: vec3_t = [0.0; 3];

        let owner_num = ctx.world.entity(ent).r.ownerNum;
        let myOwner_id: Option<EntityId> = if owner_num >= 0 && (owner_num as usize) < MAX_GENTITIES
        {
            Some(EntityId(owner_num as u32))
        } else {
            None
        };

        // FLAG: owner may be an NPC (pool client); deref its client raw.
        let owner_ok = match myOwner_id {
            Some(id) => ctx.world.entity(id).inuse != 0 && !ctx.world.entity(id).client.is_null(),
            None => false,
        };
        if !owner_ok {
            let now = ctx.world.level.time;
            ctx.world.entity_mut(ent).think = Some(EntThink::G_FreeEntity).into();
            ctx.world.entity_mut(ent).nextthink = now;
            return;
        }
        let myOwner_id = myOwner_id.unwrap();

        frac *= frac * frac; // yes, this is completely ridiculous...but it causes the shell to grow slowly then "explode" at the end

        radius = frac * 200.0; // 200 is max radius...the model is aprox. 100 units tall...the fx draw code mults. this by 2.

        // C's `0.6` is a double literal, so `count*0.6` runs in f64 and narrows
        // to float at the store.
        fact = (ent_count as f64 * 0.6) as f32;

        if fact < 1.0 {
            fact = 1.0;
        }

        radius *= fact;

        for i in 0..3 {
            mins[i] = ent_origin[i] - radius;
            maxs[i] = ent_origin[i] + radius;
        }

        let mut iEntityList: [c_int; MAX_GENTITIES] = [0; MAX_GENTITIES];
        let numListedEntities = trap::EntitiesInBox(
            ctx.engine,
            mp_abi::game::syscalls::G_ENTITIES_IN_BOX::GEntitiesInBoxArgs::new(
                &mins as *const vec3_t,
                &maxs as *const vec3_t,
                iEntityList.as_mut_ptr(),
                MAX_GENTITIES as c_int,
            ),
        );

        for e in 0..numListedEntities {
            let gent_id = EntityId(iEntityList[e as usize] as u32);
            // FLAG: box target may be an NPC/vehicle (pool client); deref raw.
            let gent_client = ctx.world.entity(gent_id).client;

            if ctx.world.entity(gent_id).takedamage == 0
                || ctx.world.entity(gent_id).r.contents == 0
            {
                continue;
            }

            // find the distance from the edge of the bounding box
            for i in 0..3 {
                if ent_origin[i] < ctx.world.entity(gent_id).r.absmin[i] {
                    v[i] = ctx.world.entity(gent_id).r.absmin[i] - ent_origin[i];
                } else if ent_origin[i] > ctx.world.entity(gent_id).r.absmax[i] {
                    v[i] = ent_origin[i] - ctx.world.entity(gent_id).r.absmax[i];
                } else {
                    v[i] = 0.0;
                }
            }

            // shape is an ellipsoid, so cut vertical distance in half
            v[2] *= 0.5;

            dist = VectorLength(v);

            if dist >= radius {
                // shockwave hasn't hit them yet
                continue;
            }

            if dist + (16.0 * ent_count as f32) < ent_generic_value6 as f32 {
                // shockwave has already hit this thing...
                continue;
            }

            v = ctx.world.entity(gent_id).r.currentOrigin;
            for i in 0..3 {
                dir[i] = v[i] - ent_origin[i];
            }

            // push the center of mass higher than the origin so players get knocked into the air more
            dir[2] += 12.0;

            if gent_id != myOwner_id {
                G_Damage(
                    ctx,
                    Some(gent_id),
                    Some(myOwner_id),
                    Some(myOwner_id),
                    Some(&mut dir),
                    ent_origin,
                    ent_damage,
                    DAMAGE_DEATH_KNOCKBACK,
                    ent_splash_mod,
                );
                if ctx.world.entity(gent_id).takedamage != 0 && !gent_client.is_null() {
                    if (*gent_client).ps.electrifyTime < ctx.world.level.time {
                        // electrocution effect
                        if ctx.world.entity(gent_id).s.eType == entityType_t::ET_NPC as c_int
                            && ctx.world.entity(gent_id).s.NPC_class == (CLASS_VEHICLE) as i32
                            && !ctx.world.entity(gent_id).m_pVehicle.is_null()
                            && {
                                let veh = ctx.world.entity(gent_id).m_pVehicle;
                                let vtype = (*(*veh).m_pVehicleInfo).r#type;
                                vtype == mp_bg::vehicles::vehicleType_t::VH_SPEEDER
                                    || vtype == mp_bg::vehicles::vehicleType_t::VH_WALKER
                            }
                        {
                            // do some extra stuff to speeders/walkers
                            (*gent_client).ps.electrifyTime =
                                ctx.world.level.time + ctx.world.bg_state.rng.Q_irand(3000, 4000);
                        } else if ctx.world.entity(gent_id).s.NPC_class != (CLASS_VEHICLE) as i32
                            || (!ctx.world.entity(gent_id).m_pVehicle.is_null() && {
                                let veh = ctx.world.entity(gent_id).m_pVehicle;
                                (*(*veh).m_pVehicleInfo).r#type
                                    != mp_bg::vehicles::vehicleType_t::VH_FIGHTER
                            })
                        {
                            // don't do this to fighters
                            (*gent_client).ps.electrifyTime =
                                ctx.world.level.time + ctx.world.bg_state.rng.Q_irand(300, 800);
                        }
                    }
                    if (*gent_client).ps.powerups[PW_CLOAKED as usize] != 0 {
                        // disable cloak temporarily
                        Jedi_Decloak(ctx, Some(gent_id));
                        (*gent_client).cloakToggleTime =
                            ctx.world.level.time + ctx.world.bg_state.rng.Q_irand(3000, 10000);
                    }
                }
            }
        }

        // store the last fraction so that next time around we can test against those things that fall between that last point and where the current shockwave edge is
        ctx.world.entity_mut(ent).genericValue6 = radius as c_int;

        if frac < 1.0 {
            // shock is still happening so continue letting it expand
            let now = ctx.world.level.time;
            ctx.world.entity_mut(ent).nextthink = now + 50;
        } else {
            // don't just leave the entity around
            ctx.world.entity_mut(ent).think = Some(EntThink::G_FreeEntity).into();
            let now = ctx.world.level.time;
            ctx.world.entity_mut(ent).nextthink = now;
        }
    }
}

/// Raven `DEMP2_AltDetonate`.
///
/// Source: `oracle/codemp/game/g_weapon.c:1310-1333`
pub fn DEMP2_AltDetonate(ctx: &mut GameContext, ent: EntityId) {
    let origin = ctx.world.entity(ent).r.currentOrigin;
    G_SetOrigin(ctx.world.entity_mut(ent), origin);
    {
        let e = ctx.world.entity_mut(ent);
        if e.pos1[0] == 0.0 && e.pos1[1] == 0.0 && e.pos1[2] == 0.0 {
            // don't play effect with a 0'd out directional vector
            e.pos1[1] = 1.0;
        }
    }
    // Let's just save ourself some bandwidth and play both the effect and sphere spawn in 1 event
    let origin = ctx.world.entity(ent).r.currentOrigin;
    let pos1 = ctx.world.entity(ent).pos1;
    let efEnt = G_PlayEffect((EFFECT_EXPLOSION_DEMP2ALT) as i32, origin, pos1);

    if !efEnt.is_null() {
        let count = ctx.world.entity(ent).count;
        let eid = ctx.entity_id_of(efEnt).unwrap();
        ctx.world.entity_mut(eid).s.weapon = count * 2;
    }

    let now = ctx.world.level.time;
    let e = ctx.world.entity_mut(ent);
    e.genericValue5 = now;
    e.genericValue6 = 0;
    e.nextthink = now + 50;
    e.think = Some(EntThink::DEMP2_AltRadiusDamage).into();
    e.s.eType = entityType_t::ET_GENERAL as c_int; // make us a missile no longer
}

/// Raven `WP_DEMP2_AltFire`.
///
/// Source: `oracle/codemp/game/g_weapon.c:1336-1403`
pub fn WP_DEMP2_AltFire(ctx: &mut GameContext, ent: EntityId) {
    let mut damage: c_int = DEMP2_ALT_DAMAGE;
    let mut count: c_int;
    let origcount: c_int;
    let mut fact: f32;
    let start: vec3_t = ctx.world.globals.muzzle;
    let mut end: vec3_t = [0.0; 3];
    let mut tr: trace_t = unsafe { std::mem::zeroed() };

    for i in 0..3 {
        end[i] = start[i] + DEMP2_ALT_RANGE as f32 * ctx.world.globals.forward[i];
    }

    // FLAG: firing ent may be an NPC (pool client); deref the client value raw.
    let ent_client = ctx.world.entity(ent).client;
    let charge_now = ctx.world.level.time;
    count = ((charge_now - unsafe { (*ent_client).ps.weaponChargeTime }) as f32 / DEMP2_CHARGE_UNIT)
        as c_int;

    origcount = count;

    if count < 1 {
        count = 1;
    } else if count > 3 {
        count = 3;
    }

    // C's `0.8` is a double literal, so `count*0.8` runs in f64 and narrows
    // to float at the store.
    fact = (count as f64 * 0.8) as f32;
    if fact < 1.0 {
        fact = 1.0;
    }
    damage = (damage as f32 * fact) as c_int;

    if origcount == 0 {
        // this was just a tap-fire
        damage = 1;
    }

    let ent_num = ctx.world.entity(ent).s.number;
    trap::Trace(
        ctx.engine,
        mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
            &mut tr,
            &start as *const vec3_t,
            std::ptr::null(),
            std::ptr::null(),
            &end as *const vec3_t,
            ent_num,
            MASK_SHOT,
        ),
    );

    let mid = G_Spawn(ctx);
    G_SetOrigin(ctx.world.entity_mut(mid), tr.endpos);
    // In SP the impact actually travels as a missile based on the trace fraction, but we're
    // just going to be instant. -rww

    let now = ctx.world.level.time;
    let m = ctx.world.entity_mut(mid);
    m.pos1 = tr.plane.normal;

    m.count = count;

    m.classname = c"demp2_alt_proj".as_ptr() as *mut c_char;
    m.s.weapon = WP_DEMP2;

    m.think = Some(EntThink::DEMP2_AltDetonate).into();
    m.nextthink = now;

    m.splashDamage = damage;
    m.damage = damage;
    m.splashMethodOfDeath = MOD_DEMP2 as c_int;
    m.methodOfDeath = MOD_DEMP2 as c_int;
    m.splashRadius = DEMP2_ALT_SPLASHRADIUS;

    m.r.ownerNum = ent_num;

    m.dflags = DAMAGE_DEATH_KNOCKBACK;
    m.clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;

    // we don't want it to ever bounce
    m.bounceCount = 0;
}

/// Raven `WP_FireDEMP2`.
///
/// Source: `oracle/codemp/game/g_weapon.c:1406-1417`
pub fn WP_FireDEMP2(ctx: &mut GameContext, ent: EntityId, altFire: bool) {
    if altFire {
        WP_DEMP2_AltFire(ctx, ent);
    } else {
        WP_DEMP2_MainFire(ctx, ent);
    }
}

/// Raven `WP_FlechetteMainFire`.
///
/// Source: `oracle/codemp/game/g_weapon.c:1430-1467`
pub fn WP_FlechetteMainFire(ctx: &mut GameContext, ent: EntityId) {
    let mut fwd: vec3_t = [0.0; 3];
    let mut angs: vec3_t = [0.0; 3];

    for i in 0..FLECHETTE_SHOTS {
        vectoangles(ctx.world.globals.forward, &mut angs);

        if i != 0 {
            // do nothing on the first shot, it will hit the crosshairs
            // C: `crandom()` is `double`; each `+=` runs in `double`, narrows to float.
            angs[PITCH] = (angs[PITCH] as f64
                + ctx.world.bg_state.rng.crandom() * FLECHETTE_SPREAD as f64)
                as f32;
            angs[YAW] = (angs[YAW] as f64
                + ctx.world.bg_state.rng.crandom() * FLECHETTE_SPREAD as f64)
                as f32;
        }

        AngleVectors(angs, Some(&mut fwd), None, None);

        let muzzle = ctx.world.globals.muzzle;
        let mid = CreateMissile(ctx, muzzle, fwd, FLECHETTE_VEL as f32, 10000, ent, false);
        let m = ctx.world.entity_mut(mid);

        m.classname = c"flech_proj".as_ptr() as *mut c_char;
        m.s.weapon = WP_FLECHETTE;

        m.r.maxs = [
            (FLECHETTE_SIZE) as f32,
            (FLECHETTE_SIZE) as f32,
            (FLECHETTE_SIZE) as f32,
        ];
        for k in 0..3 {
            m.r.mins[k] = -m.r.maxs[k];
        }

        m.damage = FLECHETTE_DAMAGE;
        m.dflags = DAMAGE_DEATH_KNOCKBACK;
        m.methodOfDeath = MOD_FLECHETTE as c_int;
        m.clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;

        // we don't want it to bounce forever
        let bounce = ctx.world.bg_state.rng.Q_irand(5, 8);
        let m = ctx.world.entity_mut(mid);
        m.bounceCount = bounce;

        m.flags |= FL_BOUNCE_SHRAPNEL;
    }
}

/// Raven `prox_mine_think`.
///
/// Source: `oracle/codemp/game/g_weapon.c:1470-1506`
pub fn prox_mine_think(ctx: &mut GameContext, ent: EntityId) {
    let mut blow = qfalse;

    // if it isn't time to auto-explode, do a small proximity check
    if ctx.world.entity(ent).delay > ctx.world.level.time {
        let mut ent_list: [*mut gentity_t; MAX_GENTITIES] = [std::ptr::null_mut(); MAX_GENTITIES];
        let origin = ctx.world.entity(ent).r.currentOrigin;
        let count = G_RadiusList(
            ctx,
            origin,
            (FLECHETTE_MINE_RADIUS_CHECK) as f32,
            Some(ent),
            qtrue,
            ent_list.as_mut_ptr(),
        );

        for i in 0..count {
            let e = ent_list[i as usize];
            let e_id = ctx.entity_id_of(e).unwrap();
            if !ctx.world.entity(e_id).client.is_null()
                && ctx.world.entity(e_id).health > 0
                && ctx.world.entity(ent).activator.is_some()
                && ctx.world.entity(e_id).s.number
                    != ctx
                        .world
                        .entity(ctx.world.entity(ent).activator.unwrap())
                        .s
                        .number
            {
                blow = qtrue;
                break;
            }
        }
    } else {
        // well, we must die now
        blow = qtrue;
    }

    if blow != qfalse {
        let now = ctx.world.level.time;
        let e = ctx.world.entity_mut(ent);
        e.think = Some(EntThink::laserTrapExplode).into();
        e.nextthink = now + 200;
    } else {
        // we probably don't need to do this thinking logic very often...maybe this is fast enough?
        let now = ctx.world.level.time;
        ctx.world.entity_mut(ent).nextthink = now + 500;
    }
}

/// Raven `WP_TraceSetStart`.
///
/// Source: `oracle/codemp/game/g_weapon.c:1509-1541`
// Oracle writes the wall-corrected point back through the `start` out-param
// (`VectorCopy(tr.endpos, start)`); return it so callers pick up the adjustment.
pub fn WP_TraceSetStart(
    ctx: &mut GameContext,
    ent: EntityId,
    start: vec3_t,
    mins: vec3_t,
    maxs: vec3_t,
) -> vec3_t {
    let mut start = start;
    let mut entMins: vec3_t = [0.0; 3];
    let mut entMaxs: vec3_t = [0.0; 3];
    {
        let e = ctx.world.entity(ent);
        for i in 0..3 {
            entMins[i] = e.r.currentOrigin[i] + e.r.mins[i];
            entMaxs[i] = e.r.currentOrigin[i] + e.r.maxs[i];
        }
    }

    if G_BoxInBounds(start, mins, maxs, entMins, entMaxs) != qfalse {
        return start;
    }

    // FLAG: firing ent may be an NPC (pool client); read the client pointer value
    // and deref it raw as Raven does.
    let ent_client = ctx.world.entity(ent).client;
    if ent_client.is_null() {
        return start;
    }

    let mut tr: trace_t = unsafe { std::mem::zeroed() };
    let ps_origin = unsafe { (*ent_client).ps.origin };
    let ent_num = ctx.world.entity(ent).s.number;
    trap::Trace(
        ctx.engine,
        mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
            &mut tr,
            &ps_origin as *const vec3_t,
            &mins as *const vec3_t,
            &maxs as *const vec3_t,
            &start as *const vec3_t,
            ent_num,
            MASK_SOLID | CONTENTS_SHOTCLIP,
        ),
    );

    if tr.startsolid != 0 || tr.allsolid != 0 {
        return start;
    }

    if tr.fraction < 1.0 {
        start = tr.endpos;
    }
    start
}

/// Raven `WP_ExplosiveDie`.
///
/// Source: `oracle/codemp/game/g_weapon.c:1543-1546`
pub fn WP_ExplosiveDie(
    ctx: &mut GameContext,
    self_: EntityId,
    inflictor: Option<EntityId>,
    attacker: Option<EntityId>,
    damage: c_int,
    r#mod: c_int,
) {
    // Raven: `inflictor`/`attacker`/`damage`/`mod` are unused by the body.
    laserTrapExplode(ctx, self_);
}

/// Raven `WP_flechette_alt_blow`.
///
/// Source: `oracle/codemp/game/g_weapon.c:1549-1557`
pub fn WP_flechette_alt_blow(ctx: &mut GameContext, ent: EntityId) {
    let e = ctx.world.entity_mut(ent);
    e.s.pos.trDelta[0] = 1.0;
    e.s.pos.trDelta[1] = 0.0;
    e.s.pos.trDelta[2] = 0.0;

    laserTrapExplode(ctx, ent);
}

/// Raven `WP_CreateFlechetteBouncyThing`.
///
/// Source: `oracle/codemp/game/g_weapon.c:1560-1599`
pub fn WP_CreateFlechetteBouncyThing(
    ctx: &mut GameContext,
    start: vec3_t,
    fwd: vec3_t,
    self_: EntityId,
) {
    let vel = 700.0 + ctx.world.bg_state.rng.random() * 700.0;
    let life = 1500.0 + ctx.world.bg_state.rng.random() * 2000.0;
    let mid = CreateMissile(ctx, start, fwd, vel, life as c_int, self_, true);
    let m = ctx.world.entity_mut(mid);

    m.think = Some(EntThink::WP_flechette_alt_blow).into();

    m.activator = Some(self_);

    m.s.weapon = WP_FLECHETTE;
    m.classname = c"flech_alt".as_ptr() as *mut c_char;
    m.mass = (4) as f32;

    // How 'bout we give this thing a size...
    m.r.mins = [-3.0, -3.0, -3.0];
    m.r.maxs = [3.0, 3.0, 3.0];
    m.clipmask = MASK_SHOT;

    m.touch = Some(EntTouch::touch_NULL).into();

    // normal ones bounce, alt ones explode on impact
    m.s.pos.trType = TR_GRAVITY;

    m.flags |= FL_BOUNCE_HALF;
    m.s.eFlags |= EF_ALT_FIRING;

    m.bounceCount = 50;

    m.damage = FLECHETTE_ALT_DAMAGE;
    m.dflags = 0;
    m.splashDamage = FLECHETTE_ALT_SPLASH_DAM;
    m.splashRadius = FLECHETTE_ALT_SPLASH_RAD;

    m.r.svFlags = SVF_USE_CURRENT_ORIGIN;

    m.methodOfDeath = MOD_FLECHETTE_ALT_SPLASH as c_int;
    m.splashMethodOfDeath = MOD_FLECHETTE_ALT_SPLASH as c_int;

    m.pos2 = start;
}

/// Raven `WP_FlechetteAltFire`.
///
/// Source: `oracle/codemp/game/g_weapon.c:1602-1623`
pub fn WP_FlechetteAltFire(ctx: &mut GameContext, self_: EntityId) {
    let mut dir: vec3_t;
    let mut fwd: vec3_t = [0.0; 3];
    let mut start: vec3_t = ctx.world.globals.muzzle;
    let mut angs: vec3_t = [0.0; 3];

    vectoangles(ctx.world.globals.forward, &mut angs);

    start = WP_TraceSetStart(ctx, self_, start, vec3_origin, vec3_origin); // make sure our start point isn't on the other side of a wall

    for _i in 0..2 {
        dir = angs;

        dir[PITCH] -= ctx.world.bg_state.rng.random() * 4.0 + 8.0; // make it fly upwards
                                                                   // C: `crandom() * 2` is `double`; narrows back to the `float` component.
        dir[YAW] = (dir[YAW] as f64 + ctx.world.bg_state.rng.crandom() * 2.0) as f32;
        AngleVectors(dir, Some(&mut fwd), None, None);

        WP_CreateFlechetteBouncyThing(ctx, start, fwd, self_);
    }
}

/// Raven `WP_FireFlechette`.
///
/// Source: `oracle/codemp/game/g_weapon.c:1626-1638`
pub fn WP_FireFlechette(ctx: &mut GameContext, ent: EntityId, altFire: bool) {
    if altFire {
        // WP_FlechetteProxMine( ent );
        WP_FlechetteAltFire(ctx, ent);
    } else {
        WP_FlechetteMainFire(ctx, ent);
    }
}

/// Raven `rocketThink`.
///
/// Source: `oracle/codemp/game/g_weapon.c:1651-1811`
pub fn rocketThink(ctx: &mut GameContext, ent: EntityId) {
    let up: vec3_t = [0.0, 0.0, 1.0];
    let mut right: vec3_t = [0.0; 3];
    let mut org: vec3_t;
    let mut targetdir: vec3_t = [0.0; 3];
    let mut newdir: vec3_t;
    let mut dot: f32;
    let mut dot2: f32;
    let mut vel: f32 = if ctx.world.entity(ent).spawnflags & 1 != 0 {
        ctx.world.entity(ent).speed
    } else {
        ROCKET_VELOCITY as f32
    };

    if ctx.world.entity(ent).genericValue1 != 0
        && ctx.world.entity(ent).genericValue1 < ctx.world.level.time
    {
        // time's up, we're done, remove us
        if ctx.world.entity(ent).genericValue2 != 0 {
            // explode when die
            let owner_id = EntityId(ctx.world.entity(ent).r.ownerNum as u32);
            RocketDie(
                ctx,
                ent,
                Some(owner_id),
                Some(owner_id),
                0,
                MOD_UNKNOWN as c_int,
            );
        } else {
            // just remove when die
            G_FreeEntity(ctx, Some(ent));
        }
        return;
    }

    // FLAG: enemy is an arbitrary entity (pool client possible); deref the client
    // pointer value raw as Raven does.
    let no_enemy = ctx.world.entity(ent).enemy.is_none() || {
        let eid = ctx.world.entity(ent).enemy.unwrap();
        let enemy_client = ctx.world.entity(eid).client;
        enemy_client.is_null()
            || ctx.world.entity(eid).health <= 0
            || unsafe { (*enemy_client).ps.powerups[PW_CLOAKED as usize] } != 0
    };
    if no_enemy {
        // no enemy or enemy not a client or enemy dead or enemy cloaked
        if ctx.world.entity(ent).genericValue1 == 0 {
            // doesn't have its own self-kill time
            let now = ctx.world.level.time;
            let e = ctx.world.entity_mut(ent);
            e.nextthink = now + 10000;
            e.think = Some(EntThink::G_FreeEntity).into();
        }
        return;
    }

    let enemy_id = ctx.world.entity(ent).enemy.unwrap();

    if ctx.world.entity(ent).spawnflags & 1 != 0 {
        // vehicle rocket
        // FLAG: enemy pool client deref stays raw.
        let enemy_client = ctx.world.entity(enemy_id).client;
        if !enemy_client.is_null() && unsafe { (*enemy_client).NPC_class } == CLASS_VEHICLE {
            // tracking another vehicle
            if unsafe { (*enemy_client).ps.speed } as f32 + 4000.0 > vel {
                vel = unsafe { (*enemy_client).ps.speed } as f32 + 4000.0;
            }
        }
    }

    if ctx.world.entity(enemy_id).inuse != 0 {
        let ent_angle = ctx.world.entity(ent).angle;
        let newDirMult = if ent_angle != 0.0 {
            ent_angle * 2.0
        } else {
            1.0
        };
        let oldDirMult = if ent_angle != 0.0 {
            (1.0 - ent_angle) * 2.0
        } else {
            1.0
        };

        org = ctx.world.entity(enemy_id).r.currentOrigin;
        {
            let en = ctx.world.entity(enemy_id);
            org[2] += (en.r.mins[2] + en.r.maxs[2]) * 0.5;
        }

        let ent_origin = ctx.world.entity(ent).r.currentOrigin;
        _VectorSubtract(org, ent_origin, &mut targetdir);
        VectorNormalize(&mut targetdir);

        // Now the rocket can't do a 180 in space, so we'll limit the turn to about 45 degrees.
        let ent_movedir = ctx.world.entity(ent).movedir;
        dot = _DotProduct(targetdir, ent_movedir);
        if ctx.world.entity(ent).spawnflags & 1 != 0 {
            // vehicle rocket
            if ctx.world.entity(ent).radius > -1.0 {
                // can lose the lock if DotProduct drops below this number
                if dot < ctx.world.entity(ent).radius {
                    // lost the lock!!!
                    return;
                }
            }
        }

        // a dot of 1.0 means right-on-target.
        newdir = [0.0; 3];
        if dot < 0.0 {
            // Go in the direction opposite, start a 180.
            CrossProduct(ent_movedir, up, &mut right);
            dot2 = _DotProduct(targetdir, right);

            if dot2 > 0.0 {
                // Turn 45 degrees right.
                _VectorMA(ent_movedir, 0.4 * newDirMult, right, &mut newdir);
            } else {
                // Turn 45 degrees left.
                _VectorMA(ent_movedir, -0.4 * newDirMult, right, &mut newdir);
            }

            // Yeah we've adjusted horizontally, but let's split the difference vertically, so we kinda try to move towards it.
            newdir[2] = ((targetdir[2] * newDirMult) + (ent_movedir[2] * oldDirMult)) * 0.5;

            // let's also slow down a lot
            vel *= 0.5;
        } else if dot < 0.70 {
            // Still a bit off, so we turn a bit softer
            _VectorMA(ent_movedir, 0.5 * newDirMult, targetdir, &mut newdir);
        } else {
            // getting close, so turn a bit harder
            _VectorMA(ent_movedir, 0.9 * newDirMult, targetdir, &mut newdir);
        }

        // add crazy drunkenness
        let ent_random = ctx.world.entity(ent).random;
        for i in 0..3 {
            // C: `crandom() * ent->random * 0.25f` is `double`; narrows to float.
            newdir[i] = (newdir[i] as f64
                + ctx.world.bg_state.rng.crandom() * ent_random as f64 * 0.25)
                as f32;
        }

        // decay the randomness
        ctx.world.entity_mut(ent).random = ent_random * 0.9;

        // FLAG: enemy pool client deref stays raw.
        let enemy_client = ctx.world.entity(enemy_id).client;
        if !enemy_client.is_null()
            && unsafe { (*enemy_client).ps.groundEntityNum } != ENTITYNUM_NONE as c_int
        {
            // tracking a client who's on the ground, aim at the floor...?
            // Try to crash into the ground if we get close enough to do splash damage
            let ent_origin = ctx.world.entity(ent).r.currentOrigin;
            let dis = Distance(ent_origin, org);

            if dis < 128.0 {
                // the closer we get, the more we push the rocket down, heh heh.
                newdir[2] -= (1.0 - (dis / 128.0)) * 0.6;
            }
        }

        VectorNormalize(&mut newdir);

        _VectorScale(
            newdir,
            vel * 0.5,
            &mut ctx.world.entity_mut(ent).s.pos.trDelta,
        );
        ctx.world.entity_mut(ent).movedir = newdir;
        snap_vector(&mut ctx.world.entity_mut(ent).s.pos.trDelta); // save net bandwidth
        let ent_origin = ctx.world.entity(ent).r.currentOrigin;
        let now = ctx.world.level.time;
        let e = ctx.world.entity_mut(ent);
        e.s.pos.trBase = ent_origin;
        e.s.pos.trTime = now;
    }

    let now = ctx.world.level.time;
    ctx.world.entity_mut(ent).nextthink = now + ROCKET_ALT_THINK_TIME;
    // Nothing at all spectacular happened, continue.
}

/// Raven `RocketDie`.
///
/// Source: `oracle/codemp/game/g_weapon.c:1814-1823`
pub fn RocketDie(
    ctx: &mut GameContext,
    self_: EntityId,
    inflictor: Option<EntityId>,
    attacker: Option<EntityId>,
    damage: c_int,
    r#mod: c_int,
) {
    {
        let e = ctx.world.entity_mut(self_);
        e.die = FnId::NONE;
        e.r.contents = 0;
    }

    G_ExplodeMissile(ctx, self_);

    let now = ctx.world.level.time;
    let e = ctx.world.entity_mut(self_);
    e.think = Some(EntThink::G_FreeEntity).into();
    e.nextthink = now;
}

/// Raven `WP_FireRocket`.
///
/// Source: `oracle/codemp/game/g_weapon.c:1826-1908`
pub fn WP_FireRocket(ctx: &mut GameContext, ent: EntityId, altFire: bool) {
    let damage: c_int = ROCKET_DAMAGE;
    let mut vel: f32 = ROCKET_VELOCITY as f32;
    let mut dif: c_int = 0;
    let mut rTime: f32;

    let muzzle = ctx.world.globals.muzzle;
    let forward = ctx.world.globals.forward;
    if altFire {
        vel *= 0.5;
    }

    let mid = CreateMissile(ctx, muzzle, forward, vel, 10000, ent, altFire);

    // FLAG: firing ent may be an NPC (pool client); deref the client value raw.
    let ent_client = ctx.world.entity(ent).client;
    if !ent_client.is_null()
        && unsafe { (*ent_client).ps.rocketLockIndex } != ENTITYNUM_NONE as c_int
    {
        let lockTimeInterval = (if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
            2400.0
        } else {
            1200.0
        }) / 16.0;
        rTime = unsafe { (*ent_client).ps.rocketLockTime } as f32;

        if rTime == -1.0 {
            rTime = unsafe { (*ent_client).ps.rocketLastValidTime } as f32;
        }
        dif = ((ctx.world.level.time as f32 - rTime) / lockTimeInterval) as c_int;

        if dif < 0 {
            dif = 0;
        }

        // It's 10 even though it locks client-side at 8, because we want them to
        // have a sturdy lock first, and because there's a slight difference in
        // time between server and client
        if dif >= 10 && rTime != -1.0 {
            let enemy_idx = unsafe { (*ent_client).ps.rocketLockIndex } as usize;
            let enemy_id = EntityId(enemy_idx as u32);
            ctx.world.entity_mut(mid).enemy = Some(enemy_id);

            // FLAG: enemy pool client deref stays raw.
            let enemy_client = ctx.world.entity(enemy_id).client;
            if !enemy_client.is_null()
                && ctx.world.entity(enemy_id).health > 0
                && OnSameTeam(ctx, Some(ent), Some(enemy_id)) == qfalse
            {
                // if enemy became invalid, died, or is on the same team, then don't seek it
                let now = ctx.world.level.time;
                let m = ctx.world.entity_mut(mid);
                m.angle = 0.5;
                m.think = Some(EntThink::rocketThink).into();
                m.nextthink = now + ROCKET_ALT_THINK_TIME;
            }
        }

        unsafe {
            (*ent_client).ps.rocketLockIndex = ENTITYNUM_NONE as c_int;
            (*ent_client).ps.rocketLockTime = (0) as f32;
            (*ent_client).ps.rocketTargetTime = (0) as f32;
        }
    }

    let m = ctx.world.entity_mut(mid);
    m.classname = c"rocket_proj".as_ptr() as *mut c_char;
    m.s.weapon = WP_ROCKET_LAUNCHER;

    // Make it easier to hit things
    m.r.maxs = [
        (ROCKET_SIZE) as f32,
        (ROCKET_SIZE) as f32,
        (ROCKET_SIZE) as f32,
    ];
    for i in 0..3 {
        m.r.mins[i] = -m.r.maxs[i];
    }

    m.damage = damage;
    m.dflags = DAMAGE_DEATH_KNOCKBACK;
    if altFire {
        m.methodOfDeath = MOD_ROCKET_HOMING as c_int;
        m.splashMethodOfDeath = MOD_ROCKET_HOMING_SPLASH as c_int;
    } else {
        m.methodOfDeath = MOD_ROCKET as c_int;
        m.splashMethodOfDeath = MOD_ROCKET_SPLASH as c_int;
    }
    //===testing being able to shoot rockets out of the air==================================
    m.health = 10;
    m.takedamage = qtrue;
    m.r.contents = MASK_SHOT;
    m.die = Some(EntDie::RocketDie).into();
    //===testing being able to shoot rockets out of the air==================================

    m.clipmask = MASK_SHOT;
    m.splashDamage = ROCKET_SPLASH_DAMAGE;
    m.splashRadius = ROCKET_SPLASH_RADIUS;

    // we don't want it to ever bounce
    m.bounceCount = 0;
}

/// Raven `thermalDetonatorExplode`.
///
/// Source: `oracle/codemp/game/g_weapon.c:1936-1970`
pub fn thermalDetonatorExplode(ctx: &mut GameContext, ent: EntityId) {
    if ctx.world.entity(ent).count == 0 {
        let snd = G_SoundIndex("sound/weapons/thermal/warning.wav");
        G_Sound(ctx, Some(ent), CHAN_WEAPON, snd);
        let now = ctx.world.level.time;
        let e = ctx.world.entity_mut(ent);
        e.count = 1;
        e.genericValue5 = now + 500;
        e.think = Some(EntThink::thermalThinkStandard).into();
        e.nextthink = now;
        e.r.svFlags |= SVF_BROADCAST; // so everyone hears/sees the explosion?
    } else {
        let dir: vec3_t = [0.0, 0.0, 1.0];
        let mut origin: vec3_t = [0.0; 3];

        let now = ctx.world.level.time;
        let pos = ctx.world.entity(ent).s.pos;
        mp_bg::bg_misc::BG_EvaluateTrajectory(&pos, now, &mut origin);
        origin[2] += 8.0;
        snap_vector(&mut origin);
        G_SetOrigin(ctx.world.entity_mut(ent), origin);

        ctx.world.entity_mut(ent).s.eType = (ET_GENERAL) as i32;
        let parm = DirToByte(dir);
        G_AddEvent(ctx.world.entity_mut(ent), EV_MISSILE_MISS as c_int, parm);
        ctx.world.entity_mut(ent).freeAfterEvent = qtrue;

        let parent_eid = ctx.world.entity(ent).parent;
        let currentOrigin = ctx.world.entity(ent).r.currentOrigin;
        let splashDamage = ctx.world.entity(ent).splashDamage;
        let splashRadius = ctx.world.entity(ent).splashRadius;
        let splashMOD = ctx.world.entity(ent).splashMethodOfDeath;
        if G_RadiusDamage(
            ctx,
            currentOrigin,
            parent_eid,
            splashDamage as f32,
            splashRadius as f32,
            Some(ent),
            Some(ent),
            splashMOD,
        ) {
            // FLAG: owner is arbitrary (r.ownerNum); pool client deref stays raw.
            let owner_id = EntityId(ctx.world.entity(ent).r.ownerNum as u32);
            let owner_client = ctx.world.entity(owner_id).client;
            unsafe {
                (*owner_client).accuracy_hits += 1;
            }
        }

        let ent_ptr = &mut ctx.world.g_entities[ent.index()] as *mut gentity_t;
        trap::LinkEntity(
            ctx.engine,
            mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(ent_ptr.cast()),
        );
    }
}

/// Raven `thermalThinkStandard`.
///
/// Source: `oracle/codemp/game/g_weapon.c:1972-1983`
pub fn thermalThinkStandard(ctx: &mut GameContext, ent: EntityId) {
    if ctx.world.entity(ent).genericValue5 < ctx.world.level.time {
        let now = ctx.world.level.time;
        let e = ctx.world.entity_mut(ent);
        e.think = Some(EntThink::thermalDetonatorExplode).into();
        e.nextthink = now;
        return;
    }
    G_RunObject(ctx, ent);
    let now = ctx.world.level.time;
    ctx.world.entity_mut(ent).nextthink = now;
}

/// Raven `WP_FireThermalDetonator`.
///
/// Source: `oracle/codemp/game/g_weapon.c:1986-2072`
pub fn WP_FireThermalDetonator(
    ctx: &mut GameContext,
    ent: EntityId,
    altFire: bool,
) -> *mut gentity_t {
    // Return stays raw `*mut gentity_t` (return conversion is a later pass).
    let dir: vec3_t = ctx.world.globals.forward;
    let mut start: vec3_t = ctx.world.globals.muzzle;
    let mut chargeAmount: f32 = 1.0; // default of full charge

    let bid = G_Spawn(ctx);
    let bolt = ctx.entity_mut(bid) as *mut gentity_t;
    let now = ctx.world.level.time;

    {
        let b = ctx.world.entity_mut(bid);
        b.physicsObject = qtrue;

        b.classname = c"thermal_detonator".as_ptr() as *mut c_char;
        b.think = Some(EntThink::thermalThinkStandard).into();
        b.nextthink = now;
        b.touch = Some(EntTouch::touch_NULL).into();

        // How 'bout we give this thing a size...
        b.r.mins = [-3.0, -3.0, -3.0];
        b.r.maxs = [3.0, 3.0, 3.0];
        b.clipmask = MASK_SHOT;
    }

    let bmins = ctx.world.entity(bid).r.mins;
    let bmaxs = ctx.world.entity(bid).r.maxs;
    start = W_TraceSetStart(ctx, ent, start, bmins, bmaxs); // make sure our start point isn't on the other side of a wall

    // FLAG: firing ent may be an NPC (pool client); deref the client value raw.
    let ent_client = ctx.world.entity(ent).client;
    if !ent_client.is_null() {
        chargeAmount = (now - unsafe { (*ent_client).ps.weaponChargeTime }) as f32;
    }

    // get charge amount
    chargeAmount /= TD_VELOCITY as f32;

    if chargeAmount > 1.0 {
        chargeAmount = 1.0;
    } else if chargeAmount < TD_MIN_CHARGE {
        chargeAmount = TD_MIN_CHARGE;
    }

    let ent_num = ctx.world.entity(ent).s.number;
    let ent_health = ctx.world.entity(ent).health;

    {
        let b = ctx.world.entity_mut(bid);
        // normal ones bounce, alt ones explode on impact
        b.genericValue5 = now + TD_TIME; // How long 'til she blows
        b.s.pos.trType = TR_GRAVITY;
        b.parent = Some(ent);
        b.r.ownerNum = (ent_num as u32) as i32;
        _VectorScale(dir, TD_VELOCITY as f32 * chargeAmount, &mut b.s.pos.trDelta);

        if ent_health >= 0 {
            b.s.pos.trDelta[2] += 120.0;
        }

        if !altFire {
            b.flags |= FL_BOUNCE_HALF;
        }

        b.s.loopSound = G_SoundIndex("sound/weapons/thermal/thermloop.wav");
        b.s.loopIsSoundset = qfalse;

        b.damage = TD_DAMAGE;
        b.dflags = 0;
        b.splashDamage = TD_SPLASH_DAM;
        b.splashRadius = TD_SPLASH_RAD;

        b.s.eType = (ET_MISSILE) as i32;
        b.r.svFlags = SVF_USE_CURRENT_ORIGIN;
        b.s.weapon = WP_THERMAL;

        b.methodOfDeath = MOD_THERMAL as c_int;
        b.splashMethodOfDeath = MOD_THERMAL_SPLASH as c_int;

        b.s.pos.trTime = now; // move a bit on the very first frame
        b.s.pos.trBase = start;

        snap_vector(&mut b.s.pos.trDelta); // save net bandwidth
        b.r.currentOrigin = start;

        b.pos2 = start;

        b.bounceCount = -5;
    }

    bolt
}

/// Raven `WP_DropThermal`.
///
/// Source: `oracle/codemp/game/g_weapon.c:2074-2078`
pub fn WP_DropThermal(ctx: &mut GameContext, ent: EntityId) -> *mut gentity_t {
    // Return stays raw `*mut gentity_t` (return conversion is a later pass).
    // FLAG: firing ent may be an NPC (pool client); read viewangles via raw deref.
    let ent_client = ctx.world.entity(ent).client;
    let viewangles = unsafe { (*ent_client).ps.viewangles };
    AngleVectors(
        viewangles,
        Some(&mut ctx.world.globals.forward),
        Some(&mut ctx.world.globals.vright),
        Some(&mut ctx.world.globals.up),
    );
    WP_FireThermalDetonator(ctx, ent, false)
}

/// Raven `WP_LobFire`.
///
/// Source: `oracle/codemp/game/g_weapon.c:2082-2226`
pub fn WP_LobFire(
    ctx: &mut GameContext,
    self_: EntityId,
    start: vec3_t,
    target: vec3_t,
    mins: vec3_t,
    maxs: vec3_t,
    clipmask: c_int,
    velocity: &mut vec3_t,
    tracePath: qboolean,
    ignoreEntNum: c_int,
    enemyNum: c_int,
    minSpeed: f32,
    maxSpeed: f32,
    idealSpeed: f32,
    mustHit: qboolean,
) -> qboolean {
    // for the galak mech NPC
    unsafe {
        let mut idealSpeed = idealSpeed;
        let mut minSpeed = minSpeed;
        let mut maxSpeed = maxSpeed;
        let speedInc: f32 = 100.0;
        let mut shotSpeed: f32;
        let mut bestImpactDist: f32 = Q3_INFINITE as f32;
        let mut targetDir: vec3_t = [0.0; 3];
        let mut shotVel: vec3_t = [0.0; 3];
        let mut failCase: vec3_t = [0.0; 3];
        let mut trace: trace_t = std::mem::zeroed();
        let mut tr: trajectory_t = std::mem::zeroed();
        let mut blocked: qboolean;
        let timeStep: c_int = 500;
        let mut hitCount: c_int = 0;
        let maxHits: c_int = 7;
        let mut lastPos: vec3_t;
        let mut testPos: vec3_t = [0.0; 3];

        if idealSpeed == 0.0 {
            idealSpeed = 300.0;
        } else if idealSpeed < speedInc {
            idealSpeed = speedInc;
        }
        shotSpeed = idealSpeed;
        let skipNum: c_int = ((idealSpeed - speedInc) / speedInc) as c_int;
        if minSpeed == 0.0 {
            minSpeed = 100.0;
        }
        if maxSpeed == 0.0 {
            maxSpeed = 900.0;
        }
        let _ = (minSpeed, maxSpeed); // Raven never reads these back after clamping shotSpeed via skipNum/speedInc

        while hitCount < maxHits {
            _VectorSubtract(target, start, &mut targetDir);
            let targetDist = VectorNormalize(&mut targetDir);

            _VectorScale(targetDir, shotSpeed, &mut shotVel);
            let mut travelTime = targetDist / shotSpeed;
            // C's `0.5` is a double literal, so the whole `travelTime * 0.5 *
            // g_gravity.value` product runs in f64 and narrows at the `+=`.
            shotVel[2] += (travelTime as f64 * 0.5 * ctx.world.cvars.g_gravity.value as f64) as f32;

            if hitCount == 0 {
                // save the first (ideal) one as the failCase (fallback value)
                if mustHit == qfalse {
                    // default is fine as a return value
                    failCase = shotVel;
                }
            }

            if tracePath != qfalse {
                // do a rough trace of the path
                blocked = qfalse;

                tr.trBase = start;
                tr.trDelta = shotVel;
                tr.trType = TR_GRAVITY;
                tr.trTime = ctx.world.level.time;
                travelTime *= 1000.0;
                lastPos = start;

                let mut elapsedTime: c_int = timeStep;
                while (elapsedTime as f32) < travelTime.floor() + timeStep as f32 {
                    if elapsedTime as f32 > travelTime {
                        // cap it
                        elapsedTime = travelTime.floor() as c_int;
                    }
                    mp_bg::bg_misc::BG_EvaluateTrajectory(
                        &tr,
                        ctx.world.level.time + elapsedTime,
                        &mut testPos,
                    );
                    trap::Trace(
                        ctx.engine,
                        mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                            &mut trace,
                            &lastPos as *const vec3_t,
                            &mins as *const vec3_t,
                            &maxs as *const vec3_t,
                            &testPos as *const vec3_t,
                            ignoreEntNum,
                            clipmask,
                        ),
                    );

                    if trace.allsolid != 0 || trace.startsolid != 0 {
                        blocked = qtrue;
                        break;
                    }
                    if trace.fraction < 1.0 {
                        // hit something
                        if trace.entityNum as c_int == enemyNum {
                            // hit the enemy, that's perfect!
                            break;
                        } else if trace.plane.normal[2] > 0.7
                            && DistanceSquared(trace.endpos, target) < 4096.0
                        {
                            // hit within 64 of desired location, should be okay
                            break;
                        } else {
                            // FIXME: maybe find the extents of this brush and go above or below it on next try somehow?
                            let impactDist = DistanceSquared(trace.endpos, target);
                            if impactDist < bestImpactDist {
                                bestImpactDist = impactDist;
                                failCase = shotVel;
                            }
                            blocked = qtrue;
                            // see if we should store this as the failCase
                            if (trace.entityNum as c_int) < ENTITYNUM_WORLD as c_int {
                                // hit an ent
                                let trace_id = EntityId(trace.entityNum as u32);
                                if ctx.world.entity(trace_id).takedamage != 0
                                    && OnSameTeam(ctx, Some(self_), Some(trace_id)) == qfalse
                                {
                                    // hit something breakable, so that's okay
                                    // we haven't found a clear shot yet so use this as the failcase
                                    failCase = shotVel;
                                }
                            }
                            break;
                        }
                    }
                    if elapsedTime == travelTime.floor() as c_int {
                        // reached end, all clear
                        break;
                    } else {
                        // all clear, try next slice
                        lastPos = testPos;
                    }
                    elapsedTime += timeStep;
                }
                if blocked != qfalse {
                    // hit something, adjust speed (which will change arc)
                    hitCount += 1;
                    shotSpeed = idealSpeed + ((hitCount - skipNum) as f32 * speedInc); // from min to max (skipping ideal)
                    if hitCount >= skipNum {
                        // skip ideal since that was the first value we tested
                        shotSpeed += speedInc;
                    }
                } else {
                    // made it!
                    break;
                }
            } else {
                // no need to check the path, go with first calc
                break;
            }
        }

        if hitCount >= maxHits {
            // NOTE: worst case scenario, use the one that impacted closest to the target (or just use the first try...?)
            *velocity = failCase;
            return qfalse;
        }
        *velocity = shotVel;
        qtrue
    }
}

/// Raven `laserTrapExplode`.
///
/// Source: `oracle/codemp/game/g_weapon.c:2244-2280`
pub fn laserTrapExplode(ctx: &mut GameContext, self_: EntityId) {
    let mut v: vec3_t;
    ctx.world.entity_mut(self_).takedamage = qfalse;

    if let Some(activator_id) = ctx.world.entity(self_).activator {
        let currentOrigin = ctx.world.entity(self_).r.currentOrigin;
        let splashDamage = ctx.world.entity(self_).splashDamage;
        let splashRadius = ctx.world.entity(self_).splashRadius;
        G_RadiusDamage(
            ctx,
            currentOrigin,
            Some(activator_id),
            splashDamage as f32,
            splashRadius as f32,
            Some(self_),
            Some(self_),
            MOD_TRIP_MINE_SPLASH as c_int, /* MOD_LT_SPLASH */
        );
    }

    if ctx.world.entity(self_).s.weapon != WP_FLECHETTE {
        G_AddEvent(ctx.world.entity_mut(self_), EV_MISSILE_MISS as c_int, 0);
    }

    v = ctx.world.entity(self_).s.pos.trDelta;
    // Explode outward from the surface

    if ctx.world.entity(self_).s.time == -2 {
        v = [0.0, 0.0, 0.0];
    }

    let currentOrigin = ctx.world.entity(self_).r.currentOrigin;
    if ctx.world.entity(self_).s.weapon == WP_FLECHETTE {
        G_PlayEffect((EFFECT_EXPLOSION_FLECHETTE) as i32, currentOrigin, v);
    } else {
        G_PlayEffect((EFFECT_EXPLOSION_TRIPMINE) as i32, currentOrigin, v);
    }

    let now = ctx.world.level.time;
    let e = ctx.world.entity_mut(self_);
    e.think = Some(EntThink::G_FreeEntity).into();
    e.nextthink = now;
}

/// Raven `laserTrapDelayedExplode`.
///
/// Source: `oracle/codemp/game/g_weapon.c:2282-2294`
pub fn laserTrapDelayedExplode(
    ctx: &mut GameContext,
    self_: EntityId,
    inflictor: Option<EntityId>,
    attacker: Option<EntityId>,
    damage: c_int,
    meansOfDeath: c_int,
) {
    // Raven's `self->enemy = attacker` (the prior port wrapped a laundered
    // pointer; `attacker` already carries the nullable handle).
    let now = ctx.world.level.time;
    {
        let e = ctx.world.entity_mut(self_);
        e.enemy = attacker;
        e.think = Some(EntThink::laserTrapExplode).into();
        e.nextthink = now + FRAMETIME;
        e.takedamage = qfalse;
    }
    if let Some(attacker_id) = attacker {
        if ctx.world.entity(attacker_id).s.number == 0 {
            // less damage when shot by player
            let e = ctx.world.entity_mut(self_);
            e.splashDamage /= 3;
            e.splashRadius /= 3;
        }
    }
}

// Laser-trap consts (`touchLaserTrap`/`CreateLaserTrap`/`WP_PlaceLaserTrap`).
// Source: `oracle/codemp/game/g_weapon.c:2235-2242`
const LT_DAMAGE: c_int = 100;
const LT_SPLASH_RAD: c_int = 256; // Raven's `256.0f` assigned straight into the int `splashRadius` field.
const LT_SPLASH_DAM: c_int = 105;
const LT_SIZE: f32 = 1.5;
const LT_ALT_TIME: c_int = 2000;
const LT_ACTIVATION_DELAY: c_int = 1000;
const LT_DELAY_TIME: c_int = 50;

/// Raven `touchLaserTrap`.
///
/// Source: `oracle/codemp/game/g_weapon.c:2296-2318`
pub fn touchLaserTrap(
    ctx: &mut GameContext,
    ent: EntityId,
    other: Option<EntityId>,
    trace: *mut trace_t,
) {
    let hit_ent = match other {
        Some(o) => (ctx.world.entity(o).s.number as u32) < (ENTITYNUM_WORLD) as u32,
        None => false,
    };
    if hit_ent {
        let other_id = other.unwrap();
        // just explode if we hit any entity. This way we don't have things
        // happening like tripmines floating in the air after getting stuck
        // to a moving door
        if ctx.world.entity(ent).activator != Some(other_id) {
            // `trace` is the engine-provided raw out-param; its derefs stay raw.
            let normal = unsafe { (*trace).plane.normal };
            let now = ctx.world.level.time;
            let e = ctx.world.entity_mut(ent);
            e.touch = FnId::NONE;
            e.nextthink = now + FRAMETIME;
            e.think = Some(EntThink::laserTrapExplode).into();
            e.s.pos.trDelta = normal;
        }
    } else {
        ctx.world.entity_mut(ent).touch = FnId::NONE;
        let trace_entnum = unsafe { (*trace).entityNum };
        if trace_entnum != ENTITYNUM_NONE as i16 {
            ctx.world.entity_mut(ent).enemy = Some(EntityId(trace_entnum as u32));
        }
        let endpos = unsafe { (*trace).endpos };
        let normal = unsafe { (*trace).plane.normal };
        laserTrapStick(ctx, ent, endpos, normal);
    }
}

/// Raven `proxMineThink`.
///
/// Source: `oracle/codemp/game/g_weapon.c:2320-2365`
pub fn proxMineThink(ctx: &mut GameContext, ent: EntityId) {
    let mut owner: Option<EntityId> = None;

    if (ctx.world.entity(ent).r.ownerNum as u32) < (ENTITYNUM_WORLD) as u32 {
        owner = Some(EntityId(ctx.world.entity(ent).r.ownerNum as u32));
    }

    let now = ctx.world.level.time;
    ctx.world.entity_mut(ent).nextthink = now;

    // FLAG: owner pool client deref stays raw (read the client pointer value).
    let owner_client = match owner {
        Some(o) => ctx.world.entity(o).client,
        None => std::ptr::null_mut(),
    };
    if ctx.world.entity(ent).genericValue15 < now
        || owner.is_none()
        || ctx.world.entity(owner.unwrap()).inuse == qfalse
        || owner_client.is_null()
        || unsafe { (*owner_client).pers.connected } != CON_CONNECTED
    {
        // time to die!
        ctx.world.entity_mut(ent).think = Some(EntThink::laserTrapExplode).into();
        return;
    }

    let mut i: c_int = 0;
    while i < MAX_CLIENTS as c_int {
        // eh, just check for clients, don't care about anyone else...
        let cl_id = EntityId(i as u32);
        // FLAG: client-slot loop (i < MAX_CLIENTS); deref the entity's client pointer raw.
        let cl_client = ctx.world.entity(cl_id).client;

        if ctx.world.entity(cl_id).inuse != qfalse
            && !cl_client.is_null()
            && unsafe { (*cl_client).pers.connected } == CON_CONNECTED
            && owner != Some(cl_id)
            && unsafe { (*cl_client).sess.sessionTeam } != TEAM_SPECTATOR
            && unsafe { (*cl_client).tempSpectate } < now
            && ctx.world.entity(cl_id).health > 0
        {
            if OnSameTeam(ctx, owner, Some(cl_id)) == qfalse
                || ctx.world.cvars.g_friendlyFire.integer != 0
            {
                // not on the same team, or friendly fire is enabled
                let mut v: vec3_t = [0.0; 3];
                let ent_origin = ctx.world.entity(ent).r.currentOrigin;
                let cl_ps_origin = unsafe { (*cl_client).ps.origin };
                _VectorSubtract(ent_origin, cl_ps_origin, &mut v);
                let splashRadius = ctx.world.entity(ent).splashRadius;
                if VectorLength(v) < splashRadius as f32 / 2.0f32 {
                    ctx.world.entity_mut(ent).think = Some(EntThink::laserTrapExplode).into();
                    return;
                }
            }
        }
        i += 1;
    }
}

/// Raven `laserTrapThink`.
///
/// Source: `oracle/codemp/game/g_weapon.c:2367-2400`
pub fn laserTrapThink(ctx: &mut GameContext, ent: EntityId) {
    // just relink it every think
    let ent_ptr = &mut ctx.world.g_entities[ent.index()] as *mut gentity_t;
    trap::LinkEntity(
        ctx.engine,
        mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(ent_ptr.cast()),
    );

    // turn on the beam effect
    if ctx.world.entity(ent).s.eFlags & EF_FIRING == 0 {
        // arm me
        let snd = G_SoundIndex("sound/weapons/laser_trap/warning.wav");
        G_Sound(ctx, Some(ent), CHAN_WEAPON, snd);
        ctx.world.entity_mut(ent).s.eFlags |= EF_FIRING;
    }
    let now = ctx.world.level.time;
    {
        let e = ctx.world.entity_mut(ent);
        e.think = Some(EntThink::laserTrapThink).into();
        e.nextthink = now + FRAMETIME;
    }

    // Find the main impact point
    let mut end: vec3_t = [0.0; 3];
    let trBase = ctx.world.entity(ent).s.pos.trBase;
    let movedir = ctx.world.entity(ent).movedir;
    _VectorMA(trBase, 1024.0, movedir, &mut end);
    let mut tr: trace_t = unsafe { std::mem::zeroed() };
    let currentOrigin = ctx.world.entity(ent).r.currentOrigin;
    let ent_num = ctx.world.entity(ent).s.number;
    trap::Trace(
        ctx.engine,
        mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
            &mut tr,
            &currentOrigin as *const vec3_t,
            std::ptr::null(),
            std::ptr::null(),
            &end as *const vec3_t,
            ent_num,
            MASK_SHOT,
        ),
    );

    let trace_id = EntityId(tr.entityNum as u32);

    ctx.world.entity_mut(ent).s.time = -1; // let all clients know to draw a beam from this guy

    // `client` is only null-checked here (no deref).
    let trace_client = ctx.world.entity(trace_id).client;
    if !trace_client.is_null() || tr.startsolid != 0 {
        // go boom
        let now = ctx.world.level.time;
        let e = ctx.world.entity_mut(ent);
        e.touch = FnId::NONE;
        e.nextthink = now + LT_DELAY_TIME;
        e.think = Some(EntThink::laserTrapExplode).into();
    }
}

/// Raven `laserTrapStick`.
///
/// Source: `oracle/codemp/game/g_weapon.c:2402-2469`
pub fn laserTrapStick(ctx: &mut GameContext, ent: EntityId, endpos: vec3_t, normal: vec3_t) {
    G_SetOrigin(ctx.world.entity_mut(ent), endpos);

    let now = ctx.world.level.time;
    // This does nothing, cg_missile makes assumptions about direction of travel controlling angles
    let mut apos_base: vec3_t = [0.0; 3];
    vectoangles(normal, &mut apos_base);
    {
        let e = ctx.world.entity_mut(ent);
        e.pos1 = normal;

        e.s.apos.trDelta = [0.0, 0.0, 0.0];
        // This will orient the object to face in the direction of the normal
        e.s.pos.trDelta = normal;
        e.s.pos.trTime = now;

        e.s.apos.trBase = apos_base;
        e.s.apos.trDelta = [0.0, 0.0, 0.0];
        e.s.apos.trType = TR_STATIONARY;
        e.s.angles = e.s.apos.trBase;
        e.r.currentAngles = e.s.angles;
    }

    let snd = G_SoundIndex("sound/weapons/laser_trap/stick.wav");
    G_Sound(ctx, Some(ent), CHAN_WEAPON, snd);
    if ctx.world.entity(ent).count != 0 {
        // a tripwire
        // add draw line flag
        let e = ctx.world.entity_mut(ent);
        e.movedir = normal;
        e.think = Some(EntThink::laserTrapThink).into();
        e.nextthink = now + LT_ACTIVATION_DELAY; // delay the activation
        e.touch = Some(EntTouch::touch_NULL).into();
        // make it shootable
        e.takedamage = qtrue;
        e.health = 5;
        e.die = Some(EntDie::laserTrapDelayedExplode).into();

        // shove the box through the wall
        e.r.mins = [-LT_SIZE * 2.0, -LT_SIZE * 2.0, -LT_SIZE * 2.0];
        e.r.maxs = [LT_SIZE * 2.0, LT_SIZE * 2.0, LT_SIZE * 2.0];

        // so that the owner can blow it up with projectiles
        e.r.svFlags |= SVF_OWNERNOTSHARED;
    } else {
        {
            let e = ctx.world.entity_mut(ent);
            e.touch = Some(EntTouch::touchLaserTrap).into();
            e.think = Some(EntThink::proxMineThink).into(); // laserTrapExplode
            e.genericValue15 = now + 30000; // auto-explode after 30 seconds.
            e.nextthink = now + LT_ALT_TIME; // How long 'til she blows

            // make it shootable
            e.takedamage = qtrue;
            e.health = 5;
            e.die = Some(EntDie::laserTrapDelayedExplode).into();

            // shove the box through the wall
            e.r.mins = [-LT_SIZE * 2.0, -LT_SIZE * 2.0, -LT_SIZE * 2.0];
            e.r.maxs = [LT_SIZE * 2.0, LT_SIZE * 2.0, LT_SIZE * 2.0];

            // so that the owner can blow it up with projectiles
            e.r.svFlags |= SVF_OWNERNOTSHARED;
        }

        if ctx.world.entity(ent).s.eFlags & EF_FIRING == 0 {
            // arm me
            let snd = G_SoundIndex("sound/weapons/laser_trap/warning.wav");
            G_Sound(ctx, Some(ent), CHAN_WEAPON, snd);
            let e = ctx.world.entity_mut(ent);
            e.s.eFlags |= EF_FIRING;
            e.s.time = -1;
            e.s.bolt2 = 1;
        }
    }
}

/// Raven `TrapThink`.
///
/// Raven: "laser trap think".
/// Source: `oracle/codemp/game/g_weapon.c:2471-2475`
pub fn TrapThink(ctx: &mut GameContext, ent: EntityId) {
    // laser trap think
    let now = ctx.world.level.time;
    ctx.world.entity_mut(ent).nextthink = now + 50;
    G_RunObject(ctx, ent);
}

/// Raven `CreateLaserTrap`.
///
/// Source: `oracle/codemp/game/g_weapon.c:2477-2531`
pub fn CreateLaserTrap(ctx: &mut GameContext, laserTrap: EntityId, start: vec3_t, owner: EntityId) {
    // create a laser trap entity
    let owner_num = ctx.world.entity(owner).s.number;
    let now = ctx.world.level.time;
    let modelidx = G_ModelIndex("models/weapons2/laser_trap/laser_trap_w.glm");
    {
        let lt = ctx.world.entity_mut(laserTrap);
        lt.classname = c"laserTrap".as_ptr() as *mut c_char;
        lt.flags |= FL_BOUNCE_HALF;
        lt.s.eFlags |= EF_MISSILE_STICK;
        lt.splashDamage = LT_SPLASH_DAM;
        lt.splashRadius = LT_SPLASH_RAD;
        lt.damage = LT_DAMAGE;
        lt.methodOfDeath = MOD_TRIP_MINE_SPLASH as c_int;
        lt.splashMethodOfDeath = MOD_TRIP_MINE_SPLASH as c_int;
        lt.s.eType = (ET_GENERAL) as i32;
        lt.r.svFlags = SVF_USE_CURRENT_ORIGIN;
        lt.s.weapon = WP_TRIP_MINE;
        lt.s.pos.trType = TR_GRAVITY;
        lt.r.contents = MASK_SHOT;
        lt.parent = Some(owner);
        lt.activator = Some(owner);
        lt.r.ownerNum = (owner_num as u32) as i32;
        lt.r.mins = [-LT_SIZE, -LT_SIZE, -LT_SIZE];
        lt.r.maxs = [LT_SIZE, LT_SIZE, LT_SIZE];
        lt.clipmask = MASK_SHOT;
        lt.s.solid = 2;
        lt.s.modelindex = modelidx;
        lt.s.modelGhoul2 = 1;
        lt.s.g2radius = 40;

        lt.s.genericenemyindex = owner_num + MAX_GENTITIES as c_int;

        lt.health = 1;

        lt.s.time = 0;

        lt.s.pos.trTime = now; // move a bit on the very first frame
        lt.s.pos.trBase = start;
        snap_vector(&mut lt.s.pos.trBase); // save net bandwidth

        snap_vector(&mut lt.s.pos.trDelta); // save net bandwidth
        lt.r.currentOrigin = start;

        lt.s.apos.trType = TR_GRAVITY;
        lt.s.apos.trTime = now;
    }

    let yaw = (ctx.world.bg_state.rng.rand() % 360) as f32;
    let pitch = (ctx.world.bg_state.rng.rand() % 360) as f32;
    let roll = (ctx.world.bg_state.rng.rand() % 360) as f32;
    let flip = ctx.world.bg_state.rng.rand() % 10 < 5;
    let final_yaw = if flip { -yaw } else { yaw };

    {
        let lt = ctx.world.entity_mut(laserTrap);
        lt.s.apos.trBase[YAW] = final_yaw;
        lt.s.apos.trBase[PITCH] = pitch;
        lt.s.apos.trBase[ROLL] = roll;

        lt.pos2 = start;
        lt.touch = Some(EntTouch::touchLaserTrap).into();
        lt.think = Some(EntThink::TrapThink).into();
        lt.nextthink = now + 50;
    }
}

/// Raven `WP_PlaceLaserTrap`.
///
/// Source: `oracle/codemp/game/g_weapon.c:2533-2626`
pub fn WP_PlaceLaserTrap(ctx: &mut GameContext, ent: EntityId, alt_fire: bool) {
    let dir: vec3_t = ctx.world.globals.forward;
    let start: vec3_t = ctx.world.globals.muzzle;
    // `FOFS(classname)` — byte offset of `gentity_t::classname` (Raven macro, `g_local.h`).
    let fofs_classname = core::mem::offset_of!(gentity_t, classname) as c_int;

    let mut foundLaserTraps: [c_int; MAX_GENTITIES] = [ENTITYNUM_NONE as c_int; MAX_GENTITIES];
    let mut trapcount: c_int = 0;

    let laserTrap_id = G_Spawn(ctx);

    // limit to 10 placed at any one time
    // see how many there are now
    let mut found: *mut gentity_t = std::ptr::null_mut();
    loop {
        found = G_Find(
            ctx,
            ctx.entity_id_of(found),
            fofs_classname,
            c"laserTrap".as_ptr(),
        );
        if found.is_null() {
            break;
        }
        let found_id = ctx.entity_id_of(found).unwrap();
        if ctx.world.entity(found_id).parent != Some(ent) {
            continue;
        }
        foundLaserTraps[trapcount as usize] = ctx.world.entity(found_id).s.number;
        trapcount += 1;
    }
    // now remove first ones we find until there are only 9 left
    let trapcount_org = trapcount;
    let mut lowestTimeStamp = ctx.world.level.time;
    while trapcount > 9 {
        let mut removeMe: c_int = -1;
        for i in 0..trapcount_org {
            if foundLaserTraps[i as usize] == ENTITYNUM_NONE as c_int {
                continue;
            }
            let found_id = EntityId(foundLaserTraps[i as usize] as u32);
            // `laserTrap` (the newly spawned entity) is always valid.
            if ctx.world.entity(found_id).setTime < lowestTimeStamp {
                removeMe = i;
                lowestTimeStamp = ctx.world.entity(found_id).setTime;
            }
        }
        if removeMe != -1 {
            // remove it... or blow it?
            let victim_id = EntityId(foundLaserTraps[removeMe as usize] as u32);
            G_FreeEntity(ctx, Some(victim_id));
            foundLaserTraps[removeMe as usize] = ENTITYNUM_NONE as c_int;
            trapcount -= 1;
        } else {
            break;
        }
    }

    // now make the new one
    CreateLaserTrap(ctx, laserTrap_id, start, ent);

    // set player-created-specific fields
    let now = ctx.world.level.time;
    {
        let lt = ctx.world.entity_mut(laserTrap_id);
        lt.setTime = now; // remember when we placed it

        if !alt_fire {
            // tripwire
            lt.count = 1;
        }

        // move it
        lt.s.pos.trType = TR_GRAVITY;

        if alt_fire {
            _VectorScale(dir, 512.0, &mut lt.s.pos.trDelta);
        } else {
            _VectorScale(dir, 256.0, &mut lt.s.pos.trDelta);
        }
    }

    let lt_ptr = &mut ctx.world.g_entities[laserTrap_id.index()] as *mut gentity_t;
    trap::LinkEntity(
        ctx.engine,
        mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(lt_ptr.cast()),
    );
}

/// Raven `charge_stick`.
///
/// Source: `oracle/codemp/game/g_weapon.c:2645-2738`
pub fn charge_stick(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    trace: *mut trace_t,
) {
    // Precompute the `other`-entity branch predicates (pure reads; `other == None`
    // makes them all false, matching Raven's leading `other != NULL` guards).
    let (b1, b2_ent, b3, num_lt_world) = match other {
        None => (false, false, false, false),
        Some(o) => {
            let oe = ctx.world.entity(o);
            let num_lt = (oe.s.number as u32) < (ENTITYNUM_WORLD) as u32;
            let b1 = (oe.flags & FL_BBRUSH) != 0
                && oe.s.pos.trType == TR_STATIONARY
                && oe.s.apos.trType == TR_STATIONARY;
            let b2_ent = num_lt && oe.s.eType == (ET_MOVER) as i32;
            let b3 = num_lt && (!oe.client.is_null() || oe.s.weapon == 0);
            (b1, b2_ent, b3, num_lt)
        }
    };

    if b1 {
        // a perfectly still breakable brush, let us attach directly to it!
        ctx.world.entity_mut(self_).target_ent = other;
    } else if b2_ent && unsafe { (*trace).plane.normal[2] } > 0.0 {
        // stick to it?
        let onum = ctx.world.entity(other.unwrap()).s.number;
        ctx.world.entity_mut(self_).s.groundEntityNum = onum;
    } else if b3 {
        // hit another entity that is not stickable, "bounce" off
        let mut vNor: vec3_t = unsafe { (*trace).plane.normal };
        VectorNormalize(&mut vNor);
        let td = ctx.world.entity(self_).s.pos.trDelta;
        let mut tN = [0.0f32; 3];
        VectorNPos(td, &mut tN);
        // C: `vNor[i]*(tN[i]*(((float)Q_irand(1,10))*0.1))` — the bare `0.1`
        // (double) runs the whole product chain in f64, narrowed once at the
        // `+=` store. The `vNor[1]` on the [2] component is a faithful oracle bug.
        // Source: `oracle/codemp/game/g_weapon.c:2671-2673`
        let r0 = ctx.world.bg_state.rng.Q_irand(1, 10);
        let r1 = ctx.world.bg_state.rng.Q_irand(1, 10);
        let r2 = ctx.world.bg_state.rng.Q_irand(1, 10);
        let new0 = (td[0] as f64 + vNor[0] as f64 * (tN[0] as f64 * (r0 as f64 * 0.1))) as f32;
        let new1 = (td[1] as f64 + vNor[1] as f64 * (tN[1] as f64 * (r1 as f64 * 0.1))) as f32;
        let new2 = (td[2] as f64 + vNor[1] as f64 * (tN[2] as f64 * (r2 as f64 * 0.1))) as f32;
        let mut sangles: vec3_t = [0.0; 3];
        vectoangles(vNor, &mut sangles);
        let mut apos_base: vec3_t = [0.0; 3];
        vectoangles(vNor, &mut apos_base);

        let e = ctx.world.entity_mut(self_);
        e.s.pos.trDelta[0] = new0;
        e.s.pos.trDelta[1] = new1;
        e.s.pos.trDelta[2] = new2;
        e.s.angles = sangles;
        e.s.apos.trBase = apos_base;
        e.touch = Some(EntTouch::charge_stick).into();
        return;
    } else if num_lt_world {
        // hit an entity that we just want to explode on (probably another projectile or something)
        {
            let e = ctx.world.entity_mut(self_);
            e.touch = FnId::NONE;
            e.think = FnId::NONE;
            e.nextthink = 0;

            e.takedamage = qfalse;

            e.s.apos.trDelta = [0.0, 0.0, 0.0];
            e.s.apos.trType = TR_STATIONARY;
        }

        let parent_eid = ctx.world.entity(self_).parent;
        let currentOrigin = ctx.world.entity(self_).r.currentOrigin;
        let splashDamage = ctx.world.entity(self_).splashDamage;
        let splashRadius = ctx.world.entity(self_).splashRadius;
        G_RadiusDamage(
            ctx,
            currentOrigin,
            parent_eid,
            splashDamage as f32,
            splashRadius as f32,
            Some(self_),
            Some(self_),
            MOD_DET_PACK_SPLASH as c_int,
        );
        let v: vec3_t = unsafe { (*trace).plane.normal };
        let currentOrigin = ctx.world.entity(self_).r.currentOrigin;
        {
            let e = ctx.world.entity_mut(self_);
            e.pos2 = v;
            e.count = -1;
        }
        G_PlayEffect((EFFECT_EXPLOSION_DETPACK) as i32, currentOrigin, v);

        let now = ctx.world.level.time;
        let e = ctx.world.entity_mut(self_);
        e.think = Some(EntThink::G_FreeEntity).into();
        e.nextthink = now;
        return;
    }

    // if we get here I guess we hit the world so we can stick to it

    let now = ctx.world.level.time;
    {
        let e = ctx.world.entity_mut(self_);
        e.touch = FnId::NONE;
        e.think = Some(EntThink::DetPackBlow).into();
        e.nextthink = now + 30000;

        e.s.apos.trDelta = [0.0, 0.0, 0.0];
        e.s.apos.trType = TR_STATIONARY;

        e.s.pos.trType = TR_STATIONARY;
        e.s.origin = e.r.currentOrigin;
        e.s.pos.trBase = e.r.currentOrigin;
        e.s.pos.trDelta = [0.0, 0.0, 0.0];

        e.s.apos.trDelta = [0.0, 0.0, 0.0];
    }

    // `trace` is the engine-provided raw out-param; normalize in place as Raven does.
    unsafe { VectorNormalize(&mut (*trace).plane.normal) };

    let tnorm = unsafe { (*trace).plane.normal };
    let mut angles: vec3_t = [0.0; 3];
    vectoangles(tnorm, &mut angles);
    {
        let e = ctx.world.entity_mut(self_);
        e.s.angles = angles;
        e.r.currentAngles = e.s.angles;
        e.s.apos.trBase = e.s.angles;

        e.pos2 = tnorm;
        e.count = -1;
    }

    let snd = G_SoundIndex("sound/weapons/detpack/stick.wav");
    G_Sound(ctx, Some(self_), CHAN_WEAPON, snd);

    let currentOrigin = ctx.world.entity(self_).r.currentOrigin;
    let self_num = ctx.world.entity(self_).s.number;
    let tent_id = G_TempEntity(ctx, currentOrigin, EV_MISSILE_MISS as c_int);
    {
        let t = ctx.world.entity_mut(tent_id);
        t.s.weapon = 0;
        t.parent = Some(self_);
        t.r.ownerNum = (self_num as u32) as i32;
    }

    // so that the owner can blow it up with projectiles
    ctx.world.entity_mut(self_).r.svFlags |= SVF_OWNERNOTSHARED;
}

/// Raven `DetPackBlow`.
///
/// Source: `oracle/codemp/game/g_weapon.c:2740-2766`
pub fn DetPackBlow(ctx: &mut GameContext, self_: EntityId) {
    let mut v: vec3_t;

    {
        let e = ctx.world.entity_mut(self_);
        e.pain = FnId::NONE;
        e.die = FnId::NONE;
        e.takedamage = qfalse;
    }

    if let Some(target_id) = ctx.world.entity(self_).target_ent {
        // we were attached to something, do *direct* damage to it!
        let owner_id = EntityId(ctx.world.entity(self_).r.ownerNum as u32);
        let currentOrigin = ctx.world.entity(self_).r.currentOrigin;
        let damage = ctx.world.entity(self_).damage;
        G_Damage(
            ctx,
            Some(target_id),
            Some(self_),
            Some(owner_id),
            // §19: C passes an UNINITIALIZED `vec3_t v` as G_Damage's dir here;
            // that read is UB — we pass a defined zero vector instead.
            Some(&mut [0.0_f32, 0.0, 0.0]),
            currentOrigin,
            damage,
            0,
            MOD_DET_PACK_SPLASH as c_int,
        );
    }
    let parent_eid = ctx.world.entity(self_).parent;
    let currentOrigin = ctx.world.entity(self_).r.currentOrigin;
    let splashDamage = ctx.world.entity(self_).splashDamage;
    let splashRadius = ctx.world.entity(self_).splashRadius;
    G_RadiusDamage(
        ctx,
        currentOrigin,
        parent_eid,
        splashDamage as f32,
        splashRadius as f32,
        Some(self_),
        Some(self_),
        MOD_DET_PACK_SPLASH as c_int,
    );
    v = [0.0, 0.0, 1.0];

    if ctx.world.entity(self_).count == -1 {
        v = ctx.world.entity(self_).pos2;
    }

    let currentOrigin = ctx.world.entity(self_).r.currentOrigin;
    G_PlayEffect((EFFECT_EXPLOSION_DETPACK) as i32, currentOrigin, v);

    let now = ctx.world.level.time;
    let e = ctx.world.entity_mut(self_);
    e.think = Some(EntThink::G_FreeEntity).into();
    e.nextthink = now;
}

/// Raven `DetPackPain`.
///
/// Source: `oracle/codemp/game/g_weapon.c:2768-2773`
pub fn DetPackPain(
    ctx: &mut GameContext,
    self_: EntityId,
    attacker: Option<EntityId>,
    damage: c_int,
) {
    let now = ctx.world.level.time;
    let delay = ctx.world.bg_state.rng.Q_irand(50, 100);
    let e = ctx.world.entity_mut(self_);
    e.think = Some(EntThink::DetPackBlow).into();
    e.nextthink = now + delay;
    e.takedamage = qfalse;
}

/// Raven `DetPackDie`.
///
/// Source: `oracle/codemp/game/g_weapon.c:2775-2780`
pub fn DetPackDie(
    ctx: &mut GameContext,
    self_: EntityId,
    inflictor: Option<EntityId>,
    attacker: Option<EntityId>,
    damage: c_int,
    r#mod: c_int,
) {
    let now = ctx.world.level.time;
    let delay = ctx.world.bg_state.rng.Q_irand(50, 100);
    let e = ctx.world.entity_mut(self_);
    e.think = Some(EntThink::DetPackBlow).into();
    e.nextthink = now + delay;
    e.takedamage = qfalse;
}

/// Raven `drop_charge`.
///
/// Source: `oracle/codemp/game/g_weapon.c:2782-2849`
pub fn drop_charge(ctx: &mut GameContext, self_: EntityId, start: vec3_t, dir: vec3_t) {
    let mut dir = dir;
    VectorNormalize(&mut dir);

    let bid = G_Spawn(ctx);
    let self_num = ctx.world.entity(self_).s.number;
    let now = ctx.world.level.time;
    let modelidx = G_ModelIndex("models/weapons2/detpack/det_pack_proj.glm");
    let mut angles: vec3_t = [0.0; 3];
    vectoangles(dir, &mut angles);
    {
        let b = ctx.world.entity_mut(bid);
        b.classname = c"detpack".as_ptr() as *mut c_char;
        b.nextthink = now + FRAMETIME;
        b.think = Some(EntThink::G_RunObject).into();
        b.s.eType = (ET_GENERAL) as i32;
        b.s.g2radius = 100;
        b.s.modelGhoul2 = 1;
        b.s.modelindex = modelidx;

        b.parent = Some(self_);
        b.r.ownerNum = (self_num as u32) as i32;
        b.damage = 100;
        b.splashDamage = 200;
        b.splashRadius = (200.0) as i32;
        b.methodOfDeath = MOD_DET_PACK_SPLASH as c_int;
        b.splashMethodOfDeath = MOD_DET_PACK_SPLASH as c_int;
        b.clipmask = MASK_SHOT;
        b.s.solid = 2;
        b.r.contents = MASK_SHOT;
        b.touch = Some(EntTouch::charge_stick).into();

        b.physicsObject = qtrue;

        b.s.genericenemyindex = self_num + MAX_GENTITIES as c_int;
        // rww - so client prediction knows we own this and won't hit it

        b.r.mins = [-2.0, -2.0, -2.0];
        b.r.maxs = [2.0, 2.0, 2.0];

        b.health = 1;
        b.takedamage = qtrue;
        b.pain = Some(EntPain::DetPackPain).into();
        b.die = Some(EntDie::DetPackDie).into();

        b.s.weapon = WP_DET_PACK;

        b.setTime = now;
    }

    G_SetOrigin(ctx.world.entity_mut(bid), start);
    {
        let b = ctx.world.entity_mut(bid);
        b.s.pos.trType = TR_GRAVITY;
        b.s.pos.trBase = start;
        _VectorScale(dir, 300.0, &mut b.s.pos.trDelta);
        b.s.pos.trTime = now;

        b.s.apos.trType = TR_GRAVITY;
        b.s.apos.trTime = now;
    }

    let yaw = (ctx.world.bg_state.rng.rand() % 360) as f32;
    let pitch = (ctx.world.bg_state.rng.rand() % 360) as f32;
    let roll = (ctx.world.bg_state.rng.rand() % 360) as f32;
    let flip = ctx.world.bg_state.rng.rand() % 10 < 5;
    let final_yaw = if flip { -yaw } else { yaw };

    {
        let b = ctx.world.entity_mut(bid);
        b.s.apos.trBase[YAW] = final_yaw;
        b.s.apos.trBase[PITCH] = pitch;
        b.s.apos.trBase[ROLL] = roll;

        b.s.angles = angles;
        b.s.apos.trBase = b.s.angles;
        b.s.apos.trDelta = [300.0, 0.0, 0.0];
        b.s.apos.trTime = now;
    }

    let b_ptr = &mut ctx.world.g_entities[bid.index()] as *mut gentity_t;
    trap::LinkEntity(
        ctx.engine,
        mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(b_ptr.cast()),
    );
}

/// Raven `BlowDetpacks`.
///
/// Source: `oracle/codemp/game/g_weapon.c:2851-2869`
pub fn BlowDetpacks(ctx: &mut GameContext, ent: EntityId) {
    let fofs_classname = core::mem::offset_of!(gentity_t, classname) as c_int;
    // FLAG: firing ent may be an NPC (pool client); deref the client value raw.
    let ent_client = ctx.world.entity(ent).client;
    if unsafe { (*ent_client).ps.hasDetPackPlanted } != qfalse {
        let mut found: *mut gentity_t = std::ptr::null_mut();
        loop {
            found = G_Find(
                ctx,
                ctx.entity_id_of(found),
                fofs_classname,
                c"detpack".as_ptr(),
            );
            if found.is_null() {
                break;
            }
            // loop through all ents and blow the crap out of them!
            let found_id = ctx.entity_id_of(found).unwrap();
            if ctx.world.entity(found_id).parent == Some(ent) {
                let currentOrigin = ctx.world.entity(found_id).r.currentOrigin;
                {
                    let f = ctx.world.entity_mut(found_id);
                    f.s.origin = currentOrigin;
                    f.think = Some(EntThink::DetPackBlow).into();
                }
                // C: `level.time + 100 + random()*200` — the int sum promotes
                // to f32 against `random()*200`, truncating once at the store.
                // Source: `oracle/codemp/game/g_weapon.c:2863`
                let now = ctx.world.level.time;
                let r = ctx.world.bg_state.rng.random();
                ctx.world.entity_mut(found_id).nextthink =
                    ((now + 100) as f32 + r * 200.0) as c_int;
                let snd = G_SoundIndex("sound/weapons/detpack/warning.wav");
                G_Sound(ctx, Some(found_id), CHAN_BODY, snd);
            }
        }
        unsafe {
            (*ent_client).ps.hasDetPackPlanted = qfalse;
        }
    }
}

/// Raven `CheatsOn`.
///
/// Source: `oracle/codemp/game/g_weapon.c:2871-2878`
pub fn CheatsOn(ctx: &mut GameContext) -> qboolean {
    if ctx.world.cvars.g_cheats.integer == 0 {
        return qfalse;
    }
    qtrue
}

/// Raven `WP_DropDetPack`.
///
/// Source: `oracle/codemp/game/g_weapon.c:2880-2964`
pub fn WP_DropDetPack(ctx: &mut GameContext, ent: Option<EntityId>, alt_fire: bool) {
    let Some(ent) = ent else {
        return;
    };
    // FLAG: firing ent may be an NPC (pool client); read the client pointer value
    // and deref it raw as Raven does.
    let ent_client = ctx.world.entity(ent).client;
    if ent_client.is_null() {
        return;
    }

    let fofs_classname = core::mem::offset_of!(gentity_t, classname) as c_int;
    let mut foundDetPacks: [c_int; MAX_GENTITIES] = [ENTITYNUM_NONE as c_int; MAX_GENTITIES];
    let mut trapcount: c_int = 0;

    // limit to 10 placed at any one time
    // see how many there are now
    let mut found: *mut gentity_t = std::ptr::null_mut();
    loop {
        found = G_Find(
            ctx,
            ctx.entity_id_of(found),
            fofs_classname,
            c"detpack".as_ptr(),
        );
        if found.is_null() {
            break;
        }
        let found_id = ctx.entity_id_of(found).unwrap();
        if ctx.world.entity(found_id).parent != Some(ent) {
            continue;
        }
        foundDetPacks[trapcount as usize] = ctx.world.entity(found_id).s.number;
        trapcount += 1;
    }
    // now remove first ones we find until there are only 9 left
    let trapcount_org = trapcount;
    let mut lowestTimeStamp = ctx.world.level.time;
    while trapcount > 9 {
        let mut removeMe: c_int = -1;
        for i in 0..trapcount_org {
            if foundDetPacks[i as usize] == ENTITYNUM_NONE as c_int {
                continue;
            }
            let found_id = EntityId(foundDetPacks[i as usize] as u32);
            if ctx.world.entity(found_id).setTime < lowestTimeStamp {
                removeMe = i;
                lowestTimeStamp = ctx.world.entity(found_id).setTime;
            }
        }
        if removeMe != -1 {
            // remove it... or blow it?
            if CheatsOn(ctx) == qfalse {
                // Let them have unlimited if cheats are enabled
                let victim_id = EntityId(foundDetPacks[removeMe as usize] as u32);
                G_FreeEntity(ctx, Some(victim_id));
            }
            foundDetPacks[removeMe as usize] = ENTITYNUM_NONE as c_int;
            trapcount -= 1;
        } else {
            break;
        }
    }

    if alt_fire {
        BlowDetpacks(ctx, ent);
    } else {
        // FLAG: firing ent pool client viewangles deref stays raw.
        let viewangles = unsafe { (*ent_client).ps.viewangles };
        AngleVectors(
            viewangles,
            Some(&mut ctx.world.globals.forward),
            Some(&mut ctx.world.globals.vright),
            Some(&mut ctx.world.globals.up),
        );

        let forward = ctx.world.globals.forward;
        let vright = ctx.world.globals.vright;
        let up = ctx.world.globals.up;
        // STAGE-2b: irreducible — `&mut world.globals.muzzle` out-param aliases the
        // `ctx` passed to `CalcMuzzlePoint`; changing that fn's signature is out of
        // scope here (one-file rule), so the raw-derived out-param stays.
        let muzzle_out = unsafe { &mut (*ctx.world_raw()).globals.muzzle };
        CalcMuzzlePoint(ctx, ent, forward, vright, up, muzzle_out);

        VectorNormalize(&mut ctx.world.globals.forward);
        _VectorMA(
            ctx.world.globals.muzzle,
            -4.0,
            ctx.world.globals.forward,
            &mut ctx.world.globals.muzzle,
        );
        let muzzle = ctx.world.globals.muzzle;
        let forward = ctx.world.globals.forward;
        drop_charge(ctx, ent, muzzle, forward);

        // FLAG: firing ent pool client deref stays raw.
        unsafe {
            (*ent_client).ps.hasDetPackPlanted = qtrue;
        }
    }
}

/// Raven `WP_FireConcussionAlt`.
///
/// Source: `oracle/codemp/game/g_weapon.c:2967-3229`
pub fn WP_FireConcussionAlt(ctx: &mut GameContext, ent: EntityId) {
    // a rail-gun-like beam
    unsafe {
        let damage: c_int = CONC_ALT_DAMAGE;
        let traces: c_int = DISRUPTOR_ALT_TRACES;
        let mut render_impact = true;
        let mut start: vec3_t;
        let mut end: vec3_t = [0.0; 3];
        let mut muzzle2 = ctx.world.globals.muzzle;
        let mut tr: trace_t = std::mem::zeroed();
        let shotRange: f32 = 8192.0;
        let mut hitDodged = false;
        let mut shot_mins: vec3_t = [-1.0, -1.0, -1.0];
        let mut shot_maxs: vec3_t = [1.0, 1.0, 1.0];

        // FLAG: firing ent may be an NPC (pool client); deref its client raw.
        let ent_client = ctx.world.entity(ent).client;
        let ent_num = ctx.world.entity(ent).s.number;

        // Shove us backwards for half a second
        _VectorMA(
            (*ent_client).ps.velocity,
            -200.0,
            ctx.world.globals.forward,
            &mut (*ent_client).ps.velocity,
        );
        (*ent_client).ps.groundEntityNum = ENTITYNUM_NONE as c_int;
        if (*ent_client).ps.pm_flags & PMF_DUCKED != 0 {
            // hunkered down
            (*ent_client).ps.pm_time = 100;
        } else {
            (*ent_client).ps.pm_time = 250;
        }

        muzzle2 = ctx.world.globals.muzzle; // making a backup copy

        start = ctx.world.globals.muzzle;
        start = W_TraceSetStart(ctx, ent, start, [0.0; 3], [0.0; 3]);

        let mut skip: c_int = ent_num;

        for _i in 0..traces {
            _VectorMA(start, shotRange, ctx.world.globals.forward, &mut end);

            if ctx.world.cvars.d_projectileGhoul2Collision.integer != 0 {
                trap::G2Trace(
                    ctx.engine,
                    mp_abi::game::syscalls::G_G2TRACE::GG2TraceArgs::new(
                        &mut tr,
                        &start as *const vec3_t,
                        &shot_mins as *const vec3_t,
                        &shot_maxs as *const vec3_t,
                        &end as *const vec3_t,
                        skip,
                        MASK_SHOT,
                        G2TRFLAG_DOGHOULTRACE | G2TRFLAG_GETSURFINDEX | G2TRFLAG_HITCORPSES,
                        ctx.world.cvars.g_g2TraceLod.integer,
                    ),
                );
            } else {
                trap::Trace(
                    ctx.engine,
                    mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                        &mut tr,
                        &start as *const vec3_t,
                        &shot_mins as *const vec3_t,
                        &shot_maxs as *const vec3_t,
                        &end as *const vec3_t,
                        skip,
                        MASK_SHOT,
                    ),
                );
            }

            let traceEnt_id = EntityId(tr.entityNum as u32);
            // FLAG: trace target may be an NPC (pool client); deref its client raw.
            let traceEnt_client = ctx.world.entity(traceEnt_id).client;

            if ctx.world.cvars.d_projectileGhoul2Collision.integer != 0
                && ctx.world.entity(traceEnt_id).inuse != 0
                && !traceEnt_client.is_null()
            {
                // g2 collision checks -rww
                if ctx.world.entity(traceEnt_id).inuse != 0
                    && !traceEnt_client.is_null()
                    && !ctx.world.entity(traceEnt_id).ghoul2.is_null()
                {
                    (*traceEnt_client).g2LastSurfaceHit = tr.surfaceFlags;
                    (*traceEnt_client).g2LastSurfaceTime = ctx.world.level.time;
                }
                if !ctx.world.entity(traceEnt_id).ghoul2.is_null() {
                    tr.surfaceFlags = 0;
                }
            }
            if (tr.surfaceFlags & SURF_NOIMPACT) != 0 {
                render_impact = false;
            }

            if tr.entityNum == ent_num as i16 {
                // should never happen, but basically we don't want to consider a
                // hit to ourselves? Get ready for an attempt to trace through
                // another person
                muzzle2 = tr.endpos;
                start = tr.endpos;
                skip = tr.entityNum as c_int;
                continue;
            }

            if tr.fraction >= 1.0 {
                // draw the beam but don't do anything else
                break;
            }

            if ctx.world.entity(traceEnt_id).s.weapon == WP_SABER {
                // FIXME: need a more reliable way to know we hit a jedi?
                hitDodged = Jedi_DodgeEvasion(ctx, Some(traceEnt_id), Some(ent), &mut tr, HL_NONE);
                // acts like we didn't even hit him
            }
            if !hitDodged {
                if render_impact {
                    if (tr.entityNum < ENTITYNUM_WORLD as i16
                        && ctx.world.entity(traceEnt_id).takedamage != 0)
                        || Q_stricmp(
                            ctx.world.entity(traceEnt_id).classname,
                            c"misc_model_breakable".as_ptr(),
                        ) == 0
                        || ctx.world.entity(traceEnt_id).s.eType == (ET_MOVER) as i32
                    {
                        // Create a simple impact type mark that doesn't last long in the world
                        if !traceEnt_client.is_null() && LogAccuracyHit(ctx, traceEnt_id, Some(ent))
                        {
                            // NOTE: hitting multiple ents can still get you over 100% accuracy
                            (*ent_client).accuracy_hits += 1;
                        }

                        let dmg_dir = Some(&mut (*ctx.world_raw()).globals.forward); // STAGE-2b: irreducible — &mut world.globals.forward aliases the ctx passed to the same call.
                        let noKnockBack = ctx.world.entity(traceEnt_id).flags & FL_NO_KNOCKBACK; // will be set if they die, I want to know if it was on *before* they died
                        if !traceEnt_client.is_null()
                            && (*traceEnt_client).NPC_class == CLASS_GALAKMECH
                        {
                            let dmg_dir = Some(&mut (*ctx.world_raw()).globals.forward); // STAGE-2b: irreducible — &mut world.globals.forward aliases the ctx passed to the same call.
                                                                                         // hehe
                            G_Damage(
                                ctx,
                                Some(traceEnt_id),
                                Some(ent),
                                Some(ent),
                                dmg_dir,
                                tr.endpos,
                                10,
                                DAMAGE_NO_KNOCKBACK | DAMAGE_NO_HIT_LOC,
                                MOD_CONC_ALT as c_int,
                            );
                            break;
                        }
                        let dmg_dir = Some(&mut (*ctx.world_raw()).globals.forward); // STAGE-2b: irreducible — &mut world.globals.forward aliases the ctx passed to the same call.
                        G_Damage(
                            ctx,
                            Some(traceEnt_id),
                            Some(ent),
                            Some(ent),
                            dmg_dir,
                            tr.endpos,
                            damage,
                            DAMAGE_NO_KNOCKBACK | DAMAGE_NO_HIT_LOC,
                            MOD_CONC_ALT as c_int,
                        );

                        // do knockback and knockdown manually
                        if !traceEnt_client.is_null() {
                            // only if we hit a client
                            let mut pushDir: vec3_t = ctx.world.globals.forward;
                            if pushDir[2] < 0.2 {
                                pushDir[2] = 0.2;
                            } // hmm, re-normalize?  nah...

                            if ctx.world.entity(traceEnt_id).health > 0 {
                                // alive
                                if noKnockBack == 0
                                    && ctx.world.entity(traceEnt_id).localAnimIndex == 0
                                    && (*traceEnt_client).ps.forceHandExtend
                                        != (HANDEXTEND_KNOCKDOWN) as i32
                                    && mp_bg::bg_pmove::BG_KnockDownable(&mut (*traceEnt_client).ps)
                                        != qfalse
                                {
                                    // knock-downable
                                    let mut plPDif: vec3_t = [0.0; 3];
                                    let mut pStr: f32;

                                    // cap it and stuff, base the strength and whether or not we
                                    // can knockdown on the distance from the shooter to the target
                                    _VectorSubtract(
                                        (*traceEnt_client).ps.origin,
                                        (*ent_client).ps.origin,
                                        &mut plPDif,
                                    );
                                    pStr = 500.0 - VectorLength(plPDif);
                                    if pStr < 150.0 {
                                        pStr = 150.0;
                                    }
                                    if pStr > 200.0 {
                                        (*traceEnt_client).ps.forceHandExtend =
                                            (HANDEXTEND_KNOCKDOWN) as i32;
                                        (*traceEnt_client).ps.forceHandExtendTime =
                                            ctx.world.level.time + 1100;
                                        (*traceEnt_client).ps.forceDodgeAnim = 0;
                                        // this toggles between 1 and 0, when it's 1 we should play the get up anim
                                    }
                                    (*traceEnt_client).ps.otherKiller = ent_num;
                                    (*traceEnt_client).ps.otherKillerTime =
                                        ctx.world.level.time + 5000;
                                    (*traceEnt_client).ps.otherKillerDebounceTime =
                                        ctx.world.level.time + 100;
                                    (*traceEnt_client).otherKillerMOD = MOD_UNKNOWN as c_int;
                                    (*traceEnt_client).otherKillerVehWeapon = 0;
                                    (*traceEnt_client).otherKillerWeaponType = WP_NONE;

                                    (*traceEnt_client).ps.velocity[0] += pushDir[0] * pStr;
                                    (*traceEnt_client).ps.velocity[1] += pushDir[1] * pStr;
                                    (*traceEnt_client).ps.velocity[2] = pStr;
                                }
                            }
                        }

                        if ctx.world.entity(traceEnt_id).s.eType == (ET_MOVER) as i32 {
                            // stop the traces on any mover
                            break;
                        }
                    } else {
                        // we only make this mark on things that can't break or move
                        break; // hit solid, but doesn't take damage, so stop the shot...we _could_ allow it to shoot through walls, might be cool?
                    }
                } else {
                    // not rendering impact, must be a skybox or other similar thing?
                    break; // don't try anymore traces
                }
            }
            // Get ready for an attempt to trace through another person
            muzzle2 = tr.endpos;
            start = tr.endpos;
            skip = tr.entityNum as c_int;
            hitDodged = false;
        }
        // just draw one beam all the way to the end

        // now go along the trail and make sight events
        let mut dir: vec3_t = [0.0; 3];
        _VectorSubtract(tr.endpos, ctx.world.globals.muzzle, &mut dir);

        // let's pack all this junk into a single tempent, and send it off.
        let tent_id = G_TempEntity(ctx, tr.endpos, (EV_CONC_ALT_IMPACT) as i32);
        let muzzle = ctx.world.globals.muzzle;
        let forward = ctx.world.globals.forward;
        let tent = ctx.entity_mut(tent_id);
        tent.s.eventParm = DirToByte(tr.plane.normal);
        tent.s.owner = ent_num;
        tent.s.angles = dir;
        tent.s.origin2 = muzzle;
        tent.s.angles2 = forward;
    }
}

/// Raven `WP_FireConcussion`.
///
/// Source: `oracle/codemp/game/g_weapon.c:3232-3276`
pub fn WP_FireConcussion(ctx: &mut GameContext, ent: EntityId) {
    // a fast rocket-like projectile
    let damage: c_int = CONC_DAMAGE;
    let vel: f32 = CONC_VELOCITY as f32;

    let mut start: vec3_t = ctx.world.globals.muzzle;
    start = W_TraceSetStart(ctx, ent, start, [0.0; 3], [0.0; 3]); // make sure our start point isn't on the other side of a wall
    let forward = ctx.world.globals.forward;

    let mid = CreateMissile(ctx, start, forward, vel, 10000, ent, false);
    let m = ctx.world.entity_mut(mid);

    m.classname = c"conc_proj".as_ptr() as *mut c_char;
    m.s.weapon = WP_CONCUSSION;
    m.mass = (10) as f32;

    // Make it easier to hit things
    m.r.maxs = [
        (ROCKET_SIZE) as f32,
        (ROCKET_SIZE) as f32,
        (ROCKET_SIZE) as f32,
    ];
    for i in 0..3 {
        m.r.mins[i] = -m.r.maxs[i];
    }

    m.damage = damage;
    m.dflags = DAMAGE_EXTRA_KNOCKBACK;

    m.methodOfDeath = MOD_CONC as c_int;
    m.splashMethodOfDeath = MOD_CONC as c_int;

    m.clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
    m.splashDamage = CONC_SPLASH_DAMAGE;
    m.splashRadius = CONC_SPLASH_RADIUS;

    // we don't want it to ever bounce
    m.bounceCount = 0;
}

/// Raven `WP_FireStunBaton`.
///
/// Source: `oracle/codemp/game/g_weapon.c:3282-3357`
pub fn WP_FireStunBaton(ctx: &mut GameContext, ent: EntityId, alt_fire: bool) {
    unsafe {
        let mut muzzleStun: vec3_t;

        // FLAG: firing ent may be an NPC (pool client); read the client pointer
        // value and deref it raw as Raven does (recipe 2b).
        let ent_client = ctx.world.entity(ent).client;
        let ent_num = ctx.world.entity(ent).s.number;

        if ent_client.is_null() {
            muzzleStun = ctx.world.entity(ent).r.currentOrigin;
            muzzleStun[2] += 8.0;
        } else {
            muzzleStun = (*ent_client).ps.origin;
            muzzleStun[2] += (*ent_client).ps.viewheight as f32 - 6.0;
        }

        let mut tmp = muzzleStun;
        _VectorMA(tmp, 20.0, ctx.world.globals.forward, &mut muzzleStun);
        tmp = muzzleStun;
        _VectorMA(tmp, 4.0, ctx.world.globals.vright, &mut muzzleStun);

        let mut end: vec3_t = [0.0; 3];
        _VectorMA(
            muzzleStun,
            (STUN_BATON_RANGE) as f32,
            ctx.world.globals.forward,
            &mut end,
        );

        let maxs: vec3_t = [6.0, 6.0, 6.0];
        let mut mins: vec3_t = [0.0; 3];
        _VectorScale(maxs, -1.0, &mut mins);

        let mut tr: trace_t = std::mem::zeroed();
        trap::Trace(
            ctx.engine,
            mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                &mut tr,
                &muzzleStun as *const vec3_t,
                &mins as *const vec3_t,
                &maxs as *const vec3_t,
                &end as *const vec3_t,
                ent_num,
                MASK_SHOT,
            ),
        );

        if (tr.entityNum as u32) >= (ENTITYNUM_WORLD) as u32 {
            return;
        }

        // Raven's `traceEnt = &g_entities[tr.entityNum]` (never NULL); the
        // always-true NULL guard is dropped.
        let tr_ent_id = EntityId(tr.entityNum as u32);
        // FLAG: trace target may be an NPC (pool client); deref its client raw.
        let tr_ent_client = ctx.world.entity(tr_ent_id).client;

        if ctx.world.entity(tr_ent_id).takedamage != 0 && !tr_ent_client.is_null() {
            // see if either party is involved in a duel
            if (*tr_ent_client).ps.duelInProgress != 0 && (*tr_ent_client).ps.duelIndex != ent_num {
                return;
            }

            if !ent_client.is_null()
                && (*ent_client).ps.duelInProgress != 0
                && (*ent_client).ps.duelIndex != ctx.world.entity(tr_ent_id).s.number
            {
                return;
            }
        }

        if ctx.world.entity(tr_ent_id).takedamage != 0 {
            G_PlayEffect((EFFECT_STUNHIT) as i32, tr.endpos, tr.plane.normal);

            let sound_idx = G_SoundIndex(&format!(
                    "sound/weapons/melee/punch{}",
                    ctx.world.bg_state.rng.Q_irand(1, 4)
                ));
            G_Sound(ctx, Some(tr_ent_id), CHAN_WEAPON, sound_idx);
            let dmg_dir = Some(&mut (*ctx.world_raw()).globals.forward); // STAGE-2b: irreducible — &mut world.globals.forward aliases the ctx passed to the same call.
            G_Damage(
                ctx,
                Some(tr_ent_id),
                Some(ent),
                Some(ent),
                dmg_dir,
                tr.endpos,
                STUN_BATON_DAMAGE,
                DAMAGE_NO_KNOCKBACK | DAMAGE_HALF_ABSORB,
                MOD_STUN_BATON as c_int,
            );

            if !tr_ent_client.is_null() {
                // if it's a player then use the shock effect
                if (*tr_ent_client).NPC_class == CLASS_VEHICLE {
                    // not on vehicles
                    // FLAG: `m_pVehicle`/`vehicleInfo_t` have no accessor; deref raw.
                    let pVeh = ctx.world.entity(tr_ent_id).m_pVehicle;
                    if pVeh.is_null()
                        || (*(*pVeh).m_pVehicleInfo).r#type
                            == mp_bg::vehicles::vehicleType_t::VH_ANIMAL
                        || (*(*pVeh).m_pVehicleInfo).r#type
                            == mp_bg::vehicles::vehicleType_t::VH_FLIER
                    {
                        // can zap animals
                        (*tr_ent_client).ps.electrifyTime =
                            ctx.world.level.time + ctx.world.bg_state.rng.Q_irand(3000, 4000);
                    }
                } else {
                    (*tr_ent_client).ps.electrifyTime = ctx.world.level.time + 700;
                }
            }
        }
    }
}

/// Raven `WP_FireMelee`.
///
/// Source: `oracle/codemp/game/g_weapon.c:3363-3445`
pub fn WP_FireMelee(ctx: &mut GameContext, ent: EntityId, alt_fire: bool) {
    unsafe {
        // FLAG: firing ent may be an NPC (pool client); deref its client raw.
        let ent_client = ctx.world.entity(ent).client;
        let ent_num = ctx.world.entity(ent).s.number;

        if !ent_client.is_null() && (*ent_client).ps.torsoAnim == (BOTH_MELEE2) as i32 {
            // right
            if (*ent_client).ps.brokenLimbs & (1 << (BROKENLIMB_RARM as i32)) != 0 {
                return;
            }
        } else {
            // left
            if (*ent_client).ps.brokenLimbs & (1 << (BROKENLIMB_LARM as i32)) != 0 {
                return;
            }
        }

        let mut muzzlePunch: vec3_t;
        if ent_client.is_null() {
            muzzlePunch = ctx.world.entity(ent).r.currentOrigin;
            muzzlePunch[2] += 8.0;
        } else {
            muzzlePunch = (*ent_client).ps.origin;
            muzzlePunch[2] += (*ent_client).ps.viewheight as f32 - 6.0;
        }

        let mut tmp = muzzlePunch;
        _VectorMA(tmp, 20.0, ctx.world.globals.forward, &mut muzzlePunch);
        tmp = muzzlePunch;
        _VectorMA(tmp, 4.0, ctx.world.globals.vright, &mut muzzlePunch);

        let mut end: vec3_t = [0.0; 3];
        _VectorMA(
            muzzlePunch,
            (MELEE_RANGE) as f32,
            ctx.world.globals.forward,
            &mut end,
        );

        let maxs: vec3_t = [6.0, 6.0, 6.0];
        let mut mins: vec3_t = [0.0; 3];
        _VectorScale(maxs, -1.0, &mut mins);

        let mut tr: trace_t = std::mem::zeroed();
        trap::Trace(
            ctx.engine,
            mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                &mut tr,
                &muzzlePunch as *const vec3_t,
                &mins as *const vec3_t,
                &maxs as *const vec3_t,
                &end as *const vec3_t,
                ent_num,
                MASK_SHOT,
            ),
        );

        if tr.entityNum != ENTITYNUM_NONE as i16 {
            // hit something
            let tr_ent_id = EntityId(tr.entityNum as u32);
            // FLAG: trace target may be an NPC (pool client); deref its client raw.
            let tr_ent_client = ctx.world.entity(tr_ent_id).client;

            let sound_idx = G_SoundIndex(&format!(
                    "sound/weapons/melee/punch{}",
                    ctx.world.bg_state.rng.Q_irand(1, 4)
                ));
            G_Sound(ctx, Some(ent), CHAN_AUTO, sound_idx);

            if ctx.world.entity(tr_ent_id).takedamage != 0 && !tr_ent_client.is_null() {
                // special duel checks
                if (*tr_ent_client).ps.duelInProgress != 0
                    && (*tr_ent_client).ps.duelIndex != ent_num
                {
                    return;
                }

                if !ent_client.is_null()
                    && (*ent_client).ps.duelInProgress != 0
                    && (*ent_client).ps.duelIndex != ctx.world.entity(tr_ent_id).s.number
                {
                    return;
                }
            }

            if ctx.world.entity(tr_ent_id).takedamage != 0 {
                // damage them, do more damage if we're in the second right hook
                let mut dmg: c_int = MELEE_SWING1_DAMAGE;

                let dmg_dir = Some(&mut (*ctx.world_raw()).globals.forward); // STAGE-2b: irreducible — &mut world.globals.forward aliases the ctx passed to the same call.
                if !ent_client.is_null() && (*ent_client).ps.torsoAnim == (BOTH_MELEE2) as i32 {
                    // do a tad bit more damage on the second swing
                    dmg = MELEE_SWING2_DAMAGE;
                }

                if G_HeavyMelee(ctx, Some(ent)) != qfalse {
                    // 2x damage for heavy melee class
                    dmg *= 2;
                }

                G_Damage(
                    ctx,
                    Some(tr_ent_id),
                    Some(ent),
                    Some(ent),
                    dmg_dir,
                    tr.endpos,
                    dmg,
                    DAMAGE_NO_ARMOR,
                    MOD_MELEE as c_int,
                );
            }
        }
    }
}

/// Raven `SnapVectorTowards`.
///
/// Source: `oracle/codemp/game/g_weapon.c:3464-3474`
pub fn SnapVectorTowards(v: vec3_t, to: vec3_t) -> vec3_t {
    let mut v = v;
    for i in 0..3 {
        if to[i] <= v[i] {
            v[i] = v[i] as c_int as f32;
        } else {
            v[i] = v[i] as c_int as f32 + 1.0;
        }
    }
    v
}

/// Raven `LogAccuracyHit`.
///
/// Source: `oracle/codemp/game/g_weapon.c:3485-3516`
pub fn LogAccuracyHit(ctx: &mut GameContext, target: EntityId, attacker: Option<EntityId>) -> bool {
    if ctx.world.entity(target).takedamage == 0 {
        return false;
    }

    if Some(target) == attacker {
        return false;
    }

    if ctx.world.entity(target).client.is_null() {
        return false;
    }

    let Some(attacker_id) = attacker else {
        return false;
    };

    if ctx.world.entity(attacker_id).client.is_null() {
        return false;
    }

    // FLAG: target pool client deref stays raw (read the client pointer value).
    let targetClient = ctx.world.entity(target).client;
    if unsafe { (*targetClient).ps.stats[statIndex_t::STAT_HEALTH as usize] } <= 0 {
        return false;
    }

    if OnSameTeam(ctx, Some(target), Some(attacker_id)) != qfalse {
        return false;
    }

    true
}

/// Raven `CalcMuzzlePoint`.
///
/// Source: `oracle/codemp/game/g_weapon.c:3530-3551`
pub fn CalcMuzzlePoint(
    ctx: &mut GameContext,
    ent: EntityId,
    forward: vec3_t,
    right: vec3_t,
    up: vec3_t,
    muzzlePoint: &mut vec3_t,
) {
    let weapontype: c_int = ctx.world.entity(ent).s.weapon;
    *muzzlePoint = ctx.world.entity(ent).s.pos.trBase;

    let muzzleOffPoint: vec3_t = WP_MuzzlePoint[weapontype as usize];

    if weapontype > WP_NONE && weapontype < WP_NUM_WEAPONS {
        // Use the table to generate the muzzlepoint;
        // Crouching.  Use the add-to-Z method to adjust vertically.
        let tmp = *muzzlePoint;
        _VectorMA(tmp, muzzleOffPoint[0], forward, muzzlePoint);
        let tmp = *muzzlePoint;
        _VectorMA(tmp, muzzleOffPoint[1], right, muzzlePoint);
        // FLAG: firing ent pool client viewheight deref stays raw.
        let ent_client = ctx.world.entity(ent).client;
        muzzlePoint[2] += unsafe { (*ent_client).ps.viewheight } as f32 + muzzleOffPoint[2];
    }

    // Referee probe: CalcMuzzlePoint pre-snap muzzle, trBase, and aim forward.
    let now = ctx.world.level.time;
    let ent_num = ctx.world.entity(ent).s.number;
    let trBase = ctx.world.entity(ent).s.pos.trBase;
    probe!(
        "MUZZLE",
        "t={} en={} w={} pre={:08x},{:08x},{:08x} tb={:08x},{:08x},{:08x} fw={:08x},{:08x},{:08x}",
        now,
        ent_num,
        weapontype,
        muzzlePoint[0].to_bits(),
        muzzlePoint[1].to_bits(),
        muzzlePoint[2].to_bits(),
        trBase[0].to_bits(),
        trBase[1].to_bits(),
        trBase[2].to_bits(),
        forward[0].to_bits(),
        forward[1].to_bits(),
        forward[2].to_bits(),
    );
    // snap to integer coordinates for more efficient network bandwidth usage
    snap_vector(muzzlePoint);
}

/// Raven `CalcMuzzlePointOrigin`.
///
/// Set muzzle location relative to pivoting eye.
///
/// Source: `oracle/codemp/game/g_weapon.c:3560-3566`
pub fn CalcMuzzlePointOrigin(
    ent: &gentity_t,
    origin: vec3_t,
    forward: vec3_t,
    right: vec3_t,
    up: vec3_t,
    muzzlePoint: vec3_t,
) -> vec3_t {
    // FLAG: pool client viewheight deref stays raw (read the client pointer value).
    let client = ent.client;
    let mut muzzlePoint = ent.s.pos.trBase;
    muzzlePoint[2] += unsafe { (*client).ps.viewheight } as f32;
    // VectorMA( muzzlePoint, 14, forward, muzzlePoint )
    muzzlePoint[0] += 14.0 * forward[0];
    muzzlePoint[1] += 14.0 * forward[1];
    muzzlePoint[2] += 14.0 * forward[2];
    // Snap to integer coordinates for more efficient network bandwidth
    // usage. Raven's `SnapVector` rounds via x87 `fistp` (round-to-nearest,
    // ties-even); `snap_vector` is the codebase's rint idiom.
    snap_vector(&mut muzzlePoint);
    muzzlePoint
}

/// Raven `WP_TouchVehMissile`.
///
/// Source: `oracle/codemp/game/g_weapon.c:3569-3578`
pub fn WP_TouchVehMissile(
    ctx: &mut GameContext,
    ent: EntityId,
    other: Option<EntityId>,
    trace: *mut trace_t,
) {
    // `trace` is the engine-provided raw out-param; copy it by value.
    let mut myTrace: trace_t = unsafe { *trace };
    if let Some(other_id) = other {
        myTrace.entityNum = ctx.world.entity(other_id).s.number as i16;
    }
    G_MissileImpact(ctx, ent, &mut myTrace);
}

/// Raven `WP_CalcVehMuzzle`.
///
/// Source: `oracle/codemp/game/g_weapon.c:3580-3608`
pub fn WP_CalcVehMuzzle(ctx: &mut GameContext, ent: EntityId, muzzleNum: c_int) {
    unsafe {
        // FLAG: vehicle entity; `m_pVehicle` and the (pool) `gclient_t` have no
        // accessor — deref raw through copied pointer values (recipe 2b/2c).
        let pVeh = ctx.world.entity(ent).m_pVehicle;
        let ent_client = ctx.world.entity(ent).client;
        let mut boltMatrix: mdxaBone_t = std::mem::zeroed();
        let mut vehAngles: vec3_t;

        // Raven `assert(pVeh)`; UB if null — the one defined behavior here is to bail.
        if pVeh.is_null() {
            return;
        }

        if (*pVeh).m_iMuzzleTime[muzzleNum as usize] == ctx.world.level.time {
            // already done for this frame, don't need to do it again
            return;
        }
        // Uh... how about we set this, hunh...?  :)
        (*pVeh).m_iMuzzleTime[muzzleNum as usize] = ctx.world.level.time;

        vehAngles = (*ent_client).ps.viewangles;
        if !(*pVeh).m_pVehicleInfo.is_null()
            && ((*(*pVeh).m_pVehicleInfo).r#type == mp_bg::vehicles::vehicleType_t::VH_ANIMAL
                || (*(*pVeh).m_pVehicleInfo).r#type == mp_bg::vehicles::vehicleType_t::VH_WALKER
                || (*(*pVeh).m_pVehicleInfo).r#type == mp_bg::vehicles::vehicleType_t::VH_SPEEDER)
        {
            vehAngles[PITCH] = 0.0;
            vehAngles[ROLL] = 0.0;
        }

        let ent_ghoul2 = ctx.world.entity(ent).ghoul2;
        let ent_modelScale = ctx.world.entity(ent).modelScale;
        trap::G2API_GetBoltMatrix_NoRecNoRot(
            ctx.engine,
            mp_abi::game::syscalls::G_G2_GETBOLT_NOREC_NOROT::GG2GetboltNorecNorotArgs::new(
                ent_ghoul2,
                0,
                (*pVeh).m_iMuzzleTag[muzzleNum as usize],
                &mut boltMatrix,
                &vehAngles as *const vec3_t,
                &(*ent_client).ps.origin as *const vec3_t,
                ctx.world.level.time,
                core::ptr::null_mut(),
                &ent_modelScale as *const vec3_t,
            ),
        );
        use mp_qshared::shared::Eorientations;
        BG_GiveMeVectorFromMatrix(
            &boltMatrix as *const mdxaBone_t,
            Eorientations::ORIGIN as c_int,
            &mut (*pVeh).m_vMuzzlePos[muzzleNum as usize],
        );
        BG_GiveMeVectorFromMatrix(
            &boltMatrix as *const mdxaBone_t,
            Eorientations::NEGATIVE_Y as c_int,
            &mut (*pVeh).m_vMuzzleDir[muzzleNum as usize],
        );
    }
}

/// Raven `WP_VehWeapSetSolidToOwner`.
///
/// Source: `oracle/codemp/game/g_weapon.c:3610-3625`
pub fn WP_VehWeapSetSolidToOwner(ctx: &mut GameContext, self_: EntityId) {
    ctx.world.entity_mut(self_).r.svFlags |= SVF_OWNERNOTSHARED;
    if ctx.world.entity(self_).genericValue1 != 0 {
        // expire after a time
        if ctx.world.entity(self_).genericValue2 != 0 {
            // blow up when your lifetime is up
            ctx.world.entity_mut(self_).think = Some(EntThink::G_ExplodeMissile).into();
        // FIXME: custom func?
        } else {
            // just remove yourself
            ctx.world.entity_mut(self_).think = Some(EntThink::G_FreeEntity).into();
            // FIXME: custom func?
        }
        let now = ctx.world.level.time;
        let gv1 = ctx.world.entity(self_).genericValue1;
        ctx.world.entity_mut(self_).nextthink = now + gv1;
    }
}

/// Raven `WP_FireVehicleWeapon`.
///
/// Source: `oracle/codemp/game/g_weapon.c:3628-3848`
pub fn WP_FireVehicleWeapon(
    ctx: &mut GameContext,
    ent: EntityId,
    start: vec3_t,
    dir: vec3_t,
    vehWeapon: *mut vehWeaponInfo_t,
    alt_fire: bool,
    isTurretWeap: bool,
) -> *mut gentity_t {
    // Return stays raw `*mut gentity_t` (return conversion is a later pass);
    // `vehWeapon` is not a gentity handle so it stays raw.
    unsafe {
        // FLAG: vehicle entity; `m_pVehicle` and the (pool) `gclient_t` have no
        // accessor — read the pointer/scalar values once and deref raw (recipe 2b/2c).
        let ent_num = ctx.world.entity(ent).s.number;
        let ent_vehicle = ctx.world.entity(ent).m_pVehicle;
        let ent_client = ctx.world.entity(ent).client;
        // FLAG: `missile` is the raw `*mut gentity_t` return value; conversion of
        // the return handle is a later pass, so it stays raw here.
        let mut missile: *mut gentity_t = std::ptr::null_mut();

        // FIXME: add some randomness...?  Inherent inaccuracy stat of weapon?  Pilot skill?
        if vehWeapon.is_null() {
            // invalid vehicle weapon
            return std::ptr::null_mut();
        } else if (*vehWeapon).bIsProjectile != qfalse {
            // projectile entity
            let mut start = start;
            let maxs: vec3_t = [
                (*vehWeapon).fWidth / 2.0,
                (*vehWeapon).fWidth / 2.0,
                (*vehWeapon).fHeight / 2.0,
            ];
            let mut mins: vec3_t = [0.0; 3];
            _VectorScale(maxs, -1.0, &mut mins);

            // make sure our start point isn't on the other side of a wall
            start = W_TraceSetStart(ctx, ent, start, mins, maxs);

            // FIXME: CUSTOM MODEL?
            // QUERY: alt_fire true or not?  Does it matter?
            let mid = CreateMissile(ctx, start, dir, (*vehWeapon).fSpeed, 10000, ent, false);
            missile = ctx.entity_mut(mid);

            (*missile).classname = c"vehicle_proj".as_ptr() as *mut c_char;

            (*missile).s.genericenemyindex = ent_num + MAX_GENTITIES as c_int;
            (*missile).damage = (*vehWeapon).iDamage;
            (*missile).splashDamage = (*vehWeapon).iSplashDamage;
            (*missile).splashRadius = ((*vehWeapon).fSplashRadius) as i32;

            // FIXME: externalize some of these properties?
            (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
            (*missile).clipmask = MASK_SHOT;
            // Maybe by checking flags...?
            if (*vehWeapon).bSaberBlockable != qfalse {
                (*missile).clipmask |= CONTENTS_LIGHTSABER;
            }

            // Make it easier to hit things
            (*missile).r.mins = mins;
            (*missile).r.maxs = maxs;
            // some slightly different stuff for things with bboxes
            if (*vehWeapon).fWidth != 0.0 || (*vehWeapon).fHeight != 0.0 {
                // we assume it's a rocket-like thing
                (*missile).s.weapon = WP_ROCKET_LAUNCHER; // does this really matter?
                (*missile).methodOfDeath = MOD_VEHICLE as c_int; // MOD_ROCKET;
                (*missile).splashMethodOfDeath = MOD_VEHICLE as c_int; // MOD_ROCKET;// ?SPLASH;

                // we don't want it to ever bounce
                (*missile).bounceCount = 0;

                (*missile).mass = (10) as f32;
            } else {
                // a blaster-laser-like thing
                (*missile).s.weapon = WP_BLASTER; // does this really matter?
                (*missile).methodOfDeath = MOD_VEHICLE as c_int; // count as a heavy weap
                (*missile).splashMethodOfDeath = MOD_VEHICLE as c_int; // ?SPLASH;
                                                                       // we don't want it to bounce forever
                (*missile).bounceCount = 8;
            }

            if (*vehWeapon).bHasGravity != qfalse {
                // TESTME: is this all we need to do?
                (*missile).s.weapon = WP_THERMAL; // does this really matter?
                (*missile).s.pos.trType = TR_GRAVITY;
            }

            if (*vehWeapon).bIonWeapon != qfalse {
                // so it disables ship shields and sends them out of control
                (*missile).s.weapon = WP_DEMP2;
            }

            if (*vehWeapon).iHealth != 0 {
                // the missile can take damage
                // don't do this - ships hit them first and have no trace.plane.normal to
                // bounce off it at and end up in the middle of the asteroid...
            }

            // pilot should own this projectile on server if we have a pilot
            let pVeh = ent_vehicle;
            if !pVeh.is_null() && !(*pVeh).m_pPilot.is_null() {
                // owned by vehicle pilot
                (*missile).r.ownerNum =
                    ((*((*pVeh).m_pPilot as *mut gentity_t)).s.number as u32) as i32;
            } else {
                // owned by vehicle?
                (*missile).r.ownerNum = (ent_num as u32) as i32;
            }

            // set veh as cgame side owner for purpose of fx overrides
            (*missile).s.owner = ent_num;
            if alt_fire {
                // use the second weapon's iShotFX
                (*missile).s.eFlags |= EF_ALT_FIRING;
            }
            if isTurretWeap {
                // look for the turret weapon info on cgame side, not vehicle weapon info
                (*missile).s.weapon = WP_TURRET;
            }
            if (*vehWeapon).iLifeTime != 0 {
                // expire after a time
                if (*vehWeapon).bExplodeOnExpire != qfalse {
                    // blow up when your lifetime is up
                    (*missile).think = Some(EntThink::G_ExplodeMissile).into(); // FIXME: custom func?
                } else {
                    // just remove yourself
                    (*missile).think = Some(EntThink::G_FreeEntity).into(); // FIXME: custom func?
                }
                (*missile).nextthink = ctx.world.level.time + (*vehWeapon).iLifeTime;
            }
            (*missile).s.otherEntityNum2 = vehWeapon
                .offset_from(&(&ctx.world.bg_state.g_vehWeaponInfo)[0] as *const vehWeaponInfo_t)
                as c_int;
            (*missile).s.eFlags |= EF_JETPACK_ACTIVE;
            // homing
            if (*vehWeapon).fHoming != 0.0 {
                // homing missile
                if !ent_client.is_null()
                    && (*ent_client).ps.rocketLockIndex != ENTITYNUM_NONE as c_int
                {
                    let mut dif: c_int = 0;
                    let mut rTime: f32;
                    rTime = (*ent_client).ps.rocketLockTime as f32;

                    if rTime == -1.0 {
                        rTime = (*ent_client).ps.rocketLastValidTime as f32;
                    }

                    if (*vehWeapon).iLockOnTime == 0 {
                        // no minimum lock-on time
                        dif = 10; // guaranteed lock-on
                    } else {
                        let lockTimeInterval = (*vehWeapon).iLockOnTime as f32 / 16.0;
                        dif = ((ctx.world.level.time as f32 - rTime) / lockTimeInterval) as c_int;
                    }

                    if dif < 0 {
                        dif = 0;
                    }

                    // It's 10 even though it locks client-side at 8, because we want them to
                    // have a sturdy lock first, and because there's a slight difference in
                    // time between server and client
                    if dif >= 10 && rTime != -1.0 {
                        let enemy_id = EntityId((*ent_client).ps.rocketLockIndex as u32);
                        // FLAG: locked target may be an NPC (pool client); deref raw.
                        let enemy_client = ctx.world.entity(enemy_id).client;
                        (*missile).enemy = Some(enemy_id);

                        if !enemy_client.is_null()
                            && ctx.world.entity(enemy_id).health > 0
                            && OnSameTeam(ctx, Some(ent), Some(enemy_id)) == qfalse
                        {
                            // if enemy became invalid, died, or is on the same team, then don't seek it
                            (*missile).spawnflags |= 1; // just to let it know it should be faster...
                            (*missile).speed = (*vehWeapon).fSpeed;
                            (*missile).angle = (*vehWeapon).fHoming;
                            (*missile).radius = (*vehWeapon).fHomingFOV;
                            // crap, if we have a lifetime, need to store that somewhere else on
                            // ent and have rocketThink func check it every frame...
                            if (*vehWeapon).iLifeTime != 0 {
                                // expire after a time
                                (*missile).genericValue1 =
                                    ctx.world.level.time + (*vehWeapon).iLifeTime;
                                (*missile).genericValue2 = (*vehWeapon).bExplodeOnExpire as c_int;
                            }
                            // now go ahead and use the rocketThink func
                            (*missile).think = Some(EntThink::rocketThink).into(); // FIXME: custom func?
                            (*missile).nextthink =
                                ctx.world.level.time + VEH_HOMING_MISSILE_THINK_TIME;
                            (*missile).s.eFlags |= EF_RADAROBJECT; // FIXME: externalize
                            if ctx.world.entity(enemy_id).s.NPC_class == (CLASS_VEHICLE) as i32 {
                                // let vehicle know we've locked on to them
                                (*missile).s.otherEntityNum = ctx.world.entity(enemy_id).s.number;
                            }
                        }
                    }

                    (*missile).movedir = dir;
                    (*missile).random = 1.0; // FIXME: externalize?
                }
            }
            if (*vehWeapon).fSpeed == 0.0 {
                // a mine or something?
                if (*vehWeapon).iHealth != 0 {
                    // the missile can take damage
                    (*missile).health = (*vehWeapon).iHealth;
                    (*missile).takedamage = qtrue;
                    (*missile).r.contents = MASK_SHOT;
                    (*missile).die = Some(EntDie::RocketDie).into();
                }
                // only do damage when someone touches us
                (*missile).s.weapon = WP_THERMAL; // does this really matter?
                G_SetOrigin(&mut *(missile), start);
                (*missile).touch = Some(EntTouch::WP_TouchVehMissile).into();
                (*missile).s.eFlags |= EF_RADAROBJECT; // FIXME: externalize
                                                       // crap, if we have a lifetime, need to store that somewhere else on ent
                                                       // and have rocketThink func check it every frame...
                if (*vehWeapon).iLifeTime != 0 {
                    // expire after a time
                    (*missile).genericValue1 = (*vehWeapon).iLifeTime;
                    (*missile).genericValue2 = (*vehWeapon).bExplodeOnExpire as c_int;
                }
                // now go ahead and use the setsolidtoowner func
                (*missile).think = Some(EntThink::WP_VehWeapSetSolidToOwner).into();
                (*missile).nextthink = ctx.world.level.time + 3000;
            }
        } else {
            // traceline
            // FIXME: implement
        }

        missile
    }
}

/// Raven `G_VehMuzzleFireFX`.
///
/// Source: `oracle/codemp/game/g_weapon.c:3851-3881`
pub fn G_VehMuzzleFireFX(
    ctx: &mut GameContext,
    ent: EntityId,
    broadcaster: Option<EntityId>,
    muzzlesFired: c_int,
) {
    unsafe {
        // FLAG: vehicle entity; `m_pVehicle` and the (pool) `gclient_t` have no
        // accessor — deref raw through copied pointer values (recipe 2b/2c).
        let pVeh = ctx.world.entity(ent).m_pVehicle;

        if pVeh.is_null() {
            return;
        }

        let ent_client = ctx.world.entity(ent).client;
        let ent_num = ctx.world.entity(ent).s.number;

        let b_id: EntityId;
        if broadcaster.is_none() {
            // oh well. We will WASTE A TEMPENT.
            let bt = G_TempEntity(ctx, (*ent_client).ps.origin, EV_VEH_FIRE as c_int);
            b_id = bt;
        } else {
            // joy
            b_id = broadcaster.unwrap();
        }

        // this guy owns it
        ctx.world.entity_mut(b_id).s.owner = ent_num;

        // this is the bitfield of all muzzles fired this time
        // NOTE: just need MAX_VEHICLE_MUZZLES bits for this... should be cool
        // since it's currently 12 and we're sending it in 16 bits
        ctx.world.entity_mut(b_id).s.trickedentindex = muzzlesFired;

        if broadcaster.is_some() {
            // add the event
            G_AddEvent(ctx.world.entity_mut(b_id), EV_VEH_FIRE as c_int, 0);
        }
    }
}

/// Raven `G_EstimateCamPos`.
///
/// Source: `oracle/codemp/game/g_weapon.c:3883-3959`
pub fn G_EstimateCamPos(
    ctx: &mut GameContext,
    viewAngles: vec3_t,
    // Read AND written (Raven bumps the caller's buffer by viewheight at
    // g_weapon.c:3918) — `&mut` per the settled vec3 out-param rule.
    cameraFocusLoc: &mut vec3_t,
    viewheight: f32,
    thirdPersonRange: f32,
    thirdPersonHorzOffset: f32,
    vertOffset: f32,
    pitchOffset: f32,
    ignoreEntNum: c_int,
    camPos: &mut vec3_t,
) {
    unsafe {
        // `MASK_SOLID`/`CONTENTS_PLAYERCLIP` come from the prelude's `surface_flags`
        // re-export. Raven: `int MASK_CAMERACLIP = (MASK_SOLID|CONTENTS_PLAYERCLIP);`
        let MASK_CAMERACLIP: c_int = MASK_SOLID | CONTENTS_PLAYERCLIP;
        let CAMERA_SIZE: f32 = 4.0;
        let cameramins: vec3_t = [-CAMERA_SIZE, -CAMERA_SIZE, -CAMERA_SIZE];
        let cameramaxs: vec3_t = [CAMERA_SIZE, CAMERA_SIZE, CAMERA_SIZE];

        let mut cameraFocusAngles = viewAngles;
        cameraFocusAngles[PITCH] += pitchOffset;
        if ctx.world.cvars.bg_fighterAltControl.integer == 0 {
            // clamp view pitch
            cameraFocusAngles[PITCH] = AngleNormalize180(cameraFocusAngles[PITCH]);
            if cameraFocusAngles[PITCH] > 80.0 {
                cameraFocusAngles[PITCH] = 80.0;
            } else if cameraFocusAngles[PITCH] < -80.0 {
                cameraFocusAngles[PITCH] = -80.0;
            }
        }
        let mut camerafwd: vec3_t = [0.0; 3];
        let mut cameraup: vec3_t = [0.0; 3];
        AngleVectors(
            cameraFocusAngles,
            Some(&mut camerafwd),
            None,
            Some(&mut cameraup),
        );

        cameraFocusLoc[2] += viewheight;

        let mut cameraIdealTarget = *cameraFocusLoc;
        cameraIdealTarget[2] += vertOffset;

        // NOTE: on cgame, this uses the thirdpersontargetdamp value, we ignore that here
        let mut cameraCurTarget = cameraIdealTarget;
        let mut trace: trace_t = std::mem::zeroed();
        trap::Trace(
            ctx.engine,
            mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                &mut trace,
                &*cameraFocusLoc as *const vec3_t,
                &cameramins as *const vec3_t,
                &cameramaxs as *const vec3_t,
                &cameraCurTarget as *const vec3_t,
                ignoreEntNum,
                MASK_CAMERACLIP,
            ),
        );
        if trace.fraction < 1.0 {
            cameraCurTarget = trace.endpos;
        }

        let mut cameraIdealLoc: vec3_t = [0.0; 3];
        _VectorMA(
            cameraIdealTarget,
            -thirdPersonRange,
            camerafwd,
            &mut cameraIdealLoc,
        );
        // NOTE: on cgame, this uses the thirdpersoncameradamp value, we ignore that here
        let mut cameraCurLoc = cameraIdealLoc;
        trap::Trace(
            ctx.engine,
            mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                &mut trace,
                &cameraCurTarget as *const vec3_t,
                &cameramins as *const vec3_t,
                &cameramaxs as *const vec3_t,
                &cameraCurLoc as *const vec3_t,
                ignoreEntNum,
                MASK_CAMERACLIP,
            ),
        );
        if trace.fraction < 1.0 {
            cameraCurLoc = trace.endpos;
        }

        let mut diff: vec3_t = [0.0; 3];
        _VectorSubtract(cameraCurTarget, cameraCurLoc, &mut diff);
        {
            let dist = VectorNormalize(&mut diff);
            // under normal circumstances, should never be 0.00000 and so on.
            if dist == 0.0 || diff[0] == 0.0 || diff[1] == 0.0 {
                // must be hitting something, need some value to calc angles, so use cam forward
                diff = camerafwd;
            }
        }

        let mut camAngles: vec3_t = [0.0; 3];
        vectoangles(diff, &mut camAngles);

        if thirdPersonHorzOffset != 0.0 {
            let mut viewaxis: [vec3_t; 3] = [[0.0; 3]; 3];
            AnglesToAxis(camAngles, viewaxis.as_mut_ptr());
            let tmp = cameraCurLoc;
            _VectorMA(tmp, thirdPersonHorzOffset, viewaxis[1], &mut cameraCurLoc);
        }

        *camPos = cameraCurLoc;
    }
}

/// Raven `WP_GetVehicleCamPos`.
///
/// Source: `oracle/codemp/game/g_weapon.c:3961-4020`
pub fn WP_GetVehicleCamPos(
    ctx: &mut GameContext,
    ent: EntityId,
    pilot: EntityId,
    camPos: &mut [f32; 3],
) {
    unsafe {
        // FLAG: vehicle + pilot; `m_pVehicle`/`vehicleInfo_t` and the (pool)
        // `gclient_t`s have no accessor — deref raw through copied pointers.
        let pVeh = ctx.world.entity(ent).m_pVehicle;
        let ent_client = ctx.world.entity(ent).client;
        let pilot_client = ctx.world.entity(pilot).client;
        let pilot_num = ctx.world.entity(pilot).s.number;
        let vehInfo = (*pVeh).m_pVehicleInfo as *mut mp_bg::vehicles::vehicleInfo_t;
        let mut thirdPersonHorzOffset: f32 = (*vehInfo).cameraHorzOffset;
        let mut thirdPersonRange: f32 = (*vehInfo).cameraRange;
        let mut pitchOffset: f32 = (*vehInfo).cameraPitchOffset;
        let mut vertOffset: f32 = (*vehInfo).cameraVertOffset;

        if (*ent_client).ps.hackingTime != 0 {
            thirdPersonHorzOffset +=
                ((*ent_client).ps.hackingTime as f32 / MAX_STRAFE_TIME) * -80.0;
            // C: `fabs(((float)hackingTime)/MAX_STRAFE_TIME)*100.0f` — libm `fabs`
            // promotes to double, the double product rounds once at the f32 `+=`.
            // Source: `oracle/codemp/game/g_weapon.c:3971`
            thirdPersonRange = (thirdPersonRange as f64
                + (((*ent_client).ps.hackingTime as f32 / MAX_STRAFE_TIME) as f64).abs() * 100.0)
                as f32;
        }

        if (*vehInfo).cameraPitchDependantVertOffset != qfalse {
            if (*pilot_client).ps.viewangles[PITCH] > 0.0 {
                vertOffset = 130.0 + (*pilot_client).ps.viewangles[PITCH] * -10.0;
                if vertOffset < -170.0 {
                    vertOffset = -170.0;
                }
            } else if (*pilot_client).ps.viewangles[PITCH] < 0.0 {
                vertOffset = 130.0 + (*pilot_client).ps.viewangles[PITCH] * -5.0;
                if vertOffset > 130.0 {
                    vertOffset = 130.0;
                }
            } else {
                vertOffset = 30.0;
            }
            if (*pilot_client).ps.viewangles[PITCH] > 0.0 {
                pitchOffset = (*pilot_client).ps.viewangles[PITCH] * -0.75;
            } else if (*pilot_client).ps.viewangles[PITCH] < 0.0 {
                pitchOffset = (*pilot_client).ps.viewangles[PITCH] * -0.75;
            } else {
                pitchOffset = 0.0;
            }
        }

        // Control Scheme 3 Method:
        // Raven passes `pilot->client->ps.origin` directly, so the viewheight
        // bump inside G_EstimateCamPos lands in the pilot's ps.origin buffer.
        G_EstimateCamPos(
            ctx,
            (*ent_client).ps.viewangles,
            &mut (*pilot_client).ps.origin,
            (*pilot_client).ps.viewheight as f32,
            thirdPersonRange,
            thirdPersonHorzOffset,
            vertOffset,
            pitchOffset,
            pilot_num,
            camPos,
        );
    }
}

/// Raven `WP_VehLeadCrosshairVeh`.
///
/// Source: `oracle/codemp/game/g_weapon.c:4022-4047`
pub fn WP_VehLeadCrosshairVeh(
    ctx: &mut GameContext,
    camTraceEnt: Option<EntityId>,
    newEnd: &mut [f32; 3],
    dir: [f32; 3],
    shotStart: [f32; 3],
    shotDir: &mut [f32; 3],
) {
    unsafe {
        if ctx.world.cvars.g_vehAutoAimLead.integer != 0 {
            // FLAG: camera-trace target may be an NPC/vehicle (pool client);
            // read the client pointer value and deref it raw (recipe 2b).
            let cam_client = match camTraceEnt {
                Some(id) => ctx.world.entity(id).client,
                None => core::ptr::null_mut(),
            };
            if !cam_client.is_null() && (*cam_client).NPC_class == CLASS_VEHICLE {
                let dot = _DotProduct((*cam_client).ps.velocity, dir);
                let distAdjust = dot;
                let mut predPos = [0.0f32; 3];
                let mut predShotDir = [0.0f32; 3];

                if distAdjust > 500.0f32
                    || DistanceSquared((*cam_client).ps.origin, shotStart) > 7000000.0f32
                {
                    _VectorMA(*newEnd, distAdjust, dir, &mut predPos);
                    _VectorSubtract(predPos, shotStart, &mut predShotDir);
                    VectorNormalize(&mut predShotDir);
                    let dot = _DotProduct(predShotDir, *shotDir);
                    if dot >= 0.75f32 {
                        *newEnd = predPos;
                    }
                }
            }
        }
        _VectorSubtract(*newEnd, shotStart, shotDir);
        VectorNormalize(shotDir);
    }
}

/// Raven `WP_VehCheckTraceFromCamPos`.
///
/// Source: `oracle/codemp/game/g_weapon.c:4052-4113`
pub fn WP_VehCheckTraceFromCamPos(
    ctx: &mut GameContext,
    ent: Option<EntityId>,
    shotStart: [f32; 3],
    shotDir: &mut [f32; 3],
) -> qboolean {
    // STAGE-1: Option param (body null-checks ent), raw re-derived verbatim (Stage-2 debt).
    // 2c-W6 FLAG (left): SEAM-BG-REENTRY — the body casts the raw `ent`
    // (`ent as *mut bgEntity_t`) into `BG_VehTraceFromCamPos` alongside the
    // `ctx.world_raw()` `GameCallbacksImpl`/`bg_state` seam adapters (recipe
    // rule 5). The ent handle is irreducibly raw at that bg boundary, so this
    // function is left for the seam pass.
    let ent: *mut gentity_t = unsafe { ent_id::resolve(ctx.world.g_entities.as_mut_ptr(), ent) };
    unsafe {
        let mut shotDir = shotDir;
        // FIXME: only if dynamicCrosshair and dynamicCrosshairPrecision is on!
        if ent.is_null() || (*ent).m_pVehicle.is_null() {
            return qfalse;
        }
        let pVeh = (*ent).m_pVehicle;
        if (*pVeh).m_pVehicleInfo.is_null() || (*pVeh).m_pPilot.is_null() {
            return qfalse;
        }
        let pilot = (*pVeh).m_pPilot as *mut gentity_t;
        if (*pilot).client.is_null() || (*pilot).s.number >= MAX_CLIENTS as c_int {
            // not being driven / being driven, but not by a real client, no need to worry about crosshair
            return qfalse;
        }

        let vehInfo = (*pVeh).m_pVehicleInfo as *mut mp_bg::vehicles::vehicleInfo_t;
        if ((*vehInfo).r#type == mp_bg::vehicles::vehicleType_t::VH_FIGHTER
            && ctx.world.globals.g_cullDistance > MAX_XHAIR_DIST_ACCURACY)
            || (*vehInfo).r#type == mp_bg::vehicles::vehicleType_t::VH_WALKER
        {
            // FIRST: simulate the normal crosshair trace from the center of the veh straight forward
            let mut trace: trace_t = std::mem::zeroed();
            let mut dir: vec3_t = [0.0; 3];
            let mut start: vec3_t;
            let mut end: vec3_t = [0.0; 3];
            if (*vehInfo).r#type == mp_bg::vehicles::vehicleType_t::VH_WALKER {
                // for some reason, the walker always draws the crosshair out from the first muzzle point
                AngleVectors((*((*ent).client)).ps.viewangles, Some(&mut dir), None, None);
                start = (*ent).r.currentOrigin;
                start[2] += (*vehInfo).height - DEFAULT_MINS_2 - 48.0;
            } else {
                let mut ang: vec3_t = [0.0; 3];
                if (*vehInfo).r#type == mp_bg::vehicles::vehicleType_t::VH_SPEEDER {
                    ang = [0.0, *(*pVeh).m_vOrientation.add(1), 0.0];
                } else {
                    ang = [
                        *(*pVeh).m_vOrientation.add(0),
                        *(*pVeh).m_vOrientation.add(1),
                        *(*pVeh).m_vOrientation.add(2),
                    ];
                }
                AngleVectors(ang, Some(&mut dir), None, None);
                start = (*ent).r.currentOrigin;
            }
            _VectorMA(start, ctx.world.globals.g_cullDistance, dir, &mut end);
            trap::Trace(
                ctx.engine,
                mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                    &mut trace,
                    &start as *const vec3_t,
                    &[0.0f32, 0.0f32, 0.0f32] as *const vec3_t,
                    &[0.0f32, 0.0f32, 0.0f32] as *const vec3_t,
                    &end as *const vec3_t,
                    (*ent).s.number,
                    CONTENTS_SOLID | CONTENTS_BODY,
                ),
            );

            if (*vehInfo).r#type == mp_bg::vehicles::vehicleType_t::VH_WALKER {
                // just use the result of that one trace since walkers don't do the extra trace
                _VectorSubtract(trace.endpos, shotStart, shotDir);
                VectorNormalize(shotDir);
                return qtrue;
            } else {
                // NOW do the trace from the camPos and compare with above trace
                let mut extraTrace: trace_t = std::mem::zeroed();
                let mut newEnd: vec3_t = [0.0; 3];
                // `BG_VehTraceFromCamPos` is a bg-tier free fn (`&BgState`/
                // `&dyn BgTraps`); its `WP_GetVehicleCamPos` upcall needs game
                // state, so it also takes `&mut dyn GameCallbacks`. This
                // game-tier caller builds both adapters from `ctx`.
                let camTraceEntNum = mp_bg::bg_pmove::BG_VehTraceFromCamPos(
                    &mut extraTrace,
                    // S5-6 seam cast: the bg fn now takes `mp_bg`'s narrow
                    // `bgEntity_t`; the game's `gentity_t` head is layout-identical.
                    ent as *mut mp_bg::public::bg_entity::bgEntity_t,
                    (*ent).r.currentOrigin,
                    shotStart,
                    end,
                    &mut newEnd,
                    &mut *shotDir,
                    trace.fraction * ctx.world.globals.g_cullDistance,
                    // STAGE-2b: irreducible — the ruling-21 `GameCallbacksImpl` seam
                    // adapter holds a raw `*mut GameWorld`, so `bg_state` is read raw
                    // to coexist with the `world:` field it fills in the same call.
                    &(*ctx.world_raw()).bg_state,
                    &GameBgTraps::new(ctx.engine),
                    &mut GameCallbacksImpl {
                        world: ctx.world_raw(),
                        engine: ctx.engine,
                    },
                );
                if camTraceEntNum != 0 {
                    let camTraceEnt =
                        &mut ctx.world.g_entities[(camTraceEntNum - 1) as usize] as *mut gentity_t;
                    WP_VehLeadCrosshairVeh(
                        ctx,
                        ctx.entity_id_of(camTraceEnt),
                        &mut newEnd,
                        dir,
                        shotStart,
                        shotDir,
                    );
                    return qtrue;
                }
            }
        }
        qfalse
    }
}

/// Raven `FireVehicleWeapon`.
///
/// Source: `oracle/codemp/game/g_weapon.c:4116-4413`
pub fn FireVehicleWeapon(ctx: &mut GameContext, ent: EntityId, alt_fire: bool) {
    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    // 2c-W6 FLAG (left): the body launders `let pVeh = &mut *pVeh` (a `&mut
    // Vehicle_t` with no accessor) and holds it across many `&mut ctx` calls
    // (recipe 2d). Cleanly removing the ent re-derive requires a `Vehicle_t`
    // accessor that does not exist yet; adding one would touch another file
    // (recipe rule 8), so this function is deferred to a later wave.
    let ent: *mut gentity_t = ctx.entity_mut(ent);
    unsafe {
        let pVeh = (*ent).m_pVehicle;
        if pVeh.is_null() {
            return;
        }

        let pVeh = &mut *pVeh;

        if pVeh.m_iRemovedSurfaces != 0 {
            return;
        }

        if pVeh.m_pVehicleInfo.as_ref().unwrap().r#type == VH_WALKER
            && (*((*ent).client)).ps.electrifyTime > ctx.world.level.time
        {
            return;
        }

        if !pVeh.m_pVehicleInfo.is_null()
            && (pVeh.m_pVehicleInfo.as_ref().unwrap().r#type != VH_FIGHTER
                || (pVeh.m_ulFlags & (VEH_WINGSOPEN as u64)) != 0)
        {
            let mut weaponNum: c_int = 0;
            let mut vehWeaponIndex = VEH_WEAPON_NONE;
            let mut delay: c_int = 1000;
            let mut aimCorrect = qfalse;
            let mut linkedFiring = qfalse;

            if !alt_fire {
                weaponNum = 0;
            } else {
                weaponNum = 1;
            }

            vehWeaponIndex = pVeh.m_pVehicleInfo.as_ref().unwrap().weapon[weaponNum as usize].ID;

            if pVeh.weaponStatus[weaponNum as usize].ammo <= 0 {
                if !pVeh.m_pPilot.is_null() && (*pVeh.m_pPilot).s.number < MAX_CLIENTS as c_int {
                    let mut i = 0;
                    while i < MAX_VEHICLE_MUZZLES as c_int {
                        if pVeh.m_pVehicleInfo.as_ref().unwrap().weapMuzzle[i as usize]
                            != vehWeaponIndex
                        {
                            i += 1;
                            continue;
                        }
                        if pVeh.m_iMuzzleTag[i as usize] != -1
                            && pVeh.m_iMuzzleWait[i as usize] < ctx.world.level.time
                        {
                            G_AddEvent(
                                &mut *(pVeh.m_pPilot as *mut gentity_t),
                                (EV_NOAMMO) as i32,
                                weaponNum,
                            );
                            break;
                        }
                        i += 1;
                    }
                }
                return;
            }

            delay = pVeh.m_pVehicleInfo.as_ref().unwrap().weapon[weaponNum as usize].delay;
            aimCorrect = pVeh.m_pVehicleInfo.as_ref().unwrap().weapon[weaponNum as usize].aimCorrect
                as qboolean;
            if pVeh.m_pVehicleInfo.as_ref().unwrap().weapon[weaponNum as usize].linkable == 2
                || (pVeh.m_pVehicleInfo.as_ref().unwrap().weapon[weaponNum as usize].linkable == 1
                    && pVeh.weaponStatus[weaponNum as usize].linked != 0)
            {
                linkedFiring = qtrue;
            }

            if vehWeaponIndex <= VEH_WEAPON_BASE || vehWeaponIndex >= MAX_VEH_WEAPONS as c_int {
                return;
            }

            let mut numMuzzles: c_int = 0;
            let mut numMuzzlesReady: c_int = 0;
            let mut cumulativeDelay: c_int = 0;
            let mut cumulativeAmmo: c_int = 0;
            let mut sentAmmoWarning = qfalse;

            let vehWeapon: *mut vehWeaponInfo_t =
                &mut (&mut ctx.world.bg_state.g_vehWeaponInfo)[vehWeaponIndex as usize];

            if pVeh.m_pVehicleInfo.as_ref().unwrap().weapon[weaponNum as usize].linkable == 2 {
                cumulativeDelay = delay;
            }

            let mut i = 0;
            while i < MAX_VEHICLE_MUZZLES as c_int {
                if pVeh.m_pVehicleInfo.as_ref().unwrap().weapMuzzle[i as usize] != vehWeaponIndex {
                    i += 1;
                    continue;
                }
                if pVeh.m_iMuzzleTag[i as usize] != -1
                    && pVeh.m_iMuzzleWait[i as usize] < ctx.world.level.time
                {
                    numMuzzlesReady += 1;
                }
                if pVeh.m_pVehicleInfo.as_ref().unwrap().weapMuzzle
                    [pVeh.weaponStatus[weaponNum as usize].nextMuzzle as usize]
                    != vehWeaponIndex
                {
                    pVeh.weaponStatus[weaponNum as usize].nextMuzzle = i;
                }
                if linkedFiring != 0 {
                    cumulativeAmmo += (*vehWeapon).iAmmoPerShot;
                    if pVeh.m_pVehicleInfo.as_ref().unwrap().weapon[weaponNum as usize].linkable
                        != 2
                    {
                        cumulativeDelay += delay;
                    }
                }
                numMuzzles += 1;
                i += 1;
            }

            if linkedFiring != 0 {
                if numMuzzlesReady != numMuzzles {
                    return;
                } else if pVeh.weaponStatus[weaponNum as usize].ammo < cumulativeAmmo {
                    if !pVeh.m_pPilot.is_null() && (*pVeh.m_pPilot).s.number < MAX_CLIENTS as c_int
                    {
                        G_AddEvent(
                            &mut *(pVeh.m_pPilot as *mut gentity_t),
                            (EV_NOAMMO) as i32,
                            weaponNum,
                        );
                    }
                    return;
                }
            }

            let mut muzzlesFired: c_int = 0;
            let mut missile: *mut gentity_t = std::ptr::null_mut();
            let mut clearRocketLockEntity = qfalse;

            'try_fire: {
                let mut i = 0;
                while i < MAX_VEHICLE_MUZZLES as c_int {
                    if pVeh.m_pVehicleInfo.as_ref().unwrap().weapMuzzle[i as usize]
                        != vehWeaponIndex
                    {
                        i += 1;
                        continue;
                    }
                    if linkedFiring == 0 && i != pVeh.weaponStatus[weaponNum as usize].nextMuzzle {
                        i += 1;
                        continue;
                    }

                    if pVeh.m_iMuzzleTag[i as usize] != -1
                        && pVeh.m_iMuzzleWait[i as usize] < ctx.world.level.time
                    {
                        if pVeh.weaponStatus[weaponNum as usize].ammo < (*vehWeapon).iAmmoPerShot {
                            if sentAmmoWarning == 0 {
                                sentAmmoWarning = qtrue;
                                if !pVeh.m_pPilot.is_null()
                                    && (*pVeh.m_pPilot).s.number < MAX_CLIENTS as c_int
                                {
                                    G_AddEvent(
                                        &mut *(pVeh.m_pPilot as *mut gentity_t),
                                        (EV_NOAMMO) as i32,
                                        weaponNum,
                                    );
                                }
                            }
                        } else {
                            WP_CalcVehMuzzle(ctx, ctx.entity_id_of(ent).unwrap(), i);
                            let mut start = pVeh.m_vMuzzlePos[i as usize];
                            let mut dir = pVeh.m_vMuzzleDir[i as usize];
                            if WP_VehCheckTraceFromCamPos(
                                ctx,
                                ctx.entity_id_of(ent),
                                start,
                                &mut dir,
                            ) != 0
                            {
                            } else if aimCorrect != 0 {
                                let mut trace: trace_t = std::mem::zeroed();
                                let mut end = [0.0f32; 3];
                                let mut ang = [0.0f32; 3];
                                let mut fixedDir = [0.0f32; 3];

                                if pVeh.m_pVehicleInfo.as_ref().unwrap().r#type == VH_SPEEDER {
                                    VectorSet(
                                        &mut ang,
                                        0.0f32,
                                        *pVeh.m_vOrientation.add(1),
                                        0.0f32,
                                    );
                                } else {
                                    ang = *(pVeh.m_vOrientation as *const vec3_t);
                                }
                                AngleVectors(ang, Some(&mut fixedDir), None, None);
                                _VectorMA((*ent).r.currentOrigin, 32768.0f32, fixedDir, &mut end);
                                trap::Trace(
                                    ctx.engine,
                                    mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                                        &mut trace,
                                        &(*ent).r.currentOrigin as *const vec3_t,
                                        &vec3_origin as *const vec3_t,
                                        &vec3_origin as *const vec3_t,
                                        &end as *const vec3_t,
                                        (*ent).s.number,
                                        MASK_SHOT,
                                    ),
                                );
                                if trace.fraction < 1.0f32
                                    && trace.allsolid == 0
                                    && trace.startsolid == 0
                                {
                                    let mut newEnd = [0.0f32; 3];
                                    newEnd = trace.endpos;
                                    WP_VehLeadCrosshairVeh(
                                        ctx,
                                        EntityId::from_num(trace.entityNum as c_int),
                                        &mut newEnd,
                                        fixedDir,
                                        start,
                                        &mut dir,
                                    );
                                }
                            }

                            muzzlesFired |= 1 << i;

                            missile = WP_FireVehicleWeapon(
                                ctx,
                                ctx.entity_id_of(ent).unwrap(),
                                start,
                                dir,
                                vehWeapon as *mut _,
                                alt_fire,
                                false,
                            );
                            if (*vehWeapon).fHoming != (0) as f32 {
                                clearRocketLockEntity = qtrue;
                            }
                        }

                        if linkedFiring != 0 {
                            i += 1;
                            continue;
                        }

                        if numMuzzles > 1 {
                            let mut nextMuzzle = pVeh.weaponStatus[weaponNum as usize].nextMuzzle;
                            loop {
                                nextMuzzle += 1;
                                if nextMuzzle >= MAX_VEHICLE_MUZZLES as c_int {
                                    nextMuzzle = 0;
                                }
                                if nextMuzzle == pVeh.weaponStatus[weaponNum as usize].nextMuzzle {
                                    break;
                                }
                                if pVeh.m_pVehicleInfo.as_ref().unwrap().weapMuzzle
                                    [nextMuzzle as usize]
                                    == vehWeaponIndex
                                {
                                    pVeh.weaponStatus[weaponNum as usize].nextMuzzle = nextMuzzle;
                                    break;
                                }
                            }
                        }

                        pVeh.m_iMuzzleWait
                            [pVeh.weaponStatus[weaponNum as usize].nextMuzzle as usize] =
                            ctx.world.level.time + delay;
                        pVeh.weaponStatus[weaponNum as usize].ammo -= (*vehWeapon).iAmmoPerShot;
                        if !pVeh.m_pParentEntity.is_null()
                            && !(pVeh.m_pParentEntity as *mut gentity_t).is_null()
                            && !(*(pVeh.m_pParentEntity as *mut gentity_t)).client.is_null()
                        {
                            (*((*(pVeh.m_pParentEntity as *mut gentity_t)).client))
                                .ps
                                .ammo[weaponNum as usize] =
                                pVeh.weaponStatus[weaponNum as usize].ammo;
                        }
                        // Oracle `goto tryFire;` — after firing one muzzle in the
                        // non-linked case, bail out of the muzzle loop entirely
                        // (skipping the cumulative ammo/delay pass) so only one
                        // muzzle fires per frame, round-robin.
                        break 'try_fire;
                    }
                    i += 1;
                }

                if cumulativeAmmo != 0 {
                    pVeh.weaponStatus[weaponNum as usize].ammo -= cumulativeAmmo;
                    if !pVeh.m_pParentEntity.is_null()
                        && !(pVeh.m_pParentEntity as *mut gentity_t).is_null()
                        && !(*(pVeh.m_pParentEntity as *mut gentity_t)).client.is_null()
                    {
                        (*((*(pVeh.m_pParentEntity as *mut gentity_t)).client))
                            .ps
                            .ammo[weaponNum as usize] = pVeh.weaponStatus[weaponNum as usize].ammo;
                    }
                }
                if cumulativeDelay != 0 {
                    let mut i = 0;
                    while i < MAX_VEHICLE_MUZZLES as c_int {
                        if pVeh.m_pVehicleInfo.as_ref().unwrap().weapMuzzle[i as usize]
                            != vehWeaponIndex
                        {
                            i += 1;
                            continue;
                        }
                        pVeh.m_iMuzzleWait[i as usize] = ctx.world.level.time + cumulativeDelay;
                        i += 1;
                    }
                }
            } // 'try_fire (oracle `tryFire:` label)

            if clearRocketLockEntity != 0 {
                (*((*ent).client)).ps.rocketLockIndex = ENTITYNUM_NONE as c_int;
                (*((*ent).client)).ps.rocketLockTime = (0) as f32;
                (*((*ent).client)).ps.rocketTargetTime = (0) as f32;
            }

            if !vehWeapon.is_null() && muzzlesFired > 0 {
                G_VehMuzzleFireFX(
                    ctx,
                    ctx.entity_id_of(ent).unwrap(),
                    ctx.entity_id_of(missile),
                    muzzlesFired,
                );
            }
        }
    }
}

/// Raven `FireWeapon`.
///
/// Source: `oracle/codemp/game/g_weapon.c:4424-4608`
pub fn FireWeapon(ctx: &mut GameContext, ent: Option<EntityId>, altFire: bool) {
    // Raven's `FireWeapon` is only called for a valid firing entity; the body
    // dereferences `ent->client` unconditionally (g_weapon.c:4426), so `ent`
    // resolves to a live handle here (matches the existing `.unwrap()` sites).
    let ent_eid = ent.unwrap();
    let ent_num = ctx.world.entity(ent_eid).s.number;
    let weapon = ctx.world.entity(ent_eid).s.weapon;
    unsafe {
        // FLAG: firing ent may be an NPC (pool client); deref its client raw.
        let ent_client = ctx.world.entity(ent_eid).client;

        if (*ent_client).ps.powerups[PW_QUAD as usize] != 0 {
            ctx.world.globals.s_quadFactor = ctx.world.cvars.g_quadfactor.value;
        } else {
            ctx.world.globals.s_quadFactor = 1.0f32;
        }

        if weapon != WP_SABER && weapon != WP_STUN_BATON && weapon != WP_MELEE {
            if weapon == WP_FLECHETTE {
                (*ent_client).accuracy_shots += FLECHETTE_SHOTS;
            } else {
                (*ent_client).accuracy_shots += 1;
            }
        }

        if !ent_client.is_null() && (*ent_client).NPC_class == CLASS_VEHICLE {
            FireVehicleWeapon(ctx, ent_eid, altFire);
            return;
        }

        // Raven's file statics `forward, vright, up` / `muzzle` (g_weapon.c:13-14)
        let forward = ctx.world.globals.forward;
        let vright = ctx.world.globals.vright;
        let up = ctx.world.globals.up;
        let muzzle_out = &mut (*ctx.world_raw()).globals.muzzle; // STAGE-2b: irreducible — &mut world.globals.muzzle out-param aliases the ctx passed to CalcMuzzlePoint.
                                                                 // are `GameGlobals` fields here; seed them exactly where the oracle seeds
                                                                 // its statics so the WP_Fire* readers below observe them.
                                                                 // Source: `oracle/codemp/game/g_weapon.c:4448-4512`

        if weapon == WP_EMPLACED_GUN && (*ent_client).ps.emplacedIndex != 0 {
            let emp_id = EntityId((*ent_client).ps.emplacedIndex as u32);

            if ctx.world.entity(emp_id).inuse != 0 {
                let mut yaw = 0.0f32;
                let mut viewAngCap = [0.0f32; 3];
                let mut override_val = 0;

                viewAngCap = (*ent_client).ps.viewangles;
                if viewAngCap[0] > 40.0f32 {
                    viewAngCap[0] = 40.0f32;
                }

                override_val = BG_EmplacedView(
                    (*ent_client).ps.viewangles,
                    ctx.world.entity(emp_id).s.angles,
                    &mut yaw,
                    ctx.world.entity(emp_id).s.origin2[0],
                );

                if override_val != 0 {
                    viewAngCap[1] = yaw;
                }

                AngleVectors(
                    viewAngCap,
                    Some(&mut ctx.world.globals.forward),
                    Some(&mut ctx.world.globals.vright),
                    Some(&mut ctx.world.globals.up),
                );
            } else {
                AngleVectors(
                    (*ent_client).ps.viewangles,
                    Some(&mut ctx.world.globals.forward),
                    Some(&mut ctx.world.globals.vright),
                    Some(&mut ctx.world.globals.up),
                );
            }
        } else if ent_num < MAX_CLIENTS as c_int
            && (*ent_client).ps.m_iVehicleNum != 0
            && weapon == WP_BLASTER
        {
            let mut vehTurnAngles = [0.0f32; 3];
            let vehEnt_id = EntityId((*ent_client).ps.m_iVehicleNum as u32);

            if ctx.world.entity(vehEnt_id).inuse != 0
                && !ctx.world.entity(vehEnt_id).client.is_null()
                && !ctx.world.entity(vehEnt_id).m_pVehicle.is_null()
            {
                // FLAG: `m_pVehicle` has no accessor; deref raw through the copied pointer.
                let veh = ctx.world.entity(vehEnt_id).m_pVehicle;
                vehTurnAngles = *((*veh).m_vOrientation as *const vec3_t);
                vehTurnAngles[0] = (*ent_client).ps.viewangles[0];
            } else {
                vehTurnAngles = (*ent_client).ps.viewangles;
            }
            if (*ent_client).pers.cmd.rightmove > 0 {
                vehTurnAngles[1] -= 90.0f32;
            } else if (*ent_client).pers.cmd.rightmove < 0 {
                vehTurnAngles[1] += 90.0f32;
            }

            AngleVectors(
                vehTurnAngles,
                Some(&mut ctx.world.globals.forward),
                Some(&mut ctx.world.globals.vright),
                Some(&mut ctx.world.globals.up),
            );
        } else {
            AngleVectors(
                (*ent_client).ps.viewangles,
                Some(&mut ctx.world.globals.forward),
                Some(&mut ctx.world.globals.vright),
                Some(&mut ctx.world.globals.up),
            );
        }

        // C passes the live file-scope forward/vright/up arrays; re-copy after
        // the AngleVectors branch above (the entry-time copies are stale).
        let forward = ctx.world.globals.forward;
        let vright = ctx.world.globals.vright;
        let up = ctx.world.globals.up;
        CalcMuzzlePoint(ctx, ent_eid, forward, vright, up, muzzle_out);

        match weapon {
            WP_STUN_BATON => {
                WP_FireStunBaton(ctx, ent_eid, altFire);
            }
            WP_MELEE => {
                WP_FireMelee(ctx, ent_eid, altFire);
            }
            WP_SABER => {}
            WP_BRYAR_PISTOL => {
                WP_FireBryarPistol(ctx, ent_eid, altFire);
            }
            WP_CONCUSSION => {
                if altFire {
                    WP_FireConcussionAlt(ctx, ent_eid);
                } else {
                    WP_FireConcussion(ctx, ent_eid);
                }
            }
            WP_BRYAR_OLD => {
                WP_FireBryarPistol(ctx, ent_eid, altFire);
            }
            WP_BLASTER => {
                WP_FireBlaster(ctx, ent_eid, altFire);
            }
            WP_DISRUPTOR => {
                WP_FireDisruptor(ctx, Some(ent_eid), altFire);
            }
            WP_BOWCASTER => {
                WP_FireBowcaster(ctx, ent_eid, altFire);
            }
            WP_REPEATER => {
                WP_FireRepeater(ctx, ent_eid, altFire);
            }
            WP_DEMP2 => {
                WP_FireDEMP2(ctx, ent_eid, altFire);
            }
            WP_FLECHETTE => {
                WP_FireFlechette(ctx, ent_eid, altFire);
            }
            WP_ROCKET_LAUNCHER => {
                WP_FireRocket(ctx, ent_eid, altFire);
            }
            WP_THERMAL => {
                WP_FireThermalDetonator(ctx, ent_eid, altFire);
            }
            WP_TRIP_MINE => {
                WP_PlaceLaserTrap(ctx, ent_eid, altFire);
            }
            WP_DET_PACK => {
                WP_DropDetPack(ctx, Some(ent_eid), altFire);
            }
            WP_EMPLACED_GUN => {
                if !ent_client.is_null() && (*ent_client).ewebIndex != 0 {
                } else {
                    WP_FireEmplaced(ctx, ent_eid, altFire);
                }
            }
            _ => {}
        }
    }

    G_LogWeaponFire(ctx, ent_num, weapon);
}

/// Raven `WP_FireEmplaced`.
///
/// Source: `oracle/codemp/game/g_weapon.c:4611-4660`
pub fn WP_FireEmplaced(ctx: &mut GameContext, ent: EntityId, altFire: bool) {
    // FLAG: firing ent may be an NPC (pool client); deref the client value raw.
    let ent_client = ctx.world.entity(ent).client;
    if ent_client.is_null() {
        return;
    }

    let emplacedIndex = unsafe { (*ent_client).ps.emplacedIndex };
    if emplacedIndex == 0 {
        return;
    }

    let gun_id = EntityId(emplacedIndex as u32);

    if ctx.world.entity(gun_id).inuse == 0 || ctx.world.entity(gun_id).health <= 0 {
        return;
    }

    let mut gunpoint = ctx.world.entity(gun_id).s.origin;
    gunpoint[2] += 46.0f32;

    let mut right = [0.0f32; 3];
    let viewangles = unsafe { (*ent_client).ps.viewangles };
    AngleVectors(viewangles, None, Some(&mut right), None);

    let mut side = 0;
    if ctx.world.entity(gun_id).genericValue10 != 0 {
        _VectorMA(gunpoint, 10.0f32, right, &mut gunpoint);
        side = 0;
    } else {
        _VectorMA(gunpoint, -10.0f32, right, &mut gunpoint);
        side = 1;
    }

    ctx.world.entity_mut(gun_id).genericValue10 = side;
    G_AddEvent(ctx.world.entity_mut(gun_id), (EV_FIRE_WEAPON) as i32, side);

    let mut angs = [0.0f32; 3];
    let mut dir = [0.0f32; 3];
    // Oracle uses the file-static `forward` (set by FireWeapon from the
    // capped/yaw-overridden emplaced view angles), not a fresh
    // AngleVectors of the raw viewangles — matching the sibling fire fns.
    vectoangles(ctx.world.globals.forward, &mut angs);
    AngleVectors(angs, Some(&mut dir), None, None);

    WP_FireEmplacedMissile(ctx, gun_id, gunpoint, dir, altFire, Some(ent));
}

/// Raven `emplaced_gun_use`.
///
/// Source: `oracle/codemp/game/g_weapon.c:4691-4802`
pub fn emplaced_gun_use(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    trace: *mut trace_t,
) {
    if ctx.world.entity(self_).health <= 0 {
        return;
    }

    if ctx.world.entity(self_).activator.is_some() {
        return;
    }

    // Raven derefs `activator` (== `other`) unconditionally below; it is never
    // NULL for this use path (§19: the NULL case is UB — bail out).
    let Some(activator) = other else {
        return;
    };

    // FLAG: activator pool client deref stays raw (read the client pointer value).
    let activator_client = ctx.world.entity(activator).client;
    if activator_client.is_null() {
        return;
    }

    if unsafe { (*activator_client).ps.emplacedTime } > (ctx.world.level.time) as f32 {
        return;
    }

    if unsafe { (*activator_client).ps.forceHandExtend } != (HANDEXTEND_NONE) as i32 {
        return;
    }

    let zoffset = 50.0f32;
    let self_origin_z = ctx.world.entity(self_).s.origin[2];
    if unsafe { (*activator_client).ps.origin[2] } > self_origin_z + zoffset - 8.0f32 {
        return;
    }

    if (unsafe { (*activator_client).ps.pm_flags } & PMF_DUCKED) != 0 {
        return;
    }

    if unsafe { (*activator_client).ps.isJediMaster } != 0 {
        return;
    }

    let mut vLen = [0.0f32; 3];
    let self_origin = ctx.world.entity(self_).s.origin;
    let act_ps_origin = unsafe { (*activator_client).ps.origin };
    _VectorSubtract(self_origin, act_ps_origin, &mut vLen);
    let ownLen = VectorLength(vLen);

    if ownLen > 64.0f32 {
        return;
    }

    let mut fwd1 = [0.0f32; 3];
    let mut fwd2 = [0.0f32; 3];
    let act_viewangles = unsafe { (*activator_client).ps.viewangles };
    AngleVectors(act_viewangles, Some(&mut fwd1), None, None);
    let self_pos1 = ctx.world.entity(self_).pos1;
    AngleVectors(self_pos1, Some(&mut fwd2), None, None);

    let mut dot = _DotProduct(fwd1, fwd2);

    if dot < -0.2f32 {
        TryHeal(ctx, Some(activator), Some(self_));
        return;
    }

    let self_origin = ctx.world.entity(self_).s.origin;
    let act_ps_origin = unsafe { (*activator_client).ps.origin };
    _VectorSubtract(self_origin, act_ps_origin, &mut fwd1);
    VectorNormalize(&mut fwd1);

    dot = _DotProduct(fwd1, fwd2);

    if dot < 0.6f32 {
        TryHeal(ctx, Some(activator), Some(self_));
        return;
    }

    ctx.world.entity_mut(self_).genericValue1 = 1;

    let oldWeapon = ctx.world.entity(activator).s.weapon;
    let self_weapon = ctx.world.entity(self_).s.weapon;
    let self_num = ctx.world.entity(self_).s.number;

    unsafe {
        (*activator_client).ps.weapon = self_weapon;
        (*activator_client).ps.weaponstate = (WEAPON_READY) as i32;
        (*activator_client).ps.stats[STAT_WEAPONS as usize] |= 1 << WP_EMPLACED_GUN;

        (*activator_client).ps.emplacedIndex = self_num;
    }

    let act_num = ctx.world.entity(activator).s.number;
    {
        let s = ctx.world.entity_mut(self_);
        s.s.emplacedOwner = act_num;
        s.s.activeForcePass = (NUM_FORCE_POWERS + 1) as i32;
        s.s.weapon = oldWeapon;
    }

    ctx.world.entity_mut(activator).r.ownerNum = self_num;
    ctx.world.entity_mut(self_).activator = Some(activator);

    let mut anglesToOwner = [0.0f32; 3];
    let self_currentOrigin = ctx.world.entity(self_).r.currentOrigin;
    let act_ps_origin = unsafe { (*activator_client).ps.origin };
    _VectorSubtract(self_currentOrigin, act_ps_origin, &mut anglesToOwner);
    vectoangles(anglesToOwner, &mut anglesToOwner);
}

/// Raven `emplaced_gun_realuse`.
///
/// Source: `oracle/codemp/game/g_weapon.c:4804-4807`
pub fn emplaced_gun_realuse(
    ctx: &mut GameContext,
    self_: EntityId,
    other: Option<EntityId>,
    activator: Option<EntityId>,
) {
    // Raven: `activator` is unused by the body.
    emplaced_gun_use(ctx, self_, other, std::ptr::null_mut());
}

/// Raven `emplaced_gun_pain`.
///
/// Source: `oracle/codemp/game/g_weapon.c:4810-4823`
pub fn emplaced_gun_pain(
    ctx: &mut GameContext,
    self_: EntityId,
    attacker: Option<EntityId>,
    damage: c_int,
) {
    let health = ctx.world.entity(self_).health;
    ctx.world.entity_mut(self_).s.health = health;

    if health <= 0 {
        // death effect.. for now taken care of on cgame
    } else {
        // if we have a pain behavior set then use it I guess
        G_ActivateBehavior(ctx, Some(self_), bSet_t::BSET_PAIN as c_int);
    }
}

/// Raven `emplaced_gun_update`.
///
/// Source: `oracle/codemp/game/g_weapon.c:4828-4927`
pub fn emplaced_gun_update(ctx: &mut GameContext, self_: EntityId) {
    let now = ctx.world.level.time;
    if ctx.world.entity(self_).health < 1 && ctx.world.entity(self_).genericValue5 == 0 {
        if (ctx.world.entity(self_).spawnflags & (EMPLACED_CANRESPAWN as u32) as i32) != 0 {
            let count = ctx.world.entity(self_).count;
            ctx.world.entity_mut(self_).genericValue5 = now + 4000 + count;
        }
    } else if ctx.world.entity(self_).health < 1 && ctx.world.entity(self_).genericValue5 < now {
        let e = ctx.world.entity_mut(self_);
        e.s.time = 0;
        e.genericValue4 = 0;
        e.genericValue3 = 0;
        e.health = (EMPLACED_GUN_HEALTH as f32 * 0.4f32) as c_int;
        e.s.health = e.health;
    }

    if ctx.world.entity(self_).genericValue4 != 0
        && ctx.world.entity(self_).genericValue4 < 2
        && ctx.world.entity(self_).s.time < now
    {
        let mut puffAngle = [0.0f32; 3];
        let mut explOrg = [0.0f32; 3];

        VectorSet(&mut puffAngle, 0.0f32, 0.0f32, 1.0f32);

        let currentOrigin = ctx.world.entity(self_).r.currentOrigin;
        explOrg = currentOrigin;
        explOrg[2] += 16.0f32;

        G_PlayEffect((EFFECT_EXPLOSION_DETPACK) as i32, explOrg, puffAngle);

        let r = ctx.world.bg_state.rng.Q_irand(2500, 3500);
        ctx.world.entity_mut(self_).genericValue3 = now + r;

        let currentOrigin = ctx.world.entity(self_).r.currentOrigin;
        let splashDamage = ctx.world.entity(self_).splashDamage;
        let splashRadius = ctx.world.entity(self_).splashRadius;
        G_RadiusDamage(
            ctx,
            currentOrigin,
            Some(self_),
            splashDamage as f32,
            splashRadius as f32,
            Some(self_),
            None,
            MOD_UNKNOWN as c_int,
        );

        let e = ctx.world.entity_mut(self_);
        e.s.time = -1;

        e.genericValue4 = 2;
    }

    if ctx.world.entity(self_).genericValue3 > now {
        if ctx.world.entity(self_).genericValue2 < now {
            let mut puffAngle = [0.0f32; 3];
            let mut smokeOrg = [0.0f32; 3];

            VectorSet(&mut puffAngle, 0.0f32, 0.0f32, 1.0f32);
            let currentOrigin = ctx.world.entity(self_).r.currentOrigin;
            smokeOrg = currentOrigin;

            smokeOrg[2] += 60.0f32;

            G_PlayEffect((EFFECT_SMOKE) as i32, smokeOrg, puffAngle);
            let r = ctx.world.bg_state.rng.Q_irand(250, 400);
            ctx.world.entity_mut(self_).genericValue2 = now + r;
        }
    }

    let activator = ctx.world.entity(self_).activator;
    // FLAG: activator pool client deref stays raw (read the client pointer value).
    let activator_client = match activator {
        Some(a) => ctx.world.entity(a).client,
        None => core::ptr::null_mut(),
    };

    let mut ownLen = 0.0f32;
    if activator.is_some()
        && !activator_client.is_null()
        && ctx.world.entity(activator.unwrap()).inuse != 0
    {
        let mut vLen = [0.0f32; 3];
        let self_origin = ctx.world.entity(self_).s.origin;
        let act_ps_origin = unsafe { (*activator_client).ps.origin };
        _VectorSubtract(self_origin, act_ps_origin, &mut vLen);
        ownLen = VectorLength(vLen);

        if (unsafe { (*activator_client).pers.cmd.buttons } & BUTTON_USE) == 0
            && ctx.world.entity(self_).genericValue1 != 0
        {
            ctx.world.entity_mut(self_).genericValue1 = 0;
        }

        if (unsafe { (*activator_client).pers.cmd.buttons } & BUTTON_USE) != 0
            && ctx.world.entity(self_).genericValue1 == 0
        {
            unsafe {
                (*activator_client).ps.emplacedIndex = 0;
                (*activator_client).ps.saberHolstered = 0;
            }
            ctx.world.entity_mut(self_).nextthink = now + 50;
            return;
        }
    }

    if (activator.is_some() && !activator_client.is_null())
        && (ctx.world.entity(activator.unwrap()).inuse == 0
            || unsafe { (*activator_client).ps.emplacedIndex } != ctx.world.entity(self_).s.number
            || ctx.world.entity(self_).genericValue4 != 0
            || ownLen > 64.0f32)
    {
        let self_weapon = ctx.world.entity(self_).s.weapon;
        unsafe {
            (*activator_client).ps.stats[STAT_WEAPONS as usize] &= !(1 << WP_EMPLACED_GUN);
        }
        let oldWeap = unsafe { (*activator_client).ps.weapon };
        unsafe {
            (*activator_client).ps.weapon = self_weapon;
        }
        ctx.world.entity_mut(self_).s.weapon = oldWeap;
        unsafe {
            (*activator_client).ps.emplacedTime = (now + 1000) as f32;
            (*activator_client).ps.emplacedIndex = 0;
            (*activator_client).ps.saberHolstered = 0;
        }
        ctx.world.entity_mut(activator.unwrap()).r.ownerNum = (ENTITYNUM_NONE as u32) as i32;
        ctx.world.entity_mut(self_).activator = None;

        ctx.world.entity_mut(self_).s.activeForcePass = 0;
    } else if activator.is_some() && !activator_client.is_null() {
        unsafe {
            (*activator_client).ps.weapon = WP_EMPLACED_GUN;
            (*activator_client).ps.weaponstate = (WEAPON_READY) as i32;
        }
    }
    ctx.world.entity_mut(self_).nextthink = now + 50;
}

/// Raven `emplaced_gun_die`.
///
/// Source: `oracle/codemp/game/g_weapon.c:4930-4942`
pub fn emplaced_gun_die(
    ctx: &mut GameContext,
    self_: EntityId,
    inflictor: Option<EntityId>,
    attacker: Option<EntityId>,
    damage: c_int,
    r#mod: c_int,
) {
    if ctx.world.entity(self_).genericValue4 != 0 {
        return;
    }

    let now = ctx.world.level.time;
    let e = ctx.world.entity_mut(self_);
    e.genericValue4 = 1;
    e.s.time = now + 3000;
    e.genericValue5 = 0;
}

/// Raven `SP_emplaced_gun`.
///
/// Source: `oracle/codemp/game/g_weapon.c:4944-5027`
pub fn SP_emplaced_gun(ctx: &mut GameContext, ent: EntityId) {
    let name = c"models/map_objects/mp/turret_chair.glm";

    RegisterItem(ctx, BG_FindItemForWeapon(WP_EMPLACED_GUN));

    {
        let e = ctx.world.entity_mut(ent);
        e.r.contents = CONTENTS_SOLID;
        e.s.solid = mp_qshared::common::mp::botlib::solid_t::solid_t::SOLID_BBOX as c_int;

        e.genericValue5 = 0;

        VectorSet(&mut e.r.mins, -30.0f32, -20.0f32, 8.0f32);
        VectorSet(&mut e.r.maxs, 30.0f32, 20.0f32, 60.0f32);
    }

    let origin = ctx.world.entity(ent).s.origin;
    let mins = ctx.world.entity(ent).r.mins;
    let maxs = ctx.world.entity(ent).r.maxs;
    let ent_num = ctx.world.entity(ent).s.number;
    let mut down = origin;
    down[2] -= 1024.0f32;

    let mut tr: trace_t = unsafe { std::mem::zeroed() };
    trap::Trace(
        ctx.engine,
        mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
            &mut tr,
            &origin as *const vec3_t,
            &mins as *const vec3_t,
            &maxs as *const vec3_t,
            &down as *const vec3_t,
            ent_num,
            MASK_SOLID,
        ),
    );

    if tr.fraction != 1.0f32 && tr.allsolid == 0 && tr.startsolid == 0 {
        ctx.world.entity_mut(ent).s.origin = tr.endpos;
    }

    {
        let e = ctx.world.entity_mut(ent);
        e.spawnflags |= 4; // deadsolid

        e.health = EMPLACED_GUN_HEALTH as c_int;

        if (e.spawnflags & (EMPLACED_CANRESPAWN as u32) as i32) != 0 {
            e.health = (e.health as f32 * 0.4f32) as c_int;
        }

        e.maxHealth = e.health;
    }
    G_ScaleNetHealth(ctx.world.entity_mut(ent));

    {
        let e = ctx.world.entity_mut(ent);
        e.genericValue4 = 0;

        e.takedamage = qtrue;
        e.pain = Some(EntPain::emplaced_gun_pain).into();
        e.die = Some(EntDie::emplaced_gun_die).into();

        e.splashDamage = 80;
        e.splashRadius = 128;
    }

    let mut count_out: c_int = 0;
    G_SpawnInt(
        ctx,
        c"count".as_ptr() as *const c_char,
        c"600".as_ptr() as *const c_char,
        &mut count_out,
    );
    ctx.world.entity_mut(ent).count = count_out;

    let mut constraint_out: f32 = 0.0;
    G_SpawnFloat(
        ctx,
        c"constraint".as_ptr() as *const c_char,
        c"60".as_ptr() as *const c_char,
        &mut constraint_out,
    );
    ctx.world.entity_mut(ent).s.origin2[0] = constraint_out;

    let modelidx = G_ModelIndex(name.to_str().unwrap());
    let angles = ctx.world.entity(ent).s.angles;
    let origin = ctx.world.entity(ent).s.origin;
    {
        let e = ctx.world.entity_mut(ent);
        e.s.modelindex = modelidx;
        e.s.modelGhoul2 = 1;
        e.s.g2radius = 110;

        e.s.weapon = WP_EMPLACED_GUN;
    }

    G_SetOrigin(ctx.world.entity_mut(ent), origin);

    let now = ctx.world.level.time;
    {
        let e = ctx.world.entity_mut(ent);
        e.pos1 = angles;
        e.r.currentAngles = angles;
        e.s.apos.trBase = angles;

        e.think = Some(EntThink::emplaced_gun_update).into();
        e.nextthink = now + 50;

        e.use_ = Some(EntUse::emplaced_gun_realuse).into();

        e.r.svFlags |= (SVF_PLAYER_USABLE as u32) as i32;

        e.s.pos.trType = TR_STATIONARY;

        e.s.owner = ((MAX_CLIENTS + 1) as u32) as i32;
        e.s.shouldtarget = qtrue;
    }

    let ent_ptr = &mut ctx.world.g_entities[ent.index()] as *mut gentity_t;
    trap::LinkEntity(
        ctx.engine,
        mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(ent_ptr.cast()),
    );
}
