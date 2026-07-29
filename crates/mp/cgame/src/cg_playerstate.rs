//! Port of `oracle/codemp/cgame/cg_playerstate.c` — local player-state transitions and reward/damage feedback. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_bg::public::gametype::{GT_CTF, GT_DUEL, GT_POWERDUEL, GT_SIEGE};
use mp_bg::public::pers_enum::persEnum_t::{PERS_HITS, PERS_SPAWN_COUNT, PERS_TEAM};
use mp_bg::public::pmtype::pmtype_t::PM_INTERMISSION;
use mp_bg::public::stat_index::statIndex_t::STAT_HEALTH;
use mp_bg::public::team::TEAM_SPECTATOR;
use mp_qshared::common::mp::qcommon::player_state::{playerState_t, MAX_PS_EVENTS};
use mp_qshared::shared::q_math::{
    _DotProduct, _VectorSubtract, vec3_origin, AngleVectors, VectorLength, PITCH, ROLL, YAW,
};
use mp_qshared::shared::sound_channel::CHAN_ANNOUNCER;
use mp_qshared::shared::{qtrue, vec3_t};
use mp_uishared::shared::display_state::DisplayState;

use crate::cg_event::{CG_EntityEvent, CG_PainEvent};
use crate::cg_main::CG_Printf;
use crate::cg_view::{CG_AddBufferedSound, DAMAGE_TIME};
use crate::local::cg_t::MAX_PREDICTED_EVENTS;
use crate::local::player_state_ref::PlayerStateRef;
use crate::trap;
use crate::world::cg_context::CgContext;
use crate::world::cg_world::CgWorld;

/// Raven `CG_CheckAmmo` — the low-ammo warning HUD/sound feature.
///
/// The whole body is wrapped in `#if 0 ... #endif`; the retail build shipped
/// with this switched off, so nothing here is reachable — only Raven's own
/// trailing comment survives.
///
/// Source: `oracle/codemp/cgame/cg_playerstate.c:17-78`
pub fn CG_CheckAmmo() {
    //disabled silly ammo warning stuff for now
}

