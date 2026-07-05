// PORT-COMPLETE: g_object.c 0/4
//! FAITHFUL signature skeleton for `oracle/oracle/codemp/game/g_object.c`.
//!
//! All 4 functions in this file read ambient game state (`level`, `g_gravity`,
//! `g_entities`) that is not reachable through the raw-pointer-only signatures
//! (no `GameWorld`/engine context parameter). This matches the precedent in
//! `g_main.rs`, `g_combat.rs`, and others where `raw-ptr-skeleton-no-world-handle`
//! escalations block porting. Once the seam decision is settled (how to thread
//! `GameWorld` through raw-pointer game logic functions), these will unblock.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

/// Raven `G_BounceObject`. Reflects velocity on trace plane.
///
/// Source: `oracle/oracle/codemp/game/g_object.c:14-59`
pub fn G_BounceObject(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
    trace: *mut trace_t,
) {
    let mut velocity: [f32; 3] = [0.0; 3];
    let world = unsafe { &mut *ctx.world };

    // reflect the velocity on the trace plane
    let hit_time = world.level.previousTime as c_int
        + ((world.level.time - world.level.previousTime) as f32 * unsafe { (*trace).fraction }) as c_int;

    crate::bg_misc::BG_EvaluateTrajectoryDelta(
        unsafe { &(*ent).s.pos as *const trajectory_t },
        hit_time,
        &mut velocity,
    );

    let dot = crate::q_math::_DotProduct(velocity, unsafe { (*trace).plane.normal });
    // bounceFactor = 60/ent->mass;		// NOTENOTE Mass is not yet implemented
    let bounce_factor = 1.0f32;
    let bounce_factor = if bounce_factor > 1.0f32 { 1.0f32 } else { bounce_factor };

    crate::q_math::_VectorMA(
        velocity,
        -2.0f32 * dot * bounce_factor,
        unsafe { (*trace).plane.normal },
        &mut unsafe { (*ent).s.pos.trDelta },
    );

    // FIXME: customized or material-based impact/bounce sounds
    if unsafe { (*ent).flags & FL_BOUNCE_HALF } != 0 {
        crate::q_math::_VectorScale(
            unsafe { (*ent).s.pos.trDelta },
            0.5f32,
            &mut unsafe { (*ent).s.pos.trDelta },
        );

        // check for stop
        let normal_z = unsafe { (*trace).plane.normal[2] };
        let g_grav = world.cvars.g_gravity.value;
        let delta_z = unsafe { (*ent).s.pos.trDelta[2] };

        if ((normal_z > 0.7f32 && g_grav > 0.0f32) || (normal_z < -0.7f32 && g_grav < 0.0f32)) &&
           ((delta_z < 40.0f32 && g_grav > 0.0f32) || (delta_z > -40.0f32 && g_grav < 0.0f32)) {
            // G_SetOrigin( ent, trace->endpos );
            // ent->nextthink = level.time + 500;
            unsafe { (*ent).s.apos.trType = TR_STATIONARY };
            crate::q_math::_VectorCopy(
                unsafe { (*ent).r.currentAngles },
                &mut unsafe { (*ent).s.apos.trBase },
            );
            crate::q_math::_VectorCopy(
                unsafe { (*trace).endpos },
                &mut unsafe { (*ent).r.currentOrigin },
            );
            crate::q_math::_VectorCopy(
                unsafe { (*trace).endpos },
                &mut unsafe { (*ent).s.pos.trBase },
            );
            unsafe { (*ent).s.pos.trTime = world.level.time };
            return;
        }
    }

    // NEW--It would seem that we want to set our trBase to the trace endpos
    // and set the trTime to the actual time of impact....
    // FIXME: Should we still consider adding the normal though??
    crate::q_math::_VectorCopy(
        unsafe { (*trace).endpos },
        &mut unsafe { (*ent).r.currentOrigin },
    );
    unsafe { (*ent).s.pos.trTime = hit_time };

    crate::q_math::_VectorCopy(
        unsafe { (*ent).r.currentOrigin },
        &mut unsafe { (*ent).s.pos.trBase },
    );
    crate::q_math::_VectorCopy(
        unsafe { (*trace).plane.normal },
        &mut unsafe { (*ent).pos1 },
    ); //???
}

