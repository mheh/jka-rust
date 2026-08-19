//! Port of `oracle/codemp/game/NPC_AI_Rancor.c`.
//!
//! Entity access uses a checked `ctx.world.entity(id)`/`entity_mut(id)` borrow at the point of use.
//! The gNPC_t (`NPCInfo`/`gentity_t.NPC`) and BG_Alloc'd pool-client (`gentity_t.client`) derefs have no accessor.
//! They stay raw inside tight `unsafe` blocks through a copied pointer value (`// FLAG:` sites).
//! Hoisting entity reads into role-named locals in source order keeps RNG and trap ordering the same as reading in place.
//! No referee scenario covers this file.
//! Parity rests on the compile and the golden suite.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::g_utils::G_SoundIndex;

// These define the working combat range for these suckers
const MIN_DISTANCE: c_int = 128;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;
const MAX_DISTANCE: c_int = 1024;
const MAX_DISTANCE_SQR: c_int = MAX_DISTANCE * MAX_DISTANCE;
const LSTATE_CLEAR: c_int = 0;
const LSTATE_WAITING: c_int = 1;

// `DistanceSquared` is the canonical `crate::q_math::DistanceSquared`, reached via the prelude glob.
// There is no per-file copy.

/// Raven `Rancor_SetBolts`.
///
/// Source: `oracle/codemp/game/NPC_AI_Rancor.c:19-29`
pub fn Rancor_SetBolts(ctx: &mut GameContext, self_: Option<EntityId>) {
    let Some(self_id) = self_ else {
        return;
    };
    // FLAG: NPC carries a BG_Alloc'd pool client (not level.clients); deref raw via the safe entity borrow.
    let client = ctx.world.entity(self_id).client;
    if client.is_null() {
        return;
    }
    let ghoul2 = ctx.world.entity(self_id).ghoul2 as *mut c_void;
    unsafe {
        // `gentity_t.client` stays `*mut c_void` per the deferral; overlay-cast to
        // `gclient_t` at the use site.
        let ri = &mut (*client).renderInfo;
        ri.handRBolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*r_hand");
        ri.handLBolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*l_hand");
        ri.headBolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*head_eyes");
        ri.torsoBolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "jaw_bone");
    }
}

/// Raven `NPC_Rancor_Precache`.
///
/// Source: `oracle/codemp/game/NPC_AI_Rancor.c:36-45`
pub fn NPC_Rancor_Precache(ctx: &mut GameContext) {
    for i in 1..3 {
        G_SoundIndex(ctx, &format!("sound/chars/rancor/snort_{}.wav", i));
    }
    G_SoundIndex(ctx, "sound/chars/rancor/swipehit.wav");
    G_SoundIndex(ctx, "sound/chars/rancor/chomp.wav");
}

/// Raven `Rancor_Idle`.
///
/// Source: `oracle/codemp/game/NPC_AI_Rancor.c:53-63`
pub fn Rancor_Idle(ctx: &mut GameContext) {
    // FLAG: gNPC_t (NPCInfo) has no accessor; deref stays raw.
    let npc_info = ctx.world.globals.NPCInfo;
    if !npc_info.is_null() {
        unsafe {
            (*npc_info).localState = LSTATE_CLEAR;
        }
    }

    //If we have somewhere to go, then do that
    if !crate::NPC_goal::UpdateGoal(ctx).is_null() {
        ctx.world.globals.ucmd.buttons &= !BUTTON_WALKING;
        crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);
    }
}

/// Raven `Rancor_CheckRoar`.
///
/// Source: `oracle/codemp/game/NPC_AI_Rancor.c:66-77`
pub fn Rancor_CheckRoar(ctx: &mut GameContext, self_: EntityId) -> qboolean {
    if ctx.world.entity(self_).wait == 0.0 {
        //haven't ever gotten mad yet
        ctx.world.entity_mut(self_).wait = 1.0; //do this only once
                                                // FLAG: pool client, deref raw via safe entity borrow.
        let client = ctx.world.entity(self_).client;
        unsafe {
            (*client).ps.eFlags2 |= EF2_ALERTED;
        }
        crate::npc_c::NPC_SetAnim(
            ctx,
            self_,
            SETANIM_BOTH,
            BOTH_STAND1TO2 as c_int,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );
        let legs_timer = unsafe { (*client).ps.legsTimer };
        crate::g_timer::TIMER_Set(ctx, Some(self_), c"rageTime".as_ptr(), legs_timer);
        return qtrue;
    }
    qfalse
}

/// Raven `Rancor_Patrol`.
///
/// Source: `oracle/codemp/game/NPC_AI_Rancor.c:83-108`
pub fn Rancor_Patrol(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; deref stays raw.
    let npc_info = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    if !npc_info.is_null() {
        unsafe {
            (*npc_info).localState = LSTATE_CLEAR;
        }
    }

    //If we have somewhere to go, then do that
    if !crate::NPC_goal::UpdateGoal(ctx).is_null() {
        ctx.world.globals.ucmd.buttons &= !BUTTON_WALKING;
        crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);
    } else {
        if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"patrolTime".as_ptr()) != 0 {
            let patrol_time = (ctx.world.bg_state.rng.crandom() * 5000.0 + 5000.0) as c_int;
            crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"patrolTime".as_ptr(), patrol_time);
        }
    }

    if crate::NPC_utils::NPC_CheckEnemyExt(ctx, qtrue) == qfalse {
        Rancor_Idle(ctx);
        return;
    }
    Rancor_CheckRoar(ctx, npc_id);
    let look_for_new_enemy = ctx.world.bg_state.rng.Q_irand(5000, 15000);
    crate::g_timer::TIMER_Set(
        ctx,
        Some(npc_id),
        c"lookForNewEnemy".as_ptr(),
        look_for_new_enemy,
    );
}

/// Raven `Rancor_Move`.
///
/// Source: `oracle/codemp/game/NPC_AI_Rancor.c:115-130`
pub fn Rancor_Move(ctx: &mut GameContext, visible: qboolean) {
    let npc = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw.
    let npc_info = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    unsafe {
        if !npc_info.is_null() && (*npc_info).localState != LSTATE_WAITING {
            (*npc_info).goalEntity = ctx.world.entity(npc_id).enemy;
            if crate::NPC_move::NPC_MoveToGoal(ctx, qtrue) == qfalse {
                (*npc_info).consecutiveBlockedMoves += 1;
            } else {
                (*npc_info).consecutiveBlockedMoves = 0;
            }
            (*npc_info).goalRadius = MAX_DISTANCE; // just get us within combat range
        }
    }
}

