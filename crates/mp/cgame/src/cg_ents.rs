//! Port of `oracle/codemp/cgame/cg_ents.c` — turning each snapshot entity into renderer commands. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};
use core::ptr::null_mut;

use native_string::{atof_bytes, latin1_to_string};

use mp_bg::bg_misc::{
    BG_EvaluateTrajectory, BG_EvaluateTrajectoryDelta, BG_GiveMeVectorFromMatrix,
};
use mp_bg::public::bg_itemlist::{bg_itemlist, bg_numItems};
use mp_bg::public::configstring::{CS_AMBIENT_SET, CS_EFFECTS, CS_MODELS};
use mp_bg::public::entity_effects::EF2_HYPERSPACE;
use mp_bg::public::entity_flags::{
    EF_ALT_FIRING, EF_DEAD, EF_DROPPEDWEAPON, EF_FIRING, EF_ITEMPLACEHOLDER, EF_JETPACK_ACTIVE,
    EF_MISSILE_STICK, EF_NODRAW, EF_RADAROBJECT, EF_SHADER_ANIM,
};
use mp_bg::public::entity_type::entityType_t;
use mp_bg::public::fx_state::{FX_STATE_OFF, FX_STATE_ONE_SHOT_LIMIT};
use mp_bg::public::g2_model_parts::G2_MODEL_PART;
use mp_bg::public::gametype::{GT_CTF, GT_CTY, GT_JEDIMASTER};
use mp_bg::public::holdable::{HI_BINOCULARS, HI_SEEKER, HI_SHIELD};
use mp_bg::public::hyperspace::{HYPERSPACE_TELEPORT_FRAC, HYPERSPACE_TIME};
use mp_bg::public::item_type::{IT_ARMOR, IT_HEALTH, IT_HOLDABLE, IT_POWERUP, IT_TEAM, IT_WEAPON};
use mp_bg::public::pmtype::pmtype_t;
use mp_bg::public::powerup::{
    PW_BLUEFLAG, PW_FORCE_BOON, PW_FORCE_ENLIGHTENED_DARK, PW_FORCE_ENLIGHTENED_LIGHT, PW_REDFLAG,
};
use mp_bg::public::team::{TEAM_BLUE, TEAM_FREE, TEAM_RED};
use mp_bg::weapons::weapon_t::{
    WP_BLASTER, WP_BOWCASTER, WP_DEMP2, WP_DET_PACK, WP_DISRUPTOR, WP_FLECHETTE, WP_NUM_WEAPONS,
    WP_REPEATER, WP_ROCKET_LAUNCHER, WP_SABER, WP_THERMAL, WP_TRIP_MINE,
};
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::game::class_t::class_t::CLASS_VEHICLE;
use mp_qshared::common::mp::qcommon::entity_state::entityState_t;
use mp_qshared::shared::force_powers::{FORCE_DARKSIDE, FORCE_LIGHTSIDE};
use mp_qshared::shared::q_math::{
    _VectorAdd, _VectorCopy, _VectorMA, _VectorScale, _VectorSubtract, vec3_origin, vectoangles,
    AngleVectors, AnglesToAxis, AxisClear, AxisCopy, ByteToDir, CrossProduct, LerpAngle,
    MatrixMultiply, PerpendicularVector, RotateAroundDirection, VectorClear, VectorLength,
    VectorNormalize, VectorNormalize2, VectorSet, PITCH, ROLL, YAW,
};
use mp_qshared::shared::surface_flags::SOLID_BMODEL;
use mp_qshared::shared::{
    addpolyArgStruct_t, addspriteArgStruct_t, mdxaBone_t, orientation_t, qfalse, qhandle_t, qtrue,
    sfxHandle_t, trType_t, vec3_t, Eorientations, CHAN_AUTO, CHAN_BODY, CHAN_ITEM,
    ENTITYNUM_MAX_NORMAL, MAX_CLIENTS_I32,
};

use crate::cg_main::{CG_ConfigString, CG_Error, Com_Printf};
use crate::cg_players::CG_AddRefEntityWithPowerups;
use crate::cg_weaponinit::NULL_HANDLE;
use crate::local::centity_s::{centity_t, MAX_CG_LOOPSOUNDS};
use crate::local::cg_loop_sound_s::cgLoopSound_t;
use crate::trap;
use crate::world::cg_context::CgContext;
use crate::world::cg_world::CgWorld;

// ---------------------------------------------------------------------------
// File-scope constants
//
// `cg_ents.c` includes `..\ghoul2\g2.h` and `tr_types.h`; neither header's
// bit-field layout nor its renderfx bits have a ported cross-crate home yet, so
// the handful this TU reads land here beside their readers (§C8).
// ---------------------------------------------------------------------------

/// Raven's `boltInfo` bit-field widths and the shift/mask pairs that unpack it.
/// Source: `oracle/codemp/ghoul2/G2.h:30-40`
const ENTITY_WIDTH: c_int = 12;
const MODEL_WIDTH: c_int = 10;
const BOLT_WIDTH: c_int = 10;
const MODEL_AND: c_int = (1 << MODEL_WIDTH) - 1;
const BOLT_AND: c_int = (1 << BOLT_WIDTH) - 1;
const ENTITY_AND: c_int = (1 << ENTITY_WIDTH) - 1;
const BOLT_SHIFT: c_int = 0;
const MODEL_SHIFT: c_int = BOLT_SHIFT + BOLT_WIDTH;
const ENTITY_SHIFT: c_int = MODEL_SHIFT + MODEL_WIDTH;

/// Raven `RF_DISINTEGRATE1` — does a procedural hole-ripping thing.
/// Source: `oracle/codemp/cgame/tr_types.h:47`
const RF_DISINTEGRATE1: c_int = 0x20000;

/// Raven `RF_DISINTEGRATE2` — does a procedural hole-ripping thing with
/// scaling at the ripping point.
/// Source: `oracle/codemp/cgame/tr_types.h:48`
const RF_DISINTEGRATE2: c_int = 0x40000;

/// Raven `RF_MINLIGHT` — allways have some light (viewmodel, some items).
/// Source: `oracle/codemp/cgame/tr_types.h:18`
const RF_MINLIGHT: c_int = 0x00001;

/// Raven `RF_DEPTHHACK` — for view weapon Z crunching.
/// Source: `oracle/codemp/cgame/tr_types.h:21`
const RF_DEPTHHACK: c_int = 0x00008;

/// Raven `RF_NOSHADOW` — don't add stencil shadows.
/// Source: `oracle/codemp/cgame/tr_types.h:26`
const RF_NOSHADOW: c_int = 0x00040;

/// Raven `RF_FORCE_ENT_ALPHA` — override shader alpha settings.
/// Source: `oracle/codemp/cgame/tr_types.h:36`
const RF_FORCE_ENT_ALPHA: c_int = 0x00400;

/// Raven `RF_RGB_TINT` — override shader rgb settings.
/// Source: `oracle/codemp/cgame/tr_types.h:37`
const RF_RGB_TINT: c_int = 0x00800;

/// Raven `RF_DISTORTION` — area distortion effect.
/// Source: `oracle/codemp/cgame/tr_types.h:41`
const RF_DISTORTION: c_int = 0x02000;

/// Raven `RF_SETANIMINDEX` — use `backEnd.currentEntity->e.skinNum` for
/// `R_BindAnimatedImage`.
/// Source: `oracle/codemp/cgame/tr_types.h:50`
const RF_SETANIMINDEX: c_int = 0x80000;

/// Raven `ITEM_SCALEUP_TIME` — how long a just-respawned item fades in for.
/// Source: `oracle/codemp/cgame/cg_local.h:37`
const ITEM_SCALEUP_TIME: c_int = 1000;

// Raven's `int CG_BMS_START/MID/END` are file-scope ints that nothing ever
// writes, so they land as `const`s beside their reader rather than as state.
// Only `CG_BMS_MID` has a reader in either tree; the other two are kept for the
// set (`CG_PlayDoorSound`'s `type` argument is the same stage numbering, fed
// from the event's `eventParm`).
// Source: `oracle/codemp/cgame/cg_ents.c:2735-2737`
pub const CG_BMS_START: c_int = 0;
pub const CG_BMS_MID: c_int = 1;
pub const CG_BMS_END: c_int = 2;

/// Raven `CG_PositionEntityOnTag` — modifies the entity's position and axis by
/// the given tag location.
/// Source: `oracle/codemp/cgame/cg_ents.c:26-44`
pub fn CG_PositionEntityOnTag(
    ctx: &mut CgContext,
    entity: &mut refEntity_t,
    parent: &refEntity_t,
    parentModel: qhandle_t,
    tagName: &str,
) {
    let mut lerped = orientation_t {
        origin: [0.0; 3],
        axis: [[0.0; 3]; 3],
    };

    // lerp the tag
    trap::R_LerpTag(
        ctx.engine,
        &mut lerped,
        parentModel,
        parent.oldframe,
        parent.frame,
        1.0 - parent.backlerp,
        tagName,
    );

    // FIXME: allow origin offsets along tag?
    _VectorCopy(parent.origin, &mut entity.origin);
    for i in 0..3 {
        _VectorMA(
            entity.origin,
            lerped.origin[i],
            parent.axis[i],
            &mut entity.origin,
        );
    }

    // had to cast away the const to avoid compiler problems...
    MatrixMultiply(&lerped.axis, &parent.axis, &mut entity.axis);
    entity.backlerp = parent.backlerp;
}

/// Raven `CG_PositionRotatedEntityOnTag` — modifies the entity's position and
/// axis by the given tag location, keeping the entity's own rotation.
/// Source: `oracle/codemp/cgame/cg_ents.c:55-75`
pub fn CG_PositionRotatedEntityOnTag(
    ctx: &mut CgContext,
    entity: &mut refEntity_t,
    parent: &refEntity_t,
    parentModel: qhandle_t,
    tagName: &str,
) {
    let mut lerped = orientation_t {
        origin: [0.0; 3],
        axis: [[0.0; 3]; 3],
    };
    let mut tempAxis: [vec3_t; 3] = [[0.0; 3]; 3];

    // AxisClear( entity->axis );
    // lerp the tag
    trap::R_LerpTag(
        ctx.engine,
        &mut lerped,
        parentModel,
        parent.oldframe,
        parent.frame,
        1.0 - parent.backlerp,
        tagName,
    );

    // FIXME: allow origin offsets along tag?
    _VectorCopy(parent.origin, &mut entity.origin);
    for i in 0..3 {
        _VectorMA(
            entity.origin,
            lerped.origin[i],
            parent.axis[i],
            &mut entity.origin,
        );
    }

    // had to cast away the const to avoid compiler problems...
    MatrixMultiply(&entity.axis, &lerped.axis, &mut tempAxis);
    MatrixMultiply(&tempAxis, &parent.axis, &mut entity.axis);
}

/// Raven `CG_SetEntitySoundPosition` — also called by event processing code.
///
/// Raven's `centity_t *cent` is the entity's number here — entities are owned
/// by `CgWorld` and reached by index, never by an aliasing pointer (§B5).
/// Source: `oracle/codemp/cgame/cg_ents.c:96-110`
pub fn CG_SetEntitySoundPosition(ctx: &mut CgContext, centNum: usize) {
    let cent = ctx.world.entity(centNum);
    if cent.currentState.solid == SOLID_BMODEL {
        let v = ctx.world.cgs.inlineModelMidpoints[cent.currentState.modelindex as usize];
        let mut origin: vec3_t = [0.0; 3];
        _VectorAdd(cent.lerpOrigin, v, &mut origin);
        trap::S_UpdateEntityPosition(ctx.engine, cent.currentState.number, &origin);
    } else {
        trap::S_UpdateEntityPosition(ctx.engine, cent.currentState.number, &cent.lerpOrigin);
    }
}

/// Raven `CG_S_AddLoopingSound` — set the current looping sounds on the entity.
///
/// PORT-NOTE: two Raven defects preserved verbatim. The search loop never
/// increments `i`, so it spins forever once the entity already carries a
/// looping sound with a *different* handle; and the "already playing" arm has
/// no `return` despite its comment, so a matched sound is updated and then
/// appended a second time.
/// Source: `oracle/codemp/cgame/cg_ents.c:119-160`
pub fn CG_S_AddLoopingSound(
    world: &mut CgWorld,
    entityNum: usize,
    origin: vec3_t,
    velocity: vec3_t,
    sfx: sfxHandle_t,
) {
    let cent = world.entity_mut(entityNum);
    let mut cSound: Option<usize> = None;
    // never stepped — see the PORT-NOTE above
    let i: c_int = 0;
    let mut alreadyPlaying = false;

    // first see if we're already looping this sound handle.
    while i < cent.numLoopingSounds {
        cSound = Some(i as usize);

        if cent.loopingSound[i as usize].sfx == sfx {
            alreadyPlaying = true;
            break;
        }
    }

    if let (true, Some(slot)) = (alreadyPlaying, cSound) {
        // if this is the case, just update the properties of the looping sound and return.
        let s = &mut cent.loopingSound[slot];
        _VectorCopy(origin, &mut s.origin);
        _VectorCopy(velocity, &mut s.velocity);
    } else if cent.numLoopingSounds >= MAX_CG_LOOPSOUNDS as c_int {
        // Just don't add it then I suppose. (Raven's overflow warning is _XBOX-only.)
        return;
    }

    // Add a new looping sound. Reachable with `slot == MAX_CG_LOOPSOUNDS` when
    // the "already playing" arm above fell through with the array already
    // full - Raven's version overruns `loopingSound` into whatever follows it
    // in the struct (UB); the port's fixed-size array indexing panics instead,
    // a defined trap rather than a silent overwrite (§F19).
    let slot = cent.numLoopingSounds as usize;
    cent.loopingSound[slot].entityNum = entityNum as c_int;
    _VectorCopy(origin, &mut cent.loopingSound[slot].origin);
    _VectorCopy(velocity, &mut cent.loopingSound[slot].velocity);
    cent.loopingSound[slot].sfx = sfx;

    cent.numLoopingSounds += 1;
}