/// Raven `CG_DamageFeedback` — turns a directional damage event into the
/// screen-flash/view-kick feedback (`cg.damageX`/`damageY`/`v_dmg_*`).
///
/// Source: `oracle/codemp/cgame/cg_playerstate.c:85-187`
pub fn CG_DamageFeedback(world: &mut CgWorld, yawByte: c_int, pitchByte: c_int, damage: c_int) {
    // show the attacking player's head and name in corner
    world.cg.attackerTime = world.cg.time;

    // §F19: Raven reads `cg.snap->ps` with no null check - before the first
    // snapshot that's a null deref, so the port takes the neutral early-out and
    // feeds nothing back. `cg.attackerTime` above is already set, as in Raven.
    let Some(snap) = world.cg.snap_ref() else {
        return;
    };
    // the lower on health you are, the greater the view kick will be
    let health = snap.ps.stats[STAT_HEALTH as usize];
    let snapServerTime = snap.serverTime;

    let scale: f32 = if health < 40 {
        1.0
    } else {
        // Raven's `40.0` is a double literal, so the divide happens in double
        (40.0f64 / f64::from(health)) as f32
    };
    let mut kick = damage as f32 * scale;

    if kick < 5.0 {
        kick = 5.0;
    }
    if kick > 10.0 {
        kick = 10.0;
    }

    // if yaw and pitch are both 255, make the damage always centered (falling, etc)
    if yawByte == 255 && pitchByte == 255 {
        world.cg.damageX = 0.0;
        world.cg.damageY = 0.0;
        world.cg.v_dmg_roll = 0.0;
        world.cg.v_dmg_pitch = -kick;
    } else {
        // positional
        let pitch = (f64::from(pitchByte) / 255.0 * 360.0) as f32;
        let yaw = (f64::from(yawByte) / 255.0 * 360.0) as f32;

        let mut angles: vec3_t = [0.0; 3];
        angles[PITCH] = pitch;
        angles[YAW] = yaw;
        angles[ROLL] = 0.0;

        let mut dir: vec3_t = [0.0; 3];
        AngleVectors(angles, Some(&mut dir), None, None);
        let toUs = dir;
        _VectorSubtract(vec3_origin, toUs, &mut dir);

        let mut front = _DotProduct(dir, world.cg.refdef.viewaxis[0]);
        let left = _DotProduct(dir, world.cg.refdef.viewaxis[1]);
        let up = _DotProduct(dir, world.cg.refdef.viewaxis[2]);

        dir[0] = front;
        dir[1] = left;
        dir[2] = 0.0;
        let mut dist = VectorLength(dir);
        if dist < 0.1 {
            dist = 0.1;
        }

        world.cg.v_dmg_roll = kick * left;

        world.cg.v_dmg_pitch = -kick * front;

        if front <= 0.1 {
            front = 0.1;
        }
        world.cg.damageX = -left / front;
        world.cg.damageY = up / dist;
    }

    // clamp the position
    if world.cg.damageX > 1.0 {
        world.cg.damageX = 1.0;
    }
    if world.cg.damageX < -1.0 {
        world.cg.damageX = -1.0;
    }

    if world.cg.damageY > 1.0 {
        world.cg.damageY = 1.0;
    }
    if world.cg.damageY < -1.0 {
        world.cg.damageY = -1.0;
    }

    // don't let the screen flashes vary as much
    if kick > 10.0 {
        kick = 10.0;
    }
    world.cg.damageValue = kick;
    world.cg.v_dmg_time = (world.cg.time + DAMAGE_TIME) as f32;
    world.cg.damageTime = snapServerTime as f32;

    // Raven's `//JLFRUMBLE` rumble tail is `#ifdef _XBOX`, so the PC build ends here.
}

/// Raven `CG_Respawn` — called on every local-player respawn: turns off error
/// decay for this frame's movement, opens the weapon-select HUD, and re-syncs
/// the selected weapon to whatever the server says we're carrying.
///
/// Source: `oracle/codemp/cgame/cg_playerstate.c:199-208`
pub fn CG_Respawn(world: &mut CgWorld) {
    // no error decay on player movement
    world.cg.thisFrameTeleport = qtrue;

    // display weapons available
    world.cg.weaponSelectTime = world.cg.time;

    // select the weapon the server says we are using
    // §F19: Raven derefs `cg.snap` here with no guard - with no snapshot yet the
    // selection keeps its prior value rather than reading through a null.
    let Some(weapon) = world.cg.snap_ref().map(|snap| snap.ps.weapon) else {
        return;
    };
    world.cg.weaponSelect = weapon;
}