/// Raven `Rancor_DropVictim`.
///
/// Source: `oracle/codemp/game/NPC_AI_Rancor.c:140-194`
pub fn Rancor_DropVictim(ctx: &mut GameContext, self_: EntityId) {
    //FIXME: if Rancor dies, it should drop its victim.
    //FIXME: if Rancor is removed, it must remove its victim.
    if let Some(activator_id) = ctx.world.entity(self_).activator {
        // FLAG: pool client, deref raw via safe entity borrow.
        let activator_client = ctx.world.entity(activator_id).client;
        unsafe {
            if !activator_client.is_null() {
                (*activator_client).ps.eFlags2 &= !EF2_HELD_BY_MONSTER;
                (*activator_client).ps.hasLookTarget = qfalse;
                (*activator_client).ps.lookTarget = ENTITYNUM_NONE;
                (*activator_client).ps.viewangles[ROLL] = 0.0;
                let viewangles = (*activator_client).ps.viewangles;
                crate::g_client::SetClientViewAngle(ctx.world.entity_mut(activator_id), viewangles);
                ctx.world.entity_mut(activator_id).r.currentAngles[PITCH] = 0.0;
                ctx.world.entity_mut(activator_id).r.currentAngles[ROLL] = 0.0;
                let currentAngles = ctx.world.entity(activator_id).r.currentAngles;
                crate::g_utils::G_SetAngles(ctx.world.entity_mut(activator_id), currentAngles);
            }
            if ctx.world.entity(activator_id).health <= 0 {
                //if ( self->activator->s.number )
                {
                    //never free player
                    if ctx.world.entity(self_).count == 1 {
                        //in my hand, just drop them
                        if !activator_client.is_null() {
                            (*activator_client).ps.legsTimer = 0;
                            (*activator_client).ps.torsoTimer = 0;
                            //FIXME: ragdoll?
                        }
                    } else {
                        if !activator_client.is_null() {
                            (*activator_client).ps.eFlags |= EF_NODRAW;
                            //so his corpse doesn't drop out of me...
                        }
                        //G_FreeEntity( self->activator );
                    }
                }
            } else {
                // FLAG: gNPC_t has no accessor; deref stays raw.
                let activator_npc = ctx.world.entity(activator_id).NPC;
                if !activator_npc.is_null() {
                    //start thinking again
                    (*activator_npc).nextBStateThink = ctx.world.level.time;
                }
                //clear their anim and let them fall
                (*activator_client).ps.legsTimer = 0;
                (*activator_client).ps.torsoTimer = 0;
            }
        }
        let self_enemy = ctx.world.entity(self_).enemy;
        let self_activator = ctx.world.entity(self_).activator;
        if self_enemy == self_activator {
            ctx.world.entity_mut(self_).enemy = None;
        }
        ctx.world.entity_mut(self_).activator = None;
    }
    ctx.world.entity_mut(self_).count = 0; //drop him
}

/// Raven `Rancor_Swing`.
///
/// Source: `oracle/codemp/game/NPC_AI_Rancor.c:196-306`
pub fn Rancor_Swing(ctx: &mut GameContext, tryGrab: qboolean) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // FLAG: pool client, deref raw via safe entity borrow.
    let npc_client = ctx.world.entity(npc_id).client;
    let mut radiusEntNums: [c_int; 128] = [0; 128];
    let radius = 88.0;
    let radiusSquared = radius * radius;
    let mut boltOrg: vec3_t = [0.0; 3];

    let handRBolt = unsafe { (*npc_client).renderInfo.handRBolt };
    let numEnts = crate::NPC_utils::NPC_GetEntsNearBolt(
        ctx,
        radiusEntNums.as_mut_ptr(),
        radius,
        handRBolt,
        &mut boltOrg,
    );

    for i in 0..(numEnts as usize) {
        let radiusEnt_id = EntityId(radiusEntNums[i] as u32);
        if ctx.world.entity(radiusEnt_id).inuse == 0 {
            continue;
        }

        if radiusEnt_id == npc_id {
            //Skip the rancor ent
            continue;
        }

        // FLAG: pool client, deref raw via safe entity borrow.
        let radiusEnt_client = ctx.world.entity(radiusEnt_id).client;
        if radiusEnt_client.is_null() {
            //must be a client
            continue;
        }

        unsafe {
            if ((*radiusEnt_client).ps.eFlags2 & EF2_HELD_BY_MONSTER) != 0 {
                //can't be one already being held
                continue;
            }

            if DistanceSquared(ctx.world.entity(radiusEnt_id).r.currentOrigin, boltOrg)
                <= radiusSquared
            {
                if tryGrab != 0
                    && ctx.world.entity(npc_id).count != 1 //don't have one in hand or in mouth already - FIXME: allow one in hand and any number in mouth!
                    && (*radiusEnt_client).NPC_class != CLASS_RANCOR
                    && (*radiusEnt_client).NPC_class != CLASS_GALAKMECH
                    && (*radiusEnt_client).NPC_class != CLASS_ATST
                    && (*radiusEnt_client).NPC_class != CLASS_GONK
                    && (*radiusEnt_client).NPC_class != CLASS_R2D2
                    && (*radiusEnt_client).NPC_class != CLASS_R5D2
                    && (*radiusEnt_client).NPC_class != CLASS_MARK1
                    && (*radiusEnt_client).NPC_class != CLASS_MARK2
                    && (*radiusEnt_client).NPC_class != CLASS_MOUSE
                    && (*radiusEnt_client).NPC_class != CLASS_PROBE
                    && (*radiusEnt_client).NPC_class != CLASS_SEEKER
                    && (*radiusEnt_client).NPC_class != CLASS_REMOTE
                    && (*radiusEnt_client).NPC_class != CLASS_SENTRY
                    && (*radiusEnt_client).NPC_class != CLASS_INTERROGATOR
                    && (*radiusEnt_client).NPC_class != CLASS_VEHICLE
                {
                    //grab
                    if ctx.world.entity(npc_id).count == 2 {
                        //have one in my mouth, remove him
                        crate::g_timer::TIMER_Remove(ctx, Some(npc_id), c"clearGrabbed".as_ptr());
                        Rancor_DropVictim(ctx, npc_id);
                    }
                    ctx.world.entity_mut(npc_id).enemy = Some(radiusEnt_id); //make him my new best friend
                    (*radiusEnt_client).ps.eFlags2 |= EF2_HELD_BY_MONSTER;
                    //FIXME: this makes it so that the victim can't hit us with shots!  Just use activator or something
                    (*radiusEnt_client).ps.hasLookTarget = qtrue;
                    (*radiusEnt_client).ps.lookTarget = ctx.world.entity(npc_id).s.number;
                    ctx.world.entity_mut(npc_id).activator = Some(radiusEnt_id); //remember him
                    let attacking =
                        (*npc_client).ps.legsTimer + ctx.world.bg_state.rng.Q_irand(500, 2500);
                    ctx.world.entity_mut(npc_id).count = 1; //in my hand
                                                            //wait to attack
                    crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"attacking".as_ptr(), attacking);
                    let re_health = ctx.world.entity(radiusEnt_id).health;
                    let re_pain = ctx.world.entity(radiusEnt_id).pain;
                    if re_health > 0 && re_pain.is_some() {
                        //do pain on enemy
                        let radiusEnt_ptr =
                            &mut ctx.world.g_entities[radiusEnt_id.index()] as *mut gentity_t;
                        crate::ent_fn_enums::dispatch_pain(
                            ctx,
                            re_pain.unwrap(),
                            radiusEnt_ptr,
                            npc,
                            100,
                        );
                        //GEntity_PainFunc( radiusEnt, NPC, NPC, radiusEnt->r.currentOrigin, 0, MOD_CRUSH );
                    } else if !radiusEnt_client.is_null() {
                        (*radiusEnt_client).ps.forceHandExtend = HANDEXTEND_NONE as c_int;
                        (*radiusEnt_client).ps.forceHandExtendTime = 0;
                        crate::npc_c::NPC_SetAnim(
                            ctx,
                            radiusEnt_id,
                            SETANIM_BOTH,
                            BOTH_SWIM_IDLE1 as c_int,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        );
                    }
                } else {
                    //smack
                    let mut pushDir: vec3_t = [0.0; 3];
                    let mut angs: vec3_t = [0.0; 3];

                    let sound = G_SoundIndex(ctx, "sound/chars/rancor/swipehit.wav");
                    crate::g_utils::G_Sound(ctx, Some(radiusEnt_id), CHAN_AUTO, sound);
                    //actually push the enemy
                    /*
                    //VectorSubtract( radiusEnt->r.currentOrigin, boltOrg, pushDir );
                    VectorSubtract( radiusEnt->r.currentOrigin, NPC->r.currentOrigin, pushDir );
                    pushDir[2] = Q_flrand( 100, 200 );
                    VectorNormalize( pushDir );
                    */
                    crate::q_math::_VectorCopy((*npc_client).ps.viewangles, &mut angs);
                    angs[1] += ctx.world.bg_state.rng.flrand(25.0, 50.0);
                    angs[0] = ctx.world.bg_state.rng.flrand(-25.0, -15.0);
                    crate::q_math::AngleVectors(angs, Some(&mut pushDir), None, None);
                    if (*radiusEnt_client).NPC_class != CLASS_RANCOR
                        && (*radiusEnt_client).NPC_class != CLASS_ATST
                    {
                        let damage = ctx.world.bg_state.rng.Q_irand(25, 40);
                        let re_origin = ctx.world.entity(radiusEnt_id).r.currentOrigin;
                        crate::g_combat::G_Damage(
                            ctx,
                            Some(radiusEnt_id),
                            Some(npc_id),
                            Some(npc_id),
                            Some(&mut [0.0; 3]),
                            re_origin,
                            damage,
                            DAMAGE_NO_ARMOR | DAMAGE_NO_KNOCKBACK,
                            MOD_MELEE as c_int,
                        );
                        crate::g_utils::G_Throw(ctx, radiusEnt_id, pushDir, 250.0);
                        if ctx.world.entity(radiusEnt_id).health > 0 {
                            //do pain on enemy
                            crate::g_combat::G_Knockdown(ctx, Some(radiusEnt_id));
                            //, NPC, pushDir, 100, qtrue );
                        }
                    }
                }
            }
        }
    }
}

