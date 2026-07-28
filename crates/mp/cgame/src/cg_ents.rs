//! Port of `oracle/codemp/cgame/cg_ents.c` — turning each snapshot entity into renderer commands. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_bg::bg_misc::{BG_EvaluateTrajectory, BG_GiveMeVectorFromMatrix};
use mp_bg::public::entity_type::entityType_t;
use mp_bg::public::gametype::{GT_CTF, GT_CTY};
use mp_bg::public::item_type::IT_POWERUP;
use mp_bg::public::powerup::{PW_FORCE_ENLIGHTENED_DARK, PW_FORCE_ENLIGHTENED_LIGHT};
use mp_bg::public::team::{TEAM_BLUE, TEAM_RED};
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::qcommon::entity_state::entityState_t;
use mp_qshared::shared::force_powers::{FORCE_DARKSIDE, FORCE_LIGHTSIDE};
use mp_qshared::shared::q_math::{
    _VectorAdd, _VectorCopy, _VectorMA, _VectorScale, _VectorSubtract, vectoangles, AngleVectors,
    AnglesToAxis, MatrixMultiply, VectorNormalize, VectorSet, PITCH, ROLL, YAW,
};
use mp_qshared::shared::surface_flags::SOLID_BMODEL;
use mp_qshared::shared::{
    addpolyArgStruct_t, mdxaBone_t, orientation_t, qfalse, qhandle_t, qtrue, sfxHandle_t, vec3_t,
    Eorientations, ENTITYNUM_MAX_NORMAL, MAX_CLIENTS_I32,
};

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
