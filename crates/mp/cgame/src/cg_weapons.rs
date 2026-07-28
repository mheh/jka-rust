//! Port of `oracle/codemp/cgame/cg_weapons.c` — weapon selection, the viewmodel, and weapon fire effects. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case, non_upper_case_globals)]

use core::ffi::{c_int, c_void};
use core::ptr::null_mut;

use mp_bg::bg_misc::BG_GiveMeVectorFromMatrix;
use mp_bg::public::anim_number::animNumber_t;
use mp_bg::public::bg_itemlist::bg_itemlist;
use mp_bg::public::entity_flags::{EF_ALT_FIRING, EF_JETPACK_ACTIVE};
use mp_bg::public::entity_type::entityType_t::{ET_NPC, ET_PLAYER};
use mp_bg::public::force_hand_anims::forceHandAnims_t::HANDEXTEND_NONE;
use mp_bg::public::item_kind::ItemKind;
use mp_bg::public::stat_index::statIndex_t::{STAT_HEALTH, STAT_WEAPONS};
use mp_bg::public::viewheight::{CROUCH_VIEWHEIGHT, DEFAULT_VIEWHEIGHT};
use mp_bg::weapons::weapon_data::weaponData;
use mp_bg::weapons::weapon_t::{
    WP_DEMP2, WP_DET_PACK, WP_DISRUPTOR, WP_EMPLACED_GUN, WP_MELEE, WP_SABER, WP_STUN_BATON,
};
use mp_bg::weapons::wp_muzzle_point::WP_MuzzlePoint;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::qcommon::player_state::MAX_WEAPONS;
use mp_qshared::shared::q_math::{
    _VectorCopy, _VectorMA, AngleVectors, ByteToDir, VectorClear, PITCH, ROLL, YAW,
};
use mp_qshared::shared::{mdxaBone_t, qfalse, qtrue, vec3_t, Eorientations, MAX_CLIENTS_I32};

use crate::local::centity_s::centity_t;
use crate::local::client_info_t::clientInfo_t;
use crate::trap;
use crate::world::cg_context::CgContext;
use crate::world::cg_world::CgWorld;

// ---------------------------------------------------------------------------
// FILE-SCOPE CONSTANTS
// Source: `oracle/codemp/cgame/cg_weapons.c:126,890-892`
// ---------------------------------------------------------------------------

/// Raven `ICON_WEAPONS` — the weapon-prong selector for the icon HUD's
/// `cgs.media.currentBackground`.
/// Source: `oracle/codemp/cgame/cg_weapons.c:890`
pub const ICON_WEAPONS: c_int = 0;

/// Raven `ICON_FORCE`.
/// Source: `oracle/codemp/cgame/cg_weapons.c:891`
pub const ICON_FORCE: c_int = 1;

/// Raven `ICON_INVENTORY`.
/// Source: `oracle/codemp/cgame/cg_weapons.c:892`
pub const ICON_INVENTORY: c_int = 2;

// `cg_local.h` timing `#define`s this file reads. They have no ported
// cross-crate home yet, so they land beside their readers — the treatment
// `cg_players.rs` gave `RF_THIRD_PERSON`.

/// Raven `LAND_DEFLECT_TIME` — how long the landing dip drops the viewmodel.
/// Source: `oracle/codemp/cgame/cg_local.h:31`
pub const LAND_DEFLECT_TIME: c_int = 150;

/// Raven `LAND_RETURN_TIME` — the recovery tail after the dip.
/// Source: `oracle/codemp/cgame/cg_local.h:32`
pub const LAND_RETURN_TIME: c_int = 300;

/// Raven `WEAPON_SELECT_TIME` — how long a weapon/force/inventory pick keeps
/// the icon HUD up, in msec.
/// Source: `oracle/codemp/cgame/cg_local.h:36`
pub const WEAPON_SELECT_TIME: c_int = 1400;

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

