// PORT-COMPLETE: NPC_AI_Rancor.c 4/12
//! FAITHFUL port of `oracle/oracle/codemp/game/NPC_AI_Rancor.c`.
//!
//! Landed from the `fnskel.py` signature skeleton. 4 functions are transcribed
//! faithfully from packet + prelude alone; the remaining 12 are parked (see
//! the `PORT-ESCALATION` topics below), matching the precedent set in
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

/// Raven `DistanceSquared` (`static ID_INLINE`, header-inline helper; ported
/// inline here per the ruling — plain-C branch only, `_XBOX` asm branch
/// skipped).
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:1527-1532`
fn DistanceSquared(p1: vec3_t, p2: vec3_t) -> f32 {
    let v = [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]];
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

// PORT-ESCALATION(ambient-state): calls `trap_G2API_AddBolt` (needs &Engine);
// the faithful context-free signature carries no engine handle to reach the
// trap seam through.
/// Raven `Rancor_SetBolts`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:19-29`
pub fn Rancor_SetBolts(self_: *mut gentity_t) {
    todo!("Port Rancor_SetBolts — parked: ambient-state")
}

/// Raven `NPC_Rancor_Precache`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:36-45`
pub fn NPC_Rancor_Precache() {
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

// PORT-ESCALATION(ambient-state): reads/writes the `NPC`/`NPCInfo`/`ucmd`
// ambient globals; no channel from this context-free faithful signature.
/// Raven `Rancor_Idle`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:53-63`
pub fn Rancor_Idle() {
    todo!("Port Rancor_Idle — parked: ambient-state")
}

/// Raven `Rancor_CheckRoar`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:66-77`
pub fn Rancor_CheckRoar(self_: *mut gentity_t) -> qboolean {
    // Raven `BOTH_STAND1TO2` / `SETANIM_BOTH` / `SETANIM_FLAG_OVERRIDE` /
    // `SETANIM_FLAG_HOLD` (`bg_public.h`/`anims.h`); no landed const in this
    // crate yet, so the literal values are transcribed at the call site per
    // house style for resolved-elsewhere anim/flag constants.
    //TODO: Port BOTH_STAND1TO2
    // Source: oracle/oracle/codemp/game/anims.h
    const BOTH_STAND1TO2: c_int = 0;
    const SETANIM_BOTH: c_int = 2;
    const SETANIM_FLAG_OVERRIDE: c_int = 1;
    const SETANIM_FLAG_HOLD: c_int = 2;
    // Raven `EF2_ALERTED` (`bg_public.h`); no landed const in this crate yet.
    //TODO: Port EF2_ALERTED
    // Source: oracle/oracle/codemp/game/bg_public.h
    const EF2_ALERTED: c_int = 0x00000002;

    unsafe {
        if (*self_).wait == 0.0 {
            //haven't ever gotten mad yet
            (*self_).wait = 1.0; //do this only once
            let client = (*self_).client as *mut gclient_t;
            (*client).ps.eFlags2 |= EF2_ALERTED;
            crate::npc_c::NPC_SetAnim(
                self_,
                SETANIM_BOTH,
                BOTH_STAND1TO2,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );
            let legs_timer = (*client).ps.legsTimer;
            crate::g_timer::TIMER_Set(self_, c"rageTime".as_ptr(), legs_timer);
            return QTRUE;
        }
        QFALSE
    }
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC`/`NPCInfo`/`ucmd`
// ambient globals and calls `crandom`/timer helpers keyed off the ambient
// `NPC`; no channel from this context-free faithful signature.
/// Raven `Rancor_Patrol`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:83-108`
pub fn Rancor_Patrol() {
    todo!("Port Rancor_Patrol — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC`/`NPCInfo` ambient
// globals; no channel from this context-free faithful signature.
/// Raven `Rancor_Move`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:115-130`
pub fn Rancor_Move(visible: qboolean) {
    todo!("Port Rancor_Move — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads the `level.time` ambient global; no
// channel from this context-free faithful signature.
/// Raven `Rancor_DropVictim`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:140-194`
pub fn Rancor_DropVictim(self_: *mut gentity_t) {
    todo!("Port Rancor_DropVictim — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC` ambient global
// (`Rancor_Swing` operates on the file-scope `NPC` pointer, not a parameter);
// no channel from this context-free faithful signature.
/// Raven `Rancor_Swing`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:196-306`
pub fn Rancor_Swing(tryGrab: qboolean) {
    todo!("Port Rancor_Swing — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads the `NPC` ambient global; no channel
// from this context-free faithful signature.
/// Raven `Rancor_Smash`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:308-367`
pub fn Rancor_Smash() {
    todo!("Port Rancor_Smash — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads the `NPC` ambient global; no channel
// from this context-free faithful signature.
/// Raven `Rancor_Bite`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:369-428`
pub fn Rancor_Bite() {
    todo!("Port Rancor_Bite — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC` ambient global; no
// channel from this context-free faithful signature.
/// Raven `Rancor_Attack`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:431-614`
pub fn Rancor_Attack(distance: f32, doCharge: qboolean) {
    todo!("Port Rancor_Attack — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC`/`NPCInfo` ambient
// globals; no channel from this context-free faithful signature.
/// Raven `Rancor_Combat`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:617-695`
pub fn Rancor_Combat() {
    todo!("Port Rancor_Combat — parked: ambient-state")
}

/// Raven `NPC_Rancor_Pain`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:703-782`
pub fn NPC_Rancor_Pain(self_: *mut gentity_t, attacker: *mut gentity_t, damage: c_int) {
    // Raven anim/flag consts (`bg_public.h`/`anims.h`); no landed const in this
    // crate yet, so the literal values are transcribed at the call site per
    // house style for resolved-elsewhere anim/flag constants.
    //TODO: Port BOTH_STAND1TO2, BOTH_MELEE1, BOTH_MELEE2, BOTH_ATTACK2, BOTH_PAIN1, BOTH_PAIN2
    // Source: oracle/oracle/codemp/game/anims.h
    const BOTH_STAND1TO2: c_int = 0;
    const BOTH_MELEE1: c_int = 1;
    const BOTH_MELEE2: c_int = 2;
    const BOTH_ATTACK2: c_int = 3;
    const BOTH_PAIN1: c_int = 4;
    const BOTH_PAIN2: c_int = 5;
    const SETANIM_BOTH: c_int = 2;
    const SETANIM_FLAG_OVERRIDE: c_int = 1;
    const SETANIM_FLAG_HOLD: c_int = 2;
    //TODO: Port CLASS_RANCOR
    // Source: oracle/oracle/codemp/game/g_local.h (class_t already landed as
    // `crate::teams::class::class_t`; using the landed enum value here).
    use crate::teams::class::class_t::CLASS_RANCOR;
    //TODO: Port FL_NOTARGET
    use crate::entity::flags::FL_NOTARGET;

    unsafe {
        let mut hit_by_rancor = QFALSE;
        if !attacker.is_null()
            && !(*attacker).client.is_null()
            && (*((*attacker).client as *mut gclient_t)).NPC_class == CLASS_RANCOR
        {
            hit_by_rancor = QTRUE;
        }

        if !attacker.is_null()
            && (*attacker).inuse != QFALSE
            && attacker != (*self_).enemy
            && ((*attacker).flags & FL_NOTARGET) == 0
        {
            if (*self_).count == 0 {
                let enemy = (*self_).enemy;
                let self_npc = (*self_).NPC as *mut gNPC_t;
                let take_attacker = ((*attacker).s.number == 0 && crate::q_math::Q_irand(0, 3) == 0)
                    || enemy.is_null()
                    || (!enemy.is_null() && (*enemy).health == 0)
                    || (!enemy.is_null()
                        && !(*enemy).client.is_null()
                        && (*((*enemy).client as *mut gclient_t)).NPC_class == CLASS_RANCOR)
                    || (!self_npc.is_null()
                        && (*self_npc).consecutiveBlockedMoves >= 10
                        && !enemy.is_null()
                        && DistanceSquared(
                            (*attacker).r.currentOrigin,
                            (*self_).r.currentOrigin,
                        ) < DistanceSquared(
                            (*enemy).r.currentOrigin,
                            (*self_).r.currentOrigin,
                        ));
                if take_attacker {
                    //if my enemy is dead (or attacked by player) and I'm not
                    //still holding/eating someone, turn on the attacker
                    //FIXME: if can't nav to my enemy, take this guy if I can
                    //nav to him
                    crate::NPC_combat::G_SetEnemy(self_, attacker);
                    crate::g_timer::TIMER_Set(
                        self_,
                        c"lookForNewEnemy".as_ptr(),
                        crate::q_math::Q_irand(5000, 15000),
                    );
                    if hit_by_rancor != QFALSE {
                        //stay mad at this Rancor for 2-5 secs before looking
                        //for attacker enemies
                        crate::g_timer::TIMER_Set(
                            self_,
                            c"rancorInfight".as_ptr(),
                            crate::q_math::Q_irand(2000, 5000),
                        );
                    }
                }
            }
        }

        let client = (*self_).client as *mut gclient_t;
        //hit by rancor, hit while holding live victim, or took a lot of damage
        if (hit_by_rancor != QFALSE
            || ((*self_).count == 1 && !(*self_).activator.is_null() && crate::q_math::Q_irand(0, 4) == 0)
            || crate::q_math::Q_irand(0, 200) < damage)
            && (*client).ps.legsAnim != BOTH_STAND1TO2
            && crate::g_timer::TIMER_Done(self_, c"takingPain".as_ptr()) != QFALSE
        {
            if Rancor_CheckRoar(self_) == QFALSE {
                if (*client).ps.legsAnim != BOTH_MELEE1
                    && (*client).ps.legsAnim != BOTH_MELEE2
                    && (*client).ps.legsAnim != BOTH_ATTACK2
                {
                    //cant interrupt one of the big attack anims
                    //if going to bite our victim, only victim can interrupt that anim
                    if (*self_).health > 100 || hit_by_rancor != QFALSE {
                        crate::g_timer::TIMER_Remove(self_, c"attacking".as_ptr());

                        let self_npc = (*self_).NPC as *mut gNPC_t;
                        (*self_).s.angles = (*self_npc).lastPathAngles;

                        if (*self_).count == 1 {
                            crate::npc_c::NPC_SetAnim(
                                self_,
                                SETANIM_BOTH,
                                BOTH_PAIN2,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            );
                        } else {
                            crate::npc_c::NPC_SetAnim(
                                self_,
                                SETANIM_BOTH,
                                BOTH_PAIN1,
                                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                            );
                        }
                        let legs_timer = (*client).ps.legsTimer;
                        crate::g_timer::TIMER_Set(
                            self_,
                            c"takingPain".as_ptr(),
                            legs_timer + crate::q_math::Q_irand(0, 500),
                        );

                        if !self_npc.is_null() {
                            (*self_npc).localState = LSTATE_WAITING;
                        }
                    }
                }
            }
        }
    }
}

// PORT-ESCALATION(ambient-state): reads the `NPC` ambient global and calls
// `trap_Trace` (needs &Engine); no channel from this context-free faithful
// signature.
/// Raven `Rancor_CheckDropVictim`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:784-802`
pub fn Rancor_CheckDropVictim() {
    todo!("Port Rancor_CheckDropVictim — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads the `NPC`/`g_entities` ambient
// globals; no channel from this context-free faithful signature.
/// Raven `Rancor_Crush`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:805-821`
pub fn Rancor_Crush() {
    todo!("Port Rancor_Crush — parked: ambient-state")
}

// PORT-ESCALATION(ambient-state): reads/writes the `NPC`/`NPCInfo`/`level`
// ambient globals; no channel from this context-free faithful signature.
/// Raven `NPC_BSRancor_Default`.
///
/// Source: `oracle/oracle/codemp/game/NPC_AI_Rancor.c:828-955`
pub fn NPC_BSRancor_Default() {
    todo!("Port NPC_BSRancor_Default — parked: ambient-state")
}
