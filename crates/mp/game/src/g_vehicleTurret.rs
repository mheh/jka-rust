// PORT-COMPLETE: g_vehicleTurret.c

//! FAITHFUL port of `oracle/codemp/game/g_vehicleTurret.c`.
//!
//! Filled by the jampgame mega-pass; functions reach file-scope game state
//! (`level`, `g_entities`, cvars) and engine traps through the threaded
//! `GameContext`/`GameWorld` handle.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

// Raven `qboolean` is `c_int`; keep the source spelling at assignment sites.
// Source: `oracle/codemp/game/q_shared.h`

use crate::entity::flags::{FL_BBRUSH, FL_NOTARGET};
use crate::g_team::OnSameTeam;
use crate::g_utils::G_RadiusList;
use crate::g_weapon::{G_VehMuzzleFireFX, WP_CalcVehMuzzle, WP_FireVehicleWeapon};
use crate::q_math::{
    _VectorCopy, _VectorMA, _VectorSubtract, vectoangles, AngleNormalize180, AnglesSubtract,
    VectorLengthSquared, VectorNormalize,
};
use crate::q_shared::{Q_stricmp, Q_strncmp};
use crate::trap;
use crate::NPC_utils::NPC_SetBoneAngles;
use mp_bg::public::team::TEAM_SPECTATOR;
use mp_bg::weapons::weapon_t::WP_TURRET;
use mp_qshared::common::mp::qcommon::usercmd_button::{BUTTON_ALT_ATTACK, BUTTON_ATTACK};
use mp_qshared::shared::limits::{ENTITYNUM_NONE, ENTITYNUM_WORLD};
use mp_qshared::shared::surface_flags::MASK_SHOT;

// `PITCH`/`YAW` resolve via the crate prelude glob (`crate::q_math`); the
// shadowing local copies were removed by the placeholder-const sweep.

/// Raven `VEH_TurretCheckFire`.
///
/// If it's time to fire and we have an enemy, then gun 'em down! pushDebounce time controls next fire time.
///
/// Source: `oracle/codemp/game/g_vehicleTurret.c:12-59`
pub fn VEH_TurretCheckFire(
    ctx: &mut GameContext,
    pVeh: *mut Vehicle_t,
    parent: EntityId,
    turretStats: *mut turretStats_t,
    vehWeapon: *mut vehWeaponInfo_t,
    turretNum: c_int,
    curMuzzle: c_int,
) {
    unsafe {
        // if it's time to fire and we have an enemy, then gun 'em down!  pushDebounce time controls next fire time
        if (*pVeh).m_iMuzzleTag[curMuzzle as usize] == -1 {
            // invalid muzzle?
            return;
        }

        if (*pVeh).m_iMuzzleWait[curMuzzle as usize] >= ctx.world.level.time {
            // can't fire yet
            return;
        }

        if (*pVeh).turretStatus[turretNum as usize].ammo < (*vehWeapon).iAmmoPerShot {
            // no ammo, can't fire
            return;
        }

        // FIXME: check to see if I'm aiming generally where I want to
        let mut nextMuzzle: c_int = 0;
        let muzzlesFired: c_int = 1 << curMuzzle;
        let missile: *mut gentity_t;
        WP_CalcVehMuzzle(ctx, parent, curMuzzle);

        // FIXME: some variation in fire dir
        missile = WP_FireVehicleWeapon(
            ctx,
            parent,
            (*pVeh).m_vMuzzlePos[curMuzzle as usize],
            (*pVeh).m_vMuzzleDir[curMuzzle as usize],
            vehWeapon,
            turretNum != 0,
            true,
        );

        // play the weapon's muzzle effect if we have one
        G_VehMuzzleFireFX(ctx, parent, ctx.entity_id_of(missile), muzzlesFired);

        // take the ammo away
        (*pVeh).turretStatus[turretNum as usize].ammo -= (*vehWeapon).iAmmoPerShot;
        // toggle to the next muzzle on this turret, if there is one
        nextMuzzle =
            if (curMuzzle + 1) == (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].iMuzzle[0] {
                (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].iMuzzle[1]
            } else {
                (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].iMuzzle[0]
            };
        if nextMuzzle != 0 {
            // a valid muzzle to toggle to
            (*pVeh).turretStatus[turretNum as usize].nextMuzzle = nextMuzzle - 1;
            // -1 because you type muzzles 1-10 in the .veh file
        }
        // add delay to the next muzzle so it doesn't fire right away on the next frame
        (*pVeh).m_iMuzzleWait[(*pVeh).turretStatus[turretNum as usize].nextMuzzle as usize] =
            ctx.world.level.time + (*turretStats).iDelay;
    }
}