/// Raven `CG_SetGhoul2InfoRef` — copies the ghoul2 half of one `refEntity_t`
/// onto another.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:15-21`
pub fn CG_SetGhoul2InfoRef(ent: &mut refEntity_t, s1: &refEntity_t) {
    ent.ghoul2 = s1.ghoul2;
    _VectorCopy(s1.modelScale, &mut ent.modelScale);
    ent.radius = s1.radius;
    _VectorCopy(s1.angles, &mut ent.angles);
}

/// Raven `CG_MapTorsoToWeaponFrame` — which viewmodel frame goes with the
/// torso animation we're on. `-1` means "no matching frame".
///
/// `ci` is Raven's unused parameter; the busy-holster block is gated on
/// `#define WEAPON_FORCE_BUSY_HOLSTER` (`cg_weapons.c:126`), which this file
/// always defines, so it is unconditional here.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:140-202`
pub fn CG_MapTorsoToWeaponFrame(
    world: &mut CgWorld,
    _ci: &clientInfo_t,
    frame: c_int,
    animNum: c_int,
) -> c_int {
    // no snapshot means no hand extension, which is the reset arm below
    let forceHandExtend = world
        .cg
        .snap_ref()
        .map_or(HANDEXTEND_NONE as c_int, |snap| snap.ps.forceHandExtend);

    if forceHandExtend != HANDEXTEND_NONE as c_int || world.weapons.cgWeapFrameTime > world.cg.time
    {
        // the reason for the after delay is so that it doesn't snap the weapon frame to the "idle" (0) frame
        // for a very quick moment
        if world.weapons.cgWeapFrame < 6 {
            world.weapons.cgWeapFrame = 6;
            world.weapons.cgWeapFrameTime = world.cg.time + 10;
        }

        if world.weapons.cgWeapFrameTime < world.cg.time && world.weapons.cgWeapFrame < 10 {
            world.weapons.cgWeapFrame += 1;
            world.weapons.cgWeapFrameTime = world.cg.time + 10;
        }

        if forceHandExtend != HANDEXTEND_NONE as c_int && world.weapons.cgWeapFrame == 10 {
            world.weapons.cgWeapFrameTime = world.cg.time + 100;
        }

        return world.weapons.cgWeapFrame;
    } else {
        world.weapons.cgWeapFrame = 0;
        world.weapons.cgWeapFrameTime = 0;
    }

    let animations = &world.bg_state.bgHumanoidAnimations;

    if animNum == animNumber_t::TORSO_DROPWEAP1 as c_int {
        let firstFrame = animations[animNum as usize].firstFrame as c_int;
        if frame >= firstFrame && frame < firstFrame + 5 {
            return frame - firstFrame + 6;
        }
    } else if animNum == animNumber_t::TORSO_RAISEWEAP1 as c_int {
        let firstFrame = animations[animNum as usize].firstFrame as c_int;
        if frame >= firstFrame && frame < firstFrame + 4 {
            return frame - firstFrame + 6 + 4;
        }
    } else if animNum == animNumber_t::BOTH_ATTACK1 as c_int
        || animNum == animNumber_t::BOTH_ATTACK2 as c_int
        || animNum == animNumber_t::BOTH_ATTACK3 as c_int
        || animNum == animNumber_t::BOTH_ATTACK4 as c_int
        || animNum == animNumber_t::BOTH_ATTACK10 as c_int
        || animNum == animNumber_t::BOTH_THERMAL_THROW as c_int
    {
        let firstFrame = animations[animNum as usize].firstFrame as c_int;
        if frame >= firstFrame && frame < firstFrame + 6 {
            return 1 + (frame - firstFrame);
        }
    }

    -1
}

