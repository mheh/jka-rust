// PORT-COMPLETE: NPC_AI_Rancor.c 4/12
//! FAITHFUL port of `oracle/oracle/codemp/game/NPC_AI_Rancor.c`.
//!
//! Landed from the `fnskel.py` signature skeleton. 4 functions are transcribed
//! faithfully from packet + prelude alone; the remaining 12 are parked (see
//! the `PORT-NOTE` topics below), matching the precedent set in
//! `NPC_AI_Jedi.rs`/`NPC_AI_Stormtrooper.rs`/`NPC_AI_GalakMech.rs`: almost
//! every body in this file reaches the file-scope AI globals (`NPC`,
//! `NPCInfo`, `ucmd`, `level`, `g_entities`) or a `trap_*` seam call, and the
//! faithful context-free signatures have no channel to reach either (fork
//! ruling 1 makes the AI globals `GameWorld`/`GameContext` state, but these
//! resolved cross-file signatures are equally context-free).
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

// These define the working combat range for these suckers (`NPC_AI_Rancor.c:10-17`).
const MIN_DISTANCE: c_int = 128;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;
const MAX_DISTANCE: c_int = 1024;
const MAX_DISTANCE_SQR: c_int = MAX_DISTANCE * MAX_DISTANCE;
const LSTATE_CLEAR: c_int = 0;
const LSTATE_WAITING: c_int = 1;

// `DistanceSquared` is the canonical `crate::q_math::DistanceSquared`, reached
// via the prelude glob (no per-file copy). Cross-module callers that referenced
// `crate::NPC_AI_Rancor::DistanceSquared` now use `crate::q_math::DistanceSquared`.

/// Raven `Rancor_SetBolts`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:19-29`
pub fn Rancor_SetBolts(
    ctx: GameContext<'_>,self_: *mut gentity_t) {
    unsafe {
        if !self_.is_null() && !(*self_).client.is_null() {
            // `gentity_t.client` stays `*mut c_void` per the deferral; overlay-cast to
            // `gclient_t` at the use site.
            let ri = &mut (*((*self_).client as *mut gclient_t)).renderInfo;
            ri.handRBolt = trap::G2API_AddBolt(
                ctx.engine,
                mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new((*self_).ghoul2 as *mut c_void, 0, c"*r_hand".to_owned()),
            );
            ri.handLBolt = trap::G2API_AddBolt(
                ctx.engine,
                mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new((*self_).ghoul2 as *mut c_void, 0, c"*l_hand".to_owned()),
            );
            ri.headBolt = trap::G2API_AddBolt(
                ctx.engine,
                mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new((*self_).ghoul2 as *mut c_void, 0, c"*head_eyes".to_owned()),
            );
            ri.torsoBolt = trap::G2API_AddBolt(
                ctx.engine,
                mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new((*self_).ghoul2 as *mut c_void, 0, c"jaw_bone".to_owned()),
            );
        }
    }
}

/// Raven `NPC_Rancor_Precache`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:36-45`
pub fn NPC_Rancor_Precache(ctx: GameContext<'_>) {
    for i in 1..3 {
        crate::g_utils::G_SoundIndex(
            std::ffi::CString::new(format!("sound/chars/rancor/snort_{}.wav", i))
                .unwrap()
                .as_ptr(),
        );
    }
    crate::g_utils::G_SoundIndex(c"sound/chars/rancor/swipehit.wav".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/chars/rancor/chomp.wav".as_ptr());
}

/// Raven `Rancor_Idle`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:53-63`
pub fn Rancor_Idle(ctx: GameContext<'_>) {
    unsafe {
        let npc_info = (*ctx.world).globals.NPCInfo;
        if !npc_info.is_null() {
            (*npc_info).localState = LSTATE_CLEAR;
        }

        //If we have somewhere to go, then do that
        if !crate::NPC_goal::UpdateGoal(ctx).is_null() {
            (*ctx.world).globals.ucmd.buttons &= !BUTTON_WALKING;
            crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);
        }
    }
}

/// Raven `Rancor_CheckRoar`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:66-77`
pub fn Rancor_CheckRoar(
    ctx: GameContext<'_>,self_: *mut gentity_t) -> qboolean {
    unsafe {
        if (*self_).wait == 0 {
            //haven't ever gotten mad yet
            (*self_).wait = 1;//do this only once
            (*(*self_).client).ps.eFlags2 |= EF2_ALERTED;
            crate::npc_c::NPC_SetAnim(self_, SETANIM_BOTH, BOTH_STAND1TO2, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
            crate::g_timer::TIMER_Set(ctx, self_, c"rageTime".as_ptr(), (*(*self_).client).ps.legsTimer);
            return qtrue;
        }
        qfalse
    }
}

/// Raven `Rancor_Patrol`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:83-108`
pub fn Rancor_Patrol(ctx: GameContext<'_>) {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let npc_info = (*ctx.world).globals.NPCInfo;

        if !npc_info.is_null() {
            (*npc_info).localState = LSTATE_CLEAR;
        }

        //If we have somewhere to go, then do that
        if !crate::NPC_goal::UpdateGoal(ctx).is_null() {
            (*ctx.world).globals.ucmd.buttons &= !BUTTON_WALKING;
            crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);
        } else {
            if crate::g_timer::TIMER_Done(ctx, npc, c"patrolTime".as_ptr()) != 0 {
                crate::g_timer::TIMER_Set(ctx, npc, c"patrolTime".as_ptr(), ((*ctx.world).bg_state.rng.crandom() * 5000.0 + 5000.0) as c_int);
            }
        }

        if crate::NPC_utils::NPC_CheckEnemyExt(ctx, qtrue) == qfalse {
            Rancor_Idle(ctx);
            return;
        }
        Rancor_CheckRoar(ctx, npc);
        crate::g_timer::TIMER_Set(ctx, npc, c"lookForNewEnemy".as_ptr(), crate::q_math::Q_irand(5000, 15000));
    }
}