/// Raven `CG_CheckLocalSounds` — fires local pain/reward/announcer sounds off
/// the delta between this frame's and the previous frame's `playerState_t`.
///
/// The "hit changes" block's own trap calls are all commented out in Raven
/// (retail shipped silent), so the `armor`/`health` bit-unpack that fed them
/// is a pure dead store here - dropped. `JK2AWARDS` is never defined for the
/// MP build (no `#define JK2AWARDS` anywhere in oracle), so the whole reward
/// block is dead too; Raven's own `#else reward = qfalse;` is what actually
/// ran, which is what this keeps.
///
/// Source: `oracle/codemp/cgame/cg_playerstate.c:311-487`
pub fn CG_CheckLocalSounds(ctx: &mut CgContext, ps: &playerState_t, ops: &playerState_t) {
    // don't play the sounds if the player just changed teams
    if ps.persistant[PERS_TEAM as usize] != ops.persistant[PERS_TEAM as usize] {
        return;
    }

    // hit changes - Raven's own trap_S_StartLocalSound calls here are all
    // commented out in retail, so there's nothing left to port but the
    // player-hit/team-hit branch labels themselves.
    if ps.persistant[PERS_HITS as usize] > ops.persistant[PERS_HITS as usize] {
        // hit an enemy - shield-pierced vs clean-hit sound was already disabled
    } else if ps.persistant[PERS_HITS as usize] < ops.persistant[PERS_HITS as usize] {
        // hit a teammate - team-hit sound was already disabled
    }

    // health changes of more than -3 should make pain sounds
    if ctx.world.cvars.cg_oldPainSounds.integer != 0
        && ps.stats[STAT_HEALTH as usize] < ops.stats[STAT_HEALTH as usize] - 3
        && ps.stats[STAT_HEALTH as usize] > 0
    {
        let clientNum = ctx.world.cg.predictedPlayerState.clientNum as usize;
        CG_PainEvent(ctx, clientNum, ps.stats[STAT_HEALTH as usize]);
    }

    // if we are going into the intermission, don't start any voices
    if ctx.world.cg.intermissionStarted != 0 {
        return;
    }

    // JK2AWARDS is never defined for this build - Raven's own `#else reward =
    // qfalse;` ran, and the lead-change block that reads it is fully
    // commented out (`/* ... */`) besides, so `reward` itself is a dead store.

    // timelimit warnings
    if ctx.world.cgs.timelimit > 0 && ctx.world.playerstate.cgAnnouncerTime < ctx.world.cg.time {
        let msec = ctx.world.cg.time - ctx.world.cgs.levelStartTime;
        if (ctx.world.cg.timelimitWarnings & 4) == 0
            && msec > (ctx.world.cgs.timelimit * 60 + 2) * 1000
        {
            ctx.world.cg.timelimitWarnings |= 1 | 2 | 4;
            // sudden-death sound is disabled in Raven too (commented out)
        } else if (ctx.world.cg.timelimitWarnings & 2) == 0
            && msec > (ctx.world.cgs.timelimit - 1) * 60 * 1000
        {
            ctx.world.cg.timelimitWarnings |= 1 | 2;
            trap::S_StartLocalSound(
                ctx.engine,
                ctx.world.cgs.media.oneMinuteSound,
                CHAN_ANNOUNCER,
            );
            ctx.world.playerstate.cgAnnouncerTime = ctx.world.cg.time + 3000;
        } else if ctx.world.cgs.timelimit > 5
            && (ctx.world.cg.timelimitWarnings & 1) == 0
            && msec > (ctx.world.cgs.timelimit - 5) * 60 * 1000
        {
            ctx.world.cg.timelimitWarnings |= 1;
            trap::S_StartLocalSound(
                ctx.engine,
                ctx.world.cgs.media.fiveMinuteSound,
                CHAN_ANNOUNCER,
            );
            ctx.world.playerstate.cgAnnouncerTime = ctx.world.cg.time + 3000;
        }
    }

    // fraglimit warnings
    if ctx.world.cgs.fraglimit > 0
        && ctx.world.cgs.gametype < GT_CTF
        && ctx.world.cgs.gametype != GT_DUEL
        && ctx.world.cgs.gametype != GT_POWERDUEL
        && ctx.world.cgs.gametype != GT_SIEGE
        && ctx.world.playerstate.cgAnnouncerTime < ctx.world.cg.time
    {
        let highScore = ctx.world.cgs.scores1;
        if (ctx.world.cg.fraglimitWarnings & 4) == 0 && highScore == ctx.world.cgs.fraglimit - 1 {
            ctx.world.cg.fraglimitWarnings |= 1 | 2 | 4;
            let sfx = ctx.world.cgs.media.oneFragSound;
            CG_AddBufferedSound(ctx.world, sfx);
            ctx.world.playerstate.cgAnnouncerTime = ctx.world.cg.time + 3000;
        } else if ctx.world.cgs.fraglimit > 2
            && (ctx.world.cg.fraglimitWarnings & 2) == 0
            && highScore == ctx.world.cgs.fraglimit - 2
        {
            ctx.world.cg.fraglimitWarnings |= 1 | 2;
            let sfx = ctx.world.cgs.media.twoFragSound;
            CG_AddBufferedSound(ctx.world, sfx);
            ctx.world.playerstate.cgAnnouncerTime = ctx.world.cg.time + 3000;
        } else if ctx.world.cgs.fraglimit > 3
            && (ctx.world.cg.fraglimitWarnings & 1) == 0
            && highScore == ctx.world.cgs.fraglimit - 3
        {
            ctx.world.cg.fraglimitWarnings |= 1;
            let sfx = ctx.world.cgs.media.threeFragSound;
            CG_AddBufferedSound(ctx.world, sfx);
            ctx.world.playerstate.cgAnnouncerTime = ctx.world.cg.time + 3000;
        }
    }
}