/// Raven `CG_CalculateWeaponPosition` — where the viewmodel sits this frame:
/// the view origin/angles plus bob, the landing dip and the idle drift.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:210-255`
pub fn CG_CalculateWeaponPosition(world: &CgWorld, origin: &mut vec3_t, angles: &mut vec3_t) {
    let mut scale: f32;

    _VectorCopy(world.cg.refdef.vieworg, origin);
    _VectorCopy(world.cg.refdef.viewangles, angles);

    // on odd legs, invert some angles
    if (world.cg.bobcycle & 1) != 0 {
        scale = -world.cg.xyspeed;
    } else {
        scale = world.cg.xyspeed;
    }

    // gun angles from bobbing
    angles[ROLL] = (angles[ROLL] as f64 + (scale * world.cg.bobfracsin) as f64 * 0.005) as f32;
    angles[YAW] = (angles[YAW] as f64 + (scale * world.cg.bobfracsin) as f64 * 0.01) as f32;
    angles[PITCH] =
        (angles[PITCH] as f64 + (world.cg.xyspeed * world.cg.bobfracsin) as f64 * 0.005) as f32;

    // drop the weapon when landing
    let delta = world.cg.time - world.cg.landTime;
    if delta < LAND_DEFLECT_TIME {
        origin[2] = (origin[2] as f64
            + world.cg.landChange as f64 * 0.25 * delta as f64 / LAND_DEFLECT_TIME as f64)
            as f32;
    } else if delta < LAND_DEFLECT_TIME + LAND_RETURN_TIME {
        origin[2] = (origin[2] as f64
            + world.cg.landChange as f64
                * 0.25
                * (LAND_DEFLECT_TIME + LAND_RETURN_TIME - delta) as f64
                / LAND_RETURN_TIME as f64) as f32;
    }

    // Raven's stair-climb drop sits under `#if 0` — dead source, not ported.

    // idle drift
    scale = world.cg.xyspeed + 40.0;
    let fracsin = (world.cg.time as f64 * 0.001).sin() as f32;
    angles[ROLL] = (angles[ROLL] as f64 + (scale * fracsin) as f64 * 0.01) as f32;
    angles[YAW] = (angles[YAW] as f64 + (scale * fracsin) as f64 * 0.01) as f32;
    angles[PITCH] = (angles[PITCH] as f64 + (scale * fracsin) as f64 * 0.01) as f32;
}

/// Raven `CG_LightningBolt` — a no-op gate.
///
/// Raven: "NOTENOTE No lightning gun-ish stuff yet." Every use of the local
/// beam (the CPMA "true lightning" trace and the impact flare) is commented
/// out, so all that survives is the durational-weapon test and its early
/// return. Kept because it is Raven's shape and the later waves' callers still
/// make the call.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:269-366`
pub fn CG_LightningBolt(cent: &centity_t, _origin: &vec3_t) {
    // Must be a durational weapon that continuously generates an effect.
    let durational =
        cent.currentState.weapon == WP_DEMP2 && (cent.currentState.eFlags & EF_ALT_FIRING) != 0;
    if !durational {
        return;
    }

    // Raven zeroes a local `refEntity_t beam` here and then never uses it.
}

/// Raven `CG_AddWeaponWithPowerups` — hands the gun to the renderer, then a
/// second time in an electrocution shell while the player is being shocked.
///
/// `powerups` is Raven's unused parameter.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:374-392`
pub fn CG_AddWeaponWithPowerups(ctx: &mut CgContext, gun: &mut refEntity_t, _powerups: c_int) {
    let engine = ctx.engine;

    // add powerup effects
    trap::R_AddRefEntityToScene(engine, gun);

    if ctx.world.cg.predictedPlayerState.electrifyTime > ctx.world.cg.time {
        // add electrocution shell
        let preShader = gun.customShader;
        if (ctx.world.bg_state.rng.rand() & 1) != 0 {
            gun.customShader = ctx.world.cgs.media.electricBodyShader;
        } else {
            gun.customShader = ctx.world.cgs.media.electricBody2Shader;
        }
        trap::R_AddRefEntityToScene(engine, gun);
        gun.customShader = preShader; // set back just to be safe
    }
}