/// Raven `Rancor_Move`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:115-130`
pub fn Rancor_Move(
    ctx: GameContext<'_>,visible: qboolean) {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let npc_info = (*ctx.world).globals.NPCInfo;

        if !npc_info.is_null() && (*npc_info).localState != LSTATE_WAITING {
            (*npc_info).goalEntity = (*npc).enemy;
            if !crate::NPC_move::NPC_MoveToGoal(ctx, qtrue) {
                (*npc_info).consecutiveBlockedMoves += 1;
            } else {
                (*npc_info).consecutiveBlockedMoves = 0;
            }
            (*npc_info).goalRadius = MAX_DISTANCE;	// just get us within combat range
        }
    }
}

/// Raven `Rancor_DropVictim`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:140-194`
pub fn Rancor_DropVictim(
    ctx: GameContext<'_>,self_: *mut gentity_t) {
    //FIXME: if Rancor dies, it should drop its victim.
    //FIXME: if Rancor is removed, it must remove its victim.
    unsafe {
        let activator = crate::ent_id::resolve((*ctx.world).entities.as_mut_ptr(), (*self_).activator);
        if !activator.is_null() {
            if !(*activator).client.is_null() {
                let activator_client = (*activator).client as *mut gclient_t;
                (*activator_client).ps.eFlags2 &= !EF2_HELD_BY_MONSTER;
                (*activator_client).ps.hasLookTarget = qfalse;
                (*activator_client).ps.lookTarget = ENTITYNUM_NONE;
                (*activator_client).ps.viewangles[ROLL] = 0.0;
                crate::g_client::SetClientViewAngle(activator, (*activator_client).ps.viewangles);
                (*activator).r.currentAngles[PITCH] = 0.0;
                (*activator).r.currentAngles[ROLL] = 0.0;
                crate::g_utils::G_SetAngles(activator, (*activator).r.currentAngles);
            }
            if (*activator).health <= 0 {
                //if ( self->activator->s.number )
                {//never free player
                    if (*self_).count == 1 {
                        //in my hand, just drop them
                        if !(*activator).client.is_null() {
                            let activator_client = (*activator).client as *mut gclient_t;
                            (*activator_client).ps.legsTimer = 0;
                            (*activator_client).ps.torsoTimer = 0;
                            //FIXME: ragdoll?
                        }
                    } else {
                        if !(*activator).client.is_null() {
                            (*((*activator).client as *mut gclient_t)).ps.eFlags |= EF_NODRAW;//so his corpse doesn't drop out of me...
                        }
                        //G_FreeEntity( self->activator );
                    }
                }
            } else {
                if !(*activator).NPC.is_null() {
                    //start thinking again
                    (*((*activator).NPC as *mut gNPC_t)).nextBStateThink = (*ctx.world).level.time;
                }
                //clear their anim and let them fall
                let activator_client = (*activator).client as *mut gclient_t;
                (*activator_client).ps.legsTimer = 0;
                (*activator_client).ps.torsoTimer = 0;
            }
            if (*self_).enemy == (*self_).activator {
                (*self_).enemy = None;
            }
            (*self_).activator = None;
        }
        (*self_).count = 0;//drop him
    }
}

/// Raven `Rancor_Swing`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:196-306`
pub fn Rancor_Swing(
    ctx: GameContext<'_>,tryGrab: qboolean) {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let mut radiusEntNums: [c_int; 128] = [0; 128];
        let radius = 88.0;
        let radiusSquared = radius * radius;
        let mut boltOrg: vec3_t = [0.0; 3];

        let numEnts = crate::NPC_utils::NPC_GetEntsNearBolt(ctx, radiusEntNums.as_mut_ptr(), radius, (*(*npc).client).renderInfo.handRBolt, boltOrg);

        for i in 0..(numEnts as usize) {
            let radiusEnt = (*ctx.world).entities.get_unchecked_mut(radiusEntNums[i] as usize) as *mut gentity_t;
            if !(*radiusEnt).inuse {
                continue;
            }

            if radiusEnt == npc {
                //Skip the rancor ent
                continue;
            }

            if (*radiusEnt).client.is_null() {
                //must be a client
                continue;
            }

            if ((*(*radiusEnt).client).ps.eFlags2 & EF2_HELD_BY_MONSTER) != 0 {
                //can't be one already being held
                continue;
            }

            if DistanceSquared((*radiusEnt).r.currentOrigin, boltOrg) <= radiusSquared {
                if tryGrab != 0
                    && (*npc).count != 1 //don't have one in hand or in mouth already - FIXME: allow one in hand and any number in mouth!
                    && (*(*radiusEnt).client).NPC_class != CLASS_RANCOR
                    && (*(*radiusEnt).client).NPC_class != CLASS_GALAKMECH
                    && (*(*radiusEnt).client).NPC_class != CLASS_ATST
                    && (*(*radiusEnt).client).NPC_class != CLASS_GONK
                    && (*(*radiusEnt).client).NPC_class != CLASS_R2D2
                    && (*(*radiusEnt).client).NPC_class != CLASS_R5D2
                    && (*(*radiusEnt).client).NPC_class != CLASS_MARK1
                    && (*(*radiusEnt).client).NPC_class != CLASS_MARK2
                    && (*(*radiusEnt).client).NPC_class != CLASS_MOUSE
                    && (*(*radiusEnt).client).NPC_class != CLASS_PROBE
                    && (*(*radiusEnt).client).NPC_class != CLASS_SEEKER
                    && (*(*radiusEnt).client).NPC_class != CLASS_REMOTE
                    && (*(*radiusEnt).client).NPC_class != CLASS_SENTRY
                    && (*(*radiusEnt).client).NPC_class != CLASS_INTERROGATOR
                    && (*(*radiusEnt).client).NPC_class != CLASS_VEHICLE {
                    //grab
                    if (*npc).count == 2 {
                        //have one in my mouth, remove him
                        crate::g_timer::TIMER_Remove(ctx, npc, c"clearGrabbed".as_ptr());
                        Rancor_DropVictim(ctx, npc);
                    }
                    (*npc).enemy = radiusEnt;//make him my new best friend
                    (*(*radiusEnt).client).ps.eFlags2 |= EF2_HELD_BY_MONSTER;
                    //FIXME: this makes it so that the victim can't hit us with shots!  Just use activator or something
                    (*(*radiusEnt).client).ps.hasLookTarget = qtrue;
                    (*(*radiusEnt).client).ps.lookTarget = (*npc).s.number;
                    (*npc).activator = radiusEnt;//remember him
                    (*npc).count = 1;//in my hand
                    //wait to attack
                    crate::g_timer::TIMER_Set(ctx, npc, c"attacking".as_ptr(), (*(*npc).client).ps.legsTimer + crate::q_math::Q_irand(500, 2500));
                    if (*radiusEnt).health > 0 && (*radiusEnt).pain.is_some() {
                        //do pain on enemy
                        crate::ent_fn_enums::dispatch_pain(ctx, (*radiusEnt).pain.unwrap(), radiusEnt, npc, 100);
                        //GEntity_PainFunc( radiusEnt, NPC, NPC, radiusEnt->r.currentOrigin, 0, MOD_CRUSH );
                    } else if !(*radiusEnt).client.is_null() {
                        (*(*radiusEnt).client).ps.forceHandExtend = HANDEXTEND_NONE;
                        (*(*radiusEnt).client).ps.forceHandExtendTime = 0;
                        crate::npc_c::NPC_SetAnim(radiusEnt, SETANIM_BOTH, BOTH_SWIM_IDLE1, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                    }
                } else {
                    //smack
                    let mut pushDir: vec3_t = [0.0; 3];
                    let mut angs: vec3_t = [0.0; 3];

                    crate::g_utils::G_Sound(ctx, radiusEnt, CHAN_AUTO, crate::g_utils::G_SoundIndex(c"sound/chars/rancor/swipehit.wav".as_ptr()));
                    //actually push the enemy
                    /*
                    //VectorSubtract( radiusEnt->r.currentOrigin, boltOrg, pushDir );
                    VectorSubtract( radiusEnt->r.currentOrigin, NPC->r.currentOrigin, pushDir );
                    pushDir[2] = Q_flrand( 100, 200 );
                    VectorNormalize( pushDir );
                    */
                    crate::q_math::_VectorCopy(&(*(*npc).client).ps.viewangles, &mut angs);
                    angs[1] += crate::q_math::flrand(25.0, 50.0);
                    angs[0] = crate::q_math::flrand(-25.0, -15.0);
                    crate::q_math::AngleVectors(angs, Some(&mut pushDir), None, None);
                    if (*(*radiusEnt).client).NPC_class != CLASS_RANCOR
                        && (*(*radiusEnt).client).NPC_class != CLASS_ATST {
                        crate::g_combat::G_Damage(radiusEnt, npc, npc, Some(&mut [0.0; 3]), (*radiusEnt).r.currentOrigin, crate::q_math::Q_irand(25, 40), DAMAGE_NO_ARMOR|DAMAGE_NO_KNOCKBACK, MOD_MELEE);
                        crate::g_combat::G_Throw(ctx, radiusEnt, pushDir, 250.0);
                        if (*radiusEnt).health > 0 {
                            //do pain on enemy
                            crate::g_combat::G_Knockdown(radiusEnt);//, NPC, pushDir, 100, qtrue );
                        }
                    }
                }
            }
        }
    }
}