/// Raven `G_RunObject`. Main object physics simulation.
///
/// Source: `oracle/oracle/codemp/game/g_object.c:72-241`
pub fn G_RunObject(
    ctx: GameContext<'_>,
    ent: *mut gentity_t,
) {
    let mut origin: [f32; 3] = [0.0; 3];
    let mut old_org: [f32; 3] = [0.0; 3];
    let mut tr: trace_t = unsafe { std::mem::zeroed() };
    let mut trace_ent: *mut gentity_t = std::ptr::null_mut();

    let world = unsafe { &mut *ctx.world };

    // FIXME: floaters need to stop floating up after a while, even if gravity stays negative?
    if unsafe { (*ent).s.pos.trType } == TR_STATIONARY {
        unsafe { (*ent).s.pos.trType = TR_GRAVITY };
        crate::q_math::_VectorCopy(
            unsafe { (*ent).r.currentOrigin },
            &mut unsafe { (*ent).s.pos.trBase },
        );
        unsafe { (*ent).s.pos.trTime = world.level.previousTime };
        if world.cvars.g_gravity.value == 0.0f32 {
            unsafe { (*ent).s.pos.trDelta[2] += 100.0f32 };
        }
    }

    unsafe { (*ent).nextthink = world.level.time + FRAMETIME as c_int };

    crate::q_math::_VectorCopy(unsafe { (*ent).r.currentOrigin }, &mut old_org);
    // get current position
    crate::bg_misc::BG_EvaluateTrajectory(
        unsafe { &(*ent).s.pos as *const trajectory_t },
        world.level.time,
        &mut origin,
    );
    // Get current angles?
    crate::bg_misc::BG_EvaluateTrajectory(
        unsafe { &(*ent).s.apos as *const trajectory_t },
        world.level.time,
        &mut unsafe { (*ent).r.currentAngles },
    );

    if crate::q_math::VectorCompare(unsafe { (*ent).r.currentOrigin }, origin) != 0 {
        // error - didn't move at all!
        return;
    }

    // trace a line from the previous position to the current position,
    // ignoring interactions with the missile owner
    let trace_skip_num = if let Some(parent_id) = unsafe { (*ent).parent } {
        unsafe { (*world).g_entities[parent_id.0 as usize].s.number as c_int }
    } else {
        unsafe { (*ent).s.number as c_int }
    };

    crate::trap::Trace(
        ctx.engine,
        crate::trap::GTraceArgs::new(
            &mut tr as *mut trace_t,
            &unsafe { (*ent).r.currentOrigin } as *const vec3_t,
            &unsafe { (*ent).r.mins } as *const vec3_t,
            &unsafe { (*ent).r.maxs } as *const vec3_t,
            &origin as *const vec3_t,
            trace_skip_num,
            unsafe { (*ent).clipmask },
        ),
    );

    if !tr.startsolid && !tr.allsolid && tr.fraction > 0.0f32 {
        crate::q_math::_VectorCopy(tr.endpos, &mut unsafe { (*ent).r.currentOrigin });
        crate::trap::LinkEntity(ctx.engine, crate::trap::GLinkentityArgs::new(ent));
    } else {
        // if ( tr.startsolid )
        tr.fraction = 0.0f32;
    }

    crate::g_active::G_MoverTouchPushTriggers(ctx, ent, old_org);

    if tr.fraction == 1.0f32 {
        if world.cvars.g_gravity.value <= 0.0f32 {
            if unsafe { (*ent).s.apos.trType } == TR_STATIONARY {
                crate::q_math::_VectorCopy(
                    unsafe { (*ent).r.currentAngles },
                    &mut unsafe { (*ent).s.apos.trBase },
                );
                unsafe { (*ent).s.apos.trType = TR_LINEAR };
                unsafe { (*ent).s.apos.trDelta[1] = world.bg_state.rng.flrand(-300.0f32, 300.0f32) };
                unsafe { (*ent).s.apos.trDelta[0] = world.bg_state.rng.flrand(-10.0f32, 10.0f32) };
                unsafe { (*ent).s.apos.trDelta[2] = world.bg_state.rng.flrand(-10.0f32, 10.0f32) };
                unsafe { (*ent).s.apos.trTime = world.level.time };
            }
        }
        // friction in zero-G
        if world.cvars.g_gravity.value == 0.0f32 {
            let mut friction = 0.975f32;
            // friction -= ent->mass/1000.0f;
            if friction < 0.1f32 {
                friction = 0.1f32;
            }

            crate::q_math::_VectorScale(
                unsafe { (*ent).s.pos.trDelta },
                friction,
                &mut unsafe { (*ent).s.pos.trDelta },
            );
            crate::q_math::_VectorCopy(
                unsafe { (*ent).r.currentOrigin },
                &mut unsafe { (*ent).s.pos.trBase },
            );
            unsafe { (*ent).s.pos.trTime = world.level.time };
        }
        return;
    }

    // hit something

    // Do impact damage
    trace_ent = unsafe {
        &mut (*world).g_entities[tr.entityNum as usize] as *mut gentity_t
    };

    if tr.fraction > 0.0f32 || (trace_ent != std::ptr::null_mut() && unsafe { (*trace_ent).takedamage != 0 }) {
        if crate::q_math::VectorCompare(unsafe { (*ent).r.currentOrigin }, old_org) == 0 {
            // moved and impacted
            if trace_ent != std::ptr::null_mut() && unsafe { (*trace_ent).takedamage != 0 } {
                // hurt someone
                // G_Sound( ent, G_SoundIndex( "sound/movers/objects/objectHurt.wav" ) );
            }
            // G_Sound( ent, G_SoundIndex( "sound/movers/objects/objectHit.wav" ) );
        }

        if unsafe { (*ent).s.weapon } != WP_SABER {
            crate::g_active::DoImpact(ctx, ent, trace_ent, qtrue);
        }
    }

    if ent == std::ptr::null_mut() ||
       (unsafe { (*ent).takedamage != 0 } && unsafe { (*ent).health } <= 0) {
        // been destroyed by impact
        // chunks?
        // G_Sound( ent, G_SoundIndex( "sound/movers/objects/objectBreak.wav" ) );
        return;
    }

    // do impact physics
    if unsafe { (*ent).s.pos.trType } == TR_GRAVITY {
        // FIXME: only do this if no trDelta
        if world.cvars.g_gravity.value <= 0.0f32 || tr.plane.normal[2] < 0.7f32 {
            if unsafe { (*ent).flags & (FL_BOUNCE | FL_BOUNCE_HALF) } != 0 {
                if tr.fraction <= 0.0f32 {
                    crate::q_math::_VectorCopy(tr.endpos, &mut unsafe { (*ent).r.currentOrigin });
                    crate::q_math::_VectorCopy(tr.endpos, &mut unsafe { (*ent).s.pos.trBase });
                    unsafe { (*ent).s.pos.trDelta = [0.0f32; 3] };
                    unsafe { (*ent).s.pos.trTime = world.level.time };
                } else {
                    G_BounceObject(ctx, ent, &mut tr);
                }
            } else {
                // slide down?
                // FIXME: slide off the slope
            }
        } else {
            unsafe { (*ent).s.apos.trType = TR_STATIONARY };
            crate::npc_c::pitch_roll_for_slope(ctx, ent, None);
            // ent->r.currentAngles[0] = 0;//FIXME: match to slope
            // ent->r.currentAngles[2] = 0;//FIXME: match to slope
            crate::q_math::_VectorCopy(
                unsafe { (*ent).r.currentAngles },
                &mut unsafe { (*ent).s.apos.trBase },
            );
            // okay, we hit the floor, might as well stop or prediction will
            // make us go through the floor!
            // FIXME: this means we can't fall if something is pulled out from under us...
            G_StopObjectMoving(ent);
        }
    } else if unsafe { (*ent).s.weapon } != WP_SABER {
        unsafe { (*ent).s.apos.trType = TR_STATIONARY };
        crate::npc_c::pitch_roll_for_slope(ctx, ent, None);
        // ent->r.currentAngles[0] = 0;//FIXME: match to slope
        // ent->r.currentAngles[2] = 0;//FIXME: match to slope
        crate::q_math::_VectorCopy(
            unsafe { (*ent).r.currentAngles },
            &mut unsafe { (*ent).s.apos.trBase },
        );
    }

    // call touch func
    if let Some(touch_fn) = unsafe { (*ent).touch } {
        crate::ent_fn_enums::dispatch_touch(
            ctx,
            touch_fn,
            ent,
            trace_ent,
            unsafe { &tr as *const trace_t as *mut trace_t },
        );
    }
}