/// Raven `CG_DrawIconBackground` — drives the icon HUD's open/close animation.
///
/// Every `CG_DrawPic` in Raven's body is commented out, so what is left is the
/// `cg.iconSelectTime`/`iconHUDActive`/`iconHUDPercent` bookkeeping the rest of
/// the HUD reads. Raven's now-dead `x2`/`y2`/`xAdd`/`height`/`drawType` locals
/// go with the draws.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:895-1022`
pub fn CG_DrawIconBackground(world: &mut CgWorld) {
    // invenSelectTime/forceSelectTime are floats in cg_t but weaponSelectTime
    // is an int - Raven sums in each field's own width, then floats
    let inTime = world.cg.invenSelectTime + WEAPON_SELECT_TIME as f32;
    let wpTime = (world.cg.weaponSelectTime + WEAPON_SELECT_TIME) as f32;
    let fpTime = world.cg.forceSelectTime + WEAPON_SELECT_TIME as f32;

    // don't display if dead
    // no snapshot means no health to read, so nothing to draw
    let health = world
        .cg
        .snap_ref()
        .map_or(0, |snap| snap.ps.stats[STAT_HEALTH as usize]);
    if health <= 0 {
        return;
    }

    if world.cvars.cg_hudFiles.integer != 0 {
        // simple hud
        return;
    }

    if inTime > wpTime {
        world.cg.iconSelectTime = world.cg.invenSelectTime;
    } else {
        world.cg.iconSelectTime = world.cg.weaponSelectTime as f32;
    }

    if fpTime > inTime && fpTime > wpTime {
        world.cg.iconSelectTime = world.cg.forceSelectTime;
    }

    // Time is up for the HUD to display
    if (world.cg.iconSelectTime + WEAPON_SELECT_TIME as f32) < world.cg.time as f32 {
        // The time is up, but we still need to move the prongs back to their original position
        if world.cg.iconHUDActive != qfalse {
            let t = (world.cg.time as f32 - (world.cg.iconSelectTime + WEAPON_SELECT_TIME as f32))
                as c_int;
            world.cg.iconHUDPercent = t as f32 / 130.0;
            world.cg.iconHUDPercent = 1.0 - world.cg.iconHUDPercent;

            if world.cg.iconHUDPercent < 0.0 {
                world.cg.iconHUDActive = qfalse;
                world.cg.iconHUDPercent = 0.0;
            }
        }

        return;
    }

    if world.cg.iconHUDActive == qfalse {
        // Raven's `t` is an int, so the float subtraction truncates before the divide
        let t = (world.cg.time as f32 - world.cg.iconSelectTime) as c_int;
        world.cg.iconHUDPercent = t as f32 / 130.0;

        // Calc how far into opening sequence we are
        if world.cg.iconHUDPercent > 1.0 {
            world.cg.iconHUDActive = qtrue;
            world.cg.iconHUDPercent = 1.0;
        } else if world.cg.iconHUDPercent < 0.0 {
            world.cg.iconHUDPercent = 0.0;
        }
    } else {
        world.cg.iconHUDPercent = 1.0;
    }

    // The side-prong draws that closed Raven's body are commented out too.
}