/// Raven `CG_S_AddRealLoopingSound`.
///
/// Raven: For now just redirect, might eventually do something different.
/// Source: `oracle/codemp/cgame/cg_ents.c:169-172`
pub fn CG_S_AddRealLoopingSound(
    world: &mut CgWorld,
    entityNum: usize,
    origin: vec3_t,
    velocity: vec3_t,
    sfx: sfxHandle_t,
) {
    CG_S_AddLoopingSound(world, entityNum, origin, velocity, sfx);
}

/// Raven `CG_S_StopLoopingSound` — `sfx == -1` clears every looping sound on
/// the entity, otherwise only the named handle is removed.
///
/// PORT-NOTE: Raven's outer `i++` also runs on the removal path, so the entry
/// shifted down into slot `i` is skipped — a duplicate handle survives one
/// call. Preserved.
/// Source: `oracle/codemp/cgame/cg_ents.c:181-214`
pub fn CG_S_StopLoopingSound(world: &mut CgWorld, entityNum: usize, sfx: sfxHandle_t) {
    let cent = world.entity_mut(entityNum);

    if sfx == -1 {
        // clear all the looping sounds on the entity
        cent.numLoopingSounds = 0;
    } else {
        // otherwise, clear only the specified looping sound
        let mut i: c_int = 0;

        while i < cent.numLoopingSounds {
            if cent.loopingSound[i as usize].sfx == sfx {
                // remove it then
                let mut x = i + 1;

                while x < cent.numLoopingSounds {
                    // Raven memcpy's the slot down; `cgLoopSound_t` is plain
                    // scalars, so a field-wise copy is the same bytes.
                    let src = &cent.loopingSound[x as usize];
                    let moved = cgLoopSound_t {
                        entityNum: src.entityNum,
                        origin: src.origin,
                        velocity: src.velocity,
                        sfx: src.sfx,
                    };
                    cent.loopingSound[(x - 1) as usize] = moved;
                    x += 1;
                }
                cent.numLoopingSounds -= 1;
            }

            i += 1;
        }
    }
    // trap_S_StopLoopingSound(entityNum);
}

/// Raven `CG_S_UpdateLoopingSounds` — replays the entity's registered loops at
/// this frame's interpolated position.
/// Source: `oracle/codemp/cgame/cg_ents.c:223-255`
pub fn CG_S_UpdateLoopingSounds(ctx: &mut CgContext, entityNum: usize) {
    let cent = ctx.world.entity(entityNum);
    let mut lerpOrg: vec3_t = [0.0; 3];

    if cent.numLoopingSounds == 0 {
        return;
    }

    if cent.currentState.eType == entityType_t::ET_MOVER as c_int {
        let v = ctx.world.cgs.inlineModelMidpoints[cent.currentState.modelindex as usize];
        _VectorAdd(cent.lerpOrigin, v, &mut lerpOrg);
    } else {
        _VectorCopy(cent.lerpOrigin, &mut lerpOrg);
    }

    let mut i: c_int = 0;
    while i < cent.numLoopingSounds {
        let cSound = &cent.loopingSound[i as usize];

        // trap_S_AddLoopingSound(entityNum, cSound->origin, cSound->velocity, cSound->sfx);
        // I guess just keep using lerpOrigin for now,
        trap::S_AddLoopingSound(
            ctx.engine,
            entityNum as c_int,
            &lerpOrg,
            &cSound.velocity,
            cSound.sfx,
        );
        i += 1;
    }
}

/// Raven `CG_SetGhoul2Info` — copies the cent's ghoul2 instance and scale onto
/// the refEntity about to be handed to the renderer.
/// Source: `oracle/codemp/cgame/cg_ents.c:490-497`
pub fn CG_SetGhoul2Info(ent: &mut refEntity_t, cent: &centity_t) {
    ent.ghoul2 = cent.ghoul2;
    _VectorCopy(cent.modelScale, &mut ent.modelScale);
    ent.radius = cent.radius;
    _VectorCopy(cent.lerpAngles, &mut ent.angles);
}

/// Raven `CG_CreateBBRefEnts` — the eight bounding-box corner sprites.
///
/// PORT-NOTE: Raven's entire body sits inside a `/* … */` block (a `_DEBUG`
/// g2r visualization that was commented out), so the shipping function is a
/// no-op and the faithful port is an empty body.
/// Source: `oracle/codemp/cgame/cg_ents.c:502-568`
#[allow(unused_variables)]
pub fn CG_CreateBBRefEnts(s1: &entityState_t, origin: vec3_t) {}

/// Raven `G2_BoltToGhoul2Model` — unpacks a cent's `boltInfo` and drops the
/// refEntity onto that bolt.
///
/// PORT-NOTE: Raven's own `assert(0)` fires unconditionally here ("I put this
/// here because the cgs.gamemodels array no longer gets initialized"), so the
/// function is dead in a debug build and reads a stale model list in release.
/// Kept as a `debug_assert!`.
/// Source: `oracle/codemp/cgame/cg_ents.c:571-607`
pub fn G2_BoltToGhoul2Model(ctx: &mut CgContext, centNum: usize, ent: &mut refEntity_t) {
    // extract the wraith ID from the bolt info
    let boltInfo = ctx.world.entity(centNum).boltInfo;
    let mut modelNum = boltInfo >> MODEL_SHIFT;
    let mut boltNum = boltInfo >> BOLT_SHIFT;
    let mut entNum = boltInfo >> ENTITY_SHIFT;
    let mut boltMatrix = mdxaBone_t {
        matrix: [[0.0; 4]; 3],
    };

    modelNum &= MODEL_AND;
    boltNum &= BOLT_AND;
    entNum &= ENTITY_AND;

    // NOTENOTE I put this here because the cgs.gamemodels array no longer gets initialized.
    debug_assert!(
        false,
        "G2_BoltToGhoul2Model: cgs.gameModels is never filled"
    );

    // `ENTITY_AND` masks to 12 bits (0-4095), wider than `MAX_GENTITIES`
    // (1024) - Raven's `cg_entities[entNum]` reads past the array (UB) for
    // any encoded index >= 1024; the port's checked array indexing panics
    // instead of reading adjacent memory (§F19).
    // go away and get me the bolt position for this frame please
    let ghoul2 = ctx.world.entity(centNum).ghoul2;
    let modelScale = ctx.world.entity(centNum).modelScale;
    let angles = ctx.world.entity(entNum as usize).currentState.angles;
    let origin = ctx.world.entity(entNum as usize).currentState.origin;
    let time = ctx.world.cg.time;
    trap::G2API_GetBoltMatrix(
        ctx.engine,
        ghoul2,
        modelNum,
        boltNum,
        &mut boltMatrix,
        &angles,
        &origin,
        time,
        Some(&mut ctx.world.cgs.gameModels[0]),
        &modelScale,
    );

    // set up the axis and origin we need for the actual effect spawning
    ent.origin[0] = boltMatrix.matrix[0][3];
    ent.origin[1] = boltMatrix.matrix[1][3];
    ent.origin[2] = boltMatrix.matrix[2][3];

    ent.axis[0][0] = boltMatrix.matrix[0][0];
    ent.axis[0][1] = boltMatrix.matrix[1][0];
    ent.axis[0][2] = boltMatrix.matrix[2][0];

    ent.axis[1][0] = boltMatrix.matrix[0][1];
    ent.axis[1][1] = boltMatrix.matrix[1][1];
    ent.axis[1][2] = boltMatrix.matrix[2][1];

    ent.axis[2][0] = boltMatrix.matrix[0][2];
    ent.axis[2][1] = boltMatrix.matrix[1][2];
    ent.axis[2][2] = boltMatrix.matrix[2][2];
}

/// Raven `ScaleModelAxis` — scale the model should we need to.
/// Source: `oracle/codemp/cgame/cg_ents.c:609-627`
pub fn ScaleModelAxis(ent: &mut refEntity_t) {
    if ent.modelScale[0] != 0.0 && ent.modelScale[0] != 1.0 {
        _VectorScale(ent.axis[0], ent.modelScale[0], &mut ent.axis[0]);
        ent.nonNormalizedAxes = qtrue;
    }
    if ent.modelScale[1] != 0.0 && ent.modelScale[1] != 1.0 {
        _VectorScale(ent.axis[1], ent.modelScale[1], &mut ent.axis[1]);
        ent.nonNormalizedAxes = qtrue;
    }
    if ent.modelScale[2] != 0.0 && ent.modelScale[2] != 1.0 {
        _VectorScale(ent.axis[2], ent.modelScale[2], &mut ent.axis[2]);
        ent.nonNormalizedAxes = qtrue;
    }
}

/// Raven `CG_Disintegration` — the disruptor death effect: two passes of the
/// same refEntity plus the lumbar smoke puffs.
/// Source: `oracle/codemp/cgame/cg_ents.c:653-700`
pub fn CG_Disintegration(ctx: &mut CgContext, centNum: usize, ent: &mut refEntity_t) {
    let mut tempAng: vec3_t = [0.0; 3];
    let mut hitLoc: vec3_t = [0.0; 3];

    _VectorCopy(ctx.world.entity(centNum).currentState.origin2, &mut hitLoc);

    _VectorSubtract(hitLoc, ent.origin, &mut ent.oldorigin);

    let tempLength = VectorNormalize(&mut ent.oldorigin);
    vectoangles(ent.oldorigin, &mut tempAng);
    tempAng[YAW] -= ctx.world.entity(centNum).lerpAngles[YAW];
    AngleVectors(tempAng, Some(&mut ent.oldorigin), None, None);
    _VectorScale(ent.oldorigin, tempLength, &mut ent.oldorigin);

    ent.endTime = ctx.world.entity(centNum).dustTrailTime as f32;

    ent.renderfx |= RF_DISINTEGRATE2;
    ent.customShader = ctx.world.cgs.media.disruptorShader;
    trap::R_AddRefEntityToScene(ctx.engine, ent);

    ent.renderfx &= !RF_DISINTEGRATE2;
    ent.renderfx |= RF_DISINTEGRATE1;
    ent.customShader = 0;
    trap::R_AddRefEntityToScene(ctx.engine, ent);

    // the timescale draw only happens when the first half of the `&&` holds —
    // short-circuit keeps the rng stream in Raven's order
    let timescale = ctx.world.cvars.cg_timescale.value;
    if ctx.world.cg.time as f32 - ent.endTime < 1000.0
        && (timescale * timescale * ctx.world.bg_state.rng.random()) > 0.05
    {
        let mut fxOrg: vec3_t = [0.0; 3];
        let mut fxDir: vec3_t = [0.0; 3];
        let mut boltMatrix = mdxaBone_t {
            matrix: [[0.0; 4]; 3],
        };
        let ghoul2 = ctx.world.entity(centNum).ghoul2;
        let torsoBolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "lower_lumbar");

        VectorSet(&mut fxDir, 0.0, 1.0, 0.0);

        let lerpAngles = ctx.world.entity(centNum).lerpAngles;
        let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
        let modelScale = ctx.world.entity(centNum).modelScale;
        let time = ctx.world.cg.time;
        trap::G2API_GetBoltMatrix(
            ctx.engine,
            ghoul2,
            0,
            torsoBolt,
            &mut boltMatrix,
            &lerpAngles,
            &lerpOrigin,
            time,
            Some(&mut ctx.world.cgs.gameModels[0]),
            &modelScale,
        );
        BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, &mut fxOrg);

        let viewaxis = ctx.world.cg.refdef.viewaxis[0];
        _VectorMA(fxOrg, -18.0, viewaxis, &mut fxOrg);
        // Raven's `crandom()` macro is double-typed, so the *20 happens in f64
        // before it narrows back into the float slot
        fxOrg[2] = (fxOrg[2] as f64 + ctx.world.bg_state.rng.crandom() * 20.0) as f32;
        let deathSmoke = ctx.world.cgs.effects.mDisruptorDeathSmoke;
        trap::FX_PlayEffectID(ctx.engine, deathSmoke, &fxOrg, &fxDir, -1, -1);

        if ctx.world.bg_state.rng.random() > 0.5 {
            trap::FX_PlayEffectID(ctx.engine, deathSmoke, &fxOrg, &fxDir, -1, -1);
        }
    }
}

