//! Port of `oracle/codemp/game/g_exphysics.c`.
//!
//! Functions reach file-scope game state (`level`, `g_entities`, cvars) and engine traps through the threaded `GameContext`/`GameWorld` handle.
#![allow(non_snake_case, unused, clippy::all)]

use crate::ent_fn_enums::dispatch_touch;
use crate::prelude::*;

use mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs;
use mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs;
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;

/// Raven `MAX_GRAVITY_PULL`.
///
/// Source: `oracle/codemp/game/g_exphysics.c:16`
pub const MAX_GRAVITY_PULL: f32 = 512.0;

/// Raven `G_RunExPhys`.
///
/// Source: `oracle/codemp/game/g_exphysics.c:21-232`
pub fn G_RunExPhys(
    ctx: &mut GameContext,
    id: EntityId,
    gravity: f32,
    mass: f32,
    bounce: f32,
    autoKill: bool,
    g2Bolts: *mut c_int,
    numG2Bolts: c_int,
) {
    // `trace_t` has no zeroing constructor.
    // The `mem::zeroed` call is a plain POD init.
    let mut tr: trace_t = unsafe { core::mem::zeroed() };
    let mut projectedOrigin: vec3_t = [0.0; 3];
    let mut vNorm: vec3_t = [0.0; 3];
    let velScaling: f32 = 0.1f32;

    // C's `assert` is elided under NDEBUG (release), so shipping builds tolerate out-of-range mass.
    // `debug_assert!` mirrors that behavior.
    debug_assert!(mass <= 1.0 && mass >= 0.01);

    if gravity != 0.0 {
        let mut ground = ctx.entity(id).r.currentOrigin;
        ground[2] -= 0.1f32;

        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &ctx.entity(id).r.currentOrigin as *const vec3_t,
                &ctx.entity(id).r.mins as *const vec3_t,
                &ctx.entity(id).r.maxs as *const vec3_t,
                &ground as *const vec3_t,
                ctx.entity(id).s.number,
                ctx.entity(id).clipmask,
            ),
        );

        if tr.fraction == 1.0f32 {
            ctx.entity_mut(id).s.groundEntityNum = ENTITYNUM_NONE;
        } else {
            ctx.entity_mut(id).s.groundEntityNum = tr.entityNum as c_int;
        }

        if ctx.entity(id).s.groundEntityNum == ENTITYNUM_NONE {
            let e = ctx.entity_mut(id);
            e.epGravFactor += gravity;

            if e.epGravFactor > MAX_GRAVITY_PULL {
                e.epGravFactor = MAX_GRAVITY_PULL;
            }

            e.epVelocity[2] -= e.epGravFactor;
        } else {
            ctx.entity_mut(id).epGravFactor = 0.0;
        }
    }

    if ctx.entity(id).epVelocity[0] == 0.0
        && ctx.entity(id).epVelocity[1] == 0.0
        && ctx.entity(id).epVelocity[2] == 0.0
    {
        if ctx.entity(id).touch.is_some() {
            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut tr as *mut trace_t,
                    &ctx.entity(id).r.currentOrigin as *const vec3_t,
                    &ctx.entity(id).r.mins as *const vec3_t,
                    &ctx.entity(id).r.maxs as *const vec3_t,
                    &ctx.entity(id).r.currentOrigin as *const vec3_t,
                    ctx.entity(id).s.number,
                    ctx.entity(id).clipmask,
                ),
            );
            if tr.startsolid != 0 || tr.allsolid != 0 {
                if let Some(touch_fn) = ctx.entity(id).touch.get() {
                    let self_ptr: *mut gentity_t = ctx.entity_mut(id);
                    let other_ptr: *mut gentity_t = ctx.entity_mut(EntityId(tr.entityNum as u32));
                    dispatch_touch(ctx, touch_fn, self_ptr, other_ptr, &mut tr);
                }
            }
        }
        return;
    }

    _VectorMA(
        ctx.entity(id).r.currentOrigin,
        velScaling,
        ctx.entity(id).epVelocity,
        &mut projectedOrigin,
    );

    let e = ctx.entity_mut(id);
    let vel = e.epVelocity;
    _VectorScale(vel, 1.0f32 - mass, &mut e.epVelocity);

    vNorm = e.epVelocity;
    let mut vTotal = VectorNormalize(&mut vNorm);

    if vTotal < 1.0 && ctx.entity(id).s.groundEntityNum != ENTITYNUM_NONE {
        let e = ctx.entity_mut(id);
        e.epVelocity[0] = 0.0;
        e.epVelocity[1] = 0.0;
        e.epVelocity[2] = 0.0;
        e.epGravFactor = 0.0;
        let ent_ptr = e as *mut gentity_t;
        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent_ptr.cast()));
        return;
    }

    if !ctx.entity(id).ghoul2.is_null() && !g2Bolts.is_null() {
        let tMins: vec3_t = [-3.0, -3.0, -3.0];
        let tMaxs: vec3_t = [3.0, 3.0, 3.0];
        let mut trajDif: vec3_t = [0.0; 3];
        let mut gbmAngles: vec3_t = [0.0; 3];
        let mut boneOrg: vec3_t = [0.0; 3];
        let mut projectedBoneOrg: vec3_t = [0.0; 3];
        let mut collisionRootPos: vec3_t = [0.0; 3];
        // `mdxaBone_t` and `trace_t` here get the same POD init as `tr` above.
        let mut matrix: mdxaBone_t = unsafe { core::mem::zeroed() };
        let mut bestCollision: trace_t = unsafe { core::mem::zeroed() };
        let mut hasFirstCollision = false;

        gbmAngles[PITCH as usize] = 0.0;
        gbmAngles[ROLL as usize] = 0.0;
        gbmAngles[YAW as usize] = ctx.entity(id).s.apos.trBase[YAW as usize];

        _VectorSubtract(
            ctx.entity(id).r.currentOrigin,
            projectedOrigin,
            &mut trajDif,
        );

        for i in 0..numG2Bolts {
            // `g2Bolts` is a raw caller-owned bolt array.
            // The indexed read is the one unsafe operation in this function.
            let bolt = unsafe { *g2Bolts.add(i as usize) };
            trap::G2API_GetBoltMatrix(
                ctx.engine,
                GG2GetboltArgs::new(
                    ctx.entity(id).ghoul2,
                    0,
                    bolt,
                    &mut matrix as *mut mdxaBone_t,
                    &gbmAngles as *const vec3_t,
                    &ctx.entity(id).r.currentOrigin as *const vec3_t,
                    ctx.world.level.time,
                    core::ptr::null_mut(),
                    &ctx.entity(id).modelScale as *const vec3_t,
                ),
            );
            BG_GiveMeVectorFromMatrix(
                &matrix as *const mdxaBone_t,
                mp_qshared::shared::Eorientations::ORIGIN as c_int,
                &mut boneOrg,
            );

            _VectorAdd(boneOrg, trajDif, &mut projectedBoneOrg);

            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut tr as *mut trace_t,
                    &boneOrg as *const vec3_t,
                    &tMins as *const vec3_t,
                    &tMaxs as *const vec3_t,
                    &projectedBoneOrg as *const vec3_t,
                    ctx.entity(id).s.number,
                    ctx.entity(id).clipmask,
                ),
            );

            if tr.fraction != 1.0f32 || tr.startsolid != 0 || tr.allsolid != 0 {
                if !hasFirstCollision {
                    bestCollision = tr;
                    collisionRootPos = boneOrg;
                    hasFirstCollision = true;
                } else {
                    if tr.allsolid != 0 && bestCollision.allsolid == 0 {
                        bestCollision = tr;
                        collisionRootPos = boneOrg;
                    } else if tr.startsolid != 0
                        && bestCollision.startsolid == 0
                        && bestCollision.allsolid == 0
                    {
                        bestCollision = tr;
                        collisionRootPos = boneOrg;
                    } else if bestCollision.startsolid == 0
                        && bestCollision.allsolid == 0
                        && tr.fraction < bestCollision.fraction
                    {
                        bestCollision = tr;
                        collisionRootPos = boneOrg;
                    }
                }
            }
        }

        if hasFirstCollision {
            _VectorSubtract(collisionRootPos, bestCollision.endpos, &mut trajDif);

            _VectorAdd(
                ctx.entity(id).r.currentOrigin,
                trajDif,
                &mut projectedOrigin,
            );
        }
    }

    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut tr as *mut trace_t,
            &ctx.entity(id).r.currentOrigin as *const vec3_t,
            &ctx.entity(id).r.mins as *const vec3_t,
            &ctx.entity(id).r.maxs as *const vec3_t,
            &projectedOrigin as *const vec3_t,
            ctx.entity(id).s.number,
            ctx.entity(id).clipmask,
        ),
    );

    if tr.startsolid != 0 || tr.allsolid != 0 {
        if autoKill {
            let time = ctx.world.level.time;
            let e = ctx.entity_mut(id);
            e.think = Some(EntThink::G_FreeEntity).into();
            e.nextthink = time;
        }
        return;
    }

    G_SetOrigin(ctx.entity_mut(id), tr.endpos);
    let ent_ptr = ctx.entity_mut(id) as *mut gentity_t;
    trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent_ptr.cast()));

    if tr.fraction == 1.0f32 {
        return;
    }

    if bounce != 0.0 {
        vTotal *= bounce;

        _VectorScale(tr.plane.normal, vTotal, &mut vNorm);

        if vNorm[2] > 0.0 {
            let e = ctx.entity_mut(id);
            e.epGravFactor -= vNorm[2] * (1.0f32 - mass);
            if e.epGravFactor < 0.0 {
                e.epGravFactor = 0.0;
            }
        }

        if tr.entityNum as c_int != ENTITYNUM_NONE && ctx.entity(id).touch.is_some() {
            if let Some(touch_fn) = ctx.entity(id).touch.get() {
                let self_ptr: *mut gentity_t = ctx.entity_mut(id);
                let other_ptr: *mut gentity_t = ctx.entity_mut(EntityId(tr.entityNum as u32));
                dispatch_touch(ctx, touch_fn, self_ptr, other_ptr, &mut tr);
            }
        }

        let e = ctx.entity_mut(id);
        let vel = e.epVelocity;
        _VectorAdd(vel, vNorm, &mut e.epVelocity);
    } else {
        let e = ctx.entity_mut(id);
        e.epVelocity[0] = 0.0;
        e.epVelocity[1] = 0.0;

        if gravity == 0.0 {
            e.epVelocity[2] = 0.0;
        }
    }
}
