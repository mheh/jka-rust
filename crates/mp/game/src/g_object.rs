// PORT-COMPLETE: g_object.c
//! `oracle/codemp/game/g_object.c` — object physics (bounce/run/start/stop).
//!
//! Entity params are `EntityId` handles (§B5); bodies reach the world and
//! their entity through the `GameContext`/`GameWorld` accessors. Behavior is
//! byte-identical to the pre-migration port — referee-verified.
#![allow(non_snake_case, unused, clippy::all)]

use crate::ent_fn_enums::dispatch_touch;
use crate::prelude::*;
use crate::world::GameWorld;
use mp_abi::game::syscalls::G_LINKENTITY::GLinkentityArgs;
use mp_abi::game::syscalls::G_TRACE::GTraceArgs;
use mp_bg::bg_misc::{BG_EvaluateTrajectory, BG_EvaluateTrajectoryDelta};

/// Raven `G_BounceObject`. Reflects velocity on trace plane.
///
/// Source: `oracle/codemp/game/g_object.c:14-59`
pub fn G_BounceObject(ctx: &mut GameContext, id: EntityId, trace: &trace_t) {
    let mut velocity: [f32; 3] = [0.0; 3];

    // reflect the velocity on the trace plane
    let hit_time = ctx.world.level.previousTime as c_int
        + ((ctx.world.level.time - ctx.world.level.previousTime) as f32 * trace.fraction) as c_int;

    BG_EvaluateTrajectoryDelta(
        core::ptr::from_ref(&ctx.entity(id).s.pos),
        hit_time,
        &mut velocity,
    );

    let dot = _DotProduct(velocity, trace.plane.normal);
    // bounceFactor = 60/ent->mass;		// NOTENOTE Mass is not yet implemented
    let bounce_factor = 1.0f32;
    let bounce_factor = if bounce_factor > 1.0f32 {
        1.0f32
    } else {
        bounce_factor
    };

    _VectorMA(
        velocity,
        -2.0f32 * dot * bounce_factor,
        trace.plane.normal,
        &mut ctx.entity_mut(id).s.pos.trDelta,
    );

    // FIXME: customized or material-based impact/bounce sounds
    if ctx.entity(id).flags & FL_BOUNCE_HALF != 0 {
        let trDelta = ctx.entity(id).s.pos.trDelta;
        _VectorScale(trDelta, 0.5f32, &mut ctx.entity_mut(id).s.pos.trDelta);

        // check for stop
        let normal_z = trace.plane.normal[2];
        let g_grav = ctx.world.cvars.g_gravity.value;
        let delta_z = ctx.entity(id).s.pos.trDelta[2];

        if ((normal_z > 0.7f32 && g_grav > 0.0f32) || (normal_z < -0.7f32 && g_grav < 0.0f32))
            && ((delta_z < 40.0f32 && g_grav > 0.0f32) || (delta_z > -40.0f32 && g_grav < 0.0f32))
        {
            // G_SetOrigin( ent, trace->endpos );
            // ent->nextthink = level.time + 500;
            let time = ctx.world.level.time;
            let e = ctx.entity_mut(id);
            e.s.apos.trType = TR_STATIONARY;
            e.s.apos.trBase = e.r.currentAngles;
            e.r.currentOrigin = trace.endpos;
            e.s.pos.trBase = trace.endpos;
            e.s.pos.trTime = time;
            return;
        }
    }

    // NEW--It would seem that we want to set our trBase to the trace endpos
    // and set the trTime to the actual time of impact....
    // FIXME: Should we still consider adding the normal though??
    let e = ctx.entity_mut(id);
    e.r.currentOrigin = trace.endpos;
    e.s.pos.trTime = hit_time;
    e.s.pos.trBase = e.r.currentOrigin;
    e.pos1 = trace.plane.normal;
    //???
}