/// Raven `CG_RenderTimeEntBolt` — parks a siege "time" entity on the carrying
/// player's left hand; returns false when the caller should skip the world
/// render (the HUD path draws it instead).
/// Source: `oracle/codemp/cgame/cg_ents.c:703-745`
pub fn CG_RenderTimeEntBolt(ctx: &mut CgContext, centNum: usize) -> bool {
    let clientNum = ctx.world.entity(centNum).currentState.boltToPlayer - 1;

    if clientNum >= MAX_CLIENTS_I32 || clientNum < 0 {
        debug_assert!(false, "CG_RenderTimeEntBolt: boltToPlayer out of range");
        return false;
    }

    let cl = clientNum as usize;

    if ctx.world.entity(cl).ghoul2.is_null() {
        debug_assert!(
            false,
            "CG_RenderTimeEntBolt: carrier has no ghoul2 instance"
        );
        return false;
    }

    if clientNum == ctx.world.cg.predictedPlayerState.clientNum
        && ctx.world.cg.renderingThirdPerson == qfalse
    {
        // If in first person and you have it then render the thing spinning around on your hud.
        let number = ctx.world.entity(centNum).currentState.number;
        // set it to render at the end of the frame.
        ctx.world.draw.cgSiegeEntityRender = number;
        return false;
    }

    let ghoul2 = ctx.world.entity(cl).ghoul2;
    let getBolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "lhand");

    let mut matrix = mdxaBone_t {
        matrix: [[0.0; 4]; 3],
    };
    let turAngles = ctx.world.entity(cl).turAngles;
    let lerpOrigin = ctx.world.entity(cl).lerpOrigin;
    let modelScale = ctx.world.entity(cl).modelScale;
    let time = ctx.world.cg.time;
    trap::G2API_GetBoltMatrix(
        ctx.engine,
        ghoul2,
        0,
        getBolt,
        &mut matrix,
        &turAngles,
        &lerpOrigin,
        time,
        Some(&mut ctx.world.cgs.gameModels[0]),
        &modelScale,
    );

    let mut boltOrg: vec3_t = [0.0; 3];
    let mut boltAng: vec3_t = [0.0; 3];
    BG_GiveMeVectorFromMatrix(&matrix, Eorientations::ORIGIN as c_int, &mut boltOrg);
    BG_GiveMeVectorFromMatrix(&matrix, Eorientations::NEGATIVE_Y as c_int, &mut boltAng);
    vectoangles(boltAng, &mut boltAng);
    boltAng[PITCH] = 0.0;
    boltAng[ROLL] = 0.0;

    let cent = ctx.world.entity_mut(centNum);
    _VectorCopy(boltOrg, &mut cent.lerpOrigin);
    _VectorCopy(boltAng, &mut cent.lerpAngles);

    true
}

/// Raven `CG_AddRadarEnt` — queues an entity for this frame's radar draw.
/// Source: `oracle/codemp/cgame/cg_ents.c:799-809`
pub fn CG_AddRadarEnt(world: &mut CgWorld, centNum: usize) {
    if world.cg.radarEntityCount as usize == world.cg.radarEntities.len() {
        // Raven's "CG_AddRadarEnt full" warning is _DEBUG-only; the ship build
        // just drops the entity.
        return;
    }
    let number = world.entity(centNum).currentState.number;
    let slot = world.cg.radarEntityCount as usize;
    world.cg.radarEntities[slot] = number as i16;
    world.cg.radarEntityCount += 1;
}

/// Raven `CG_AddBracketedEnt` — queues an entity for this frame's bracket draw.
/// Source: `oracle/codemp/cgame/cg_ents.c:811-821`
pub fn CG_AddBracketedEnt(world: &mut CgWorld, centNum: usize) {
    if world.cg.bracketedEntityCount as usize == world.cg.bracketedEntities.len() {
        // Raven's "CG_AddBracketedEnt full" warning is _DEBUG-only.
        return;
    }
    let number = world.entity(centNum).currentState.number;
    let slot = world.cg.bracketedEntityCount as usize;
    world.cg.bracketedEntities[slot] = number as i16;
    world.cg.bracketedEntityCount += 1;
}

/// Raven `CG_Speaker` — speaker entities can automatically play sounds.
/// Source: `oracle/codemp/cgame/cg_ents.c:1835-1854`
pub fn CG_Speaker(ctx: &mut CgContext, centNum: usize) {
    if ctx.world.entity(centNum).currentState.trickedentindex != 0 {
        let number = ctx.world.entity(centNum).currentState.number as usize;
        CG_S_StopLoopingSound(ctx.world, number, -1);
    }

    // FIXME: use something other than clientNum...
    if ctx.world.entity(centNum).currentState.clientNum == 0 {
        return; // not auto triggering
    }

    if ctx.world.cg.time < ctx.world.entity(centNum).miscTime {
        return;
    }

    let number = ctx.world.entity(centNum).currentState.number;
    let eventParm = ctx.world.entity(centNum).currentState.eventParm;
    let sfx = ctx.world.cgs.gameSounds[eventParm as usize];
    trap::S_StartSound(ctx.engine, None, number, CHAN_ITEM, sfx);

    //	ent->s.frame = ent->wait * 10;
    //	ent->s.clientNum = ent->random * 10;
    let time = ctx.world.cg.time;
    let frame = ctx.world.entity(centNum).currentState.frame;
    let clientNum = ctx.world.entity(centNum).currentState.clientNum;
    let crandom = ctx.world.bg_state.rng.crandom();
    // `crandom()` is double-typed, so the whole tail is a double sum that
    // truncates back into the int slot
    ctx.world.entity_mut(centNum).miscTime =
        ((time + frame * 100) as f64 + (clientNum * 100) as f64 * crandom) as c_int;
}

/// Raven `CG_GreyItem` — true when an enlightenment powerup belongs to the
/// other force side, so the icon draws greyed out.
/// Source: `oracle/codemp/cgame/cg_ents.c:1856-1878`
pub fn CG_GreyItem(r#type: c_int, tag: c_int, plSide: c_int) -> bool {
    if r#type == IT_POWERUP
        && (tag == PW_FORCE_ENLIGHTENED_LIGHT || tag == PW_FORCE_ENLIGHTENED_DARK)
    {
        if plSide == FORCE_LIGHTSIDE {
            if tag == PW_FORCE_ENLIGHTENED_DARK {
                return true;
            }
        } else if plSide == FORCE_DARKSIDE {
            if tag == PW_FORCE_ENLIGHTENED_LIGHT {
                return true;
            }
        }
    }

    false
}