/// Raven `G_StopObjectMoving`. Stops an object from moving.
///
/// Source: `oracle/oracle/codemp/game/g_object.c:244-258`
pub fn G_StopObjectMoving(
    object: *mut gentity_t,
) {
    unsafe { (*object).s.pos.trType = TR_STATIONARY };
    crate::q_math::_VectorCopy(
        unsafe { (*object).r.currentOrigin },
        &mut unsafe { (*object).s.origin },
    );
    crate::q_math::_VectorCopy(
        unsafe { (*object).r.currentOrigin },
        &mut unsafe { (*object).s.pos.trBase },
    );
    unsafe { (*object).s.pos.trDelta = [0.0f32; 3] };

    // Stop spinning (commented out in Raven)
    // VectorClear( self->s.apos.trDelta );
    // vectoangles(trace->plane.normal, self->s.angles);
    // VectorCopy(self->s.angles, self->r.currentAngles );
    // VectorCopy(self->s.angles, self->s.apos.trBase);
}

/// Raven `G_StartObjectMoving`. Starts an object moving with direction and speed.
///
/// Source: `oracle/oracle/codemp/game/g_object.c:260-287`
pub fn G_StartObjectMoving(
    ctx: GameContext<'_>,
    object: *mut gentity_t,
    dir: vec3_t,
    speed: f32,
    trType: trType_t,
) {
    // PORT-NOTE(fork-9-signature-mismatch): skeleton has dir: vec3_t but fork-9 ruling
    // settled to &mut vec3_t for params written through VectorNormalize. Code below
    // assumes &mut semantics but compiles with by-value param; mutations invisible to caller.
    let mut dir_mut = dir;
    crate::q_math::VectorNormalize(&mut dir_mut);

    let world = unsafe { &mut *ctx.world };

    // object->s.eType = ET_GENERAL;
    unsafe { (*object).s.pos.trType = trType };
    crate::q_math::_VectorCopy(
        unsafe { (*object).r.currentOrigin },
        &mut unsafe { (*object).s.pos.trBase },
    );
    crate::q_math::_VectorScale(
        dir_mut,
        speed,
        &mut unsafe { (*object).s.pos.trDelta },
    );
    unsafe { (*object).s.pos.trTime = world.level.time };

    // FIXME: incorporate spin?
    // vectoangles(dir, object->s.angles);
    // VectorCopy(object->s.angles, object->s.apos.trBase);
    // VectorSet(object->s.apos.trDelta, 300, 0, 0 );
    // object->s.apos.trTime = level.time;

    // FIXME: make these objects go through G_RunObject automatically, like missiles do
    if unsafe { (*object).think.is_none() } {
        unsafe { (*object).nextthink = world.level.time + FRAMETIME as c_int };
        unsafe { (*object).think = Some(EntThink::G_RunObject) };
    } else {
        // You're responsible for calling RunObject
    }
}
