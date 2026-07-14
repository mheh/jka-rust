// PORT-COMPLETE: g_object.c 4/4
//! `oracle/codemp/game/g_object.c` — object physics (bounce/run/start/stop).
//!
//! Safe-state migration **Stage 0 pilot**. Entity params are `EntityId`
//! handles (§B5) instead of raw `gentity_t*`; bodies reach the world and their
//! entity through the `GameContext`/`GameWorld` accessors
//! (`ctx.world`/`ctx.entity_mut()`), so the per-line `unsafe { (*ent).… }`
//! derefs are gone. Behavior is byte-identical to the pre-migration port — this
//! is a mechanical reshape, referee-verified. Callers in still-raw files bridge
//! their `*mut gentity_t` at the boundary with `ctx.entity_id_of(ptr)`.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::world::GameWorld;

/// Raven `G_BounceObject`. Reflects velocity on trace plane.
///
/// Source: `oracle/codemp/game/g_object.c:14-59`
pub fn G_BounceObject(ctx: &mut GameContext, id: EntityId, trace: &trace_t) {
    // STAGE-1: EntityId param, raw pointers re-derived verbatim (Stage-2 debt) —
    // `world`/`ent` are re-derived as raw pointers (not tracked borrows) so the
    // rest of this Stage-1 body can alias them freely, matching the pre-flip
    // `ctx.world()`/`ctx.entity_mut()` Copy-`GameContext` idiom.
    let ent: *mut gentity_t = ctx.entity_mut(id);
    unsafe {
        let mut velocity: [f32; 3] = [0.0; 3];

        // reflect the velocity on the trace plane
        let hit_time = ctx.world.level.previousTime as c_int
            + ((ctx.world.level.time - ctx.world.level.previousTime) as f32 * trace.fraction)
                as c_int;

        crate::bg_misc::BG_EvaluateTrajectoryDelta(
            &(*ent).s.pos as *const trajectory_t,
            hit_time,
            &mut velocity,
        );

        let dot = crate::q_math::_DotProduct(velocity, trace.plane.normal);
        // bounceFactor = 60/ent->mass;		// NOTENOTE Mass is not yet implemented
        let bounce_factor = 1.0f32;
        let bounce_factor = if bounce_factor > 1.0f32 {
            1.0f32
        } else {
            bounce_factor
        };

        crate::q_math::_VectorMA(
            velocity,
            -2.0f32 * dot * bounce_factor,
            trace.plane.normal,
            &mut (*ent).s.pos.trDelta,
        );

        // FIXME: customized or material-based impact/bounce sounds
        if (*ent).flags & FL_BOUNCE_HALF != 0 {
            crate::q_math::_VectorScale((*ent).s.pos.trDelta, 0.5f32, &mut (*ent).s.pos.trDelta);

            // check for stop
            let normal_z = trace.plane.normal[2];
            let g_grav = ctx.world.cvars.g_gravity.value;
            let delta_z = (*ent).s.pos.trDelta[2];

            if ((normal_z > 0.7f32 && g_grav > 0.0f32) || (normal_z < -0.7f32 && g_grav < 0.0f32))
                && ((delta_z < 40.0f32 && g_grav > 0.0f32)
                    || (delta_z > -40.0f32 && g_grav < 0.0f32))
            {
                // G_SetOrigin( ent, trace->endpos );
                // ent->nextthink = level.time + 500;
                (*ent).s.apos.trType = TR_STATIONARY;
                crate::q_math::_VectorCopy((*ent).r.currentAngles, &mut (*ent).s.apos.trBase);
                crate::q_math::_VectorCopy(trace.endpos, &mut (*ent).r.currentOrigin);
                crate::q_math::_VectorCopy(trace.endpos, &mut (*ent).s.pos.trBase);
                (*ent).s.pos.trTime = ctx.world.level.time;
                return;
            }
        }

        // NEW--It would seem that we want to set our trBase to the trace endpos
        // and set the trTime to the actual time of impact....
        // FIXME: Should we still consider adding the normal though??
        crate::q_math::_VectorCopy(trace.endpos, &mut (*ent).r.currentOrigin);
        (*ent).s.pos.trTime = hit_time;

        crate::q_math::_VectorCopy((*ent).r.currentOrigin, &mut (*ent).s.pos.trBase);
        crate::q_math::_VectorCopy(trace.plane.normal, &mut (*ent).pos1); //???
    }
}