/// Raven `CG_Item` — draws a pickup: the holo cone over weapons/powerups, the
/// simple-item sprite, or the real model with its bob, spin and respawn fade.
///
/// PORT-NOTE: Raven's `wi` local is only read inside the barrel block at the
/// bottom, which is commented out ("rww - As far as I can see, this is
/// useless"). The dropped-weapon arm's `wi = &cg_weapons[item->giTag]` is
/// therefore a bare address-of with no read, so it is dropped rather than
/// transcribed into an index that would trap for a dropped non-weapon whose
/// `giTag` runs past `MAX_WEAPONS`.
/// Source: `oracle/codemp/cgame/cg_ents.c:1885-2327`
pub fn CG_Item(ctx: &mut CgContext, centNum: usize) {
    let mut ent: refEntity_t;

    let modelindex = ctx.world.entity(centNum).currentState.modelindex;
    if modelindex >= bg_numItems {
        CG_Error(ctx, &format!("Bad item index {modelindex} on entity"));
        return;
    }

    // Ghoul2 Insert Start
    let eFlags = ctx.world.entity(centNum).currentState.eFlags;
    if (eFlags & EF_NODRAW) != 0 && (eFlags & EF_ITEMPLACEHOLDER) != 0 {
        ctx.world.entity_mut(centNum).currentState.eFlags &= !EF_NODRAW;
    }

    if modelindex == 0 {
        return;
    }

    let item = &bg_itemlist[modelindex as usize];

    // Raven derefs `cg.snap` for the force side at three points below with no
    // null check; with no snapshot the port reads side 0, which greys nothing
    // (§F19).
    let forceSide = ctx
        .world
        .cg
        .snap_ref()
        .map_or(0, |snap| snap.ps.fd.forceSide);

    if (item.giType() == IT_WEAPON || item.giType() == IT_POWERUP)
        && (ctx.world.entity(centNum).currentState.eFlags & EF_DROPPEDWEAPON) == 0
        && ctx.world.cvars.cg_simpleItems.integer == 0
    {
        let mut uNorm: vec3_t = [0.0; 3];

        VectorClear(&mut uNorm);

        uNorm[2] = 1.0;

        ent = refEntity_t::zeroed();

        ent.customShader = 0;
        _VectorCopy(ctx.world.entity(centNum).lerpOrigin, &mut ent.origin);
        let angles = ctx.world.entity(centNum).currentState.angles;
        _VectorCopy(angles, &mut ctx.world.entity_mut(centNum).lerpAngles);
        let lerpAngles = ctx.world.entity(centNum).lerpAngles;
        AnglesToAxis(lerpAngles, ent.axis.as_mut_ptr());
        ent.hModel = ctx.world.cgs.media.itemHoloModel;

        let doGrey = CG_GreyItem(item.giType(), item.giTag(), forceSide);

        if doGrey {
            ent.renderfx |= RF_RGB_TINT;

            ent.shaderRGBA[0] = 150;
            ent.shaderRGBA[1] = 150;
            ent.shaderRGBA[2] = 150;
        }

        trap::R_AddRefEntityToScene(ctx.engine, &ent);

        if !doGrey {
            let itemCone = ctx.world.cgs.effects.itemCone;
            let origin = ent.origin;
            trap::FX_PlayEffectID(ctx.engine, itemCone, &origin, &uNorm, -1, -1);
        }
    }

    // if set to invisible, skip
    if (ctx.world.entity(centNum).currentState.eFlags & EF_NODRAW) != 0 {
        return;
    }
    // Ghoul2 Insert End

    if ctx.world.cvars.cg_simpleItems.integer != 0 && item.giType() != IT_TEAM {
        ent = refEntity_t::zeroed();
        ent.reType = refEntityType_t::RT_SPRITE;
        _VectorCopy(ctx.world.entity(centNum).lerpOrigin, &mut ent.origin);
        ent.radius = 14.0;
        ent.customShader = ctx.world.cg_items[modelindex as usize].icon;
        ent.shaderRGBA[0] = 255;
        ent.shaderRGBA[1] = 255;
        ent.shaderRGBA[2] = 255;

        ent.origin[2] += 16.0;

        if item.giType() != IT_POWERUP || item.giTag() != PW_FORCE_BOON {
            ent.renderfx |= RF_FORCE_ENT_ALPHA;
        }

        if (ctx.world.entity(centNum).currentState.eFlags & EF_ITEMPLACEHOLDER) != 0 {
            if item.giType() == IT_POWERUP && item.giTag() == PW_FORCE_BOON {
                return;
            }
            ent.shaderRGBA[0] = 200;
            ent.shaderRGBA[1] = 200;
            ent.shaderRGBA[2] = 200;
            // `sin` and the literals are doubles, so the pulse is a double that
            // truncates into the byte slot
            ent.shaderRGBA[3] =
                (150.0 + (ctx.world.cg.time as f64 * 0.01).sin() * 30.0) as i32 as u8;
        } else {
            ent.shaderRGBA[3] = 255;
        }

        if CG_GreyItem(item.giType(), item.giTag(), forceSide) {
            ent.shaderRGBA[0] = 100;
            ent.shaderRGBA[1] = 100;
            ent.shaderRGBA[2] = 100;

            ent.shaderRGBA[3] = 200;

            if item.giTag() == PW_FORCE_ENLIGHTENED_LIGHT {
                ent.customShader =
                    trap::R_RegisterShader(ctx.engine, "gfx/misc/mp_light_enlight_disable");
            } else {
                ent.customShader =
                    trap::R_RegisterShader(ctx.engine, "gfx/misc/mp_dark_enlight_disable");
            }
        }
        trap::R_AddRefEntityToScene(ctx.engine, &ent);
        return;
    }

    if (item.giType() == IT_WEAPON || item.giType() == IT_POWERUP)
        && (ctx.world.entity(centNum).currentState.eFlags & EF_DROPPEDWEAPON) == 0
    {
        ctx.world.entity_mut(centNum).lerpOrigin[2] += 16.0;
    }

    if ((ctx.world.entity(centNum).currentState.eFlags & EF_DROPPEDWEAPON) == 0
        || item.giType() == IT_POWERUP)
        && (item.giType() == IT_WEAPON || item.giType() == IT_POWERUP)
    {
        // items bob up and down continuously
        let number = ctx.world.entity(centNum).currentState.number;
        let scale = (0.005 + number as f64 * 0.00001) as f32;
        // `(cg.time + 1000) * scale` stays single-precision, then widens for
        // `cos`; the `4 +` sum is double and narrows on the `+=`
        let time = ctx.world.cg.time;
        let bob = 4.0 + ((((time + 1000) as f32) * scale) as f64).cos() * 4.0;
        let z = ctx.world.entity(centNum).lerpOrigin[2];
        ctx.world.entity_mut(centNum).lerpOrigin[2] = (z as f64 + bob) as f32;
    } else {
        if item.giType() == IT_HOLDABLE {
            if item.giTag() == HI_SEEKER {
                ctx.world.entity_mut(centNum).lerpOrigin[2] += 5.0;
            }
            if item.giTag() == HI_SHIELD {
                ctx.world.entity_mut(centNum).lerpOrigin[2] += 2.0;
            }
            if item.giTag() == HI_BINOCULARS {
                ctx.world.entity_mut(centNum).lerpOrigin[2] += 2.0;
            }
        }
        if item.giType() == IT_HEALTH {
            ctx.world.entity_mut(centNum).lerpOrigin[2] += 2.0;
        }
        if item.giType() == IT_ARMOR && item.quantity == 100 {
            ctx.world.entity_mut(centNum).lerpOrigin[2] += 7.0;
        }
    }

    ent = refEntity_t::zeroed();

    if ((ctx.world.entity(centNum).currentState.eFlags & EF_DROPPEDWEAPON) == 0
        || item.giType() == IT_POWERUP)
        && (item.giType() == IT_WEAPON || item.giType() == IT_POWERUP)
    {
        //only weapons and powerups rotate now
        // autorotate at one of two speeds
        let autoAngles = ctx.world.cg.autoAngles;
        _VectorCopy(autoAngles, &mut ctx.world.entity_mut(centNum).lerpAngles);
        AxisCopy(ctx.world.cg.autoAxis.as_mut_ptr(), ent.axis.as_mut_ptr());
    } else {
        let angles = ctx.world.entity(centNum).currentState.angles;
        _VectorCopy(angles, &mut ctx.world.entity_mut(centNum).lerpAngles);
        let lerpAngles = ctx.world.entity(centNum).lerpAngles;
        AnglesToAxis(lerpAngles, ent.axis.as_mut_ptr());
    }

    // the weapons have their origin where they attatch to player
    // models, so we need to offset them or they will rotate
    // eccentricly
    if (ctx.world.entity(centNum).currentState.eFlags & EF_DROPPEDWEAPON) == 0 {
        if item.giType() == IT_WEAPON {
            let mid = ctx.world.cg_weapons[item.giTag() as usize].weaponMidpoint;
            let axis = ent.axis;
            let cent = ctx.world.entity_mut(centNum);
            cent.lerpOrigin[0] -= mid[0] * axis[0][0] + mid[1] * axis[1][0] + mid[2] * axis[2][0];
            cent.lerpOrigin[1] -= mid[0] * axis[0][1] + mid[1] * axis[1][1] + mid[2] * axis[2][1];
            cent.lerpOrigin[2] -= mid[0] * axis[0][2] + mid[1] * axis[1][2] + mid[2] * axis[2][2];

            cent.lerpOrigin[2] += 8.0; // an extra height boost
        }
    } else {
        let zDrop = match item.giTag() {
            WP_BLASTER => 12.0,
            WP_DISRUPTOR => 13.0,
            WP_BOWCASTER => 16.0,
            WP_REPEATER => 12.0,
            WP_DEMP2 => 10.0,
            WP_FLECHETTE => 6.0,
            WP_ROCKET_LAUNCHER => 11.0,
            WP_THERMAL => 12.0,
            WP_TRIP_MINE => 16.0,
            WP_DET_PACK => 16.0,
            _ => 8.0,
        };
        ctx.world.entity_mut(centNum).lerpOrigin[2] -= zDrop;
    }

    ent.hModel = ctx.world.cg_items[modelindex as usize].models[0];
    // Ghoul2 Insert Start
    ent.ghoul2 = ctx.world.cg_items[modelindex as usize].g2Models[0];
    ent.radius = ctx.world.cg_items[modelindex as usize].radius[0];
    _VectorCopy(ctx.world.entity(centNum).lerpAngles, &mut ent.angles);
    // Ghoul2 Insert End
    _VectorCopy(ctx.world.entity(centNum).lerpOrigin, &mut ent.origin);
    _VectorCopy(ctx.world.entity(centNum).lerpOrigin, &mut ent.oldorigin);

    ent.nonNormalizedAxes = qfalse;

    // if just respawned, slowly scale up

    let msec = ctx.world.cg.time - ctx.world.entity(centNum).miscTime;

    if CG_GreyItem(item.giType(), item.giTag(), forceSide) {
        ent.renderfx |= RF_RGB_TINT;

        ent.shaderRGBA[0] = 150;
        ent.shaderRGBA[1] = 150;
        ent.shaderRGBA[2] = 150;

        ent.renderfx |= RF_FORCE_ENT_ALPHA;

        ent.shaderRGBA[3] = 200;

        if item.giTag() == PW_FORCE_ENLIGHTENED_LIGHT {
            ent.customShader =
                trap::R_RegisterShader(ctx.engine, "gfx/misc/mp_light_enlight_disable");
        } else {
            ent.customShader =
                trap::R_RegisterShader(ctx.engine, "gfx/misc/mp_dark_enlight_disable");
        }

        trap::R_AddRefEntityToScene(ctx.engine, &ent);
        return;
    }

    let eFlags = ctx.world.entity(centNum).currentState.eFlags;
    if (eFlags & EF_ITEMPLACEHOLDER) != 0 {
        // item has been picked up
        if (eFlags & EF_DEAD) != 0 {
            // if item had been droped, don't show at all
            return;
        }

        ent.renderfx |= RF_RGB_TINT;
        ent.shaderRGBA[0] = 0;
        ent.shaderRGBA[1] = 200;
        ent.shaderRGBA[2] = 85;
        ent.customShader = ctx.world.cgs.media.itemRespawningPlaceholder;
    }

    // increase the size of the weapons when they are presented as items
    if item.giType() == IT_WEAPON {
        let axis0 = ent.axis[0];
        _VectorScale(axis0, 1.5, &mut ent.axis[0]);
        let axis1 = ent.axis[1];
        _VectorScale(axis1, 1.5, &mut ent.axis[1]);
        let axis2 = ent.axis[2];
        _VectorScale(axis2, 1.5, &mut ent.axis[2]);
        ent.nonNormalizedAxes = qtrue;
        //trap_S_AddLoopingSound( cent->currentState.number, cent->lerpOrigin, vec3_origin, cgs.media.weaponHoverSound );
    }

    if (eFlags & EF_DROPPEDWEAPON) == 0
        && (item.giType() == IT_WEAPON || item.giType() == IT_POWERUP)
    {
        ent.renderfx |= RF_MINLIGHT;
    }

    if item.giType() != IT_TEAM
        && msec >= 0
        && msec < ITEM_SCALEUP_TIME
        && (eFlags & EF_ITEMPLACEHOLDER) == 0
        && (eFlags & EF_DROPPEDWEAPON) == 0
    {
        // if just respawned, fade in, but don't do this for flags.
        let alpha = msec as f32 / ITEM_SCALEUP_TIME as f32;
        let mut a = (alpha as f64 * 255.0) as c_int;
        if a <= 0 {
            a = 1;
        }

        ent.shaderRGBA[3] = a as u8;
        if item.giType() != IT_POWERUP || item.giTag() != PW_FORCE_BOON {
            //boon model uses a different blending mode for the sprite inside and doesn't look proper with this method
            ent.renderfx |= RF_FORCE_ENT_ALPHA;
        }
        trap::R_AddRefEntityToScene(ctx.engine, &ent);

        ent.renderfx &= !RF_FORCE_ENT_ALPHA;

        // Now draw the static shader over it.
        // Alpha in over half the time, out over half.

        //alpha = sin(M_PI*alpha);
        // PORT-NOTE: Raven recomputes `a = alpha*255`, flips it to `255 - a`
        // and clamps it to 1..255 here, but both blocks that read it back into
        // `shaderRGBA` are commented out - the recompute is dead, so it is
        // recorded here instead of transcribed.

        ent.customShader = ctx.world.cgs.media.itemRespawningRezOut;

        ent.renderfx |= RF_RGB_TINT;
        ent.shaderRGBA[0] = 0;
        ent.shaderRGBA[1] = 200;
        ent.shaderRGBA[2] = 85;

        trap::R_AddRefEntityToScene(ctx.engine, &ent);
    } else {
        // add to refresh list  -- normal item
        if item.giType() == IT_TEAM && (item.giTag() == PW_REDFLAG || item.giTag() == PW_BLUEFLAG) {
            ent.modelScale[0] = 0.7;
            ent.modelScale[1] = 0.7;
            ent.modelScale[2] = 0.7;
            ScaleModelAxis(&mut ent);
        }
        trap::R_AddRefEntityToScene(ctx.engine, &ent);
    }

    // accompanying rings / spheres for powerups
    if ctx.world.cvars.cg_simpleItems.integer == 0 {
        let mut spinAngles: vec3_t = [0.0; 3];

        VectorClear(&mut spinAngles);

        if item.giType() == IT_HEALTH || item.giType() == IT_POWERUP {
            // Raven assigns `ent.hModel` inside the condition, so the second
            // model handle lands on the refEntity either way
            ent.hModel = ctx.world.cg_items[modelindex as usize].models[1];
            if ent.hModel != 0 {
                if item.giType() == IT_POWERUP {
                    ent.origin[2] += 12.0;
                    spinAngles[1] = ((ctx.world.cg.time & 1023) * 360) as f32 / -1024.0;
                }
                AnglesToAxis(spinAngles, ent.axis.as_mut_ptr());

                trap::R_AddRefEntityToScene(ctx.engine, &ent);
            }
        }
    }
}

/// Raven `CG_CreateDistortionTrailPart` — one segment of the merr-sonn trail's
/// screen-distortion ribbon.
/// Source: `oracle/codemp/cgame/cg_ents.c:2331-2393`
pub fn CG_CreateDistortionTrailPart(ctx: &mut CgContext, centNum: usize, scale: f32, pos: vec3_t) {
    let mut ang: vec3_t = [0.0; 3];

    if ctx.world.cvars.cg_renderToTextureFX.integer == 0 {
        return;
    }
    let mut ent = refEntity_t::zeroed();

    _VectorCopy(pos, &mut ent.origin);

    let vieworg = ctx.world.cg.refdef.vieworg;
    _VectorSubtract(ent.origin, vieworg, &mut ent.axis[0]);
    let vLen = VectorLength(ent.axis[0]);
    if VectorNormalize(&mut ent.axis[0]) <= 0.1 {
        // Entity is right on vieworg.  quit.
        return;
    }

    _VectorCopy(ctx.world.entity(centNum).lerpAngles, &mut ang);
    ang[PITCH] += 90.0;
    AnglesToAxis(ang, ent.axis.as_mut_ptr());

    //radius must be a power of 2, and is the actual captured texture size
    if vLen < 512.0 {
        ent.radius = 256.0;
    } else if vLen < 1024.0 {
        ent.radius = 128.0;
    } else if vLen < 2048.0 {
        ent.radius = 64.0;
    } else {
        ent.radius = 32.0;
    }

    ent.modelScale[0] = scale;
    ent.modelScale[1] = scale;
    ent.modelScale[2] = scale * 16.0;
    ScaleModelAxis(&mut ent);

    ent.hModel = trap::R_RegisterModel(ctx.engine, "models/weapons2/merr_sonn/trailmodel.md3");
    ent.customShader = ctx.world.cgs.media.itemRespawningRezOut; //cgs.media.cloakedShader;//cgs.media.halfShieldShader;

    // Raven's `#if 1` alpha arm is the live one; the `#else` bare
    // `RF_DISTORTION` never compiles. The RGBA values are float literals in
    // Raven, exact in the byte slots.
    ent.renderfx = RF_DISTORTION | RF_FORCE_ENT_ALPHA;
    ent.shaderRGBA[0] = 255;
    ent.shaderRGBA[1] = 255;
    ent.shaderRGBA[2] = 255;
    ent.shaderRGBA[3] = 100;

    trap::R_AddRefEntityToScene(ctx.engine, &ent);
}