/// Raven `VEH_TurretAnglesToEnemy`.
///
/// Source: `oracle/codemp/game/g_vehicleTurret.c:61-86`
pub fn VEH_TurretAnglesToEnemy(
    pVeh: *mut Vehicle_t,
    curMuzzle: c_int,
    fSpeed: f32,
    turretEnemy: &gentity_t,
    bAILead: qboolean,
    desiredAngles: &mut vec3_t,
) {
    unsafe {
        let mut enemyDir = [0f32; 3];
        let mut org = [0f32; 3];
        _VectorCopy(turretEnemy.r.currentOrigin, &mut org);
        if bAILead != qfalse {
            //we want to lead them a bit
            let mut diff = [0f32; 3];
            let mut velocity = [0f32; 3];
            _VectorSubtract(org, (*pVeh).m_vMuzzlePos[curMuzzle as usize], &mut diff);
            let dist = VectorNormalize(&mut diff);
            if !turretEnemy.client.is_null() {
                // FLAG: turretEnemy may be an NPC/vehicle carrying a BG_Alloc'd pool
                // client (trap 2b); deref the client pointer raw, as Raven does.
                let tec = turretEnemy.client;
                _VectorCopy((*tec).ps.velocity, &mut velocity);
            } else {
                _VectorCopy(turretEnemy.s.pos.trDelta, &mut velocity);
            }
            _VectorMA(org, dist / fSpeed, velocity, &mut org);
        }

        //FIXME: this isn't quite right, it's aiming from the muzzle, not the center of the turret...
        _VectorSubtract(org, (*pVeh).m_vMuzzlePos[curMuzzle as usize], &mut enemyDir);
        //Get the desired absolute, world angles to our target
        vectoangles(enemyDir, desiredAngles);
    }
}