/// Raven `CG_CheckPlayerstateEvents` — replays the server-authoritative
/// `externalEvent` plus the local client's predictable-event ring buffer into
/// `CG_EntityEvent`, keeping `cg.predictableEvents`/`cg.eventSequence` in step.
///
/// Source: `oracle/codemp/cgame/cg_playerstate.c:217-250`
pub fn CG_CheckPlayerstateEvents(
    ctx: &mut CgContext,
    ds: &DisplayState,
    ps: &playerState_t,
    ops: &playerState_t,
    psRef: PlayerStateRef,
) {
    let centNum = ps.clientNum as usize;

    if ps.externalEvent != 0 && ps.externalEvent != ops.externalEvent {
        {
            let cent = ctx.world.entity_mut(centNum);
            cent.currentState.event = ps.externalEvent;
            cent.currentState.eventParm = ps.externalEventParm;
        }
        let position = ctx.world.entity(centNum).lerpOrigin;
        CG_EntityEvent(ctx, ds, centNum, &position);
    }

    // go through the predictable events buffer
    for i in (ps.eventSequence - MAX_PS_EVENTS as c_int)..ps.eventSequence {
        let idx = (i & (MAX_PS_EVENTS as c_int - 1)) as usize;
        // if we have a new predictable event
        // or the server told us to play another event instead of a predicted event we already issued
        // or something the server told us changed our prediction causing a different event
        if i >= ops.eventSequence
            || (i > ops.eventSequence - MAX_PS_EVENTS as c_int && ps.events[idx] != ops.events[idx])
        {
            let event = ps.events[idx];
            {
                let cent = ctx.world.entity_mut(centNum);
                cent.currentState.event = event;
                cent.currentState.eventParm = ps.eventParms[idx];
                // JLF ADDED to hopefully mark events as player event
                //
                // Raven stores the caller's `ps` pointer verbatim here
                // (`cg.predictedPlayerState` on the predict path,
                // `cg.snap->ps` on the cg_snapshot.c one) - and cg_view.c /
                // cg_players.c DO read it back. The caller says which target
                // it handed us (DEC-46.2 resolution enum).
                cent.playerState = psRef;
            }
            let position = ctx.world.entity(centNum).lerpOrigin;
            CG_EntityEvent(ctx, ds, centNum, &position);

            let predIdx = (i & (MAX_PREDICTED_EVENTS as c_int - 1)) as usize;
            ctx.world.cg.predictableEvents[predIdx] = event;

            ctx.world.cg.eventSequence += 1;
        }
    }
}