/// Raven `Rancor_Smash`.
///
/// Source: `oracle/codemp/game/NPC_AI_Rancor.c:308-367`
pub fn Rancor_Smash(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // FLAG: pool client, deref raw via safe entity borrow.
    let npc_client = ctx.world.entity(npc_id).client;
    let mut radiusEntNums: [c_int; 128] = [0; 128];
    let radius = 128.0;
    let halfRadSquared = ((radius / 2.0) * (radius / 2.0));
    let radiusSquared = radius * radius;
    let mut boltOrg: vec3_t = [0.0; 3];

    let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
    crate::NPC_senses::AddSoundEvent(ctx, Some(npc_id), npc_origin, 512.0, AEL_DANGER, qfalse); //, qtrue );

    let handLBolt = unsafe { (*npc_client).renderInfo.handLBolt };
    let numEnts = crate::NPC_utils::NPC_GetEntsNearBolt(
        ctx,
        radiusEntNums.as_mut_ptr(),
        radius,
        handLBolt,
        &mut boltOrg,
    );

    for i in 0..(numEnts as usize) {
        let radiusEnt_id = EntityId(radiusEntNums[i] as u32);
        if ctx.world.entity(radiusEnt_id).inuse == 0 {
            continue;
        }

        if radiusEnt_id == npc_id {
            //Skip the rancor ent
            continue;
        }

        // FLAG: pool client, deref raw via safe entity borrow.
        let radiusEnt_client = ctx.world.entity(radiusEnt_id).client;
        if radiusEnt_client.is_null() {
            //must be a client
            continue;
        }

        unsafe {
            if ((*radiusEnt_client).ps.eFlags2 & EF2_HELD_BY_MONSTER) != 0 {
                //can't be one being held
                continue;
            }

            let distSq = DistanceSquared(ctx.world.entity(radiusEnt_id).r.currentOrigin, boltOrg);
            if distSq <= radiusSquared {
                let sound = G_SoundIndex(ctx, "sound/chars/rancor/swipehit.wav");
                crate::g_utils::G_Sound(ctx, Some(radiusEnt_id), CHAN_AUTO, sound);
                if distSq < halfRadSquared {
                    let damage = ctx.world.bg_state.rng.Q_irand(10, 25);
                    let re_origin = ctx.world.entity(radiusEnt_id).r.currentOrigin;
                    //close enough to do damage, too
                    crate::g_combat::G_Damage(
                        ctx,
                        Some(radiusEnt_id),
                        Some(npc_id),
                        Some(npc_id),
                        Some(&mut [0.0; 3]),
                        re_origin,
                        damage,
                        DAMAGE_NO_ARMOR | DAMAGE_NO_KNOCKBACK,
                        MOD_MELEE as c_int,
                    );
                }
                if ctx.world.entity(radiusEnt_id).health > 0
                    && !radiusEnt_client.is_null()
                    && (*radiusEnt_client).NPC_class != CLASS_RANCOR
                    && (*radiusEnt_client).NPC_class != CLASS_ATST
                {
                    if distSq < halfRadSquared
                        || (*radiusEnt_client).ps.groundEntityNum != ENTITYNUM_NONE
                    {
                        //within range of my fist or withing ground-shaking range and not in the air
                        crate::g_combat::G_Knockdown(ctx, Some(radiusEnt_id));
                        //, NPC, vec3_origin, 100, qtrue );
                    }
                }
            }
        }
    }
}

/// Raven `Rancor_Bite`.
///
/// Source: `oracle/codemp/game/NPC_AI_Rancor.c:369-428`
pub fn Rancor_Bite(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // FLAG: pool client, deref raw via safe entity borrow.
    let npc_client = ctx.world.entity(npc_id).client;
    let mut radiusEntNums: [c_int; 128] = [0; 128];
    let radius = 100.0;
    let radiusSquared = radius * radius;
    let mut boltOrg: vec3_t = [0.0; 3];

    let crotchBolt = unsafe { (*npc_client).renderInfo.crotchBolt };
    let numEnts = crate::NPC_utils::NPC_GetEntsNearBolt(
        ctx,
        radiusEntNums.as_mut_ptr(),
        radius,
        crotchBolt,
        &mut boltOrg,
    ); //was gutBolt?

    for i in 0..(numEnts as usize) {
        let radiusEnt_id = EntityId(radiusEntNums[i] as u32);
        if ctx.world.entity(radiusEnt_id).inuse == 0 {
            continue;
        }

        if radiusEnt_id == npc_id {
            //Skip the rancor ent
            continue;
        }

        // FLAG: pool client, deref raw via safe entity borrow.
        let radiusEnt_client = ctx.world.entity(radiusEnt_id).client;
        if radiusEnt_client.is_null() {
            //must be a client
            continue;
        }

        unsafe {
            if ((*radiusEnt_client).ps.eFlags2 & EF2_HELD_BY_MONSTER) != 0 {
                //can't be one already being held
                continue;
            }

            if DistanceSquared(ctx.world.entity(radiusEnt_id).r.currentOrigin, boltOrg)
                <= radiusSquared
            {
                let damage = ctx.world.bg_state.rng.Q_irand(15, 30);
                let re_origin = ctx.world.entity(radiusEnt_id).r.currentOrigin;
                crate::g_combat::G_Damage(
                    ctx,
                    Some(radiusEnt_id),
                    Some(npc_id),
                    Some(npc_id),
                    Some(&mut [0.0; 3]),
                    re_origin,
                    damage,
                    DAMAGE_NO_ARMOR | DAMAGE_NO_KNOCKBACK,
                    MOD_MELEE as c_int,
                );
                if ctx.world.entity(radiusEnt_id).health <= 0 && !radiusEnt_client.is_null() {
                    //killed them, chance of dismembering
                    if ctx.world.bg_state.rng.Q_irand(0, 1) == 0 {
                        //bite something off
                        let hitLoc = ctx
                            .world
                            .bg_state
                            .rng
                            .Q_irand(G2_MODELPART_HEAD as c_int, G2_MODELPART_RLEG as c_int);
                        if hitLoc == G2_MODELPART_HEAD as c_int {
                            crate::npc_c::NPC_SetAnim(
                                ctx,
                                radiusEnt_id,
                                SETANIM_BOTH,
                                BOTH_DEATH17 as c_int,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            );
                        } else if hitLoc == G2_MODELPART_WAIST as c_int {
                            crate::npc_c::NPC_SetAnim(
                                ctx,
                                radiusEnt_id,
                                SETANIM_BOTH,
                                BOTH_DEATHBACKWARD2 as c_int,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            );
                        }
                        //radiusEnt->client->dismembered = qfalse;
                        //FIXME: the limb should just disappear, cuz I ate it
                        let re_origin2 = ctx.world.entity(radiusEnt_id).r.currentOrigin;
                        let re_torsoAnim = (*radiusEnt_client).ps.torsoAnim;
                        crate::g_combat::G_Dismember(
                            ctx,
                            radiusEnt_id,
                            Some(npc_id),
                            re_origin2,
                            hitLoc,
                            90.0,
                            0.0,
                            re_torsoAnim,
                            qtrue,
                        );
                        //G_DoDismemberment( radiusEnt, radiusEnt->r.currentOrigin, MOD_SABER, 1000, hitLoc, qtrue );
                    }
                }
                let sound = G_SoundIndex(ctx, "sound/chars/rancor/chomp.wav");
                crate::g_utils::G_Sound(ctx, Some(radiusEnt_id), CHAN_AUTO, sound);
            }
        }
    }
}