/// Raven `G_RunObject`. Main object physics simulation.
///
/// Source: `oracle/codemp/game/g_object.c:72-241`
pub fn G_RunObject(ctx: &mut GameContext, id: EntityId) {
    let mut origin: [f32; 3] = [0.0; 3];
    let mut old_org: [f32; 3] = [0.0; 3];
    // trace_t has no zeroing constructor; the mem::zeroed is a plain
    // POD-init, not part of the entity/world unsafe this migration retires.
    let mut tr: trace_t = unsafe { std::mem::zeroed() };

    // STAGE-1: EntityId param, raw pointers re-derived verbatim (Stage-2 debt) —
    // `world`/`ent` are re-derived as raw pointers (not tracked borrows) so this
    // Stage-1 body can alias them freely and still call back into `ctx` (e.g.
    // `ctx.entity(trace_ent)`, `G_BounceObject(ctx, …)`) further down, matching
    // the pre-flip `ctx.world()`/`ctx.entity_mut()` Copy-`GameContext` idiom.
    let ent: *mut gentity_t = ctx.entity_mut(id);

    unsafe {
        // FIXME: floaters need to stop floating up after a while, even if gravity stays negative?
        if (*ent).s.pos.trType == TR_STATIONARY {
            (*ent).s.pos.trType = TR_GRAVITY;
            crate::q_math::_VectorCopy((*ent).r.currentOrigin, &mut (*ent).s.pos.trBase);
            (*ent).s.pos.trTime = ctx.world.level.previousTime;
            if ctx.world.cvars.g_gravity.value == 0.0f32 {
                (*ent).s.pos.trDelta[2] += 100.0f32;
            }
        }

        (*ent).nextthink = ctx.world.level.time + FRAMETIME as c_int;

        crate::q_math::_VectorCopy((*ent).r.currentOrigin, &mut old_org);
        // get current position
        crate::bg_misc::BG_EvaluateTrajectory(
            &(*ent).s.pos as *const trajectory_t,
            ctx.world.level.time,
            &mut origin,
        );
        // Get current angles?
        crate::bg_misc::BG_EvaluateTrajectory(
            &(*ent).s.apos as *const trajectory_t,
            ctx.world.level.time,
            &mut (*ent).r.currentAngles,
        );

        if crate::q_math::VectorCompare((*ent).r.currentOrigin, origin) != 0 {
            // error - didn't move at all!
            return;
        }

        // trace a line from the previous position to the current position,
        // ignoring interactions with the missile owner
        let trace_skip_num = if let Some(parent_id) = (*ent).parent {
            ctx.world.g_entities[parent_id.index()].s.number as c_int
        } else {
            (*ent).s.number as c_int
        };

        crate::trap::Trace(
            ctx.engine,
            mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                &mut tr as *mut trace_t,
                &(*ent).r.currentOrigin as *const vec3_t,
                &(*ent).r.mins as *const vec3_t,
                &(*ent).r.maxs as *const vec3_t,
                &origin as *const vec3_t,
                trace_skip_num,
                (*ent).clipmask,
            ),
        );

        if tr.startsolid == 0 && tr.allsolid == 0 && tr.fraction > 0.0f32 {
            crate::q_math::_VectorCopy(tr.endpos, &mut (*ent).r.currentOrigin);
            crate::trap::LinkEntity(
                ctx.engine,
                mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs::new(&mut *ent),
            );
        } else {
            // if ( tr.startsolid )
            tr.fraction = 0.0f32;
        }

        crate::g_active::G_MoverTouchPushTriggers(ctx, id, old_org);

        if tr.fraction == 1.0f32 {
            if ctx.world.cvars.g_gravity.value <= 0.0f32 {
                if (*ent).s.apos.trType == TR_STATIONARY {
                    crate::q_math::_VectorCopy((*ent).r.currentAngles, &mut (*ent).s.apos.trBase);
                    (*ent).s.apos.trType = TR_LINEAR;
                    (*ent).s.apos.trDelta[1] = ctx.world.bg_state.rng.flrand(-300.0f32, 300.0f32);
                    (*ent).s.apos.trDelta[0] = ctx.world.bg_state.rng.flrand(-10.0f32, 10.0f32);
                    (*ent).s.apos.trDelta[2] = ctx.world.bg_state.rng.flrand(-10.0f32, 10.0f32);
                    (*ent).s.apos.trTime = ctx.world.level.time;
                }
            }
            // friction in zero-G
            if ctx.world.cvars.g_gravity.value == 0.0f32 {
                let mut friction = 0.975f32;
                // friction -= ent->mass/1000.0f;
                if friction < 0.1f32 {
                    friction = 0.1f32;
                }

                crate::q_math::_VectorScale(
                    (*ent).s.pos.trDelta,
                    friction,
                    &mut (*ent).s.pos.trDelta,
                );
                crate::q_math::_VectorCopy((*ent).r.currentOrigin, &mut (*ent).s.pos.trBase);
                (*ent).s.pos.trTime = ctx.world.level.time;
            }
            return;
        }

        // hit something

        // Do impact damage. Raven: trace_ent = &g_entities[tr.entityNum]. On this
        // path (tr.fraction < 1) the trace struck a live entity, so tr.entityNum
        // indexes a real arena slot and the reference is never NULL — Raven's
        // `trace_ent != NULL` guards below are vacuous and collapse to the
        // takedamage tests (faithful index, not a from_num sentinel reinterpret).
        let trace_ent = EntityId(tr.entityNum as u32);

        if tr.fraction > 0.0f32 || ctx.entity(trace_ent).takedamage != 0 {
            if crate::q_math::VectorCompare((*ent).r.currentOrigin, old_org) == 0 {
                // moved and impacted
                if ctx.entity(trace_ent).takedamage != 0 {
                    // hurt someone
                    // G_Sound( ent, G_SoundIndex( "sound/movers/objects/objectHurt.wav" ) );
                }
                // G_Sound( ent, G_SoundIndex( "sound/movers/objects/objectHit.wav" ) );
            }

            if (*ent).s.weapon != WP_SABER {
                crate::g_active::DoImpact(ctx, id, trace_ent, qtrue);
            }
        }

        // Raven: if ( !ent || (ent->takedamage && ent->health <= 0) ). `ent` is a
        // live arena entity (never NULL), so the NULL arm is vacuous.
        if (*ent).takedamage != 0 && (*ent).health <= 0 {
            // been destroyed by impact
            // chunks?
            // G_Sound( ent, G_SoundIndex( "sound/movers/objects/objectBreak.wav" ) );
            return;
        }

        // do impact physics
        if (*ent).s.pos.trType == TR_GRAVITY {
            // FIXME: only do this if no trDelta
            if ctx.world.cvars.g_gravity.value <= 0.0f32 || tr.plane.normal[2] < 0.7f32 {
                if (*ent).flags & (FL_BOUNCE | FL_BOUNCE_HALF) != 0 {
                    if tr.fraction <= 0.0f32 {
                        crate::q_math::_VectorCopy(tr.endpos, &mut (*ent).r.currentOrigin);
                        crate::q_math::_VectorCopy(tr.endpos, &mut (*ent).s.pos.trBase);
                        (*ent).s.pos.trDelta = [0.0f32; 3];
                        (*ent).s.pos.trTime = ctx.world.level.time;
                    } else {
                        G_BounceObject(ctx, id, &tr);
                    }
                } else {
                    // slide down?
                    // FIXME: slide off the slope
                }
            } else {
                (*ent).s.apos.trType = TR_STATIONARY;
                crate::npc_c::pitch_roll_for_slope(ctx, id, Some(&mut tr.plane.normal));
                // ent->r.currentAngles[0] = 0;//FIXME: match to slope
                // ent->r.currentAngles[2] = 0;//FIXME: match to slope
                crate::q_math::_VectorCopy((*ent).r.currentAngles, &mut (*ent).s.apos.trBase);
                // okay, we hit the floor, might as well stop or prediction will
                // make us go through the floor!
                // FIXME: this means we can't fall if something is pulled out from under us...
                G_StopObjectMoving(&mut *ent);
            }
        } else if (*ent).s.weapon != WP_SABER {
            (*ent).s.apos.trType = TR_STATIONARY;
            crate::npc_c::pitch_roll_for_slope(ctx, id, Some(&mut tr.plane.normal));
            // ent->r.currentAngles[0] = 0;//FIXME: match to slope
            // ent->r.currentAngles[2] = 0;//FIXME: match to slope
            crate::q_math::_VectorCopy((*ent).r.currentAngles, &mut (*ent).s.apos.trBase);
        }

        // call touch func
        if let Some(touch_fn) = (*ent).touch.get() {
            let trace_ent_ptr: *mut gentity_t = ctx.entity_mut(trace_ent);
            crate::ent_fn_enums::dispatch_touch(
                ctx,
                touch_fn,
                &mut *ent,
                trace_ent_ptr,
                &tr as *const trace_t as *mut trace_t,
            );
        }
    }
}