/// Raven `CG_WeaponCheck` — has the snapshot player enough ammo to fire weapon
/// `weap` either way?
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:1024-1033`
pub fn CG_WeaponCheck(world: &CgWorld, weap: c_int) -> bool {
    // no snapshot means no ammo we can vouch for, so the weapon isn't usable
    let Some(snap) = world.cg.snap_ref() else {
        return false;
    };
    let wd = &weaponData[weap as usize];

    if snap.ps.ammo[wd.ammoIndex as usize] < wd.energyPerShot
        && snap.ps.ammo[wd.ammoIndex as usize] < wd.altEnergyPerShot
    {
        return false;
    }

    true
}

/// Raven `CG_WeaponSelectable` — can the weapon-select HUD land on weapon `i`?
/// Ammo, the planted-det-pack exception, and actually owning it.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:1040-1066`
pub fn CG_WeaponSelectable(world: &CgWorld, i: c_int) -> bool {
    // Raven's ammo-only precheck is commented out; the real ammo test is below.
    if i == 0 {
        return false;
    }

    let ps = &world.cg.predictedPlayerState;
    let wd = &weaponData[i as usize];

    if ps.ammo[wd.ammoIndex as usize] < wd.energyPerShot
        && ps.ammo[wd.ammoIndex as usize] < wd.altEnergyPerShot
    {
        return false;
    }

    if i == WP_DET_PACK && ps.ammo[wd.ammoIndex as usize] < 1 && ps.hasDetPackPlanted == qfalse {
        return false;
    }

    if (ps.stats[STAT_WEAPONS as usize] & (1 << i)) == 0 {
        return false;
    }

    true
}

/// Raven `CG_GetClientWeaponMuzzleBoltPoint` — world position of client
/// `clIndex`'s weapon muzzle bolt. `to` is left untouched when the client has
/// no ghoul2 weapon model, exactly as Raven's early returns leave it.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:1795-1815`
pub fn CG_GetClientWeaponMuzzleBoltPoint(ctx: &mut CgContext, clIndex: c_int, to: &mut vec3_t) {
    let mut boltMatrix = mdxaBone_t {
        matrix: [[0.0; 4]; 3],
    };

    if clIndex < 0 || clIndex >= MAX_CLIENTS_I32 {
        return;
    }

    let engine = ctx.engine;
    let idx = clIndex as usize;
    let cent = ctx.world.entity(idx);
    let ghoul2 = cent.ghoul2;
    let turAngles = cent.turAngles;
    let lerpOrigin = cent.lerpOrigin;
    let modelScale = cent.modelScale;

    // Raven's null-check on `cent` itself drops — an owned array element can't
    // be null (§B5). `HasGhoul2ModelOnIndex` takes the ADDRESS of the instance
    // slot, not the token: Raven passes `&(cent->ghoul2)` and the engine casts
    // the word to `CGhoul2Info_v **` (`cl_cgame.cpp:1434`).
    if ghoul2.is_null()
        || !trap::G2_HaveWeGhoul2Models(engine, ghoul2)
        || !trap::G2API_HasGhoul2ModelOnIndex(
            engine,
            &mut ctx.world.entity_mut(idx).ghoul2 as *mut *mut c_void as *mut c_void,
            1,
        )
    {
        return;
    }

    let time = ctx.world.cg.time;
    trap::G2API_GetBoltMatrix(
        engine,
        ghoul2,
        1,
        0,
        &mut boltMatrix,
        &turAngles,
        &lerpOrigin,
        time,
        Some(&mut ctx.world.cgs.gameModels[0]),
        &modelScale,
    );
    BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, to);
}