/// Raven `VEH_TurretAim`.
///
/// Source: `oracle/codemp/game/g_vehicleTurret.c:89-190`
pub fn VEH_TurretAim(
    ctx: &mut GameContext,
    pVeh: *mut Vehicle_t,
    parent: EntityId,
    turretEnemy: Option<EntityId>,
    turretStats: *mut turretStats_t,
    vehWeapon: *mut vehWeaponInfo_t,
    turretNum: c_int,
    curMuzzle: c_int,
    desiredAngles: &mut vec3_t,
) -> qboolean {
    unsafe {
        let mut curAngles = [0f32; 3];
        let mut addAngles = [0f32; 3];
        let mut newAngles = [0f32; 3];
        let mut yawAngles = [0f32; 3];
        let mut pitchAngles = [0f32; 3];
        let mut aimCorrect: qboolean = qfalse;

        WP_CalcVehMuzzle(ctx, parent, curMuzzle);
        // get the current absolute angles of the turret right now
        vectoangles((*pVeh).m_vMuzzleDir[curMuzzle as usize], &mut curAngles);
        // subtract out the vehicle's angles to get the relative alignment
        AnglesSubtract(
            curAngles,
            *((*pVeh).m_vOrientation as *const vec3_t),
            &mut curAngles,
        );

        if turretEnemy.is_some() {
            aimCorrect = qtrue;
            // ...then we'll calculate what new aim adjustments we should attempt to make this frame
            // Aim at enemy
            VEH_TurretAnglesToEnemy(
                pVeh,
                curMuzzle,
                (*vehWeapon).fSpeed,
                ctx.world.entity(turretEnemy.unwrap()),
                (*turretStats).bAILead,
                desiredAngles,
            );
        }
        // subtract out the vehicle's angles to get the relative desired alignment
        AnglesSubtract(
            *desiredAngles,
            *((*pVeh).m_vOrientation as *const vec3_t),
            desiredAngles,
        );
        // Now clamp the desired relative angles
        // clamp yaw
        (*desiredAngles)[YAW] = AngleNormalize180((*desiredAngles)[YAW]);
        if (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].yawClampLeft != 0.0
            && (*desiredAngles)[YAW]
                > (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].yawClampLeft
        {
            aimCorrect = qfalse;
            (*desiredAngles)[YAW] =
                (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].yawClampLeft;
        }
        if (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].yawClampRight != 0.0
            && (*desiredAngles)[YAW]
                < (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].yawClampRight
        {
            aimCorrect = qfalse;
            (*desiredAngles)[YAW] =
                (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].yawClampRight;
        }
        // clamp pitch
        (*desiredAngles)[PITCH] = AngleNormalize180((*desiredAngles)[PITCH]);
        if (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].pitchClampDown != 0.0
            && (*desiredAngles)[PITCH]
                > (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].pitchClampDown
        {
            aimCorrect = qfalse;
            (*desiredAngles)[PITCH] =
                (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].pitchClampDown;
        }
        if (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].pitchClampUp != 0.0
            && (*desiredAngles)[PITCH]
                < (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].pitchClampUp
        {
            aimCorrect = qfalse;
            (*desiredAngles)[PITCH] =
                (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize].pitchClampUp;
        }
        // Now get the offset we want from our current relative angles
        AnglesSubtract(*desiredAngles, curAngles, &mut addAngles);
        // Now cap the addAngles for our fTurnSpeed
        if addAngles[PITCH] > (*turretStats).fTurnSpeed {
            // aimCorrect = qfalse;//???
            addAngles[PITCH] = (*turretStats).fTurnSpeed;
        } else if addAngles[PITCH] < -(*turretStats).fTurnSpeed {
            // aimCorrect = qfalse;//???
            addAngles[PITCH] = -(*turretStats).fTurnSpeed;
        }
        if addAngles[YAW] > (*turretStats).fTurnSpeed {
            // aimCorrect = qfalse;//???
            addAngles[YAW] = (*turretStats).fTurnSpeed;
        } else if addAngles[YAW] < -(*turretStats).fTurnSpeed {
            // aimCorrect = qfalse;//???
            addAngles[YAW] = -(*turretStats).fTurnSpeed;
        }
        // Now add the additional angles back in to our current relative angles
        // FIXME: add some AI aim error randomness...?
        newAngles[PITCH] = AngleNormalize180(curAngles[PITCH] + addAngles[PITCH]);
        newAngles[YAW] = AngleNormalize180(curAngles[YAW] + addAngles[YAW]);
        // Now set the bone angles to the new angles
        // set yaw
        if (*turretStats).yawBone != core::ptr::null_mut() {
            // VectorClear( yawAngles );
            yawAngles[0] = 0.0;
            yawAngles[1] = 0.0;
            yawAngles[2] = 0.0;
            yawAngles[(*turretStats).yawAxis as usize] = newAngles[YAW];
            NPC_SetBoneAngles(ctx, parent, (*turretStats).yawBone, yawAngles);
        }
        // set pitch
        if (*turretStats).pitchBone != core::ptr::null_mut() {
            // VectorClear( pitchAngles );
            pitchAngles[0] = 0.0;
            pitchAngles[1] = 0.0;
            pitchAngles[2] = 0.0;
            pitchAngles[(*turretStats).pitchAxis as usize] = newAngles[PITCH];
            NPC_SetBoneAngles(ctx, parent, (*turretStats).pitchBone, pitchAngles);
        }
        // force muzzle to recalc next check
        (*pVeh).m_iMuzzleTime[curMuzzle as usize] = 0;

        return aimCorrect;
    }
}