/// Raven `G_RunObject`. Main object physics simulation.
///
/// Source: `oracle/codemp/game/g_object.c:72-241`
pub fn G_RunObject(ctx: &mut GameContext, id: EntityId) {
    let mut origin: [f32; 3] = [0.0; 3];
    // trace_t has no zeroing constructor; the mem::zeroed is a plain POD-init.
    let mut tr: trace_t = unsafe { std::mem::zeroed() };

    // FIXME: floaters need to stop floating up after a while, even if gravity stays negative?
    if ctx.entity(id).s.pos.trType == TR_STATIONARY {
        let previousTime = ctx.world.level.previousTime;
        let zero_grav = ctx.world.cvars.g_gravity.value == 0.0f32;
        let e = ctx.entity_mut(id);
        e.s.pos.trType = TR_GRAVITY;
        e.s.pos.trBase = e.r.currentOrigin;
        e.s.pos.trTime = previousTime;
        if zero_grav {
            e.s.pos.trDelta[2] += 100.0f32;
        }
    }

    ctx.entity_mut(id).nextthink = ctx.world.level.time + FRAMETIME as c_int;

    let old_org = ctx.entity(id).r.currentOrigin;
    // get current position
    BG_EvaluateTrajectory(
        core::ptr::from_ref(&ctx.entity(id).s.pos),
        ctx.world.level.time,
        &mut origin,
    );
    // Get current angles?
    // Copy `s.apos` out so the read source is disjoint from the `r.currentAngles`
    // write target (Raven aliases them through one `gentity_t*`; the snapshot is
    // behavior-identical because `s.apos` is not mutated by the eval).
    let apos = ctx.entity(id).s.apos;
    BG_EvaluateTrajectory(
        core::ptr::from_ref(&apos),
        ctx.world.level.time,
        &mut ctx.entity_mut(id).r.currentAngles,
    );

    if VectorCompare(ctx.entity(id).r.currentOrigin, origin) {
        // error - didn't move at all!
        return;
    }

    // trace a line from the previous position to the current position,
    // ignoring interactions with the missile owner
    let trace_skip_num = if let Some(parent_id) = ctx.entity(id).parent {
        ctx.entity(parent_id).s.number as c_int
    } else {
        ctx.entity(id).s.number as c_int
    };

    trap::Trace(
        ctx.engine,
        GTraceArgs::new(
            &mut tr as *mut trace_t,
            core::ptr::from_ref(&ctx.entity(id).r.currentOrigin),
            core::ptr::from_ref(&ctx.entity(id).r.mins),
            core::ptr::from_ref(&ctx.entity(id).r.maxs),
            &origin as *const vec3_t,
            trace_skip_num,
            ctx.entity(id).clipmask,
        ),
    );

    if tr.startsolid == 0 && tr.allsolid == 0 && tr.fraction > 0.0f32 {
        ctx.entity_mut(id).r.currentOrigin = tr.endpos;
        trap::LinkEntity(
            ctx.engine,
            GLinkentityArgs::new(core::ptr::from_mut(ctx.entity_mut(id)).cast()),
        );
    } else {
        // if ( tr.startsolid )
        tr.fraction = 0.0f32;
    }

    G_MoverTouchPushTriggers(ctx, id, old_org);

    if tr.fraction == 1.0f32 {
        if ctx.world.cvars.g_gravity.value <= 0.0f32 {
            if ctx.entity(id).s.apos.trType == TR_STATIONARY {
                let time = ctx.world.level.time;
                let yaw_delta = ctx.world.bg_state.rng.flrand(-300.0f32, 300.0f32);
                let pitch_delta = ctx.world.bg_state.rng.flrand(-10.0f32, 10.0f32);
                let roll_delta = ctx.world.bg_state.rng.flrand(-10.0f32, 10.0f32);
                let e = ctx.entity_mut(id);
                e.s.apos.trBase = e.r.currentAngles;
                e.s.apos.trType = TR_LINEAR;
                e.s.apos.trDelta[1] = yaw_delta;
                e.s.apos.trDelta[0] = pitch_delta;
                e.s.apos.trDelta[2] = roll_delta;
                e.s.apos.trTime = time;
            }
        }
        // friction in zero-G
        if ctx.world.cvars.g_gravity.value == 0.0f32 {
            let mut friction = 0.975f32;
            // friction -= ent->mass/1000.0f;
            if friction < 0.1f32 {
                friction = 0.1f32;
            }

            let time = ctx.world.level.time;
            let e = ctx.entity_mut(id);
            let trDelta = e.s.pos.trDelta;
            _VectorScale(trDelta, friction, &mut e.s.pos.trDelta);
            e.s.pos.trBase = e.r.currentOrigin;
            e.s.pos.trTime = time;
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
        if !VectorCompare(ctx.entity(id).r.currentOrigin, old_org) {
            // moved and impacted
            if ctx.entity(trace_ent).takedamage != 0 {
                // hurt someone
                // G_Sound( ent, G_SoundIndex( "sound/movers/objects/objectHurt.wav" ) );
            }
            // G_Sound( ent, G_SoundIndex( "sound/movers/objects/objectHit.wav" ) );
        }

        if ctx.entity(id).s.weapon != WP_SABER {
            DoImpact(ctx, id, trace_ent, qtrue);
        }
    }

    // Raven: if ( !ent || (ent->takedamage && ent->health <= 0) ). `ent` is a
    // live arena entity (never NULL), so the NULL arm is vacuous.
    if ctx.entity(id).takedamage != 0 && ctx.entity(id).health <= 0 {
        // been destroyed by impact
        // chunks?
        // G_Sound( ent, G_SoundIndex( "sound/movers/objects/objectBreak.wav" ) );
        return;
    }

    // do impact physics
    if ctx.entity(id).s.pos.trType == TR_GRAVITY {
        // FIXME: only do this if no trDelta
        // `0.7` is a bare double in the oracle; promote to f64 so the
        // round-down `<` compare matches. Source: g_object.c:196
        if ctx.world.cvars.g_gravity.value <= 0.0f32 || (tr.plane.normal[2] as f64) < 0.7 {
            if ctx.entity(id).flags & (FL_BOUNCE | FL_BOUNCE_HALF) != 0 {
                if tr.fraction <= 0.0f32 {
                    let time = ctx.world.level.time;
                    let e = ctx.entity_mut(id);
                    e.r.currentOrigin = tr.endpos;
                    e.s.pos.trBase = tr.endpos;
                    e.s.pos.trDelta = [0.0f32; 3];
                    e.s.pos.trTime = time;
                } else {
                    G_BounceObject(ctx, id, &tr);
                }
            } else {
                // slide down?
                // FIXME: slide off the slope
            }
        } else {
            ctx.entity_mut(id).s.apos.trType = TR_STATIONARY;
            pitch_roll_for_slope(ctx, id, Some(&mut tr.plane.normal));
            // ent->r.currentAngles[0] = 0;//FIXME: match to slope
            // ent->r.currentAngles[2] = 0;//FIXME: match to slope
            let e = ctx.entity_mut(id);
            e.s.apos.trBase = e.r.currentAngles;
            // okay, we hit the floor, might as well stop or prediction will
            // make us go through the floor!
            // FIXME: this means we can't fall if something is pulled out from under us...
            G_StopObjectMoving(e);
        }
    } else if ctx.entity(id).s.weapon != WP_SABER {
        ctx.entity_mut(id).s.apos.trType = TR_STATIONARY;
        pitch_roll_for_slope(ctx, id, Some(&mut tr.plane.normal));
        // ent->r.currentAngles[0] = 0;//FIXME: match to slope
        // ent->r.currentAngles[2] = 0;//FIXME: match to slope
        let e = ctx.entity_mut(id);
        e.s.apos.trBase = e.r.currentAngles;
    }

    // call touch func
    if let Some(touch_fn) = ctx.entity(id).touch.get() {
        // Raw-pointer temps end `entity_mut`'s borrow of `ctx` at the coercion
        // (raw pointers carry no borrowck lifetime), so the seam dispatch (which
        // needs `ctx` plus both entity pointers) doesn't conflict.
        let self_ptr: *mut gentity_t = ctx.entity_mut(id);
        let trace_ent_ptr: *mut gentity_t = ctx.entity_mut(trace_ent);
        dispatch_touch(
            ctx,
            touch_fn,
            self_ptr,
            trace_ent_ptr,
            &tr as *const trace_t as *mut trace_t,
        );
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
    object.s.origin = object.r.currentOrigin;
    object.s.pos.trBase = object.r.currentOrigin;
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
    // Skeleton signature took `dir: vec3_t`, not the settled `&mut vec3_t`
    // shape for VectorNormalize out-params; harmless — zero live callers.
    let mut dir_mut = dir;
    VectorNormalize(&mut dir_mut);

    // object->s.eType = ET_GENERAL;
    let time = ctx.world.level.time;
    let e = ctx.entity_mut(object);
    e.s.pos.trType = trType;
    e.s.pos.trBase = e.r.currentOrigin;
    _VectorScale(dir_mut, speed, &mut e.s.pos.trDelta);
    e.s.pos.trTime = time;

    // FIXME: incorporate spin?
    // vectoangles(dir, object->s.angles);
    // VectorCopy(object->s.angles, object->s.apos.trBase);
    // VectorSet(object->s.apos.trDelta, 300, 0, 0 );
    // object->s.apos.trTime = level.time;

    // FIXME: make these objects go through G_RunObject automatically, like missiles do
    if ctx.entity(object).think.is_none() {
        let time = ctx.world.level.time;
        let e = ctx.entity_mut(object);
        e.nextthink = time + FRAMETIME as c_int;
        e.think = Some(EntThink::G_RunObject).into();
    } else {
        // You're responsible for calling RunObject
    }
}