/// Raven `CG_Mover` — the bmodel/model mover draw, plus the hyperspace brush's
/// stuck-to-the-view special case.
/// Source: `oracle/codemp/cgame/cg_ents.c:2824-2926`
pub fn CG_Mover(ctx: &mut CgContext, centNum: usize) {
    // create the render entity
    let mut ent = refEntity_t::zeroed();

    if (ctx.world.entity(centNum).currentState.eFlags2 & EF2_HYPERSPACE) != 0 {
        //I'm the hyperspace brush
        let mut drawMe = false;
        let time = ctx.world.cg.time;
        let hyperSpaceTime = ctx.world.cg.predictedVehicleState.hyperSpaceTime;
        if ctx.world.cg.predictedPlayerState.m_iVehicleNum != 0
            && hyperSpaceTime != 0
            && (time - hyperSpaceTime) < HYPERSPACE_TIME
            && (time - hyperSpaceTime) > 1000
        {
            let inIntermission = ctx.world.cg.snap_ref().map_or(false, |snap| {
                snap.ps.pm_type == pmtype_t::PM_INTERMISSION as c_int
            });
            if inIntermission {
                //in the intermission, stop drawing hyperspace ent
            } else if (ctx.world.cg.predictedVehicleState.eFlags2 & EF2_HYPERSPACE) != 0 {
                //actually hyperspacing now
                let timeFrac =
                    (time - hyperSpaceTime - 1000) as f32 / (HYPERSPACE_TIME - 1000) as f32;
                if timeFrac < (HYPERSPACE_TELEPORT_FRAC + 0.1) {
                    //still in hyperspace or just popped out
                    let alpha = if timeFrac < 0.5 { timeFrac / 0.5 } else { 1.0 };
                    drawMe = true;
                    let vieworg = ctx.world.cg.refdef.vieworg;
                    let viewaxis0 = ctx.world.cg.refdef.viewaxis[0];
                    _VectorMA(
                        vieworg,
                        1000.0 + ((1.0 - timeFrac) * 1000.0),
                        viewaxis0,
                        &mut ctx.world.entity_mut(centNum).lerpOrigin,
                    );
                    let viewangles = ctx.world.cg.refdef.viewangles;
                    VectorSet(
                        &mut ctx.world.entity_mut(centNum).lerpAngles,
                        viewangles[PITCH],
                        viewangles[YAW] - 90.0,
                        0.0,
                    ); //cos( ( cg.time + 1000 ) *  scale ) * 4 );
                    ent.shaderRGBA[0] = 255;
                    ent.shaderRGBA[1] = 255;
                    ent.shaderRGBA[2] = 255;
                    ent.shaderRGBA[3] = (alpha * 255.0) as i32 as u8;
                }
            }
        }
        if !drawMe {
            //else, never draw
            return;
        }
    }

    if (ctx.world.entity(centNum).currentState.eFlags & EF_RADAROBJECT) != 0 {
        CG_AddRadarEnt(ctx.world, centNum);
    }

    _VectorCopy(ctx.world.entity(centNum).lerpOrigin, &mut ent.origin);
    _VectorCopy(ctx.world.entity(centNum).lerpOrigin, &mut ent.oldorigin);
    AnglesToAxis(ctx.world.entity(centNum).lerpAngles, ent.axis.as_mut_ptr());

    ent.renderfx = RF_NOSHADOW;
    // Ghoul2 Insert Start
    CG_SetGhoul2Info(&mut ent, ctx.world.entity(centNum));
    // Ghoul2 Insert End
    // flicker between two skins (FIXME?)
    ent.skinNum = (ctx.world.cg.time >> 6) & 1;

    // get the model, either as a bmodel or a modelindex
    let s1_solid = ctx.world.entity(centNum).currentState.solid;
    let s1_modelindex = ctx.world.entity(centNum).currentState.modelindex;
    if s1_solid == SOLID_BMODEL {
        ent.hModel = ctx.world.cgs.inlineDrawModel[s1_modelindex as usize];
    } else {
        ent.hModel = ctx.world.cgs.gameModels[s1_modelindex as usize];
    }

    if (ctx.world.entity(centNum).currentState.eFlags & EF_SHADER_ANIM) != 0 {
        ent.renderfx |= RF_SETANIMINDEX;
        ent.skinNum = ctx.world.entity(centNum).currentState.frame;
        //ent.shaderTime = cg.time*0.001f - s1->frame/s1->time;//NOTE: s1->time is number of frames
    }

    // add to refresh list
    trap::R_AddRefEntityToScene(ctx.engine, &ent);

    // add the secondary model
    let s1_modelindex2 = ctx.world.entity(centNum).currentState.modelindex2;
    if s1_modelindex2 != 0 {
        ent.skinNum = 0;
        ent.hModel = ctx.world.cgs.gameModels[s1_modelindex2 as usize];
        let iModelScale = ctx.world.entity(centNum).currentState.iModelScale;
        if iModelScale != 0 {
            //custom model2 scale
            let modelScale = if ctx.world.entity(centNum).currentState.legsFlip != qfalse {
                //scalar
                iModelScale as f32
            } else {
                //percentage
                iModelScale as f32 / 100.0
            };
            ent.modelScale[0] = modelScale;
            ent.modelScale[1] = modelScale;
            ent.modelScale[2] = modelScale;
            ScaleModelAxis(&mut ent);
        }
        trap::R_AddRefEntityToScene(ctx.engine, &ent);
    }
}

/// Raven `CG_Beam`.
///
/// Raven: Also called as an event.
/// Source: `oracle/codemp/cgame/cg_ents.c:2935-2959`
pub fn CG_Beam(ctx: &mut CgContext, centNum: usize) {
    // create the render entity
    let mut ent = refEntity_t::zeroed();

    let cent = ctx.world.entity(centNum);
    let s1 = &cent.currentState;

    _VectorCopy(s1.pos.trBase, &mut ent.origin);
    _VectorCopy(s1.origin2, &mut ent.oldorigin);
    AxisClear(ent.axis.as_mut_ptr());
    ent.reType = refEntityType_t::RT_BEAM;

    ent.renderfx = RF_NOSHADOW;
    // Ghoul2 Insert Start
    CG_SetGhoul2Info(&mut ent, cent);

    // Ghoul2 Insert End
    // add to refresh list
    trap::R_AddRefEntityToScene(ctx.engine, &ent);
}

/// Raven `CG_Portal` — the portal-surface marker entity the renderer reads its
/// camera from.
/// Source: `oracle/codemp/cgame/cg_ents.c:2967-2998`
pub fn CG_Portal(ctx: &mut CgContext, centNum: usize) {
    // create the render entity
    let mut ent = refEntity_t::zeroed();

    let cent = ctx.world.entity(centNum);
    let s1 = &cent.currentState;

    _VectorCopy(cent.lerpOrigin, &mut ent.origin);
    _VectorCopy(s1.origin2, &mut ent.oldorigin);
    ByteToDir(s1.eventParm, &mut ent.axis[0]);
    let axis0 = ent.axis[0];
    PerpendicularVector(&mut ent.axis[1], axis0);

    // negating this tends to get the directions like they want
    // we really should have a camera roll value
    let axis1 = ent.axis[1];
    _VectorSubtract(vec3_origin, axis1, &mut ent.axis[1]);

    let axis0 = ent.axis[0];
    let axis1 = ent.axis[1];
    CrossProduct(axis0, axis1, &mut ent.axis[2]);
    ent.reType = refEntityType_t::RT_PORTALSURFACE;
    ent.oldframe = s1.powerups;
    ent.frame = s1.frame; // rotation speed
    ent.skinNum = (s1.clientNum as f64 / 256.0 * 360.0) as c_int; // roll offset
                                                                  // Ghoul2 Insert Start
    CG_SetGhoul2Info(&mut ent, cent);
    // Ghoul2 Insert End
    // add to refresh list
    trap::R_AddRefEntityToScene(ctx.engine, &ent);
}