/// Raven `CG_VehicleWeaponImpact` — does this missile belong to a vehicle
/// weapon with its own impact effect? If so it plays that effect instead of the
/// generic one.
///
/// Raven: "see if this is a missile entity that's owned by a vehicle and should
/// do a special, overridden impact effect".
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:1951-1964`
pub fn CG_VehicleWeaponImpact(ctx: &mut CgContext, cent: &centity_t) -> bool {
    // hack so we know we're a vehicle Weapon shot
    if (cent.currentState.eFlags & EF_JETPACK_ACTIVE) != 0
        && cent.currentState.otherEntityNum2 != 0
        && ctx.world.bg_state.g_vehWeaponInfo[cent.currentState.otherEntityNum2 as usize].iImpactFX
            != 0
    {
        // missile is from a special vehWeapon
        let mut normal: vec3_t = [0.0; 3];
        ByteToDir(cent.currentState.eventParm, &mut normal);

        let iImpactFX = ctx.world.bg_state.g_vehWeaponInfo
            [cent.currentState.otherEntityNum2 as usize]
            .iImpactFX;
        trap::FX_PlayEffectID(ctx.engine, iImpactFX, &cent.lerpOrigin, &normal, -1, -1);
        return true;
    }
    false
}

/// Raven `CG_CalcMuzzlePoint` — where entity `entityNum`'s shots come out of,
/// for the crosshair trace. The local player gets the full viewmodel treatment
/// (per-weapon muzzle offset, third-person vs first-person, the emplaced-gun
/// override); everyone else gets their trajectory base plus a viewheight.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:2215-2313`
pub fn CG_CalcMuzzlePoint(world: &CgWorld, entityNum: c_int, muzzle: &mut vec3_t) -> bool {
    let mut forward: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut gunpoint: vec3_t = [0.0; 3];

    // no snapshot means nothing can match our clientNum, so we fall through to
    // the generic path rather than reading a state we don't have
    let snapPs = world
        .cg
        .snap_ref()
        .map(|snap| &snap.ps)
        .filter(|ps| ps.clientNum == entityNum);

    if let Some(ps) = snapPs {
        // I'm not exactly sure why we'd be rendering someone else's crosshair, but hey.
        let weapontype = ps.weapon;
        let mut weaponMuzzle: vec3_t = [0.0; 3];
        let pEnt = world.entity(world.cg.predictedPlayerState.clientNum as usize);

        _VectorCopy(WP_MuzzlePoint[weapontype as usize], &mut weaponMuzzle);

        if weapontype == WP_DISRUPTOR
            || weapontype == WP_STUN_BATON
            || weapontype == WP_MELEE
            || weapontype == WP_SABER
        {
            VectorClear(&mut weaponMuzzle);
        }

        if world.cg.renderingThirdPerson != qfalse {
            _VectorCopy(pEnt.lerpOrigin, &mut gunpoint);
            AngleVectors(pEnt.lerpAngles, Some(&mut forward), Some(&mut right), None);
        } else {
            _VectorCopy(world.cg.refdef.vieworg, &mut gunpoint);
            AngleVectors(
                world.cg.refdef.viewangles,
                Some(&mut forward),
                Some(&mut right),
                None,
            );
        }

        if weapontype == WP_EMPLACED_GUN && ps.emplacedIndex != 0 {
            // Raven null-checks `gunEnt` here; an owned array element is never
            // null, so the check drops and the block runs unconditionally.
            let gunEnt = world.entity(ps.emplacedIndex as usize);
            let mut pitchConstraint: vec3_t = [0.0; 3];

            _VectorCopy(gunEnt.lerpOrigin, &mut gunpoint);
            gunpoint[2] += 46.0;

            if world.cg.renderingThirdPerson != qfalse {
                _VectorCopy(pEnt.lerpAngles, &mut pitchConstraint);
            } else {
                _VectorCopy(world.cg.refdef.viewangles, &mut pitchConstraint);
            }

            if pitchConstraint[PITCH] > 40.0 {
                pitchConstraint[PITCH] = 40.0;
            }
            AngleVectors(pitchConstraint, Some(&mut forward), Some(&mut right), None);
        }

        _VectorCopy(gunpoint, muzzle);

        _VectorMA(*muzzle, weaponMuzzle[0], forward, muzzle);
        _VectorMA(*muzzle, weaponMuzzle[1], right, muzzle);

        if weapontype == WP_EMPLACED_GUN && ps.emplacedIndex != 0 {
            // Do nothing
        } else if world.cg.renderingThirdPerson != qfalse {
            muzzle[2] += ps.viewheight as f32 + weaponMuzzle[2];
        } else {
            muzzle[2] += weaponMuzzle[2];
        }

        return true;
    }

    let cent = world.entity(entityNum as usize);
    if cent.currentValid == qfalse {
        return false;
    }

    _VectorCopy(cent.currentState.pos.trBase, muzzle);

    AngleVectors(
        cent.currentState.apos.trBase,
        Some(&mut forward),
        None,
        None,
    );
    let anim = cent.currentState.legsAnim;
    if anim == animNumber_t::BOTH_CROUCH1WALK as c_int
        || anim == animNumber_t::BOTH_CROUCH1IDLE as c_int
    {
        muzzle[2] += CROUCH_VIEWHEIGHT as f32;
    } else {
        muzzle[2] += DEFAULT_VIEWHEIGHT as f32;
    }

    _VectorMA(*muzzle, 14.0, forward, muzzle);

    true
}