/// Raven `VEH_TurretFindEnemies`.
///
/// Source: `oracle/codemp/game/g_vehicleTurret.c:193-302`
pub fn VEH_TurretFindEnemies(
    ctx: &mut GameContext,
    pVeh: *mut Vehicle_t,
    parent: EntityId,
    turretStats: *mut turretStats_t,
    turretNum: c_int,
    curMuzzle: c_int,
) -> qboolean {
    unsafe {
        let mut found: qboolean = qfalse;
        let mut i: c_int;
        let mut count: c_int;
        let mut bestDist: f32 = (*turretStats).fAIRange * (*turretStats).fAIRange;
        let mut enemyDist: f32;
        let mut enemyDir = [0f32; 3];
        let mut org = [0f32; 3];
        let mut org2 = [0f32; 3];
        let mut foundClient: qboolean = qfalse;
        let mut entity_list: [*mut gentity_t; MAX_GENTITIES] =
            [core::ptr::null_mut(); MAX_GENTITIES];
        let mut best_id: Option<EntityId> = None;

        // FLAG: parent is a vehicle carrying a BG_Alloc'd pool client (trap 2b);
        // its `client` pointer is deref'd raw below, so read the (stable) pointer
        // value once and keep it, exactly as Raven's `parent->client` does.
        let parent_client = ctx.world.entity(parent).client;

        WP_CalcVehMuzzle(ctx, parent, curMuzzle);
        _VectorCopy((*pVeh).m_vMuzzlePos[curMuzzle as usize], &mut org2);

        count = G_RadiusList(
            ctx,
            org2,
            (*turretStats).fAIRange,
            Some(parent),
            qtrue,
            entity_list.as_mut_ptr(),
        );

        i = 0;
        while i < count {
            let mut tr: trace_t = core::mem::zeroed();
            let target = entity_list[i as usize];
            let target_id = ctx.entity_id_of(target).unwrap();

            if target_id == parent
                || ctx.world.entity(target_id).takedamage == qfalse
                || ctx.world.entity(target_id).health <= 0
                || (ctx.world.entity(target_id).flags & FL_NOTARGET) != 0
            {
                i += 1;
                continue;
            }
            if ctx.world.entity(target_id).client.is_null() {
                // only attack clients
                if (ctx.world.entity(target_id).flags & FL_BBRUSH) == 0
                    // not a breakable brush
                    || ctx.world.entity(target_id).takedamage == qfalse
                    // is a bbrush, but invincible
                    || (!ctx.world.entity(target_id).NPC_targetname.is_null()
                        && !ctx.world.entity(parent).targetname.is_null()
                        && Q_stricmp(
                            ctx.world.entity(target_id).NPC_targetname,
                            ctx.world.entity(parent).targetname,
                        ) != 0)
                {
                    // not in invicible bbrush, but can only be broken by an NPC that is not me
                    let s = cstr("misc_turret");
                    if ctx.world.entity(target_id).s.weapon == WP_TURRET
                        && !ctx.world.entity(target_id).classname.is_null()
                        && Q_strncmp(ctx.world.entity(target_id).classname, s.as_ptr(), 11) == 0
                    {
                        // these guys we want to shoot at
                    } else {
                        i += 1;
                        continue;
                    }
                }
                // else: we will shoot at bbrushes!
            } else {
                // client is non-null here
                // FLAG: target may be an NPC carrying a pool client (trap 2b);
                // deref the client pointer raw, as Raven does.
                let tc = ctx.world.entity(target_id).client;
                if !tc.is_null() && (*tc).sess.sessionTeam == TEAM_SPECTATOR {
                    i += 1;
                    continue;
                }
            }
            if target == (*pVeh).m_pPilot as *mut gentity_t
                || ctx.world.entity(target_id).r.ownerNum == ctx.world.entity(parent).s.number
            {
                // don't get angry at my pilot or passengers?
                i += 1;
                continue;
            }
            if !parent_client.is_null() && (*parent_client).sess.sessionTeam != 0 {
                // FLAG: parent/target pool clients (trap 2b); deref raw.
                let tc = ctx.world.entity(target_id).client;
                if !tc.is_null() {
                    if (*tc).sess.sessionTeam == (*parent_client).sess.sessionTeam {
                        // A bot/client/NPC we don't want to shoot
                        i += 1;
                        continue;
                    }
                } else if ctx.world.entity(target_id).teamnodmg == (*parent_client).sess.sessionTeam
                {
                    // some other entity that's allied with us
                    i += 1;
                    continue;
                }
            }
            if trap::InPVS(
                ctx.engine,
                mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs::new(
                    &org2 as *const vec3_t,
                    &ctx.world.entity(target_id).r.currentOrigin as *const vec3_t,
                ),
            ) == qfalse
            {
                i += 1;
                continue;
            }

            _VectorCopy(ctx.world.entity(target_id).r.currentOrigin, &mut org);

            trap::Trace(
                ctx.engine,
                mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                    &mut tr as *mut trace_t,
                    &org2 as *const vec3_t,
                    core::ptr::null(),
                    core::ptr::null(),
                    &org as *const vec3_t,
                    ctx.world.entity(parent).s.number,
                    MASK_SHOT,
                ),
            );

            if tr.entityNum as c_int == ctx.world.entity(target_id).s.number
                || (tr.allsolid == 0 && tr.startsolid == 0 && tr.fraction == 1.0f32)
            {
                // Only acquire if have a clear shot, Is it in range and closer than our best?
                _VectorSubtract(
                    ctx.world.entity(target_id).r.currentOrigin,
                    org2,
                    &mut enemyDir,
                );
                enemyDist = VectorLengthSquared(enemyDir);

                if enemyDist < bestDist
                    || (!ctx.world.entity(target_id).client.is_null() && foundClient == qfalse)
                {
                    // all things equal, keep current
                    best_id = Some(target_id);
                    bestDist = enemyDist;
                    found = qtrue;
                    if !ctx.world.entity(target_id).client.is_null() {
                        // prefer clients over non-clients
                        foundClient = qtrue;
                    }
                }
            }
            i += 1;
        }

        if found != qfalse {
            let n = ctx.world.entity(best_id.unwrap()).s.number;
            (*pVeh).turretStatus[turretNum as usize].enemyEntNum = n;
        }

        return found;
    }
}

