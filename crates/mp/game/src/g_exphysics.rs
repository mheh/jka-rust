// PORT-COMPLETE: g_exphysics.c
//! FAITHFUL port of `oracle/codemp/game/g_exphysics.c`.
//!
//! Filled by the jampgame mega-pass; functions reach file-scope game state
//! (`level`, `g_entities`, cvars) and engine traps through the threaded
//! `GameContext`/`GameWorld` handle.
#![allow(non_snake_case, unused, clippy::all)]

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
    autoKill: qboolean,
    g2Bolts: *mut c_int,
    numG2Bolts: c_int,
) {
    unsafe {
        let mut tr: trace_t = core::mem::zeroed();
        let mut projectedOrigin: vec3_t = [0.0; 3];
        let mut vNorm: vec3_t = [0.0; 3];
        let mut ground: vec3_t = [0.0; 3];
        let velScaling: f32 = 0.1f32;
        let mut vTotal: f32 = 0.0f32;

        // C `assert` is elided under NDEBUG (release), so out-of-range mass is
        // tolerated in shipping builds; `debug_assert!` mirrors that.
        debug_assert!(mass <= 1.0 && mass >= 0.01);

        if gravity != 0.0 {
            _VectorCopy(ctx.world.entity(id).r.currentOrigin, &mut ground);
            ground[2] -= 0.1f32;

            trap::Trace(
                ctx.engine,
                GTraceArgs::new(
                    &mut tr as *mut trace_t,
                    &ctx.world.entity(id).r.currentOrigin as *const vec3_t,
                    &ctx.world.entity(id).r.mins as *const vec3_t,
                    &ctx.world.entity(id).r.maxs as *const vec3_t,
                    &ground as *const vec3_t,
                    ctx.world.entity(id).s.number,
                    ctx.world.entity(id).clipmask,
                ),
            );

            if tr.fraction == 1.0f32 {
                ctx.world.entity_mut(id).s.groundEntityNum = ENTITYNUM_NONE;
            } else {
                ctx.world.entity_mut(id).s.groundEntityNum = tr.entityNum as c_int;
            }

            if ctx.world.entity(id).s.groundEntityNum == ENTITYNUM_NONE {
                ctx.world.entity_mut(id).epGravFactor += gravity;

                if ctx.world.entity(id).epGravFactor > MAX_GRAVITY_PULL {
                    ctx.world.entity_mut(id).epGravFactor = MAX_GRAVITY_PULL;
                }

                let grav_factor = ctx.world.entity(id).epGravFactor;
                ctx.world.entity_mut(id).epVelocity[2] -= grav_factor;
            } else {
                ctx.world.entity_mut(id).epGravFactor = 0.0;
            }
        }

        if ctx.world.entity(id).epVelocity[0] == 0.0
            && ctx.world.entity(id).epVelocity[1] == 0.0
            && ctx.world.entity(id).epVelocity[2] == 0.0
        {
            if ctx.world.entity(id).touch.is_some() {
                trap::Trace(
                    ctx.engine,
                    GTraceArgs::new(
                        &mut tr as *mut trace_t,
                        &ctx.world.entity(id).r.currentOrigin as *const vec3_t,
                        &ctx.world.entity(id).r.mins as *const vec3_t,
                        &ctx.world.entity(id).r.maxs as *const vec3_t,
                        &ctx.world.entity(id).r.currentOrigin as *const vec3_t,
                        ctx.world.entity(id).s.number,
                        ctx.world.entity(id).clipmask,
                    ),
                );
                if tr.startsolid != 0 || tr.allsolid != 0 {
                    let touch_fn = ctx.world.entity(id).touch.get();
                    if let Some(touch_fn) = touch_fn {
                        let self_ent = ctx.world.entity_mut(id) as *mut gentity_t;
                        let other_ent =
                            &mut ctx.world.g_entities[tr.entityNum as usize] as *mut gentity_t;
                        crate::ent_fn_enums::dispatch_touch(
                            ctx, touch_fn, self_ent, other_ent, &mut tr,
                        );
                    }
                }
            }
            return;
        }

        _VectorMA(
            ctx.world.entity(id).r.currentOrigin,
            velScaling,
            ctx.world.entity(id).epVelocity,
            &mut projectedOrigin,
        );

        let vel = ctx.world.entity(id).epVelocity;
        _VectorScale(vel, 1.0f32 - mass, &mut ctx.world.entity_mut(id).epVelocity);

        _VectorCopy(ctx.world.entity(id).epVelocity, &mut vNorm);
        vTotal = VectorNormalize(&mut vNorm);

        if vTotal < 1.0 && ctx.world.entity(id).s.groundEntityNum != ENTITYNUM_NONE {
            ctx.world.entity_mut(id).epVelocity[0] = 0.0;
            ctx.world.entity_mut(id).epVelocity[1] = 0.0;
            ctx.world.entity_mut(id).epVelocity[2] = 0.0;
            ctx.world.entity_mut(id).epGravFactor = 0.0;
            let ent_ptr = ctx.world.entity_mut(id) as *mut gentity_t;
            trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent_ptr.cast()));
            return;
        }

        if !ctx.world.entity(id).ghoul2.is_null() && !g2Bolts.is_null() {
            let mut tMins: vec3_t = [-3.0, -3.0, -3.0];
            let mut tMaxs: vec3_t = [3.0, 3.0, 3.0];
            let mut trajDif: vec3_t = [0.0; 3];
            let mut gbmAngles: vec3_t = [0.0; 3];
            let mut boneOrg: vec3_t = [0.0; 3];
            let mut projectedBoneOrg: vec3_t = [0.0; 3];
            let mut collisionRootPos: vec3_t = [0.0; 3];
            let mut matrix: mdxaBone_t = core::mem::zeroed();
            let mut bestCollision: trace_t = core::mem::zeroed();
            let mut hasFirstCollision: qboolean = qfalse;
            let mut i: c_int = 0;

            gbmAngles[PITCH as usize] = 0.0;
            gbmAngles[ROLL as usize] = 0.0;
            gbmAngles[YAW as usize] = ctx.world.entity(id).s.apos.trBase[YAW as usize];

            _VectorSubtract(
                ctx.world.entity(id).r.currentOrigin,
                projectedOrigin,
                &mut trajDif,
            );

            while i < numG2Bolts {
                trap::G2API_GetBoltMatrix(
                    ctx.engine,
                    GG2GetboltArgs::new(
                        ctx.world.entity(id).ghoul2,
                        0,
                        *g2Bolts.add(i as usize),
                        &mut matrix as *mut mdxaBone_t,
                        &gbmAngles as *const vec3_t,
                        &ctx.world.entity(id).r.currentOrigin as *const vec3_t,
                        ctx.world.level.time,
                        core::ptr::null_mut(),
                        &ctx.world.entity(id).modelScale as *const vec3_t,
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
                        ctx.world.entity(id).s.number,
                        ctx.world.entity(id).clipmask,
                    ),
                );

                if tr.fraction != 1.0f32 || tr.startsolid != 0 || tr.allsolid != 0 {
                    if hasFirstCollision == qfalse {
                        bestCollision = tr;
                        _VectorCopy(boneOrg, &mut collisionRootPos);
                        hasFirstCollision = qtrue;
                    } else {
                        if tr.allsolid != 0 && bestCollision.allsolid == 0 {
                            bestCollision = tr;
                            _VectorCopy(boneOrg, &mut collisionRootPos);
                        } else if tr.startsolid != 0
                            && bestCollision.startsolid == 0
                            && bestCollision.allsolid == 0
                        {
                            bestCollision = tr;
                            _VectorCopy(boneOrg, &mut collisionRootPos);
                        } else if bestCollision.startsolid == 0
                            && bestCollision.allsolid == 0
                            && tr.fraction < bestCollision.fraction
                        {
                            bestCollision = tr;
                            _VectorCopy(boneOrg, &mut collisionRootPos);
                        }
                    }
                }

                i += 1;
            }

            if hasFirstCollision != qfalse {
                _VectorSubtract(collisionRootPos, bestCollision.endpos, &mut trajDif);

                _VectorAdd(
                    ctx.world.entity(id).r.currentOrigin,
                    trajDif,
                    &mut projectedOrigin,
                );
            }
        }

        trap::Trace(
            ctx.engine,
            GTraceArgs::new(
                &mut tr as *mut trace_t,
                &ctx.world.entity(id).r.currentOrigin as *const vec3_t,
                &ctx.world.entity(id).r.mins as *const vec3_t,
                &ctx.world.entity(id).r.maxs as *const vec3_t,
                &projectedOrigin as *const vec3_t,
                ctx.world.entity(id).s.number,
                ctx.world.entity(id).clipmask,
            ),
        );

        if tr.startsolid != 0 || tr.allsolid != 0 {
            if autoKill != qfalse {
                ctx.world.entity_mut(id).think = Some(EntThink::G_FreeEntity).into();
                let lt = ctx.world.level.time;
                ctx.world.entity_mut(id).nextthink = lt;
            }
            return;
        }

        G_SetOrigin(ctx.world.entity_mut(id), tr.endpos);
        let ent_ptr = ctx.world.entity_mut(id) as *mut gentity_t;
        trap::LinkEntity(ctx.engine, GLinkentityArgs::new(ent_ptr.cast()));

        if tr.fraction == 1.0f32 {
            return;
        }

        if bounce != 0.0 {
            vTotal *= bounce;

            _VectorScale(tr.plane.normal, vTotal, &mut vNorm);

            if vNorm[2] > 0.0 {
                ctx.world.entity_mut(id).epGravFactor -= vNorm[2] * (1.0f32 - mass);
                if ctx.world.entity(id).epGravFactor < 0.0 {
                    ctx.world.entity_mut(id).epGravFactor = 0.0;
                }
            }

            if tr.entityNum as c_int != ENTITYNUM_NONE && ctx.world.entity(id).touch.is_some() {
                let touch_fn = ctx.world.entity(id).touch.get();
                if let Some(touch_fn) = touch_fn {
                    let self_ent = ctx.world.entity_mut(id) as *mut gentity_t;
                    let other_ent =
                        &mut ctx.world.g_entities[tr.entityNum as usize] as *mut gentity_t;
                    crate::ent_fn_enums::dispatch_touch(
                        ctx, touch_fn, self_ent, other_ent, &mut tr,
                    );
                }
            }

            let vel = ctx.world.entity(id).epVelocity;
            _VectorAdd(vel, vNorm, &mut ctx.world.entity_mut(id).epVelocity);
        } else {
            ctx.world.entity_mut(id).epVelocity[0] = 0.0;
            ctx.world.entity_mut(id).epVelocity[1] = 0.0;

            if gravity == 0.0 {
                ctx.world.entity_mut(id).epVelocity[2] = 0.0;
            }
        }
    }
}