/// Raven `Rancor_Attack`.
///
/// Source: `oracle/codemp/game/NPC_AI_Rancor.c:431-614`
pub fn Rancor_Attack(ctx: &mut GameContext, distance: f32, doCharge: qboolean) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    let ent_base = ctx.world.g_entities.as_mut_ptr();
    // FLAG: pool client, deref raw via safe entity borrow.
    let npc_client = ctx.world.entity(npc_id).client;

    unsafe {
        if crate::g_timer::TIMER_Exists(ctx, Some(npc_id), c"attacking".as_ptr()) == qfalse {
            let count = ctx.world.entity(npc_id).count;
            let has_activator = ctx.world.entity(npc_id).activator.is_some();
            if count == 2 && has_activator {
            } else if count == 1 && has_activator {
                let activator_id = ctx.world.entity(npc_id).activator.unwrap();
                //holding enemy
                if ctx.world.entity(activator_id).health > 0
                    && ctx.world.bg_state.rng.Q_irand(0, 1) != 0
                {
                    //quick bite
                    crate::npc_c::NPC_SetAnim(
                        ctx,
                        npc_id,
                        SETANIM_BOTH,
                        BOTH_ATTACK1 as c_int,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                    crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"attack_dmg".as_ptr(), 450);
                } else {
                    //full eat
                    crate::npc_c::NPC_SetAnim(
                        ctx,
                        npc_id,
                        SETANIM_BOTH,
                        BOTH_ATTACK3 as c_int,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                    crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"attack_dmg".as_ptr(), 900);
                    //Make victim scream in fright
                    if ctx.world.entity(activator_id).health > 0
                        && !ctx.world.entity(activator_id).client.is_null()
                    {
                        let ev = ctx
                            .world
                            .bg_state
                            .rng
                            .Q_irand(EV_DEATH1 as c_int, EV_DEATH3 as c_int);
                        crate::g_utils::G_AddEvent(ctx.world.entity_mut(activator_id), ev, 0);
                        crate::npc_c::NPC_SetAnim(
                            ctx,
                            activator_id,
                            SETANIM_TORSO,
                            BOTH_FALLDEATH1 as c_int,
                            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                        );
                        // FLAG: gNPC_t has no accessor; deref stays raw.
                        let activator_npc = ctx.world.entity(activator_id).NPC;
                        if !activator_npc.is_null() {
                            //no more thinking for you
                            crate::g_combat::TossClientItems(ctx, npc_id);
                            (*activator_npc).nextBStateThink = Q3_INFINITE;
                        }
                    }
                }
            } else if
            // FLAG: enemy resolved raw to keep Raven's unconditional NPC->enemy->health deref.
            // This is the same null-deref UB path Raven has.
            (*crate::ent_id::resolve(ent_base, ctx.world.entity(npc_id).enemy)).health > 0
                && doCharge != 0
            {
                //charge
                let mut fwd: vec3_t = [0.0; 3];
                let mut yawAng: vec3_t = [0.0, (*npc_client).ps.viewangles[1], 0.0];
                crate::q_math::AngleVectors(yawAng, Some(&mut fwd), None, None);
                crate::q_math::_VectorScale(fwd, distance * 1.5, &mut (*npc_client).ps.velocity);
                (*npc_client).ps.velocity[2] = 150.0;
                (*npc_client).ps.groundEntityNum = ENTITYNUM_NONE;

                crate::npc_c::NPC_SetAnim(
                    ctx,
                    npc_id,
                    SETANIM_BOTH,
                    BOTH_MELEE2 as c_int,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"attack_dmg".as_ptr(), 1250);
            } else if ctx.world.bg_state.rng.Q_irand(0, 1) == 0 {
                //smash
                crate::npc_c::NPC_SetAnim(
                    ctx,
                    npc_id,
                    SETANIM_BOTH,
                    BOTH_MELEE1 as c_int,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"attack_dmg".as_ptr(), 1000);
            } else {
                //try to grab
                crate::npc_c::NPC_SetAnim(
                    ctx,
                    npc_id,
                    SETANIM_BOTH,
                    BOTH_ATTACK2 as c_int,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
                crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"attack_dmg".as_ptr(), 1000);
            }

            // Oracle computes this last, after the attack-type Q_irand draws and after NPC_SetAnim updated legsTimer.
            // Source: oracle/codemp/game/NPC_AI_Rancor.c:485
            let attacking = ((*npc_client).ps.legsTimer as f32
                + ctx.world.bg_state.rng.random() * 200.0) as c_int;
            crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"attacking".as_ptr(), attacking);
        }

        // Need to do delayed damage since the attack animations encapsulate multiple mini-attacks

        if crate::g_timer::TIMER_Done2(ctx, Some(npc_id), c"attack_dmg".as_ptr(), qtrue) != 0 {
            let mut shakePos: vec3_t = [0.0; 3];
            match (*npc_client).ps.legsAnim {
                _ if (*npc_client).ps.legsAnim == BOTH_MELEE1 as c_int => {
                    Rancor_Smash(ctx);
                    let handLBolt = (*npc_client).renderInfo.handLBolt;
                    crate::NPC_utils::G_GetBoltPosition(
                        ctx,
                        Some(npc_id),
                        handLBolt,
                        Some(&mut shakePos),
                        0,
                    );
                    crate::g_utils::G_ScreenShake(ctx, shakePos, None, 4.0, 1000, qfalse);
                    //CGCam_Shake( 1.0f*playerDist/128.0f, 1000 );
                }
                _ if (*npc_client).ps.legsAnim == BOTH_MELEE2 as c_int => {
                    Rancor_Bite(ctx);
                    crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"attack_dmg2".as_ptr(), 450);
                }
                _ if (*npc_client).ps.legsAnim == BOTH_ATTACK1 as c_int => {
                    if ctx.world.entity(npc_id).count == 1
                        && ctx.world.entity(npc_id).activator.is_some()
                    {
                        let activator_id = ctx.world.entity(npc_id).activator.unwrap();
                        let damage = ctx.world.bg_state.rng.Q_irand(25, 40);
                        let activator_origin = ctx.world.entity(activator_id).r.currentOrigin;
                        crate::g_combat::G_Damage(
                            ctx,
                            Some(activator_id),
                            Some(npc_id),
                            Some(npc_id),
                            Some(&mut [0.0; 3]),
                            activator_origin,
                            damage,
                            DAMAGE_NO_ARMOR | DAMAGE_NO_KNOCKBACK,
                            MOD_MELEE as c_int,
                        );
                        if ctx.world.entity(activator_id).health <= 0 {
                            //killed him
                            //make it look like we bit his head off
                            //NPC->activator->client->dismembered = qfalse;
                            // FLAG: pool client, deref raw via safe entity borrow.
                            let activator_client = ctx.world.entity(activator_id).client;
                            let activator_origin2 = ctx.world.entity(activator_id).r.currentOrigin;
                            let activator_torsoAnim = (*activator_client).ps.torsoAnim;
                            crate::g_combat::G_Dismember(
                                ctx,
                                activator_id,
                                Some(npc_id),
                                activator_origin2,
                                G2_MODELPART_HEAD as c_int,
                                90.0,
                                0.0,
                                activator_torsoAnim,
                                qtrue,
                            );
                            //G_DoDismemberment( NPC->activator, NPC->activator->r.currentOrigin, MOD_SABER, 1000, HL_HEAD, qtrue );
                            (*activator_client).ps.forceHandExtend = HANDEXTEND_NONE as c_int;
                            (*activator_client).ps.forceHandExtendTime = 0;
                            crate::npc_c::NPC_SetAnim(
                                ctx,
                                activator_id,
                                SETANIM_BOTH,
                                BOTH_SWIM_IDLE1 as c_int,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            );
                        }
                        let sound = G_SoundIndex(ctx, "sound/chars/rancor/chomp.wav");
                        crate::g_utils::G_Sound(ctx, Some(activator_id), CHAN_AUTO, sound);
                    }
                }
                _ if (*npc_client).ps.legsAnim == BOTH_ATTACK2 as c_int => {
                    //try to grab
                    Rancor_Swing(ctx, qtrue);
                }
                _ if (*npc_client).ps.legsAnim == BOTH_ATTACK3 as c_int => {
                    if ctx.world.entity(npc_id).count == 1
                        && ctx.world.entity(npc_id).activator.is_some()
                    {
                        let activator_id = ctx.world.entity(npc_id).activator.unwrap();
                        //cut in half
                        if !ctx.world.entity(activator_id).client.is_null() {
                            //NPC->activator->client->dismembered = qfalse;
                            // FLAG: pool client, deref raw via safe entity borrow.
                            let activator_client = ctx.world.entity(activator_id).client;
                            let activator_origin = ctx.world.entity(activator_id).r.currentOrigin;
                            let activator_torsoAnim = (*activator_client).ps.torsoAnim;
                            crate::g_combat::G_Dismember(
                                ctx,
                                activator_id,
                                Some(npc_id),
                                activator_origin,
                                G2_MODELPART_WAIST as c_int,
                                90.0,
                                0.0,
                                activator_torsoAnim,
                                qtrue,
                            );
                            //G_DoDismemberment( NPC->activator, NPC->enemy->r.currentOrigin, MOD_SABER, 1000, HL_WAIST, qtrue );
                        }
                        //KILL
                        let activator_origin2 = ctx.world.entity(activator_id).r.currentOrigin;
                        // FLAG: enemy resolved raw to keep Raven's unconditional NPC->enemy->health deref.
                        let enemy_health =
                            (*crate::ent_id::resolve(ent_base, ctx.world.entity(npc_id).enemy))
                                .health;
                        crate::g_combat::G_Damage(
                            ctx,
                            Some(activator_id),
                            Some(npc_id),
                            Some(npc_id),
                            Some(&mut [0.0; 3]),
                            activator_origin2,
                            enemy_health + 10,
                            DAMAGE_NO_PROTECTION
                                | DAMAGE_NO_ARMOR
                                | DAMAGE_NO_KNOCKBACK
                                | DAMAGE_NO_HIT_LOC,
                            MOD_MELEE as c_int,
                        ); //, HL_NONE );//
                        if !ctx.world.entity(activator_id).client.is_null() {
                            // FLAG: pool client, deref raw via safe entity borrow.
                            let activator_client = ctx.world.entity(activator_id).client;
                            (*activator_client).ps.forceHandExtend = HANDEXTEND_NONE as c_int;
                            (*activator_client).ps.forceHandExtendTime = 0;
                            crate::npc_c::NPC_SetAnim(
                                ctx,
                                activator_id,
                                SETANIM_BOTH,
                                BOTH_SWIM_IDLE1 as c_int,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            );
                        }
                        crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"attack_dmg2".as_ptr(), 1350);
                        let sound = G_SoundIndex(ctx, "sound/chars/rancor/swipehit.wav");
                        crate::g_utils::G_Sound(ctx, Some(activator_id), CHAN_AUTO, sound);
                        let activator_health = ctx.world.entity(activator_id).health;
                        crate::g_utils::G_AddEvent(
                            ctx.world.entity_mut(activator_id),
                            EV_JUMP as c_int,
                            activator_health,
                        );
                    }
                }
                _ => {}
            }
        } else if crate::g_timer::TIMER_Done2(ctx, Some(npc_id), c"attack_dmg2".as_ptr(), qtrue)
            != 0
        {
            match (*npc_client).ps.legsAnim {
                _ if (*npc_client).ps.legsAnim == BOTH_MELEE1 as c_int => {}
                _ if (*npc_client).ps.legsAnim == BOTH_MELEE2 as c_int => {
                    Rancor_Bite(ctx);
                }
                _ if (*npc_client).ps.legsAnim == BOTH_ATTACK1 as c_int => {}
                _ if (*npc_client).ps.legsAnim == BOTH_ATTACK2 as c_int => {}
                _ if (*npc_client).ps.legsAnim == BOTH_ATTACK3 as c_int => {
                    if ctx.world.entity(npc_id).count == 1
                        && ctx.world.entity(npc_id).activator.is_some()
                    {
                        let activator_id = ctx.world.entity(npc_id).activator.unwrap();
                        //swallow victim
                        let sound = G_SoundIndex(ctx, "sound/chars/rancor/chomp.wav");
                        crate::g_utils::G_Sound(ctx, Some(activator_id), CHAN_AUTO, sound);
                        //FIXME: sometimes end up with a live one in our mouths?
                        //just make sure they're dead
                        if ctx.world.entity(activator_id).health > 0 {
                            //cut in half
                            //NPC->activator->client->dismembered = qfalse;
                            // FLAG: pool client, deref raw via safe entity borrow.
                            let activator_client = ctx.world.entity(activator_id).client;
                            let activator_origin = ctx.world.entity(activator_id).r.currentOrigin;
                            let activator_torsoAnim = (*activator_client).ps.torsoAnim;
                            crate::g_combat::G_Dismember(
                                ctx,
                                activator_id,
                                Some(npc_id),
                                activator_origin,
                                G2_MODELPART_WAIST as c_int,
                                90.0,
                                0.0,
                                activator_torsoAnim,
                                qtrue,
                            );
                            //G_DoDismemberment( NPC->activator, NPC->enemy->r.currentOrigin, MOD_SABER, 1000, HL_WAIST, qtrue );
                            //KILL
                            let activator_origin2 = ctx.world.entity(activator_id).r.currentOrigin;
                            // FLAG: enemy resolved raw to keep Raven's unconditional NPC->enemy->health deref.
                            let enemy_health =
                                (*crate::ent_id::resolve(ent_base, ctx.world.entity(npc_id).enemy))
                                    .health;
                            crate::g_combat::G_Damage(
                                ctx,
                                Some(activator_id),
                                Some(npc_id),
                                Some(npc_id),
                                Some(&mut [0.0; 3]),
                                activator_origin2,
                                enemy_health + 10,
                                DAMAGE_NO_PROTECTION
                                    | DAMAGE_NO_ARMOR
                                    | DAMAGE_NO_KNOCKBACK
                                    | DAMAGE_NO_HIT_LOC,
                                MOD_MELEE as c_int,
                            ); //, HL_NONE );
                            (*activator_client).ps.forceHandExtend = HANDEXTEND_NONE as c_int;
                            (*activator_client).ps.forceHandExtendTime = 0;
                            crate::npc_c::NPC_SetAnim(
                                ctx,
                                activator_id,
                                SETANIM_BOTH,
                                BOTH_SWIM_IDLE1 as c_int,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            );
                            let activator_health = ctx.world.entity(activator_id).health;
                            crate::g_utils::G_AddEvent(
                                ctx.world.entity_mut(activator_id),
                                EV_JUMP as c_int,
                                activator_health,
                            );
                        }
                        if !ctx.world.entity(activator_id).client.is_null() {
                            //*sigh*, can't get tags right, just remove them?
                            // FLAG: pool client, deref raw via safe entity borrow.
                            let activator_client = ctx.world.entity(activator_id).client;
                            (*activator_client).ps.eFlags |= EF_NODRAW;
                        }
                        ctx.world.entity_mut(npc_id).count = 2;
                        crate::g_timer::TIMER_Set(
                            ctx,
                            Some(npc_id),
                            c"clearGrabbed".as_ptr(),
                            2600,
                        );
                    }
                }
                _ => {}
            }
        } else if (*npc_client).ps.legsAnim == BOTH_ATTACK2 as c_int {
            if (*npc_client).ps.legsTimer >= 1200 && (*npc_client).ps.legsTimer <= 1350 {
                if ctx.world.bg_state.rng.Q_irand(0, 2) != 0 {
                    Rancor_Swing(ctx, qfalse);
                } else {
                    Rancor_Swing(ctx, qtrue);
                }
            } else if (*npc_client).ps.legsTimer >= 1100 && (*npc_client).ps.legsTimer <= 1550 {
                Rancor_Swing(ctx, qtrue);
            }
        }

        // Just using this to remove the attacking flag at the right time
        crate::g_timer::TIMER_Done2(ctx, Some(npc_id), c"attacking".as_ptr(), qtrue);
    }
}