/// Raven `CG_CheckChangedPredictableEvents` — re-checks the server's replayed
/// `playerState_t.events` ring against what the client already predicted
/// (`cg.predictableEvents`), replaying `CG_EntityEvent` only where the server's
/// account diverged from the local prediction.
///
/// Source: `oracle/codemp/cgame/cg_playerstate.c:257-286`
pub fn CG_CheckChangedPredictableEvents(
    ctx: &mut CgContext,
    ds: &DisplayState,
    ps: &playerState_t,
) {
    let centNum = ps.clientNum as usize;

    for i in (ps.eventSequence - MAX_PS_EVENTS as c_int)..ps.eventSequence {
        if i >= ctx.world.cg.eventSequence {
            continue;
        }
        // if this event is not further back in than the maximum predictable events we remember
        if i > ctx.world.cg.eventSequence - MAX_PREDICTED_EVENTS as c_int {
            let idx = (i & (MAX_PS_EVENTS as c_int - 1)) as usize;
            let predIdx = (i & (MAX_PREDICTED_EVENTS as c_int - 1)) as usize;

            // if the new playerstate event is different from a previously predicted one
            if ps.events[idx] != ctx.world.cg.predictableEvents[predIdx] {
                let event = ps.events[idx];
                {
                    let cent = ctx.world.entity_mut(centNum);
                    cent.currentState.event = event;
                    cent.currentState.eventParm = ps.eventParms[idx];
                }
                let position = ctx.world.entity(centNum).lerpOrigin;
                CG_EntityEvent(ctx, ds, centNum, &position);

                ctx.world.cg.predictableEvents[predIdx] = event;

                if ctx.world.cvars.cg_showmiss.integer != 0 {
                    CG_Printf(ctx, "WARNING: changed predicted event\n");
                }
            }
        }
    }
}

/// Raven `CG_TransitionPlayerState` - runs once per new `playerState_t`
/// (predicted-locally or from a fresh snapshot), firing the respawn/damage/
/// local-sound/event side effects off the delta against last frame's state.
///
/// `psRef` names which of `cg.predictedPlayerState`/`cg.snap->ps` the caller
/// handed us, forwarded into `CG_CheckPlayerstateEvents` per DEC-46.2.
///
/// Source: `oracle/codemp/cgame/cg_playerstate.c:495-534`
pub fn CG_TransitionPlayerState(
    ctx: &mut CgContext,
    ds: &DisplayState,
    ps: &playerState_t,
    ops: &mut playerState_t,
    psRef: PlayerStateRef,
) {
    // check for changing follow mode
    if ps.clientNum != ops.clientNum {
        ctx.world.cg.thisFrameTeleport = qtrue;
        // make sure we don't get any unwanted transition effects
        *ops = *ps;
    }

    // damage events (player is getting wounded)
    if ps.damageEvent != ops.damageEvent && ps.damageCount != 0 {
        CG_DamageFeedback(ctx.world, ps.damageYaw, ps.damagePitch, ps.damageCount);
    }

    // respawning
    if ps.persistant[PERS_SPAWN_COUNT as usize] != ops.persistant[PERS_SPAWN_COUNT as usize] {
        CG_Respawn(ctx.world);
    }

    if ctx.world.cg.mapRestart != 0 {
        CG_Respawn(ctx.world);
        ctx.world.cg.mapRestart = 0;
    }

    // §F19: Raven derefs `cg.snap->ps` here with no null check - before the
    // first snapshot that's a null deref, so the port takes the neutral
    // early-out (skip the local-sounds check) rather than reading through a
    // null, same posture as CG_Respawn/CG_DamageFeedback above. The snapshot
    // read is done up front so the borrow doesn't outlive the `ctx` we hand
    // CG_CheckLocalSounds below.
    let not_intermission = ctx
        .world
        .cg
        .snap_ref()
        .is_some_and(|snap| snap.ps.pm_type != PM_INTERMISSION as c_int);
    if not_intermission && ps.persistant[PERS_TEAM as usize] != TEAM_SPECTATOR as c_int {
        CG_CheckLocalSounds(ctx, ps, ops);
    }

    // check for going low on ammo
    CG_CheckAmmo();

    // run events
    CG_CheckPlayerstateEvents(ctx, ds, ps, ops, psRef);

    // smooth the ducking viewheight change
    if ps.viewheight != ops.viewheight {
        ctx.world.cg.duckChange = (ps.viewheight - ops.viewheight) as f32;
        ctx.world.cg.duckTime = ctx.world.cg.time;
    }
}