/// Raven `Rancor_Smash`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:308-367`
pub fn Rancor_Smash(ctx: GameContext<'_>) {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let mut radiusEntNums: [c_int; 128] = [0; 128];
        let radius = 128.0;
        let halfRadSquared = ((radius/2.0)*(radius/2.0));
        let radiusSquared = radius * radius;
        let mut boltOrg: vec3_t = [0.0; 3];

        crate::NPC_senses::AddSoundEvent(ctx, npc, (*npc).r.currentOrigin, 512.0, AEL_DANGER, qfalse);//, qtrue );

        let numEnts = crate::NPC_utils::NPC_GetEntsNearBolt(ctx, radiusEntNums.as_mut_ptr(), radius, (*(*npc).client).renderInfo.handLBolt, boltOrg);

        for i in 0..(numEnts as usize) {
            let radiusEnt = (*ctx.world).entities.get_unchecked_mut(radiusEntNums[i] as usize) as *mut gentity_t;
            if !(*radiusEnt).inuse {
                continue;
            }

            if radiusEnt == npc {
                //Skip the rancor ent
                continue;
            }

            if (*radiusEnt).client.is_null() {
                //must be a client
                continue;
            }

            if ((*(*radiusEnt).client).ps.eFlags2 & EF2_HELD_BY_MONSTER) != 0 {
                //can't be one being held
                continue;
            }

            let distSq = DistanceSquared((*radiusEnt).r.currentOrigin, boltOrg);
            if distSq <= radiusSquared {
                crate::g_utils::G_Sound(ctx, radiusEnt, CHAN_AUTO, crate::g_utils::G_SoundIndex(c"sound/chars/rancor/swipehit.wav".as_ptr()));
                if distSq < halfRadSquared {
                    //close enough to do damage, too
                    crate::g_combat::G_Damage(radiusEnt, npc, npc, Some(&mut [0.0; 3]), (*radiusEnt).r.currentOrigin, crate::q_math::Q_irand(10, 25), DAMAGE_NO_ARMOR|DAMAGE_NO_KNOCKBACK, MOD_MELEE);
                }
                if (*radiusEnt).health > 0
                    && !(*radiusEnt).client.is_null()
                    && (*(*radiusEnt).client).NPC_class != CLASS_RANCOR
                    && (*(*radiusEnt).client).NPC_class != CLASS_ATST {
                    if distSq < halfRadSquared
                        || (*(*radiusEnt).client).ps.groundEntityNum != ENTITYNUM_NONE {
                        //within range of my fist or withing ground-shaking range and not in the air
                        crate::g_combat::G_Knockdown(radiusEnt);//, NPC, vec3_origin, 100, qtrue );
                    }
                }
            }
        }
    }
}