/// Raven `Rancor_Combat`.
///
/// Source: `oracle/codemp/game/NPC_AI_Rancor.c:617-695`
pub fn Rancor_Combat(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw.
    let npc_info = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(npc).unwrap();

    let enemy_id = ctx.world.entity(npc_id).enemy;
    if ctx.world.entity(npc_id).count != 0 {
        //holding my enemy
        if crate::g_timer::TIMER_Done2(ctx, Some(npc_id), c"takingPain".as_ptr(), qtrue) != 0 {
            unsafe {
                (*npc_info).localState = LSTATE_CLEAR;
            }
        } else {
            Rancor_Attack(ctx, 0.0, qfalse);
        }
        crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
        return;
    }
    // If we cannot see our target or we have somewhere to go, then do that
    if crate::NPC_utils::NPC_ClearLOS4(ctx, enemy_id) == qfalse {
        //|| UpdateGoal( ))
        let npc_enemy = ctx.world.entity(npc_id).enemy;
        unsafe {
            (*npc_info).combatMove = qtrue;
            (*npc_info).goalEntity = npc_enemy;
            (*npc_info).goalRadius = MIN_DISTANCE; //MAX_DISTANCE;	// just get us within combat range
        }

        if crate::NPC_move::NPC_MoveToGoal(ctx, qtrue) == qfalse {
            //couldn't go after him?  Look for a new one
            crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"lookForNewEnemy".as_ptr(), 0);
            unsafe {
                (*npc_info).consecutiveBlockedMoves += 1;
            }
        } else {
            unsafe {
                (*npc_info).consecutiveBlockedMoves = 0;
            }
        }
        return;
    }

    // Sometimes I have problems with facing the enemy I'm attacking, so force the issue so I don't look dumb
    crate::NPC_utils::NPC_FaceEnemy(ctx, qtrue);

    {
        let enemy_id2 = ctx.world.entity(npc_id).enemy.unwrap();
        let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
        let enemy_origin = ctx.world.entity(enemy_id2).r.currentOrigin;
        let distance = crate::q_math::Distance(npc_origin, enemy_origin);
        let npc_maxs0 = ctx.world.entity(npc_id).r.maxs[0];
        let mut advance = if distance > (npc_maxs0 + MIN_DISTANCE as f32) {
            qtrue
        } else {
            qfalse
        };
        let mut doCharge = qfalse;

        if advance != 0 {
            //have to get closer
            let npc_currentAngles1 = ctx.world.entity(npc_id).r.currentAngles[1];
            let yawOnlyAngles: vec3_t = [0.0, npc_currentAngles1, 0.0];
            if ctx.world.entity(enemy_id2).health > 0
                && (distance - 250.0).abs() <= 80.0
                && crate::NPC_senses::InFOV3(
                    ctx.world.entity(enemy_id2).r.currentOrigin,
                    ctx.world.entity(npc_id).r.currentOrigin,
                    yawOnlyAngles,
                    30,
                    30,
                ) != 0
            {
                if ctx.world.bg_state.rng.Q_irand(0, 9) == 0 {
                    //go for the charge
                    doCharge = qtrue;
                    advance = qfalse;
                }
            }
        }

        if advance != 0 && crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"attacking".as_ptr()) != 0
        {
            // waiting monsters can't attack
            if crate::g_timer::TIMER_Done2(ctx, Some(npc_id), c"takingPain".as_ptr(), qtrue) != 0 {
                unsafe {
                    (*npc_info).localState = LSTATE_CLEAR;
                }
            } else {
                Rancor_Move(ctx, 1);
            }
        } else {
            Rancor_Attack(ctx, distance, doCharge);
        }
    }
}