/// Raven `CG_InitG2Weapons` — builds the one shared ghoul2 instance per weapon.
///
/// Raven: "create one instance of all the weapons we are going to use so we can
/// just copy this info into each clients gun ghoul2 object in fast way".
///
/// Raven walks `bg_itemlist + 1` to the `NULL`-classname sentinel; our
/// `bg_itemlist` dropped that sentinel, so the walk is `[1..]`.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:2324-2361`
pub fn CG_InitG2Weapons(ctx: &mut CgContext) {
    let engine = ctx.engine;
    let mut i: usize = 0;

    ctx.world.weapons.g2WeaponInstances = [null_mut(); MAX_WEAPONS];

    for item in bg_itemlist[1..].iter() {
        let ItemKind::Weapon(giTag) = item.kind else {
            continue;
        };
        debug_assert!(giTag < MAX_WEAPONS as c_int);
        let slot = giTag as usize;

        // initialise model
        trap::G2API_InitGhoul2Model(
            engine,
            &mut ctx.world.weapons.g2WeaponInstances[slot] as *mut *mut c_void,
            item.world_model[0],
            0,
            0,
            0,
            0,
            0,
        );

        let instance = ctx.world.weapons.g2WeaponInstances[slot];
        if !instance.is_null() {
            // indicate we will be bolted to model 0 (ie the player) on bolt 0 (always the right hand) when we get copied
            trap::G2API_SetBoltInfo(engine, instance, 0, 0);
            // now set up the gun bolt on it
            if giTag == WP_SABER {
                trap::G2API_AddBolt(engine, instance, 0, "*blade1");
            } else {
                trap::G2API_AddBolt(engine, instance, 0, "*flash");
            }
            i += 1;
        }

        if i == MAX_WEAPONS {
            debug_assert!(false, "CG_InitG2Weapons ran out of weapon slots");
            break;
        }
    }
}

/// Raven `CG_ShutDownG2Weapons` — frees every shared weapon instance.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:2364-2371`
pub fn CG_ShutDownG2Weapons(ctx: &mut CgContext) {
    let engine = ctx.engine;
    for i in 0..MAX_WEAPONS {
        trap::G2API_CleanGhoul2Models(
            engine,
            &mut ctx.world.weapons.g2WeaponInstances[i] as *mut *mut c_void,
        );
    }
}

/// Raven `CG_G2WeaponInstance` — which ghoul2 weapon instance to copy for this
/// entity: a player's custom saber hilt when there is one, otherwise the shared
/// per-weapon instance.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:2373-2411`
pub fn CG_G2WeaponInstance(world: &CgWorld, cent: &centity_t, weapon: c_int) -> *mut c_void {
    let shared = world.weapons.g2WeaponInstances[weapon as usize];

    if weapon != WP_SABER {
        return shared;
    }

    if cent.currentState.eType != ET_PLAYER as c_int && cent.currentState.eType != ET_NPC as c_int {
        return shared;
    }

    let ci: Option<&clientInfo_t> = if cent.currentState.eType == ET_NPC as c_int {
        cent.npcClient.as_deref()
    } else {
        Some(&world.cgs.clientinfo[cent.currentState.number as usize])
    };

    let Some(ci) = ci else {
        return shared;
    };

    // Try to return the custom saber instance if we can.
    if ci.saber[0].model[0] != 0 && !ci.ghoul2Weapons[0].is_null() {
        return ci.ghoul2Weapons[0];
    }

    // If no custom then just use the default.
    shared
}