/// Raven `Rancor_Bite`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:369-428`
pub fn Rancor_Bite(ctx: GameContext<'_>) {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let mut radiusEntNums: [c_int; 128] = [0; 128];
        let radius = 100.0;
        let radiusSquared = radius * radius;
        let mut boltOrg: vec3_t = [0.0; 3];

        let numEnts = crate::NPC_utils::NPC_GetEntsNearBolt(ctx, radiusEntNums.as_mut_ptr(), radius, (*(*npc).client).renderInfo.crotchBolt, boltOrg);//was gutBolt?

        for i in 0..(numEnts as usize) {
            let radiusEnt = (*ctx.world).entities.get_unchecked_mut(radiusEntNums[i] as usize) as *mut gentity_t;
            if !(*radiusEnt).inuse {
                continue;
            }

            if radiusEnt == npc {
                //Skip the rancor ent
                continue;
            }

            if (*radiusEnt).client.is_null() {
                //must be a client
                continue;
            }

            if ((*(*radiusEnt).client).ps.eFlags2 & EF2_HELD_BY_MONSTER) != 0 {
                //can't be one already being held
                continue;
            }

            if DistanceSquared((*radiusEnt).r.currentOrigin, boltOrg) <= radiusSquared {
                crate::g_combat::G_Damage(radiusEnt, npc, npc, Some(&mut [0.0; 3]), (*radiusEnt).r.currentOrigin, crate::q_math::Q_irand(15, 30), DAMAGE_NO_ARMOR|DAMAGE_NO_KNOCKBACK, MOD_MELEE);
                if (*radiusEnt).health <= 0 && !(*radiusEnt).client.is_null() {
                    //killed them, chance of dismembering
                    if !crate::q_math::Q_irand(0, 1) != 0 {
                        //bite something off
                        let hitLoc = crate::q_math::Q_irand(G2_MODELPART_HEAD, G2_MODELPART_RLEG);
                        if hitLoc == G2_MODELPART_HEAD {
                            crate::npc_c::NPC_SetAnim(radiusEnt, SETANIM_BOTH, BOTH_DEATH17, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                        } else if hitLoc == G2_MODELPART_WAIST {
                            crate::npc_c::NPC_SetAnim(radiusEnt, SETANIM_BOTH, BOTH_DEATHBACKWARD2, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                        }
                        //radiusEnt->client->dismembered = qfalse;
                        //FIXME: the limb should just disappear, cuz I ate it
                        crate::g_combat::G_Dismember(ctx, radiusEnt, npc, (*radiusEnt).r.currentOrigin, hitLoc, 90.0, 0.0, (*(*radiusEnt).client).ps.torsoAnim, qtrue);
                        //G_DoDismemberment( radiusEnt, radiusEnt->r.currentOrigin, MOD_SABER, 1000, hitLoc, qtrue );
                    }
                }
                crate::g_utils::G_Sound(ctx, radiusEnt, CHAN_AUTO, crate::g_utils::G_SoundIndex(c"sound/chars/rancor/chomp.wav".as_ptr()));
            }
        }
    }
}

/// Raven `Rancor_Attack`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:431-614`
pub fn Rancor_Attack(
    ctx: GameContext<'_>,distance: f32, doCharge: qboolean) {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let ent_base = (*ctx.world).entities.as_mut_ptr();

        if !crate::g_timer::TIMER_Exists(ctx, npc, c"attacking".as_ptr()) {
            if (*npc).count == 2 && !(*npc).activator.is_none() {
            } else if (*npc).count == 1 && !(*npc).activator.is_none() {
                let activator = crate::ent_id::resolve(ent_base, (*npc).activator);
                //holding enemy
                if (*activator).health > 0 && crate::q_math::Q_irand(0, 1) != 0 {
                    //quick bite
                    crate::npc_c::NPC_SetAnim(npc, SETANIM_BOTH, BOTH_ATTACK1, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                    crate::g_timer::TIMER_Set(ctx, npc, c"attack_dmg".as_ptr(), 450);
                } else {
                    //full eat
                    crate::npc_c::NPC_SetAnim(npc, SETANIM_BOTH, BOTH_ATTACK3, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                    crate::g_timer::TIMER_Set(ctx, npc, c"attack_dmg".as_ptr(), 900);
                    //Make victim scream in fright
                    if (*activator).health > 0 && !(*activator).client.is_null() {
                        crate::g_utils::G_AddEvent(activator, crate::q_math::Q_irand(EV_DEATH1, EV_DEATH3), 0);
                        crate::npc_c::NPC_SetAnim(activator, SETANIM_TORSO, BOTH_FALLDEATH1, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                        if !(*activator).NPC.is_null() {
                            //no more thinking for you
                            crate::g_combat::TossClientItems(ctx, npc);
                            (*((*activator).NPC as *mut gNPC_t)).nextBStateThink = Q3_INFINITE;
                        }
                    }
                }
            } else if (*crate::ent_id::resolve(ent_base, (*npc).enemy)).health > 0 && doCharge != 0 {
                //charge
                let mut fwd: vec3_t = [0.0; 3];
                let mut yawAng: vec3_t = [0.0, (*(*npc).client).ps.viewangles[1], 0.0];
                crate::q_math::AngleVectors(yawAng, Some(&mut fwd), None, None);
                crate::q_math::_VectorScale(fwd, distance*1.5, &mut (*(*npc).client).ps.velocity);
                (*(*npc).client).ps.velocity[2] = 150.0;
                (*(*npc).client).ps.groundEntityNum = ENTITYNUM_NONE;

                crate::npc_c::NPC_SetAnim(npc, SETANIM_BOTH, BOTH_MELEE2, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                crate::g_timer::TIMER_Set(ctx, npc, c"attack_dmg".as_ptr(), 1250);
            } else if !crate::q_math::Q_irand(0, 1) != 0 {
                //smash
                crate::npc_c::NPC_SetAnim(npc, SETANIM_BOTH, BOTH_MELEE1, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                crate::g_timer::TIMER_Set(ctx, npc, c"attack_dmg".as_ptr(), 1000);
            } else {
                //try to grab
                crate::npc_c::NPC_SetAnim(npc, SETANIM_BOTH, BOTH_ATTACK2, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                crate::g_timer::TIMER_Set(ctx, npc, c"attack_dmg".as_ptr(), 1000);
            }

            crate::g_timer::TIMER_Set(ctx, npc, c"attacking".as_ptr(), ((*(*npc).client).ps.legsTimer as f32 + (*ctx.world).bg_state.rng.random() * 200.0) as c_int);
        }

        // Need to do delayed damage since the attack animations encapsulate multiple mini-attacks

        if crate::g_timer::TIMER_Done2(ctx, npc, c"attack_dmg".as_ptr(), qtrue) != 0 {
            let mut shakePos: vec3_t = [0.0; 3];
            match (*(*npc).client).ps.legsAnim {
            BOTH_MELEE1 => {
                Rancor_Smash(ctx);
                crate::NPC_utils::G_GetBoltPosition(ctx, npc, (*(*npc).client).renderInfo.handLBolt, shakePos, 0);
                crate::g_utils::G_ScreenShake(ctx, shakePos, core::ptr::null_mut(), 4.0, 1000, qfalse);
                //CGCam_Shake( 1.0f*playerDist/128.0f, 1000 );
            },
            BOTH_MELEE2 => {
                Rancor_Bite(ctx);
                crate::g_timer::TIMER_Set(ctx, npc, c"attack_dmg2".as_ptr(), 450);
            },
            BOTH_ATTACK1 => {
                if (*npc).count == 1 && !(*npc).activator.is_none() {
                    let activator = crate::ent_id::resolve(ent_base, (*npc).activator);
                    crate::g_combat::G_Damage(activator, npc, npc, Some(&mut [0.0; 3]), (*activator).r.currentOrigin, crate::q_math::Q_irand(25, 40), DAMAGE_NO_ARMOR|DAMAGE_NO_KNOCKBACK, MOD_MELEE);
                    if (*activator).health <= 0 {
                        //killed him
                        //make it look like we bit his head off
                        //NPC->activator->client->dismembered = qfalse;
                        let activator_client = (*activator).client as *mut gclient_t;
                        crate::g_combat::G_Dismember(ctx, activator, npc, (*activator).r.currentOrigin, G2_MODELPART_HEAD, 90.0, 0.0, (*activator_client).ps.torsoAnim, qtrue);
                        //G_DoDismemberment( NPC->activator, NPC->activator->r.currentOrigin, MOD_SABER, 1000, HL_HEAD, qtrue );
                        (*activator_client).ps.forceHandExtend = HANDEXTEND_NONE;
                        (*activator_client).ps.forceHandExtendTime = 0;
                        crate::npc_c::NPC_SetAnim(activator, SETANIM_BOTH, BOTH_SWIM_IDLE1, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                    }
                    crate::g_utils::G_Sound(ctx, activator, CHAN_AUTO, crate::g_utils::G_SoundIndex(c"sound/chars/rancor/chomp.wav".as_ptr()));
                }
            },
            BOTH_ATTACK2 => {
                //try to grab
                Rancor_Swing(ctx, qtrue);
            },
            BOTH_ATTACK3 => {
                if (*npc).count == 1 && !(*npc).activator.is_none() {
                    let activator = crate::ent_id::resolve(ent_base, (*npc).activator);
                    //cut in half
                    if !(*activator).client.is_null() {
                        //NPC->activator->client->dismembered = qfalse;
                        crate::g_combat::G_Dismember(ctx, activator, npc, (*activator).r.currentOrigin, G2_MODELPART_WAIST, 90.0, 0.0, (*((*activator).client as *mut gclient_t)).ps.torsoAnim, qtrue);
                        //G_DoDismemberment( NPC->activator, NPC->enemy->r.currentOrigin, MOD_SABER, 1000, HL_WAIST, qtrue );
                    }
                    //KILL
                    crate::g_combat::G_Damage(activator, npc, npc, Some(&mut [0.0; 3]), (*activator).r.currentOrigin, (*crate::ent_id::resolve(ent_base, (*npc).enemy)).health+10, DAMAGE_NO_PROTECTION|DAMAGE_NO_ARMOR|DAMAGE_NO_KNOCKBACK|DAMAGE_NO_HIT_LOC, MOD_MELEE);//, HL_NONE );//
                    if !(*activator).client.is_null() {
                        let activator_client = (*activator).client as *mut gclient_t;
                        (*activator_client).ps.forceHandExtend = HANDEXTEND_NONE;
                        (*activator_client).ps.forceHandExtendTime = 0;
                        crate::npc_c::NPC_SetAnim(activator, SETANIM_BOTH, BOTH_SWIM_IDLE1, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                    }
                    crate::g_timer::TIMER_Set(ctx, npc, c"attack_dmg2".as_ptr(), 1350);
                    crate::g_utils::G_Sound(ctx, activator, CHAN_AUTO, crate::g_utils::G_SoundIndex(c"sound/chars/rancor/swipehit.wav".as_ptr()));
                    crate::g_utils::G_AddEvent(activator, EV_JUMP, (*activator).health);
                }
            },
            _ => {}
            }
        } else if crate::g_timer::TIMER_Done2(ctx, npc, c"attack_dmg2".as_ptr(), qtrue) != 0 {
            match (*(*npc).client).ps.legsAnim {
            BOTH_MELEE1 => {
            },
            BOTH_MELEE2 => {
                Rancor_Bite(ctx);
            },
            BOTH_ATTACK1 => {
            },
            BOTH_ATTACK2 => {
            },
            BOTH_ATTACK3 => {
                if (*npc).count == 1 && !(*npc).activator.is_none() {
                    let activator = crate::ent_id::resolve(ent_base, (*npc).activator);
                    //swallow victim
                    crate::g_utils::G_Sound(ctx, activator, CHAN_AUTO, crate::g_utils::G_SoundIndex(c"sound/chars/rancor/chomp.wav".as_ptr()));
                    //FIXME: sometimes end up with a live one in our mouths?
                    //just make sure they're dead
                    if (*activator).health > 0 {
                        //cut in half
                        //NPC->activator->client->dismembered = qfalse;
                        let activator_client = (*activator).client as *mut gclient_t;
                        crate::g_combat::G_Dismember(ctx, activator, npc, (*activator).r.currentOrigin, G2_MODELPART_WAIST, 90.0, 0.0, (*activator_client).ps.torsoAnim, qtrue);
                        //G_DoDismemberment( NPC->activator, NPC->enemy->r.currentOrigin, MOD_SABER, 1000, HL_WAIST, qtrue );
                        //KILL
                        crate::g_combat::G_Damage(activator, npc, npc, Some(&mut [0.0; 3]), (*activator).r.currentOrigin, (*crate::ent_id::resolve(ent_base, (*npc).enemy)).health+10, DAMAGE_NO_PROTECTION|DAMAGE_NO_ARMOR|DAMAGE_NO_KNOCKBACK|DAMAGE_NO_HIT_LOC, MOD_MELEE);//, HL_NONE );
                        (*activator_client).ps.forceHandExtend = HANDEXTEND_NONE;
                        (*activator_client).ps.forceHandExtendTime = 0;
                        crate::npc_c::NPC_SetAnim(activator, SETANIM_BOTH, BOTH_SWIM_IDLE1, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                        crate::g_utils::G_AddEvent(activator, EV_JUMP, (*activator).health);
                    }
                    if !(*activator).client.is_null() {
                        //*sigh*, can't get tags right, just remove them?
                        (*((*activator).client as *mut gclient_t)).ps.eFlags |= EF_NODRAW;
                    }
                    (*npc).count = 2;
                    crate::g_timer::TIMER_Set(ctx, npc, c"clearGrabbed".as_ptr(), 2600);
                }
            },
            _ => {}
            }
        } else if (*(*npc).client).ps.legsAnim == BOTH_ATTACK2 {
            if (*(*npc).client).ps.legsTimer >= 1200 && (*(*npc).client).ps.legsTimer <= 1350 {
                if crate::q_math::Q_irand(0, 2) != 0 {
                    Rancor_Swing(ctx, qfalse);
                } else {
                    Rancor_Swing(ctx, qtrue);
                }
            } else if (*(*npc).client).ps.legsTimer >= 1100 && (*(*npc).client).ps.legsTimer <= 1550 {
                Rancor_Swing(ctx, qtrue);
            }
        }

        // Just using this to remove the attacking flag at the right time
        crate::g_timer::TIMER_Done2(ctx, npc, c"attacking".as_ptr(), qtrue);
    }
}

/// Raven `Rancor_Combat`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:617-695`
pub fn Rancor_Combat(ctx: GameContext<'_>) {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let npc_info = (*ctx.world).globals.NPCInfo;

        if (*npc).count != 0 {
            //holding my enemy
            if crate::g_timer::TIMER_Done2(ctx, npc, c"takingPain".as_ptr(), qtrue) != 0 {
                (*npc_info).localState = LSTATE_CLEAR;
            } else {
                Rancor_Attack(ctx, 0.0, qfalse);
            }
            crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
            return;
        }
        // If we cannot see our target or we have somewhere to go, then do that
        if crate::NPC_utils::NPC_ClearLOS4(ctx, (*npc).enemy) == qfalse {
            //|| UpdateGoal( ))
            (*npc_info).combatMove = qtrue;
            (*npc_info).goalEntity = (*npc).enemy;
            (*npc_info).goalRadius = MIN_DISTANCE;//MAX_DISTANCE;	// just get us within combat range

            if !crate::NPC_move::NPC_MoveToGoal(ctx, qtrue) {
                //couldn't go after him?  Look for a new one
                crate::g_timer::TIMER_Set(ctx, npc, c"lookForNewEnemy".as_ptr(), 0);
                (*npc_info).consecutiveBlockedMoves += 1;
            } else {
                (*npc_info).consecutiveBlockedMoves = 0;
            }
            return;
        }

        // Sometimes I have problems with facing the enemy I'm attacking, so force the issue so I don't look dumb
        crate::NPC_utils::NPC_FaceEnemy(ctx, qtrue);

        {
            let enemy = crate::ent_id::resolve((*ctx.world).entities.as_mut_ptr(), (*npc).enemy);
            let distance = crate::q_math::Distance((*npc).r.currentOrigin, (*enemy).r.currentOrigin);
            let advance = if distance > ((*npc).r.maxs[0] + MIN_DISTANCE as f32) { qtrue } else { qfalse };
            let mut doCharge = qfalse;

            if advance != 0 {
                //have to get closer
                let yawOnlyAngles: vec3_t = [0.0, (*npc).r.currentAngles[1], 0.0];
                if (*enemy).health > 0
                    && (distance - 250.0).abs() <= 80.0
                    && crate::NPC_senses::InFOV3((*enemy).r.currentOrigin, (*npc).r.currentOrigin, yawOnlyAngles, 30, 30) != 0 {
                    if !crate::q_math::Q_irand(0, 9) != 0 {
                        //go for the charge
                        doCharge = qtrue;
                    }
                }
            }

            if (advance != 0 || crate::g_timer::TIMER_Done(ctx, npc, c"attacking".as_ptr()) != 0) {
                // waiting monsters can't attack
                if crate::g_timer::TIMER_Done2(ctx, npc, c"takingPain".as_ptr(), qtrue) != 0 {
                    (*npc_info).localState = LSTATE_CLEAR;
                } else {
                    Rancor_Move(ctx, 1);
                }
            } else {
                Rancor_Attack(ctx, distance, doCharge);
            }
        }
    }
}

/// Raven `NPC_Rancor_Pain`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:703-782`
pub fn NPC_Rancor_Pain(
    ctx: GameContext<'_>,self_: *mut gentity_t, attacker: *mut gentity_t, damage: c_int) {
    unsafe {
        let mut hitByRancor = qfalse;
        if !attacker.is_null() && !(*attacker).client.is_null() && (*(*attacker).client).NPC_class == CLASS_RANCOR {
            hitByRancor = qtrue;
        }
        let self_enemy = crate::ent_id::resolve((*ctx.world).entities.as_mut_ptr(), (*self_).enemy);
        if !attacker.is_null()
            && (*attacker).inuse != 0
            && attacker != (*self_).enemy
            && ((*attacker).flags & FL_NOTARGET) == 0 {
            if (*self_).count == 0 {
                if ((*attacker).s.number == 0 && !crate::q_math::Q_irand(0,3) != 0)
                    || (*self_).enemy.is_none()
                    || (*self_enemy).health == 0
                    || (!(*self_enemy).client.is_null() && (*((*self_enemy).client as *mut gclient_t)).NPC_class == CLASS_RANCOR)
                    || (!(*self_).NPC.is_null() && (*(*self_).NPC).consecutiveBlockedMoves>=10 && DistanceSquared((*attacker).r.currentOrigin, (*self_).r.currentOrigin) < DistanceSquared((*self_enemy).r.currentOrigin, (*self_).r.currentOrigin)) {
                    //if my enemy is dead (or attacked by player) and I'm not still holding/eating someone, turn on the attacker
                    //FIXME: if can't nav to my enemy, take this guy if I can nav to him
                    crate::NPC_combat::G_SetEnemy(ctx, self_, attacker);
                    crate::g_timer::TIMER_Set(self_, c"lookForNewEnemy".as_ptr(), crate::q_math::Q_irand(5000, 15000));
                    if hitByRancor != 0 {
                        //stay mad at this Rancor for 2-5 secs before looking for attacker enemies
                        crate::g_timer::TIMER_Set(self_, c"rancorInfight".as_ptr(), crate::q_math::Q_irand(2000, 5000));
                    }
                }
            }
        }
        if (hitByRancor != 0 || ((*self_).count == 1 && !(*self_).activator.is_none() && !crate::q_math::Q_irand(0,4) != 0) || crate::q_math::Q_irand(0, 200) < damage)
            && (*(*self_).client).ps.legsAnim != BOTH_STAND1TO2
            && crate::g_timer::TIMER_Done(ctx, self_, c"takingPain".as_ptr()) != 0 {
            if !Rancor_CheckRoar(ctx, self_) {
                if (*(*self_).client).ps.legsAnim != BOTH_MELEE1
                    && (*(*self_).client).ps.legsAnim != BOTH_MELEE2
                    && (*(*self_).client).ps.legsAnim != BOTH_ATTACK2 {
                    //cant interrupt one of the big attack anims
                    /*
                    if ( self->count != 1
                        || attacker == self->activator
                        || (self->client->ps.legsAnim != BOTH_ATTACK1&&self->client->ps.legsAnim != BOTH_ATTACK3) )
                    */
                    {
                        //if going to bite our victim, only victim can interrupt that anim
                        if (*self_).health > 100 || hitByRancor != 0 {
                            crate::g_timer::TIMER_Remove(ctx, self_, c"attacking".as_ptr());

                            mp_qshared::shared::q_math::VectorCopy(&(*(*self_).NPC).lastPathAngles, &mut (*self_).s.angles);

                            if (*self_).count == 1 {
                                crate::npc_c::NPC_SetAnim(self_, SETANIM_BOTH, BOTH_PAIN2, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                            } else {
                                crate::npc_c::NPC_SetAnim(self_, SETANIM_BOTH, BOTH_PAIN1, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD);
                            }
                            crate::g_timer::TIMER_Set(ctx, self_, c"takingPain".as_ptr(), (*(*self_).client).ps.legsTimer + crate::q_math::Q_irand(0, 500));
                        }
                        if (*self_).count == 1 {
                            (*(*self_).NPC).localState = LSTATE_WAITING;
                        }
                    }
                }
            }
        }
    }
}

/// Raven `Rancor_CheckDropVictim`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:784-802`
pub fn Rancor_CheckDropVictim(ctx: GameContext<'_>) {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let activator = crate::ent_id::resolve((*ctx.world).entities.as_mut_ptr(), (*npc).activator);
        let mins: vec3_t = [(*activator).r.mins[0]-1.0, (*activator).r.mins[1]-1.0, 0.0];
        let maxs: vec3_t = [(*activator).r.maxs[0]+1.0, (*activator).r.maxs[1]+1.0, 1.0];
        let start: vec3_t = [(*activator).r.currentOrigin[0], (*activator).r.currentOrigin[1], (*activator).r.absmin[2]];
        let end: vec3_t = [(*activator).r.currentOrigin[0], (*activator).r.currentOrigin[1], (*activator).r.absmax[2]-1.0];
        let mut trace: trace_t = core::mem::zeroed();

        trap::Trace(ctx.engine, &mut trace, start, mins, maxs, end, (*activator).s.number, (*activator).clipmask);
        if !trace.allsolid && !trace.startsolid && trace.fraction >= 1.0 {
            Rancor_DropVictim(ctx, npc);
        }
    }
}

/// Raven `Rancor_Crush`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:805-821`
pub fn Rancor_Crush(ctx: GameContext<'_>) {
    unsafe {
        let npc = (*ctx.world).globals.NPC;

        if npc.is_null()
            || (*npc).client.is_null()
            || (*(*npc).client).ps.groundEntityNum >= ENTITYNUM_WORLD {
            //nothing to crush
            return;
        }

        let crush = &mut (*ctx.world).entities[(*(*npc).client).ps.groundEntityNum as usize];
        if crush.inuse != 0 && !crush.client.is_null() && crush.localAnimIndex == 0 {
            //a humanoid, smash them good.
            crate::g_combat::G_Damage(crush as *mut gentity_t, npc, npc, None, (*npc).r.currentOrigin, 200, 0, MOD_CRUSH);
        }
    }
}

/// Raven `NPC_BSRancor_Default`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:828-955`
pub fn NPC_BSRancor_Default(ctx: GameContext<'_>) {
    unsafe {
        let npc = (*ctx.world).globals.NPC;
        let npc_info = (*ctx.world).globals.NPCInfo;

        crate::NPC_senses::AddSightEvent(ctx, npc, (*npc).r.currentOrigin, 1024.0, AEL_DANGER_GREAT, 50.0);

        Rancor_Crush(ctx);

        (*(*npc).client).ps.eFlags2 &= !(EF2_USE_ALT_ANIM|EF2_GENERIC_NPC_FLAG);
        if (*npc).count != 0 {
            //holding someone
            (*(*npc).client).ps.eFlags2 |= EF2_USE_ALT_ANIM;
            if (*npc).count == 2 {
                //in my mouth
                (*(*npc).client).ps.eFlags2 |= EF2_GENERIC_NPC_FLAG;
            }
        } else {
            (*(*npc).client).ps.eFlags2 &= !(EF2_USE_ALT_ANIM|EF2_GENERIC_NPC_FLAG);
        }

        if crate::g_timer::TIMER_Done2(ctx, npc, c"clearGrabbed".as_ptr(), qtrue) != 0 {
            Rancor_DropVictim(ctx, npc);
        } else if (*(*npc).client).ps.legsAnim == BOTH_PAIN2
            && (*npc).count == 1
            && !(*npc).activator.is_none() {
            if crate::q_math::Q_irand(0, 3) == 0 {
                Rancor_CheckDropVictim(ctx);
            }
        }
        if crate::g_timer::TIMER_Done(ctx, npc, c"rageTime".as_ptr()) == 0 {
            //do nothing but roar first time we see an enemy
            crate::NPC_senses::AddSoundEvent(ctx, npc, (*npc).r.currentOrigin, 1024.0, AEL_DANGER_GREAT, qfalse);//, qfalse );
            crate::NPC_utils::NPC_FaceEnemy(ctx, qtrue);
            return;
        }
        if !(*npc).enemy.is_none() {
            /*
            if ( NPC->enemy->client //enemy is a client
                && (NPC->enemy->client->NPC_class == CLASS_UGNAUGHT || NPC->enemy->client->NPC_class == CLASS_JAWA )//enemy is a lowly jawa or ugnaught
                && NPC->enemy->enemy != NPC//enemy's enemy is not me
                && (!NPC->enemy->enemy || !NPC->enemy->enemy->client || NPC->enemy->enemy->client->NPC_class!=CLASS_RANCOR) )//enemy's enemy is not a client or is not a rancor (which is as scary as me anyway)
            {//they should be scared of ME and no-one else
                G_SetEnemy( NPC->enemy, NPC );
            }
            */
            if crate::g_timer::TIMER_Done(ctx, npc, c"angrynoise".as_ptr()) != 0 {
                crate::g_utils::G_Sound(ctx, npc, CHAN_AUTO, crate::g_utils::G_SoundIndex(
                    crate::q_shared::va(c"sound/chars/rancor/misc/anger%d.wav".as_ptr(), crate::q_math::Q_irand(1, 3))
                ));

                crate::g_timer::TIMER_Set(ctx, npc, c"angrynoise".as_ptr(), crate::q_math::Q_irand(5000, 10000));
            } else {
                crate::NPC_senses::AddSoundEvent(ctx, npc, (*npc).r.currentOrigin, 512.0, AEL_DANGER_GREAT, qfalse);//, qfalse );
            }
            if (*npc).count == 2 && (*(*npc).client).ps.legsAnim == BOTH_ATTACK3 {
                //we're still chewing our enemy up
                crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
                return;
            }
            //else, if he's in our hand, we eat, else if he's on the ground, we keep attacking his dead body for a while
            let npc_enemy = crate::ent_id::resolve((*ctx.world).entities.as_mut_ptr(), (*npc).enemy);
            if !(*npc_enemy).client.is_null() && (*((*npc_enemy).client as *mut gclient_t)).NPC_class == CLASS_RANCOR {
                //got mad at another Rancor, look for a valid enemy
                if crate::g_timer::TIMER_Done(ctx, npc, c"rancorInfight".as_ptr()) != 0 {
                    crate::NPC_utils::NPC_CheckEnemyExt(ctx, qtrue);
                }
            } else if (*npc).count == 0 {
                if crate::NPC_combat::ValidEnemy(ctx, (*npc).enemy) == qfalse {
                    crate::g_timer::TIMER_Remove(ctx, npc, c"lookForNewEnemy".as_ptr());//make them look again right now
                    if (*npc_enemy).inuse == 0 || (*ctx.world).level.time - (*npc_enemy).s.time > crate::q_math::Q_irand(10000, 15000) {
                        //it's been a while since the enemy died, or enemy is completely gone, get bored with him
                        (*npc).enemy = None;
                        Rancor_Patrol(ctx);
                        crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
                        return;
                    }
                }
                if crate::g_timer::TIMER_Done(ctx, npc, c"lookForNewEnemy".as_ptr()) != 0 {
                    let newEnemy = crate::NPC_combat::NPC_CheckEnemy(ctx, ((*npc_info).confusionTime < (*ctx.world).level.time) as c_int, qfalse, qfalse);
                    let sav_enemy = (*npc).enemy;
                    (*npc).enemy = None;
                    let newEnemy = newEnemy;
                    (*npc).enemy = sav_enemy;
                    if !newEnemy.is_null() && newEnemy != sav_enemy {
                        //picked up a new enemy!
                        (*npc).lastEnemy = (*npc).enemy;
                        crate::NPC_combat::G_SetEnemy(ctx, npc, newEnemy);
                        //hold this one for at least 5-15 seconds
                        crate::g_timer::TIMER_Set(ctx, npc, c"lookForNewEnemy".as_ptr(), crate::q_math::Q_irand(5000, 15000));
                    } else {
                        //look again in 2-5 secs
                        crate::g_timer::TIMER_Set(ctx, npc, c"lookForNewEnemy".as_ptr(), crate::q_math::Q_irand(2000, 5000));
                    }
                }
            }
            Rancor_Combat(ctx);
        } else {
            if crate::g_timer::TIMER_Done(ctx, npc, c"idlenoise".as_ptr()) != 0 {
                crate::g_utils::G_Sound(ctx, npc, CHAN_AUTO, crate::g_utils::G_SoundIndex(
                    crate::q_shared::va(c"sound/chars/rancor/snort_%d.wav".as_ptr(), crate::q_math::Q_irand(1, 2))
                ));

                crate::g_timer::TIMER_Set(ctx, npc, c"idlenoise".as_ptr(), crate::q_math::Q_irand(2000, 4000));
                crate::NPC_senses::AddSoundEvent(ctx, npc, (*npc).r.currentOrigin, 384.0, AEL_DANGER, qfalse);//, qfalse );
            }
            if ((*npc_info).scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
                Rancor_Patrol(ctx);
            } else {
                Rancor_Idle(ctx);
            }
        }

        crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
    }
}