/// Raven `NPC_Rancor_Pain`.
///
/// Source: `oracle/codemp/game/NPC_AI_Rancor.c:703-782`
pub fn NPC_Rancor_Pain(
    ctx: &mut GameContext,
    self_: EntityId,
    attacker: Option<EntityId>,
    damage: c_int,
) {
    // FLAG: pool client, deref raw via safe entity borrow.
    let self_client = ctx.world.entity(self_).client;

    let mut hitByRancor = qfalse;
    if let Some(attacker_id) = attacker {
        // FLAG: pool client, deref raw via safe entity borrow.
        let attacker_client = ctx.world.entity(attacker_id).client;
        if !attacker_client.is_null() && unsafe { (*attacker_client).NPC_class } == CLASS_RANCOR {
            hitByRancor = qtrue;
        }
    }

    let self_enemy_id = ctx.world.entity(self_).enemy;
    if let Some(attacker_id) = attacker {
        if ctx.world.entity(attacker_id).inuse != 0
            && Some(attacker_id) != self_enemy_id
            && (ctx.world.entity(attacker_id).flags & FL_NOTARGET) == 0
        {
            if ctx.world.entity(self_).count == 0 {
                // Raven's short-circuit OR encoded as if/else so RNG stays lazy and
                // the `self->enemy` derefs are unreachable when enemy is None.
                let should_retarget = {
                    let attacker_s_number = ctx.world.entity(attacker_id).s.number;
                    if attacker_s_number == 0 && ctx.world.bg_state.rng.Q_irand(0, 3) == 0 {
                        true
                    } else if ctx.world.entity(self_).enemy.is_none() {
                        true
                    } else {
                        let self_enemy = ctx.world.entity(self_).enemy.unwrap();
                        if ctx.world.entity(self_enemy).health == 0 {
                            true
                        } else if {
                            // FLAG: pool client, deref raw via safe entity borrow.
                            let se_client = ctx.world.entity(self_enemy).client;
                            !se_client.is_null()
                                && unsafe { (*se_client).NPC_class } == CLASS_RANCOR
                        } {
                            true
                        } else {
                            // FLAG: gNPC_t has no accessor; deref stays raw.
                            let self_npc = ctx.world.entity(self_).NPC;
                            !self_npc.is_null()
                                && unsafe { (*self_npc).consecutiveBlockedMoves } >= 10
                                && {
                                    let a_origin = ctx.world.entity(attacker_id).r.currentOrigin;
                                    let s_origin = ctx.world.entity(self_).r.currentOrigin;
                                    let se_origin = ctx.world.entity(self_enemy).r.currentOrigin;
                                    DistanceSquared(a_origin, s_origin)
                                        < DistanceSquared(se_origin, s_origin)
                                }
                        }
                    }
                };
                if should_retarget {
                    //if my enemy is dead (or attacked by player) and I'm not still holding/eating someone, turn on the attacker
                    //FIXME: if can't nav to my enemy, take this guy if I can nav to him
                    crate::NPC_combat::G_SetEnemy(ctx, self_, Some(attacker_id));
                    let look_for_new_enemy = ctx.world.bg_state.rng.Q_irand(5000, 15000);
                    crate::g_timer::TIMER_Set(
                        ctx,
                        Some(self_),
                        c"lookForNewEnemy".as_ptr(),
                        look_for_new_enemy,
                    );
                    if hitByRancor != 0 {
                        let rancor_infight = ctx.world.bg_state.rng.Q_irand(2000, 5000);
                        //stay mad at this Rancor for 2-5 secs before looking for attacker enemies
                        crate::g_timer::TIMER_Set(
                            ctx,
                            Some(self_),
                            c"rancorInfight".as_ptr(),
                            rancor_infight,
                        );
                    }
                }
            }
        }
    }

    let self_count = ctx.world.entity(self_).count;
    let self_has_activator = ctx.world.entity(self_).activator.is_some();
    let rng_or = hitByRancor != 0
        || (self_count == 1 && self_has_activator && ctx.world.bg_state.rng.Q_irand(0, 4) == 0)
        || ctx.world.bg_state.rng.Q_irand(0, 200) < damage;
    if rng_or
        && unsafe { (*self_client).ps.legsAnim } != BOTH_STAND1TO2 as c_int
        && crate::g_timer::TIMER_Done(ctx, Some(self_), c"takingPain".as_ptr()) != 0
    {
        if Rancor_CheckRoar(ctx, self_) == qfalse {
            let legs_anim = unsafe { (*self_client).ps.legsAnim };
            if legs_anim != BOTH_MELEE1 as c_int
                && legs_anim != BOTH_MELEE2 as c_int
                && legs_anim != BOTH_ATTACK2 as c_int
            {
                //cant interrupt one of the big attack anims
                /*
                if ( self->count != 1
                    || attacker == self->activator
                    || (self->client->ps.legsAnim != BOTH_ATTACK1&&self->client->ps.legsAnim != BOTH_ATTACK3) )
                */
                {
                    //if going to bite our victim, only victim can interrupt that anim
                    if ctx.world.entity(self_).health > 100 || hitByRancor != 0 {
                        crate::g_timer::TIMER_Remove(ctx, Some(self_), c"attacking".as_ptr());

                        // FLAG: gNPC_t has no accessor; deref stays raw.
                        let self_npc = ctx.world.entity(self_).NPC;
                        let lastPathAngles = unsafe { (*self_npc).lastPathAngles };
                        crate::q_math::_VectorCopy(
                            lastPathAngles,
                            &mut ctx.world.entity_mut(self_).s.angles,
                        );

                        let taking_pain = unsafe { (*self_client).ps.legsTimer }
                            + ctx.world.bg_state.rng.Q_irand(0, 500);
                        if ctx.world.entity(self_).count == 1 {
                            crate::npc_c::NPC_SetAnim(
                                ctx,
                                self_,
                                SETANIM_BOTH,
                                BOTH_PAIN2 as c_int,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            );
                        } else {
                            crate::npc_c::NPC_SetAnim(
                                ctx,
                                self_,
                                SETANIM_BOTH,
                                BOTH_PAIN1 as c_int,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            );
                        }
                        crate::g_timer::TIMER_Set(
                            ctx,
                            Some(self_),
                            c"takingPain".as_ptr(),
                            taking_pain,
                        );

                        // FLAG: gNPC_t has no accessor; deref stays raw.
                        let self_npc2 = ctx.world.entity(self_).NPC;
                        if !self_npc2.is_null() {
                            unsafe {
                                (*self_npc2).localState = LSTATE_WAITING;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Raven `Rancor_CheckDropVictim`.
///
/// Source: `oracle/codemp/game/NPC_AI_Rancor.c:784-802`
pub fn Rancor_CheckDropVictim(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    let activator_id = ctx.world.entity(npc_id).activator.unwrap();

    let a = ctx.world.entity(activator_id);
    let mins: vec3_t = [a.r.mins[0] - 1.0, a.r.mins[1] - 1.0, 0.0];
    let maxs: vec3_t = [a.r.maxs[0] + 1.0, a.r.maxs[1] + 1.0, 1.0];
    let start: vec3_t = [a.r.currentOrigin[0], a.r.currentOrigin[1], a.r.absmin[2]];
    let end: vec3_t = [
        a.r.currentOrigin[0],
        a.r.currentOrigin[1],
        a.r.absmax[2] - 1.0,
    ];
    let a_s_number = a.s.number;
    let a_clipmask = a.clipmask;
    let mut trace: trace_t = unsafe { core::mem::zeroed() };

    trap::Trace(
        ctx.engine,
        mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
            &mut trace, &start, &mins, &maxs, &end, a_s_number, a_clipmask,
        ),
    );
    if trace.allsolid == 0 && trace.startsolid == 0 && trace.fraction >= 1.0 {
        Rancor_DropVictim(ctx, npc_id);
    }
}

/// Raven `Rancor_Crush`.
///
/// Source: `oracle/codemp/game/NPC_AI_Rancor.c:805-821`
pub fn Rancor_Crush(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;

    if npc.is_null() {
        //nothing to crush
        return;
    }
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // FLAG: pool client, deref raw via safe entity borrow.
    let npc_client = ctx.world.entity(npc_id).client;
    if npc_client.is_null() {
        //nothing to crush
        return;
    }
    let ground = unsafe { (*npc_client).ps.groundEntityNum };
    if ground >= ENTITYNUM_WORLD {
        //nothing to crush
        return;
    }

    let crush_id = EntityId(ground as u32);
    // FLAG: pool client, deref raw via safe entity borrow.
    let crush_client = ctx.world.entity(crush_id).client;
    if ctx.world.entity(crush_id).inuse != 0
        && !crush_client.is_null()
        && ctx.world.entity(crush_id).localAnimIndex == 0
    {
        //a humanoid, smash them good.
        let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
        crate::g_combat::G_Damage(
            ctx,
            Some(crush_id),
            Some(npc_id),
            Some(npc_id),
            None,
            npc_origin,
            200,
            0,
            MOD_CRUSH as c_int,
        );
    }
}

/// Raven `NPC_BSRancor_Default`.
///
/// Source: `oracle/codemp/game/NPC_AI_Rancor.c:828-955`
pub fn NPC_BSRancor_Default(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw.
    let npc_info = ctx.world.globals.NPCInfo;
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // FLAG: pool client, deref raw via safe entity borrow.
    let npc_client = ctx.world.entity(npc_id).client;

    let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
    crate::NPC_senses::AddSightEvent(
        ctx,
        Some(npc_id),
        npc_origin,
        1024.0,
        AEL_DANGER_GREAT,
        50.0,
    );

    Rancor_Crush(ctx);

    unsafe {
        (*npc_client).ps.eFlags2 &= !(EF2_USE_ALT_ANIM | EF2_GENERIC_NPC_FLAG);
        if ctx.world.entity(npc_id).count != 0 {
            //holding someone
            (*npc_client).ps.eFlags2 |= EF2_USE_ALT_ANIM;
            if ctx.world.entity(npc_id).count == 2 {
                //in my mouth
                (*npc_client).ps.eFlags2 |= EF2_GENERIC_NPC_FLAG;
            }
        } else {
            (*npc_client).ps.eFlags2 &= !(EF2_USE_ALT_ANIM | EF2_GENERIC_NPC_FLAG);
        }
    }

    if crate::g_timer::TIMER_Done2(ctx, Some(npc_id), c"clearGrabbed".as_ptr(), qtrue) != 0 {
        Rancor_DropVictim(ctx, npc_id);
    } else if unsafe { (*npc_client).ps.legsAnim } == BOTH_PAIN2 as c_int
        && ctx.world.entity(npc_id).count == 1
        && ctx.world.entity(npc_id).activator.is_some()
    {
        if ctx.world.bg_state.rng.Q_irand(0, 3) == 0 {
            Rancor_CheckDropVictim(ctx);
        }
    }
    if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"rageTime".as_ptr()) == 0 {
        //do nothing but roar first time we see an enemy
        let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
        crate::NPC_senses::AddSoundEvent(
            ctx,
            Some(npc_id),
            npc_origin,
            1024.0,
            AEL_DANGER_GREAT,
            qfalse,
        ); //, qfalse );
        crate::NPC_utils::NPC_FaceEnemy(ctx, qtrue);
        return;
    }
    if ctx.world.entity(npc_id).enemy.is_some() {
        /*
        if ( NPC->enemy->client //enemy is a client
            && (NPC->enemy->client->NPC_class == CLASS_UGNAUGHT || NPC->enemy->client->NPC_class == CLASS_JAWA )//enemy is a lowly jawa or ugnaught
            && NPC->enemy->enemy != NPC//enemy's enemy is not me
            && (!NPC->enemy->enemy || !NPC->enemy->enemy->client || NPC->enemy->enemy->client->NPC_class!=CLASS_RANCOR) )//enemy's enemy is not a client or is not a rancor (which is as scary as me anyway)
        {//they should be scared of ME and no-one else
            G_SetEnemy( NPC->enemy, NPC );
        }
        */
        if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"angrynoise".as_ptr()) != 0 {
            let anger_snd = format!(
                "sound/chars/rancor/misc/anger{}.wav",
                ctx.world.bg_state.rng.Q_irand(1, 3)
            );
            let sound_index = G_SoundIndex(ctx, &anger_snd);
            crate::g_utils::G_Sound(ctx, Some(npc_id), CHAN_AUTO, sound_index);

            let angrynoise = ctx.world.bg_state.rng.Q_irand(5000, 10000);
            crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"angrynoise".as_ptr(), angrynoise);
        } else {
            let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
            crate::NPC_senses::AddSoundEvent(
                ctx,
                Some(npc_id),
                npc_origin,
                512.0,
                AEL_DANGER_GREAT,
                qfalse,
            ); //, qfalse );
        }
        if ctx.world.entity(npc_id).count == 2
            && unsafe { (*npc_client).ps.legsAnim } == BOTH_ATTACK3 as c_int
        {
            //we're still chewing our enemy up
            crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
            return;
        }
        //else, if he's in our hand, we eat, else if he's on the ground, we keep attacking his dead body for a while
        let npc_enemy_id = ctx.world.entity(npc_id).enemy.unwrap();
        // FLAG: pool client, deref raw via safe entity borrow.
        let npc_enemy_client = ctx.world.entity(npc_enemy_id).client;
        if !npc_enemy_client.is_null() && unsafe { (*npc_enemy_client).NPC_class } == CLASS_RANCOR {
            //got mad at another Rancor, look for a valid enemy
            if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"rancorInfight".as_ptr()) != 0 {
                crate::NPC_utils::NPC_CheckEnemyExt(ctx, qtrue);
            }
        } else if ctx.world.entity(npc_id).count == 0 {
            let enemy_id = ctx.world.entity(npc_id).enemy;
            if crate::NPC_combat::ValidEnemy(ctx, enemy_id) == qfalse {
                crate::g_timer::TIMER_Remove(ctx, Some(npc_id), c"lookForNewEnemy".as_ptr()); //make them look again right now
                let npc_enemy_inuse = ctx.world.entity(npc_enemy_id).inuse;
                let npc_enemy_stime = ctx.world.entity(npc_enemy_id).s.time;
                if npc_enemy_inuse == 0
                    || ctx.world.level.time - npc_enemy_stime
                        > ctx.world.bg_state.rng.Q_irand(10000, 15000)
                {
                    //it's been a while since the enemy died, or enemy is completely gone, get bored with him
                    ctx.world.entity_mut(npc_id).enemy = None;
                    Rancor_Patrol(ctx);
                    crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
                    return;
                }
            }
            if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"lookForNewEnemy".as_ptr()) != 0 {
                let sav_enemy = ctx.world.entity(npc_id).enemy;
                ctx.world.entity_mut(npc_id).enemy = None;
                // FLAG: gNPC_t has no accessor; deref stays raw.
                let check_all =
                    (unsafe { (*npc_info).confusionTime } < ctx.world.level.time) as c_int;
                let newEnemy = crate::NPC_combat::NPC_CheckEnemy(ctx, check_all, qfalse, qfalse);
                ctx.world.entity_mut(npc_id).enemy = sav_enemy;
                let newEnemy_id = ctx.entity_id_of(newEnemy);
                if !newEnemy.is_null() && newEnemy_id != sav_enemy {
                    //picked up a new enemy!
                    let npc_enemy = ctx.world.entity(npc_id).enemy;
                    ctx.world.entity_mut(npc_id).lastEnemy = npc_enemy;
                    crate::NPC_combat::G_SetEnemy(ctx, npc_id, newEnemy_id);
                    let look_for_new_enemy = ctx.world.bg_state.rng.Q_irand(5000, 15000);
                    //hold this one for at least 5-15 seconds
                    crate::g_timer::TIMER_Set(
                        ctx,
                        Some(npc_id),
                        c"lookForNewEnemy".as_ptr(),
                        look_for_new_enemy,
                    );
                } else {
                    let look_for_new_enemy = ctx.world.bg_state.rng.Q_irand(2000, 5000);
                    //look again in 2-5 secs
                    crate::g_timer::TIMER_Set(
                        ctx,
                        Some(npc_id),
                        c"lookForNewEnemy".as_ptr(),
                        look_for_new_enemy,
                    );
                }
            }
        }
        Rancor_Combat(ctx);
    } else {
        if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"idlenoise".as_ptr()) != 0 {
            let snort_snd = format!(
                "sound/chars/rancor/snort_{}.wav",
                ctx.world.bg_state.rng.Q_irand(1, 2)
            );
            let sound_index = G_SoundIndex(ctx, &snort_snd);
            crate::g_utils::G_Sound(ctx, Some(npc_id), CHAN_AUTO, sound_index);

            let idlenoise = ctx.world.bg_state.rng.Q_irand(2000, 4000);
            crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"idlenoise".as_ptr(), idlenoise);
            let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
            crate::NPC_senses::AddSoundEvent(
                ctx,
                Some(npc_id),
                npc_origin,
                384.0,
                AEL_DANGER,
                qfalse,
            ); //, qfalse );
        }
        if (unsafe { (*npc_info).scriptFlags } & SCF_LOOK_FOR_ENEMIES) != 0 {
            Rancor_Patrol(ctx);
        } else {
            Rancor_Idle(ctx);
        }
    }

    crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
}