/// Raven `VEH_TurretObeyPassengerControl`.
///
/// Source: `oracle/codemp/game/g_vehicleTurret.c:304-322`
pub fn VEH_TurretObeyPassengerControl(
    ctx: &mut GameContext,
    pVeh: *mut Vehicle_t,
    parent: EntityId,
    turretNum: c_int,
) {
    unsafe {
        let turretStats: *mut turretStats_t =
            &mut (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize];
        let passenger: *mut gentity_t =
            (*pVeh).m_ppPassengers[((*turretStats).passengerNum - 1) as usize] as *mut gentity_t;

        if !passenger.is_null() {
            let passenger_id = ctx.entity_id_of(passenger).unwrap();
            // FLAG: passenger may be an NPC carrying a pool client (trap 2b); read
            // the (stable) client pointer once and deref it raw, as Raven does.
            let passenger_client = ctx.world.entity(passenger_id).client;
            if !passenger_client.is_null() && ctx.world.entity(passenger_id).health > 0 {
                // a valid, living passenger client
                let vehWeapon: *mut vehWeaponInfo_t =
                    &mut (&mut ctx.world.bg_state.g_vehWeaponInfo)[(*turretStats).iWeapon as usize];
                let curMuzzle: c_int = (*pVeh).turretStatus[turretNum as usize].nextMuzzle;
                let mut aimAngles = [0f32; 3];
                _VectorCopy((*passenger_client).ps.viewangles, &mut aimAngles);

                VEH_TurretAim(
                    ctx,
                    pVeh,
                    parent,
                    None,
                    turretStats,
                    vehWeapon,
                    turretNum,
                    curMuzzle,
                    &mut aimAngles,
                );
                if ((*passenger_client).pers.cmd.buttons & (BUTTON_ATTACK | BUTTON_ALT_ATTACK)) != 0
                {
                    // he's pressing an attack button, so fire!
                    VEH_TurretCheckFire(
                        ctx,
                        pVeh,
                        parent,
                        turretStats,
                        vehWeapon,
                        turretNum,
                        curMuzzle,
                    );
                }
            }
        }
    }
}