/// Raven `CG_AdjustPositionForMover` — moves the given position from one time
/// to another along a mover's trajectory.
/// Source: `oracle/codemp/cgame/cg_ents.c:3008-3036`
pub fn CG_AdjustPositionForMover(
    world: &CgWorld,
    r#in: vec3_t,
    moverNum: c_int,
    fromTime: c_int,
    toTime: c_int,
    out: &mut vec3_t,
) {
    let mut oldOrigin: vec3_t = [0.0; 3];
    let mut origin: vec3_t = [0.0; 3];
    let mut deltaOrigin: vec3_t = [0.0; 3];
    let mut oldAngles: vec3_t = [0.0; 3];
    let mut angles: vec3_t = [0.0; 3];
    let mut deltaAngles: vec3_t = [0.0; 3];

    if moverNum <= 0 || moverNum >= ENTITYNUM_MAX_NORMAL {
        _VectorCopy(r#in, out);
        return;
    }

    let cent = world.entity(moverNum as usize);
    if cent.currentState.eType != entityType_t::ET_MOVER as c_int {
        _VectorCopy(r#in, out);
        return;
    }

    BG_EvaluateTrajectory(&cent.currentState.pos, fromTime, &mut oldOrigin);
    BG_EvaluateTrajectory(&cent.currentState.apos, fromTime, &mut oldAngles);

    BG_EvaluateTrajectory(&cent.currentState.pos, toTime, &mut origin);
    BG_EvaluateTrajectory(&cent.currentState.apos, toTime, &mut angles);

    _VectorSubtract(origin, oldOrigin, &mut deltaOrigin);
    _VectorSubtract(angles, oldAngles, &mut deltaAngles);

    _VectorAdd(r#in, deltaOrigin, out);

    // FIXME: origin change when on a rotating object
}

/// Raven `CG_InterpolateEntityPosition` — lerps an entity between the two
/// snapshots instead of extrapolating its trajectory.
/// Source: `oracle/codemp/cgame/cg_ents.c:3043-3070`
pub fn CG_InterpolateEntityPosition(ctx: &mut CgContext, centNum: usize) {
    let mut current: vec3_t = [0.0; 3];
    let mut next: vec3_t = [0.0; 3];

    // it would be an internal error to find an entity that interpolates without
    // a snapshot ahead of the current one
    let nextServerTime = ctx.world.cg.next_snap_ref().map(|snap| snap.serverTime);
    let Some(nextServerTime) = nextServerTime else {
        CG_Error(ctx, "CG_InterpoateEntityPosition: cg.nextSnap == NULL");
        return;
    };

    // Raven derefs `cg.snap` right below with no null check; with no current
    // snapshot the port leaves the entity's lerped position alone (§F19).
    let Some(serverTime) = ctx.world.cg.snap_ref().map(|snap| snap.serverTime) else {
        return;
    };

    let f = ctx.world.cg.frameInterpolation;

    // this will linearize a sine or parabolic curve, but it is important
    // to not extrapolate player positions if more recent data is available
    let pos = ctx.world.entity(centNum).currentState.pos;
    let nextPos = ctx.world.entity(centNum).nextState.pos;
    let apos = ctx.world.entity(centNum).currentState.apos;
    let nextApos = ctx.world.entity(centNum).nextState.apos;

    BG_EvaluateTrajectory(&pos, serverTime, &mut current);
    BG_EvaluateTrajectory(&nextPos, nextServerTime, &mut next);

    let cent = ctx.world.entity_mut(centNum);
    cent.lerpOrigin[0] = current[0] + f * (next[0] - current[0]);
    cent.lerpOrigin[1] = current[1] + f * (next[1] - current[1]);
    cent.lerpOrigin[2] = current[2] + f * (next[2] - current[2]);

    BG_EvaluateTrajectory(&apos, serverTime, &mut current);
    BG_EvaluateTrajectory(&nextApos, nextServerTime, &mut next);

    cent.lerpAngles[0] = LerpAngle(current[0], next[0], f);
    cent.lerpAngles[1] = LerpAngle(current[1], next[1], f);
    cent.lerpAngles[2] = LerpAngle(current[2], next[2], f);
}

/// Raven `CG_TeamBase` — draws the CTF flag-base model under a team entity.
/// Source: `oracle/codemp/cgame/cg_ents.c:3209-3233`
pub fn CG_TeamBase(ctx: &mut CgContext, centNum: usize) {
    if ctx.world.cgs.gametype == GT_CTF || ctx.world.cgs.gametype == GT_CTY {
        // show the flag base
        let mut model = refEntity_t::zeroed();
        let cent = ctx.world.entity(centNum);
        model.reType = refEntityType_t::RT_MODEL;
        _VectorCopy(cent.lerpOrigin, &mut model.lightingOrigin);
        _VectorCopy(cent.lerpOrigin, &mut model.origin);
        AnglesToAxis(cent.currentState.angles, model.axis.as_mut_ptr());
        if cent.currentState.modelindex == TEAM_RED {
            model.hModel = ctx.world.cgs.media.redFlagBaseModel;
        } else if cent.currentState.modelindex == TEAM_BLUE {
            model.hModel = ctx.world.cgs.media.blueFlagBaseModel;
        } else {
            model.hModel = ctx.world.cgs.media.neutralFlagBaseModel;
        }

        if cent.currentState.eType != entityType_t::ET_NPC as c_int {
            // do not do this for g2animents
            trap::R_AddRefEntityToScene(ctx.engine, &model);
        }
    }
}

/// Raven `CG_Cube` — six FX polys tracing an axis-aligned box, one pair of
/// faces per axis.
/// Source: `oracle/codemp/cgame/cg_ents.c:3760-3809`
pub fn CG_Cube(ctx: &mut CgContext, mins: vec3_t, maxs: vec3_t, color: vec3_t, alpha: f32) {
    let rot: vec3_t = [0.0, 0.0, 0.0];
    let mut vec: [usize; 3] = [0, 1, 2];
    let mut apArgs = addpolyArgStruct_t {
        p: [[0.0; 3]; 4],
        ev: [[0.0; 2]; 4],
        numVerts: 0,
        vel: [0.0; 3],
        accel: [0.0; 3],
        alpha1: 0.0,
        alpha2: 0.0,
        alphaParm: 0.0,
        rgb1: [0.0; 3],
        rgb2: [0.0; 3],
        rgbParm: 0.0,
        rotationDelta: [0.0; 3],
        bounce: 0.0,
        motionDelay: 0,
        killTime: 0,
        shader: 0,
        flags: 0,
    };

    for _axis in 0..3 {
        for i in 0..3 {
            if vec[i] > 2 {
                vec[i] = 0;
            }
        }

        apArgs.p[0][vec[1]] = mins[vec[1]];
        apArgs.p[0][vec[2]] = mins[vec[2]];

        apArgs.p[1][vec[1]] = mins[vec[1]];
        apArgs.p[1][vec[2]] = maxs[vec[2]];

        apArgs.p[2][vec[1]] = maxs[vec[1]];
        apArgs.p[2][vec[2]] = maxs[vec[2]];

        apArgs.p[3][vec[1]] = maxs[vec[1]];
        apArgs.p[3][vec[2]] = mins[vec[2]];

        //- face
        apArgs.p[0][vec[0]] = mins[vec[0]];
        apArgs.p[1][vec[0]] = mins[vec[0]];
        apArgs.p[2][vec[0]] = mins[vec[0]];
        apArgs.p[3][vec[0]] = mins[vec[0]];

        apArgs.numVerts = 4;
        apArgs.alpha1 = alpha;
        apArgs.alpha2 = alpha;
        _VectorCopy(color, &mut apArgs.rgb1);
        _VectorCopy(color, &mut apArgs.rgb2);
        _VectorCopy(rot, &mut apArgs.rotationDelta);
        apArgs.killTime = ctx.world.cg.frametime;
        apArgs.shader = ctx.world.cgs.media.solidWhite;

        trap::FX_AddPoly(ctx.engine, &mut apArgs);

        //+ face
        apArgs.p[0][vec[0]] = maxs[vec[0]];
        apArgs.p[1][vec[0]] = maxs[vec[0]];
        apArgs.p[2][vec[0]] = maxs[vec[0]];
        apArgs.p[3][vec[0]] = maxs[vec[0]];

        trap::FX_AddPoly(ctx.engine, &mut apArgs);

        for i in 0..3 {
            vec[i] += 1;
        }
    }
}

/// Raven `CG_EntityEffects` — the looping sound and constant-light glow every
/// entity type gets each frame.
/// Source: `oracle/codemp/cgame/cg_ents.c:264-327`
pub fn CG_EntityEffects(ctx: &mut CgContext, centNum: usize) {
    // update sound origins
    CG_SetEntitySoundPosition(ctx, centNum);

    // add loop sound
    let loopSound = ctx.world.entity(centNum).currentState.loopSound;
    let loopIsSoundset = ctx.world.entity(centNum).currentState.loopIsSoundset;
    let number = ctx.world.entity(centNum).currentState.number;
    if loopSound != 0 || (loopIsSoundset != qfalse && number >= MAX_CLIENTS_I32) {
        let mut realSoundIndex: sfxHandle_t = -1;

        if loopIsSoundset != qfalse && number >= MAX_CLIENTS_I32 {
            //If this is so, then first get our soundset from the index, and loopSound actually contains which part of the set to
            //use rather than a sound index (BMS_START [0], BMS_MID [1], or BMS_END [2]). Typically loop sounds will be BMS_MID.
            let soundSetIndex = ctx.world.entity(centNum).currentState.soundSetIndex;
            let soundSet = CG_ConfigString(ctx, CS_AMBIENT_SET + soundSetIndex);

            if !soundSet.is_empty() {
                realSoundIndex = trap::AS_GetBModelSound(ctx.engine, &soundSet, loopSound);
            }
        } else {
            realSoundIndex = ctx.world.cgs.gameSounds[loopSound as usize];
        }

        //rww - doors and things with looping sounds have a crazy origin (being brush models and all)
        if realSoundIndex != -1 {
            let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
            if ctx.world.entity(centNum).currentState.solid == SOLID_BMODEL {
                let modelindex = ctx.world.entity(centNum).currentState.modelindex;
                let v = ctx.world.cgs.inlineModelMidpoints[modelindex as usize];
                let mut origin: vec3_t = [0.0; 3];
                _VectorAdd(lerpOrigin, v, &mut origin);
                trap::S_AddLoopingSound(ctx.engine, number, &origin, &vec3_origin, realSoundIndex);
            } else if ctx.world.entity(centNum).currentState.eType
                != entityType_t::ET_SPEAKER as c_int
            {
                trap::S_AddLoopingSound(
                    ctx.engine,
                    number,
                    &lerpOrigin,
                    &vec3_origin,
                    realSoundIndex,
                );
            } else {
                trap::S_AddRealLoopingSound(
                    ctx.engine,
                    number,
                    &lerpOrigin,
                    &vec3_origin,
                    realSoundIndex,
                );
            }
        }
    }

    // constant light glow
    let constantLight = ctx.world.entity(centNum).currentState.constantLight;
    if constantLight != 0 {
        let cl = constantLight;
        let r = cl & 255;
        let g = (cl >> 8) & 255;
        let b = (cl >> 16) & 255;
        let i = ((cl >> 24) & 255) * 4;
        let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
        trap::R_AddLightToScene(
            ctx.engine,
            &lerpOrigin,
            i as f32,
            r as f32,
            g as f32,
            b as f32,
        );
    }
}

/// Raven `CG_Missile` — the in-flight missile draw: the ghoul2 saber piece, the
/// per-weapon trail/dlight/sound, the spin, and jedimaster's dropped-saber glow.
///
/// PORT-NOTE: Raven's `refEntity_t ent` is a raw stack local, so the
/// `cent->ghoul2` radius store at :2483 lands *before* the `memset` at :2586
/// wipes it — a dead store. The port drops the dead store and declares `ent`
/// at the memset site (wave-1 `difLen` precedent).
/// Source: `oracle/codemp/cgame/cg_ents.c:2426-2734`
pub fn CG_Missile(ctx: &mut CgContext, centNum: usize) {
    let engine = ctx.engine;

    let mut s1_weapon = ctx.world.entity(centNum).currentState.weapon;
    if s1_weapon > WP_NUM_WEAPONS && s1_weapon != G2_MODEL_PART {
        ctx.world.entity_mut(centNum).currentState.weapon = 0;
        s1_weapon = 0;
    }

    // Two indices slip past Raven's guard and read off the end of
    // `cg_weapons[MAX_WEAPONS]` (19 entries): `G2_MODEL_PART` (50) on a
    // model-part missile with no ghoul2 instance, and `WP_NUM_WEAPONS` itself
    // (19), which `> WP_NUM_WEAPONS` lets through. Both are UB in Raven; the
    // port's checked indexing traps instead of reading past the array (§F19).
    let weaponIdx = if !ctx.world.entity(centNum).ghoul2.is_null() && s1_weapon == G2_MODEL_PART {
        WP_SABER as usize
    } else {
        s1_weapon as usize
    };

    if (ctx.world.entity(centNum).currentState.eFlags & EF_RADAROBJECT) != 0 {
        CG_AddRadarEnt(ctx.world, centNum);
    }

    if s1_weapon == WP_SABER {
        let modelindex = ctx.world.entity(centNum).currentState.modelindex;
        let serverSaberHitIndex = ctx.world.entity(centNum).serverSaberHitIndex;
        let eFlags = ctx.world.entity(centNum).currentState.eFlags;

        if (modelindex != serverSaberHitIndex || ctx.world.entity(centNum).ghoul2.is_null())
            && (eFlags & EF_NODRAW) == 0
        {
            //no g2, or server changed the model we are using
            let saberModel = CG_ConfigString(ctx, CS_MODELS + modelindex);

            ctx.world.entity_mut(centNum).serverSaberHitIndex = modelindex;

            if !ctx.world.entity(centNum).ghoul2.is_null() {
                //clean if we already have one (because server changed model string index)
                trap::G2API_CleanGhoul2Models(
                    engine,
                    &mut ctx.world.entity_mut(centNum).ghoul2 as *mut *mut c_void,
                );
                ctx.world.entity_mut(centNum).ghoul2 = null_mut();
            }

            if !saberModel.is_empty() {
                trap::G2API_InitGhoul2Model(
                    engine,
                    &mut ctx.world.entity_mut(centNum).ghoul2 as *mut *mut c_void,
                    &saberModel,
                    0,
                    0,
                    0,
                    0,
                    0,
                );
            } else {
                trap::G2API_InitGhoul2Model(
                    engine,
                    &mut ctx.world.entity_mut(centNum).ghoul2 as *mut *mut c_void,
                    "models/weapons2/saber/saber_w.glm",
                    0,
                    0,
                    0,
                    0,
                    0,
                );
            }
            return;
        } else if (eFlags & EF_NODRAW) != 0 {
            return;
        }
    }

    // Raven's `ent.radius = g2radius` store sat here (:2483) — dead, wiped by
    // the memset below; dropped per the PORT-NOTE above.

    // calculate the axis
    let angles = ctx.world.entity(centNum).currentState.angles;
    _VectorCopy(angles, &mut ctx.world.entity_mut(centNum).lerpAngles);

    let s1_otherEntityNum2 = ctx.world.entity(centNum).currentState.otherEntityNum2;
    let s1_eFlags = ctx.world.entity(centNum).currentState.eFlags;
    let s1_pos = ctx.world.entity(centNum).currentState.pos;
    let s1_number = ctx.world.entity(centNum).currentState.number;
    let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
    let time = ctx.world.cg.time;

    if s1_otherEntityNum2 != 0 && s1_weapon != WP_SABER {
        //using an over-ridden trail effect!
        let mut forward: vec3_t = [0.0; 3];

        if VectorNormalize2(s1_pos.trDelta, &mut forward) == 0.0 {
            forward[2] = 1.0;
        }

        let vehWeapon = &ctx.world.bg_state.g_vehWeaponInfo[s1_otherEntityNum2 as usize];
        let iShotFX = vehWeapon.iShotFX;
        let iModel = vehWeapon.iModel;
        let iLoopSound = vehWeapon.iLoopSound;

        if (s1_eFlags & EF_JETPACK_ACTIVE) != 0 //hack so we know we're a vehicle Weapon shot
            && (iShotFX != 0 || iModel != NULL_HANDLE)
        {
            //a vehicle with an override for the weapon trail fx or model
            trap::FX_PlayEffectID(engine, iShotFX, &lerpOrigin, &forward, -1, -1);
            if iLoopSound != 0 {
                let mut velocity: vec3_t = [0.0; 3];
                BG_EvaluateTrajectoryDelta(&s1_pos, time, &mut velocity);
                trap::S_AddLoopingSound(engine, s1_number, &lerpOrigin, &velocity, iLoopSound);
            }
            //add custom model
            if iModel == NULL_HANDLE {
                return;
            }
        } else {
            //a regular missile
            let gameEffect = ctx.world.cgs.gameEffects[s1_otherEntityNum2 as usize];
            trap::FX_PlayEffectID(engine, gameEffect, &lerpOrigin, &forward, -1, -1);
            let s1_loopSound = ctx.world.entity(centNum).currentState.loopSound;
            if s1_loopSound != 0 {
                let mut velocity: vec3_t = [0.0; 3];
                BG_EvaluateTrajectoryDelta(&s1_pos, time, &mut velocity);
                trap::S_AddLoopingSound(engine, s1_number, &lerpOrigin, &velocity, s1_loopSound);
            }
            //FIXME: if has a custom model, too, then set it and do rest of code below?
            return;
        }
    } else if (s1_eFlags & EF_ALT_FIRING) != 0 {
        // add trails
        // DEFERRED: `weapon->altMissileTrailFunc( cent, weapon )`.
        // `weaponInfo_t.altMissileTrailFunc` is still the transcription-era raw
        // `extern "C"` fn ptr and every store in `cg_weaponinit.rs` holds it at
        // `None` (DEC-46.4's closed trail-fn enum has not landed), so the
        // condition can never be true and there is nothing to dispatch to yet.
        // Source: `oracle/codemp/cgame/cg_ents.c:2530-2533`

        // add dynamic light
        let altMissileDlight = ctx.world.cg_weapons[weaponIdx].altMissileDlight;
        if altMissileDlight != 0.0 {
            let color = ctx.world.cg_weapons[weaponIdx].altMissileDlightColor;
            trap::R_AddLightToScene(
                engine,
                &lerpOrigin,
                altMissileDlight,
                color[0],
                color[1],
                color[2],
            );
        }

        // add missile sound
        let altMissileSound = ctx.world.cg_weapons[weaponIdx].altMissileSound;
        if altMissileSound != 0 {
            let mut velocity: vec3_t = [0.0; 3];

            BG_EvaluateTrajectoryDelta(&s1_pos, time, &mut velocity);

            trap::S_AddLoopingSound(engine, s1_number, &lerpOrigin, &velocity, altMissileSound);
        }

        //Don't draw something without a model
        if ctx.world.cg_weapons[weaponIdx].altMissileModel == NULL_HANDLE {
            return;
        }
    } else {
        // add trails
        // DEFERRED: `weapon->missileTrailFunc( cent, weapon )` — the same held
        // raw fn-ptr field as the alt arm above.
        // Source: `oracle/codemp/cgame/cg_ents.c:2558-2561`

        // add dynamic light
        let missileDlight = ctx.world.cg_weapons[weaponIdx].missileDlight;
        if missileDlight != 0.0 {
            let color = ctx.world.cg_weapons[weaponIdx].missileDlightColor;
            trap::R_AddLightToScene(
                engine,
                &lerpOrigin,
                missileDlight,
                color[0],
                color[1],
                color[2],
            );
        }

        // add missile sound
        let missileSound = ctx.world.cg_weapons[weaponIdx].missileSound;
        if missileSound != 0 {
            let mut velocity: vec3_t = [0.0; 3];

            BG_EvaluateTrajectoryDelta(&s1_pos, time, &mut velocity);

            trap::S_AddLoopingSound(engine, s1_number, &lerpOrigin, &velocity, missileSound);
        }

        //Don't draw something without a model
        //saber uses ghoul2 model, doesn't matter
        if ctx.world.cg_weapons[weaponIdx].missileModel == NULL_HANDLE
            && s1_weapon != WP_SABER
            && s1_weapon != G2_MODEL_PART
        {
            return;
        }
    }

    // create the render entity
    let mut ent = refEntity_t::zeroed();
    _VectorCopy(lerpOrigin, &mut ent.origin);
    _VectorCopy(lerpOrigin, &mut ent.oldorigin);
    /*
    Ghoul2 Insert Start
    */
    CG_SetGhoul2Info(&mut ent, ctx.world.entity(centNum));

    /*
    Ghoul2 Insert End
    */

    // flicker between two skins
    ent.skinNum = ctx.world.cg.clientFrame & 1;
    ent.renderfx = /*weapon->missileRenderfx | */RF_NOSHADOW;

    if (s1_eFlags & EF_JETPACK_ACTIVE) == 0 {
        if s1_weapon != WP_SABER && s1_weapon != G2_MODEL_PART {
            //if ( cent->currentState.eFlags | EF_ALT_FIRING )
            //rww - why was this like this?
            if (ctx.world.entity(centNum).currentState.eFlags & EF_ALT_FIRING) != 0 {
                ent.hModel = ctx.world.cg_weapons[weaponIdx].altMissileModel;
            } else {
                ent.hModel = ctx.world.cg_weapons[weaponIdx].missileModel;
            }
        }
    }
    //add custom model
    else {
        let iModel = ctx.world.bg_state.g_vehWeaponInfo[s1_otherEntityNum2 as usize].iModel;
        if iModel != NULL_HANDLE {
            ent.hModel = iModel;
        } else {
            //wtf?  how did we get here?
            return;
        }
    }

    // spin as it moves
    if ctx.world.entity(centNum).currentState.apos.trType != trType_t::TR_INTERPOLATE {
        // convert direction of travel into axis
        if VectorNormalize2(s1_pos.trDelta, &mut ent.axis[0]) == 0.0 {
            ent.axis[0][2] = 1.0;
        }

        // spin as it moves
        if s1_pos.trType != trType_t::TR_STATIONARY {
            if (s1_eFlags & EF_MISSILE_STICK) != 0 {
                //Did this so regular missiles don't get broken
                RotateAroundDirection(ent.axis.as_mut_ptr(), time as f32 * 0.5);
            } else {
                //JFM:FLOAT FIX
                RotateAroundDirection(ent.axis.as_mut_ptr(), time as f32 * 0.25);
            }
        } else if (s1_eFlags & EF_MISSILE_STICK) != 0 {
            RotateAroundDirection(ent.axis.as_mut_ptr(), s1_pos.trTime as f32 * 0.5);
        } else {
            let s1_time = ctx.world.entity(centNum).currentState.time;
            RotateAroundDirection(ent.axis.as_mut_ptr(), s1_time as f32);
        }
    } else {
        let lerpAngles = ctx.world.entity(centNum).lerpAngles;
        AnglesToAxis(lerpAngles, ent.axis.as_mut_ptr());
    }

    if s1_weapon == WP_SABER {
        ent.radius = ctx.world.entity(centNum).currentState.g2radius as f32;
    }

    // add to refresh list, possibly with quad glow
    let s1 = ctx.world.entity(centNum).currentState;
    CG_AddRefEntityWithPowerups(ctx, &ent, &s1, TEAM_FREE);

    if s1_weapon == WP_SABER && ctx.world.cgs.gametype == GT_JEDIMASTER {
        //in jedimaster always make the saber glow when on the ground
        let mut org: vec3_t = [0.0; 3];
        //refEntity_t sRef;
        //memcpy( &sRef, &ent, sizeof( sRef ) );
        let mut fxSArgs = addspriteArgStruct_t {
            origin: [0.0; 3],
            vel: [0.0; 3],
            accel: [0.0; 3],
            scale: 0.0,
            dscale: 0.0,
            sAlpha: 0.0,
            eAlpha: 0.0,
            rotation: 0.0,
            bounce: 0.0,
            life: 0,
            shader: 0,
            flags: 0,
        };

        ent.customShader = ctx.world.cgs.media.solidWhite;
        ent.renderfx = RF_RGB_TINT;
        // `sin` and the `0.08f`/`0.1f` literals widen to double, so the whole
        // tail is a double that narrows back into the float `wv`
        let wv = (((time as f32 * 0.003) as f64).sin() * 0.08f32 as f64 + 0.1f32 as f64) as f32;
        ent.shaderRGBA[0] = (wv * 255.0) as i32 as u8;
        ent.shaderRGBA[1] = (wv * 255.0) as i32 as u8;
        ent.shaderRGBA[2] = (wv * 0.0) as i32 as u8;
        trap::R_AddRefEntityToScene(engine, &ent);

        let mut i: c_int = -4;
        while i < 10 {
            let axis2 = ent.axis[2];
            _VectorMA(ent.origin, -(i as f32), axis2, &mut org);

            _VectorCopy(org, &mut fxSArgs.origin);
            VectorClear(&mut fxSArgs.vel);
            VectorClear(&mut fxSArgs.accel);
            fxSArgs.scale = 5.5;
            fxSArgs.dscale = 5.5;
            fxSArgs.sAlpha = wv;
            fxSArgs.eAlpha = wv;
            fxSArgs.rotation = 0.0;
            fxSArgs.bounce = 0.0;
            // Raven writes the float literal `1.0f` into the int `life` slot
            fxSArgs.life = 1;
            fxSArgs.shader = ctx.world.cgs.media.yellowDroppedSaberShader;
            fxSArgs.flags = 0x08000000;

            //trap_FX_AddSprite( org, NULL, NULL, 5.5f, 5.5f, wv, wv, 0.0f, 0.0f, 1.0f, cgs.media.yellowSaberGlowShader, 0x08000000 );
            trap::FX_AddSprite(engine, &mut fxSArgs);

            i += 1;
        }

        if ctx.world.cgs.gametype == GT_JEDIMASTER {
            ent.shaderRGBA[0] = 255;
            ent.shaderRGBA[1] = 255;
            ent.shaderRGBA[2] = 0;

            ent.renderfx |= RF_DEPTHHACK;
            ent.customShader = ctx.world.cgs.media.forceSightBubble;

            trap::R_AddRefEntityToScene(engine, &ent);
        }
    }

    if (s1_eFlags & EF_FIRING) != 0 {
        //special code for adding the beam to the attached tripwire mine
        let mut beamOrg: vec3_t = [0.0; 3];

        // forward
        let axis0 = ent.axis[0];
        _VectorMA(ent.origin, 8.0, axis0, &mut beamOrg);
        let tripMineLaser = ctx.world.cgs.effects.mTripMineLaster;
        trap::FX_PlayEffectID(engine, tripMineLaser, &beamOrg, &axis0, -1, -1);
    }
}

/// Raven `CG_PlayDoorLoopSound` — the ambient-set loop a mover holds while it
/// runs.
/// Source: `oracle/codemp/cgame/cg_ents.c:2746-2784`
pub fn CG_PlayDoorLoopSound(ctx: &mut CgContext, centNum: usize) {
    let soundSetIndex = ctx.world.entity(centNum).currentState.soundSetIndex;
    if soundSetIndex == 0 {
        return;
    }

    let soundSet = CG_ConfigString(ctx, CS_AMBIENT_SET + soundSetIndex);

    if soundSet.is_empty() {
        return;
    }

    let sfx = trap::AS_GetBModelSound(ctx.engine, &soundSet, CG_BMS_MID);

    if sfx == -1 {
        return;
    }

    let mut origin: vec3_t = [0.0; 3];
    let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
    //shouldn't be in here otherwise, but just in case.
    if ctx.world.entity(centNum).currentState.eType == entityType_t::ET_MOVER as c_int {
        let modelindex = ctx.world.entity(centNum).currentState.modelindex;
        let v = ctx.world.cgs.inlineModelMidpoints[modelindex as usize];
        _VectorAdd(lerpOrigin, v, &mut origin);
    } else {
        _VectorCopy(lerpOrigin, &mut origin);
    }

    //ent->s.loopSound = sfx;
    let number = ctx.world.entity(centNum).currentState.number;
    CG_S_AddRealLoopingSound(ctx.world, number as usize, origin, vec3_origin, sfx);
}

/// Raven `CG_PlayDoorSound` — the one-shot open/close sound off the mover's
/// ambient set; `r#type` is the `CG_BMS_*` stage the event carried.
/// Source: `oracle/codemp/cgame/cg_ents.c:2792-2817`
pub fn CG_PlayDoorSound(ctx: &mut CgContext, centNum: usize, r#type: c_int) {
    let soundSetIndex = ctx.world.entity(centNum).currentState.soundSetIndex;
    if soundSetIndex == 0 {
        return;
    }

    let soundSet = CG_ConfigString(ctx, CS_AMBIENT_SET + soundSetIndex);

    if soundSet.is_empty() {
        return;
    }

    let sfx = trap::AS_GetBModelSound(ctx.engine, &soundSet, r#type);

    if sfx == -1 {
        return;
    }

    let number = ctx.world.entity(centNum).currentState.number;
    trap::S_StartSound(ctx.engine, None, number, CHAN_AUTO, sfx);
}

/// Raven `CG_CalcEntityLerpPositions` — picks this entity's frame position:
/// snapshot interpolation, trajectory extrapolation, or the ridden vehicle's
/// own special case.
///
/// PORT-NOTE: Raven's ragdoll-offset block (:3131-3189) sits inside `#if 0`, so
/// it never compiled and is not transcribed.
/// Source: `oracle/codemp/cgame/cg_ents.c:3078-3202`
pub fn CG_CalcEntityLerpPositions(ctx: &mut CgContext, centNum: usize) {
    let mut goAway = false;

    // if this player does not want to see extrapolated players
    if ctx.world.cvars.cg_smoothClients.integer == 0 {
        // make sure the clients use TR_INTERPOLATE
        if ctx.world.entity(centNum).currentState.number < MAX_CLIENTS_I32 {
            let cent = ctx.world.entity_mut(centNum);
            cent.currentState.pos.trType = trType_t::TR_INTERPOLATE;
            cent.nextState.pos.trType = trType_t::TR_INTERPOLATE;
        }
    }

    let m_iVehicleNum = ctx.world.cg.predictedPlayerState.m_iVehicleNum;
    let number = ctx.world.entity(centNum).currentState.number;
    let eType = ctx.world.entity(centNum).currentState.eType;
    let NPC_class = ctx.world.entity(centNum).currentState.NPC_class;
    let time = ctx.world.cg.time;

    if m_iVehicleNum != 0
        && m_iVehicleNum == number
        && eType == entityType_t::ET_NPC as c_int
        && NPC_class == CLASS_VEHICLE as c_int
    {
        //special case for vehicle we are riding
        let owner = ctx.world.entity(m_iVehicleNum as usize).currentState.owner;

        if owner == ctx.world.cg.predictedPlayerState.clientNum {
            //only do this if the vehicle is pilotted by this client and predicting properly
            let pos = ctx.world.entity(centNum).currentState.pos;
            let apos = ctx.world.entity(centNum).currentState.apos;
            let cent = ctx.world.entity_mut(centNum);
            BG_EvaluateTrajectory(&pos, time, &mut cent.lerpOrigin);
            BG_EvaluateTrajectory(&apos, time, &mut cent.lerpAngles);
            return;
        }
    }

    let interpolate = ctx.world.entity(centNum).interpolate;
    let trType = ctx.world.entity(centNum).currentState.pos.trType;

    if interpolate != qfalse && trType == trType_t::TR_INTERPOLATE {
        CG_InterpolateEntityPosition(ctx, centNum);
        return;
    }

    // first see if we can interpolate between two snaps for
    // linear extrapolated clients
    if interpolate != qfalse && trType == trType_t::TR_LINEAR_STOP && number < MAX_CLIENTS_I32 {
        CG_InterpolateEntityPosition(ctx, centNum);
        goAway = true;
    } else if interpolate != qfalse
        && eType == entityType_t::ET_NPC as c_int
        && NPC_class == CLASS_VEHICLE as c_int
    {
        CG_InterpolateEntityPosition(ctx, centNum);
        goAway = true;
    } else {
        // just use the current frame and evaluate as best we can
        let pos = ctx.world.entity(centNum).currentState.pos;
        let apos = ctx.world.entity(centNum).currentState.apos;
        let cent = ctx.world.entity_mut(centNum);
        BG_EvaluateTrajectory(&pos, time, &mut cent.lerpOrigin);
        BG_EvaluateTrajectory(&apos, time, &mut cent.lerpAngles);
    }

    if goAway {
        return;
    }

    // adjust for riding a mover if it wasn't rolled into the predicted
    // player state
    if number != ctx.world.cg.predictedPlayerState.clientNum {
        // Raven derefs `cg.snap` here with no null check; with no snapshot the
        // port leaves the entity's lerped position alone (§F19).
        let Some(serverTime) = ctx.world.cg.snap_ref().map(|snap| snap.serverTime) else {
            return;
        };
        let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
        let groundEntityNum = ctx.world.entity(centNum).currentState.groundEntityNum;
        let mut out: vec3_t = [0.0; 3];
        CG_AdjustPositionForMover(
            ctx.world,
            lerpOrigin,
            groundEntityNum,
            serverTime,
            time,
            &mut out,
        );
        _VectorCopy(out, &mut ctx.world.entity_mut(centNum).lerpOrigin);
    }
}

/// Raven `CG_FX` — the `target_effect` entity: replays its configstring effect
/// on its own schedule.
/// Source: `oracle/codemp/cgame/cg_ents.c:3237-3307`
pub fn CG_FX(ctx: &mut CgContext, centNum: usize) {
    let mut fxDir: vec3_t = [0.0; 3];
    let mut efxIndex: c_int = 0;

    let time = ctx.world.cg.time;
    if ctx.world.entity(centNum).miscTime > time {
        return;
    }

    // Raven's `if (!s1)` null-check on `&cent->currentState` can never fire —
    // an owned field has no null address (§B5), so it is dropped.

    let s1_modelindex2 = ctx.world.entity(centNum).currentState.modelindex2;
    if s1_modelindex2 == FX_STATE_OFF {
        // fx not active
        return;
    }

    if s1_modelindex2 < FX_STATE_ONE_SHOT_LIMIT {
        // fx is single shot
        if ctx.world.entity(centNum).muzzleFlashTime == s1_modelindex2 {
            return;
        }

        ctx.world.entity_mut(centNum).muzzleFlashTime = s1_modelindex2;
    }

    let s1_speed = ctx.world.entity(centNum).currentState.speed;
    let s1_time = ctx.world.entity(centNum).currentState.time;
    let random = ctx.world.bg_state.rng.random();
    // every term here is float-typed in C, so the sum truncates into the int
    // `miscTime` slot
    ctx.world.entity_mut(centNum).miscTime =
        (time as f32 + s1_speed + random * s1_time as f32) as c_int;

    let s1_angles = ctx.world.entity(centNum).currentState.angles;
    AngleVectors(s1_angles, Some(&mut fxDir), None, None);

    if fxDir[0] == 0.0 && fxDir[1] == 0.0 && fxDir[2] == 0.0 {
        fxDir[1] = 1.0;
    }

    let s1_modelindex = ctx.world.entity(centNum).currentState.modelindex;
    if ctx.world.cgs.gameEffects[s1_modelindex as usize] != 0 {
        efxIndex = ctx.world.cgs.gameEffects[s1_modelindex as usize];
    } else {
        let s = CG_ConfigString(ctx, CS_EFFECTS + s1_modelindex);
        if !s.is_empty() {
            efxIndex = trap::FX_RegisterEffect(ctx.engine, &s);
            ctx.world.cgs.gameEffects[s1_modelindex as usize] = efxIndex;
        }
    }

    if efxIndex != 0 {
        let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
        if ctx.world.entity(centNum).currentState.isPortalEnt != qfalse {
            trap::FX_PlayPortalEffectID(ctx.engine, efxIndex, &lerpOrigin, &fxDir, -1, -1);
        } else {
            trap::FX_PlayEffectID(ctx.engine, efxIndex, &lerpOrigin, &fxDir, -1, -1);
        }
    }
}

/// Raven's `char buf[N]` scratch buffers are NUL-terminated, and the parse
/// walks read one past the terminator on the failure paths — in C that reads
/// uninitialized stack, so the port reads a `\0` there and stops (§F19).
fn notetrack_byte(buf: &[u8], i: usize) -> u8 {
    if i < buf.len() {
        buf[i]
    } else {
        0
    }
}

/// Raven `CG_ROFF_NotetrackCallback` — runs one ROFF notetrack: `effect
/// <file> [X+Y+Z [XA-YA-ZA]]` or `sound <file>`.
///
/// PORT-NOTE: Raven's fixed `type[256]`/`argument[512]`/`addlArg[512]`/`t[64]`
/// buffers overrun on a long notetrack (UB); the port's growable buffers just
/// hold the whole token (§F19).
/// Source: `oracle/codemp/cgame/cg_ents.c:3555-3758`
pub fn CG_ROFF_NotetrackCallback(ctx: &mut CgContext, centNum: usize, notetrack: &str) {
    let mut i: usize = 0;
    let mut r: usize = 0;
    let mut anglesGathered: usize = 0;
    let mut posoffsetGathered: usize = 0;
    let mut r#type: Vec<u8> = Vec::new();
    let mut argument: Vec<u8> = Vec::new();
    let mut addlArg: Vec<u8> = Vec::new();
    let mut t: Vec<u8> = Vec::new();
    let mut addlArgs = false;
    let mut parsedAngles: vec3_t = [0.0; 3];
    let mut parsedOffset: vec3_t = [0.0; 3];
    let mut useAngles: vec3_t = [0.0; 3];
    let mut useOrigin: vec3_t = [0.0; 3];
    let mut forward: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];

    // Raven's `if (!cent || !notetrack)` guard: the entity is an index and the
    // notetrack is an owned string, so neither can be null (§B5).

    //notetrack = "effect effects/explosion1.efx 0+0+64 0-0-1";
    let nt = notetrack.as_bytes();

    while notetrack_byte(nt, i) != 0 && notetrack_byte(nt, i) != b' ' {
        r#type.push(nt[i]);
        i += 1;
    }

    if notetrack_byte(nt, i) != b' ' {
        //didn't pass in a valid notetrack type, or forgot the argument for it
        return;
    }

    i += 1;

    while notetrack_byte(nt, i) != 0 && notetrack_byte(nt, i) != b' ' {
        argument.push(nt[i]);
        r += 1;
        i += 1;
    }

    if r == 0 {
        return;
    }

    if notetrack_byte(nt, i) == b' ' {
        //additional arguments...
        addlArgs = true;

        i += 1;
        while notetrack_byte(nt, i) != 0 {
            addlArg.push(nt[i]);
            i += 1;
        }
    }

    if r#type.as_slice() == b"effect".as_slice() {
        // Raven's two `goto defaultoffsetposition` jumps skip the rest of the
        // offset parse
        let mut defaultOffsetPosition = false;

        if !addlArgs {
            //sprintf(errMsg, "Offset position argument for 'effect' type is invalid.");
            //goto functionend;
            VectorClear(&mut parsedOffset);
            defaultOffsetPosition = true;
        }

        if !defaultOffsetPosition {
            i = 0;

            while posoffsetGathered < 3 {
                r = 0;
                t.clear();
                while notetrack_byte(&addlArg, i) != 0
                    && notetrack_byte(&addlArg, i) != b'+'
                    && notetrack_byte(&addlArg, i) != b' '
                {
                    t.push(addlArg[i]);
                    r += 1;
                    i += 1;
                }
                i += 1;
                if r == 0 {
                    //failure..
                    //sprintf(errMsg, "Offset position argument for 'effect' type is invalid.");
                    //goto functionend;
                    VectorClear(&mut parsedOffset);
                    i = 0;
                    defaultOffsetPosition = true;
                    break;
                }
                parsedOffset[posoffsetGathered] = atof_bytes(&t) as f32;
                posoffsetGathered += 1;
            }
        }

        if !defaultOffsetPosition {
            if posoffsetGathered < 3 {
                // dead arm - the loop above only exits at 3 or through the
                // goto, so Raven's `functionend` tail never runs
                let errMsg = "Offset position argument for 'effect' type is invalid.";
                Com_Printf(ctx, &format!("^3Type-specific notetrack error: {errMsg}\n"));
                return;
            }

            i -= 1;

            if notetrack_byte(&addlArg, i) != b' ' {
                addlArgs = false;
            }
        }

        // defaultoffsetposition:

        let argumentStr = latin1_to_string(&argument);
        let objectID = trap::FX_RegisterEffect(ctx.engine, &argumentStr);

        if objectID != 0 {
            if addlArgs {
                //if there is an additional argument for an effect it is expected to be XANGLE-YANGLE-ZANGLE
                i += 1;
                while anglesGathered < 3 {
                    r = 0;
                    t.clear();
                    while notetrack_byte(&addlArg, i) != 0 && notetrack_byte(&addlArg, i) != b'-' {
                        t.push(addlArg[i]);
                        r += 1;
                        i += 1;
                    }
                    i += 1;

                    if r == 0 {
                        //failed to get a new part of the vector
                        anglesGathered = 0;
                        break;
                    }

                    parsedAngles[anglesGathered] = atof_bytes(&t) as f32;
                    anglesGathered += 1;
                }

                if anglesGathered != 0 {
                    _VectorCopy(parsedAngles, &mut useAngles);
                } else {
                    //failed to parse angles from the extra argument provided..
                    _VectorCopy(ctx.world.entity(centNum).lerpAngles, &mut useAngles);
                }
            } else {
                //if no constant angles, play in direction entity is facing
                _VectorCopy(ctx.world.entity(centNum).lerpAngles, &mut useAngles);
            }

            AngleVectors(
                useAngles,
                Some(&mut forward),
                Some(&mut right),
                Some(&mut up),
            );

            _VectorCopy(ctx.world.entity(centNum).lerpOrigin, &mut useOrigin);

            //forward
            useOrigin[0] += forward[0] * parsedOffset[0];
            useOrigin[1] += forward[1] * parsedOffset[0];
            useOrigin[2] += forward[2] * parsedOffset[0];

            //right
            useOrigin[0] += right[0] * parsedOffset[1];
            useOrigin[1] += right[1] * parsedOffset[1];
            useOrigin[2] += right[2] * parsedOffset[1];

            //up
            useOrigin[0] += up[0] * parsedOffset[2];
            useOrigin[1] += up[1] * parsedOffset[2];
            useOrigin[2] += up[2] * parsedOffset[2];

            trap::FX_PlayEffectID(ctx.engine, objectID, &useOrigin, &useAngles, -1, -1);
        }
    } else if r#type.as_slice() == b"sound".as_slice() {
        let argumentStr = latin1_to_string(&argument);
        let objectID = trap::S_RegisterSound(ctx.engine, &argumentStr);
        let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
        let number = ctx.world.entity(centNum).currentState.number;
        trap::S_StartSound(ctx.engine, Some(&lerpOrigin), number, CHAN_BODY, objectID);
    } else if r#type.as_slice() == b"loop".as_slice() {
        //handled server-side
    }
    //else if ...
    else if !r#type.is_empty() {
        let typeStr = latin1_to_string(&r#type);
        Com_Printf(
            ctx,
            &format!("^3Warning: \"{typeStr}\" is an invalid ROFF notetrack function\n"),
        );
    } else {
        Com_Printf(
            ctx,
            "^3Warning: Notetrack is missing function and/or arguments\n",
        );
    }
}