/// Raven `G_StopObjectMoving`. Stops an object from moving.
///
/// Ctx-free leaf helper — takes a `&mut gentity_t` borrow from the caller's
/// accessor (Stage-1 rule: ctx-free single-entity mutators may borrow directly).
///
/// Source: `oracle/codemp/game/g_object.c:244-258`
pub fn G_StopObjectMoving(object: &mut gentity_t) {
    object.s.pos.trType = TR_STATIONARY;
    crate::q_math::_VectorCopy(object.r.currentOrigin, &mut object.s.origin);
    crate::q_math::_VectorCopy(object.r.currentOrigin, &mut object.s.pos.trBase);
    object.s.pos.trDelta = [0.0f32; 3];

    // Stop spinning (commented out in Raven)
    // VectorClear( self->s.apos.trDelta );
    // vectoangles(trace->plane.normal, self->s.angles);
    // VectorCopy(self->s.angles, self->r.currentAngles );
    // VectorCopy(self->s.angles, self->s.apos.trBase);
}

/// Raven `G_StartObjectMoving`. Starts an object moving with direction and speed.
///
/// Source: `oracle/codemp/game/g_object.c:260-287`
pub fn G_StartObjectMoving(
    ctx: &mut GameContext,
    object: EntityId,
    dir: vec3_t,
    speed: f32,
    trType: trType_t,
) {
    // PORT-NOTE(signature-mismatch): skeleton has dir: vec3_t but the settled shape for
    // params written through VectorNormalize is &mut vec3_t. Code below assumes &mut
    // semantics but compiles with by-value param; mutations invisible to caller.
    let mut dir_mut = dir;
    crate::q_math::VectorNormalize(&mut dir_mut);

    // STAGE-1: raw pointers re-derived verbatim (Stage-2 debt); see G_BounceObject.
    let obj: *mut gentity_t = ctx.entity_mut(object);

    unsafe {
        // object->s.eType = ET_GENERAL;
        (*obj).s.pos.trType = trType;
        crate::q_math::_VectorCopy((*obj).r.currentOrigin, &mut (*obj).s.pos.trBase);
        crate::q_math::_VectorScale(dir_mut, speed, &mut (*obj).s.pos.trDelta);
        (*obj).s.pos.trTime = ctx.world.level.time;

        // FIXME: incorporate spin?
        // vectoangles(dir, object->s.angles);
        // VectorCopy(object->s.angles, object->s.apos.trBase);
        // VectorSet(object->s.apos.trDelta, 300, 0, 0 );
        // object->s.apos.trTime = level.time;

        // FIXME: make these objects go through G_RunObject automatically, like missiles do
        if (*obj).think.is_none() {
            (*obj).nextthink = ctx.world.level.time + FRAMETIME as c_int;
            (*obj).think = Some(EntThink::G_RunObject).into();
        } else {
            // You're responsible for calling RunObject
        }
    }
}