/// Raven `VEH_TurretThink`.
///
/// Source: `oracle/codemp/game/g_vehicleTurret.c:324-444`
pub fn VEH_TurretThink(
    ctx: &mut GameContext,
    pVeh: *mut Vehicle_t,
    parent: EntityId,
    turretNum: c_int,
) {
    unsafe {
        let mut doAim: qboolean = qfalse;
        let mut enemyDist: f32;
        let mut rangeSq: f32;
        let mut enemyDir = [0f32; 3];
        let turretStats: *mut turretStats_t =
            &mut (*(*pVeh).m_pVehicleInfo).turret[turretNum as usize];
        let mut vehWeapon: *mut vehWeaponInfo_t = core::ptr::null_mut();
        let mut turretEnemy: Option<EntityId> = None;
        let mut curMuzzle: c_int = 0; // ?

        if turretStats.is_null() || (*turretStats).iAmmoMax == 0 {
            // not a valid turret
            return;
        }

        if (*turretStats).passengerNum != 0
            && (*pVeh).m_iNumPassengers >= (*turretStats).passengerNum
        {
            // the passenger that has control of this turret is on the ship
            VEH_TurretObeyPassengerControl(ctx, pVeh, parent, turretNum);
            return;
        } else if (*turretStats).bAI == qfalse {
            // try AI
            // this turret does not think on its own.
            return;
        }
        // okay, so it has AI, but still don't think if there's no pilot!
        if (*pVeh).m_pPilot.is_null() {
            return;
        }

        vehWeapon = &mut (&mut ctx.world.bg_state.g_vehWeaponInfo)[(*turretStats).iWeapon as usize];
        rangeSq = (*turretStats).fAIRange * (*turretStats).fAIRange;
        curMuzzle = (*pVeh).turretStatus[turretNum as usize].nextMuzzle;

        if (*pVeh).turretStatus[turretNum as usize].enemyEntNum < ENTITYNUM_WORLD {
            let te_id = EntityId((*pVeh).turretStatus[turretNum as usize].enemyEntNum as u32);
            turretEnemy = Some(te_id);
            if ctx.world.entity(te_id).health < 0
                || ctx.world.entity(te_id).inuse == qfalse
                || ctx.entity_id_of((*pVeh).m_pPilot as *mut gentity_t) == Some(te_id)
                // enemy became my pilot///?
                || te_id == parent
                || ctx.world.entity(te_id).r.ownerNum == ctx.world.entity(parent).s.number // a passenger?
                || {
                    // FLAG: te may be an NPC carrying a pool client (trap 2b); deref raw.
                    let tec = ctx.world.entity(te_id).client;
                    !tec.is_null() && (*tec).sess.sessionTeam == TEAM_SPECTATOR
                }
            {
                // don't keep going after spectators, pilot, self, dead people, etc.
                turretEnemy = None;
                (*pVeh).turretStatus[turretNum as usize].enemyEntNum = ENTITYNUM_NONE;
            }
        }

        if (*pVeh).turretStatus[turretNum as usize].enemyHoldTime < ctx.world.level.time {
            if VEH_TurretFindEnemies(ctx, pVeh, parent, turretStats, turretNum, curMuzzle) != qfalse
            {
                turretEnemy = Some(EntityId(
                    (*pVeh).turretStatus[turretNum as usize].enemyEntNum as u32,
                ));
                doAim = qtrue;
            } else {
                let parent_enemy = ctx.world.entity(parent).enemy;
                if let Some(enemy_id) = parent_enemy {
                    if ctx.world.entity(enemy_id).s.number < ENTITYNUM_WORLD {
                        if ctx.world.cvars.g_gametype.integer < GT_TEAM
                            || OnSameTeam(ctx, Some(enemy_id), Some(parent)) == qfalse
                        {
                            // either not in a team game or the enemy isn't on the same team
                            turretEnemy = Some(enemy_id);
                            doAim = qtrue;
                        }
                    }
                }
            }
            if let Some(te_id) = turretEnemy {
                // found one
                if !ctx.world.entity(te_id).client.is_null() {
                    // hold on to clients for a min of 3 seconds
                    (*pVeh).turretStatus[turretNum as usize].enemyHoldTime =
                        ctx.world.level.time + 3000;
                } else {
                    // hold less
                    (*pVeh).turretStatus[turretNum as usize].enemyHoldTime =
                        ctx.world.level.time + 500;
                }
            }
        }
        if let Some(te_id) = turretEnemy {
            if ctx.world.entity(te_id).health > 0 {
                // enemy is alive
                WP_CalcVehMuzzle(ctx, parent, curMuzzle);
                _VectorSubtract(
                    ctx.world.entity(te_id).r.currentOrigin,
                    (*pVeh).m_vMuzzlePos[curMuzzle as usize],
                    &mut enemyDir,
                );
                enemyDist = VectorLengthSquared(enemyDir);

                if enemyDist < rangeSq {
                    // was in valid radius
                    if trap::InPVS(
                        ctx.engine,
                        mp_abi::game::syscalls::G_IN_PVS::GInPvsArgs::new(
                            &(*pVeh).m_vMuzzlePos[curMuzzle as usize] as *const vec3_t,
                            &ctx.world.entity(te_id).r.currentOrigin as *const vec3_t,
                        ),
                    ) != qfalse
                    {
                        // Every now and again, check to see if we can even trace to the enemy
                        let mut tr: trace_t = core::mem::zeroed();
                        let mut start = [0f32; 3];
                        let mut end = [0f32; 3];
                        _VectorCopy((*pVeh).m_vMuzzlePos[curMuzzle as usize], &mut start);

                        _VectorCopy(ctx.world.entity(te_id).r.currentOrigin, &mut end);
                        trap::Trace(
                            ctx.engine,
                            mp_abi::game::syscalls::G_TRACE::GTraceArgs::new(
                                &mut tr as *mut trace_t,
                                &start as *const vec3_t,
                                core::ptr::null(),
                                core::ptr::null(),
                                &end as *const vec3_t,
                                ctx.world.entity(parent).s.number,
                                MASK_SHOT,
                            ),
                        );

                        if tr.entityNum as c_int == ctx.world.entity(te_id).s.number
                            || (tr.allsolid == 0 && tr.startsolid == 0)
                        {
                            doAim = qtrue; // Can see our enemy
                        }
                    }
                }
            }
        }

        if doAim != qfalse {
            let mut aimAngles = [0f32; 3];
            if VEH_TurretAim(
                ctx,
                pVeh,
                parent,
                turretEnemy,
                turretStats,
                vehWeapon,
                turretNum,
                curMuzzle,
                &mut aimAngles,
            ) != qfalse
            {
                VEH_TurretCheckFire(
                    ctx,
                    pVeh,
                    parent,
                    turretStats,
                    vehWeapon,
                    turretNum,
                    curMuzzle,
                );
            }
        }
    }
}
