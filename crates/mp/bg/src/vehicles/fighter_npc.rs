//! Shared (game + cgame) fighter vehicle steering/physics.
//!
//! The bg-compatible half of `oracle/codemp/game/FighterNPC.c` (a cgame TU in
//! `JK2_cgame.vcproj`): `BG_FighterUpdate` (Raven's comment — "client must
//! explicitly call this for prediction"), `ProcessMoveCommands`,
//! `ProcessOrientCommands`, and every helper they transitively call. These run
//! inside `Pmove` (the game-tier `Update` chain for the Game host, cgame
//! prediction for the Cgame host). Game-only functions (`Board`, `Eject`,
//! `Update`, `AnimateVehicle`, `AnimateRiders`, `FighterPitchClamp`,
//! `G_CreateFighterNPC`) stay in `mp_game`'s `FighterNPC.rs`.
//!
//! `#ifdef QAGAME` islands host-switch on `pmc.bg.host`. Those reaching
//! game-only fields — the takeoff/turbo `G_EntitySound`, `FighterIsInSpace`
//! (`client->inSpaceIndex`), and the land-while-broken `G_DamageFromKiller`
//! (NULL attacker) — route through `GameCallbacks` upcalls under the Game host
//! and compile out under the Cgame host. `BG_FighterUpdate`'s `#ifdef QAGAME`
//! `Ghost` loop is hoisted into `mp_game`'s `FighterNPC::Update` wrapper (which
//! runs it before this body, preserving the game-side call order) so it never
//! needs a bg upcall.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;
use crate::vehicles::MIN_LANDING_SLOPE;
use mp_qshared::shared::q_math::{
    _VectorMA, _VectorScale, AngleNormalize180, AngleNormalize360, VectorClear,
};

// Constants used by the fighter move/orient bodies. Values from the oracle;
// defined locally (mirroring `mp_game`'s `FighterNPC.rs`).
// Source: `oracle/codemp/game/{bg_public.h,bg_vehicles.h,FighterNPC.c}`.
const HYPERSPACE_SPEED: f32 = 10000.0; // bg_public.h:1681
const FIGHTER_MIN_TAKEOFF_FRACTION: f32 = 0.7; // FighterNPC.c:369
const MAX_STRAFE_TIME: f32 = 2000.0; // bg_vehicles.h:398
                                     // Removed-surface bitflags. Source: `oracle/codemp/game/bg_vehicles.h:444-447`.
const SHIPSURF_BROKEN_C: c_int = 1 << 2; // wing 1
const SHIPSURF_BROKEN_D: c_int = 1 << 3; // wing 2
const SHIPSURF_BROKEN_E: c_int = 1 << 4; // wing 3
const SHIPSURF_BROKEN_F: c_int = 1 << 5; // wing 4

/// `Inhabited` for the base vehicle — a pilot or at least one passenger. Inlined
/// (the generic base `g_vehicles.c` body is bg-reachable), reached only under the
/// Game host where Raven's `#ifdef QAGAME` `Inhabited(pVeh)` clauses compile in.
/// Source: `oracle/codemp/game/g_vehicles.c` (`Inhabited`).
#[inline]
unsafe fn Inhabited(pVeh: *mut Vehicle_t) -> qboolean {
    if !(*pVeh).m_pPilot.is_null() || (*pVeh).m_iNumPassengers != 0 {
        qtrue
    } else {
        qfalse
    }
}

/// Raven `BG_FighterUpdate`.
///
/// Raven: "client must explicitly call this for prediction." The `#ifdef QAGAME`
/// `Ghost` loop is hoisted into `mp_game`'s `FighterNPC::Update`; the trace uses
/// the host's own pmove trace (`self.traps.trace` == `G_VehicleTrace` game-side,
/// `pm->trace` cgame-side).
/// Source: `oracle/codemp/game/FighterNPC.c:99-183`;
/// cgame arm `oracle/codemp/game/bg_pmove.c:10965-10968`
pub fn BG_FighterUpdate(
    pmc: &mut PmoveContext,
    pVeh: *mut Vehicle_t,
    _pUcmd: *const usercmd_t,
    trMins: vec3_t,
    trMaxs: vec3_t,
    gravity: f32,
) -> qboolean {
    unsafe {
        let mut bottom: vec3_t;

        // Get parent's player state (bg-reachable `bgEntity_t`).
        let parentPS: *mut playerState_t = (*(*pVeh).m_pParentEntity).playerState;
        if parentPS.is_null() {
            // PORT-NOTE: Raven `Com_Error(ERR_DROP, "NULL PS in BG_FighterUpdate")`
            // (FighterNPC.c:123) - bg has no error route, so this returns the
            // fail value instead of dropping.
            return qfalse;
        }

        // If we have a pilot, take out gravity (flying craft).
        if !(*pVeh).m_pPilot.is_null() {
            (*parentPS).gravity = 0;
        } else if (*(*pVeh).m_pVehicleInfo).gravity != 0 {
            (*parentPS).gravity = (*(*pVeh).m_pVehicleInfo).gravity;
        } else {
            (*parentPS).gravity = gravity as c_int;
        }

        // Check landing surface.
        bottom = (*parentPS).origin;
        bottom[2] -= (*(*pVeh).m_pVehicleInfo).landingHeight;

        // Trace down for the landing surface (`MASK_NPCSOLID & ~CONTENTS_BODY`).
        let parent_num = (*(*pVeh).m_pParentEntity).s.number;
        let origin = (*parentPS).origin;
        pmc.traps.trace(
            &mut (*pVeh).m_LandTrace,
            &origin,
            &trMins,
            &trMaxs,
            &bottom,
            parent_num,
            MASK_NPCSOLID & !CONTENTS_BODY,
        );

        qtrue
    }
}

/// Raven `PredictedAngularDecrement`.
///
/// Source: `oracle/codemp/game/FighterNPC.c:237-273`
pub fn PredictedAngularDecrement(scale: f32, timeMod: f32, originalAngle: f32) -> f32 {
    let mut fixedBaseDec = originalAngle * 0.05f32;
    let mut r = 0.0f32;

    if fixedBaseDec < 0.0f32 {
        fixedBaseDec = -fixedBaseDec;
    }

    fixedBaseDec *= 1.0f32 + (1.0f32 - scale);

    if fixedBaseDec < 0.1f32 {
        fixedBaseDec = 0.1f32;
    }

    fixedBaseDec *= timeMod * 0.1f32;

    if originalAngle > 0.0f32 {
        r = originalAngle - fixedBaseDec;
        if r < 0.0f32 {
            r = 0.0f32;
        }
    } else if originalAngle < 0.0f32 {
        r = originalAngle + fixedBaseDec;
        if r > 0.0f32 {
            r = 0.0f32;
        }
    }

    r
}

/// Raven `FighterOverValidLandingSurface`.
///
/// Source: `oracle/codemp/game/FighterNPC.c:289-298`
pub fn FighterOverValidLandingSurface(pVeh: *mut Vehicle_t) -> qboolean {
    unsafe {
        if (*pVeh).m_LandTrace.fraction < 1.0f32
            && (*pVeh).m_LandTrace.plane.normal[2] >= MIN_LANDING_SLOPE
        {
            qtrue
        } else {
            qfalse
        }
    }
}

/// Raven `FighterIsLanded`.
///
/// Source: `oracle/codemp/game/FighterNPC.c:300-308`
pub fn FighterIsLanded(pVeh: *mut Vehicle_t, parentPS: *mut playerState_t) -> qboolean {
    unsafe {
        if FighterOverValidLandingSurface(pVeh) == qtrue && (*parentPS).speed == 0.0f32 {
            qtrue
        } else {
            qfalse
        }
    }
}

/// Raven `FighterIsLanding`.
///
/// The `Inhabited(pVeh)` clause is `#ifdef QAGAME` (Cgame predicts only the local
/// driver, so it's always inhabited); host-gated.
/// Source: `oracle/codemp/game/FighterNPC.c:310-323`
pub fn FighterIsLanding(
    host: BgHost,
    pVeh: *mut Vehicle_t,
    parentPS: *mut playerState_t,
) -> qboolean {
    unsafe {
        if FighterOverValidLandingSurface(pVeh) == qtrue
            && (host != BgHost::Game || Inhabited(pVeh) != qfalse)
            && ((*pVeh).m_ucmd.forwardmove < 0 || (*pVeh).m_ucmd.upmove < 0)
            && (*parentPS).speed <= MIN_LANDING_SPEED
        {
            qtrue
        } else {
            qfalse
        }
    }
}

/// Raven `FighterIsLaunching`.
///
/// The `Inhabited(pVeh)` clause is `#ifdef QAGAME`; host-gated.
/// Source: `oracle/codemp/game/FighterNPC.c:325-338`
pub fn FighterIsLaunching(
    host: BgHost,
    pVeh: *mut Vehicle_t,
    parentPS: *mut playerState_t,
) -> qboolean {
    unsafe {
        if FighterOverValidLandingSurface(pVeh) == qtrue
            && (host != BgHost::Game || Inhabited(pVeh) != qfalse)
            && (*pVeh).m_ucmd.upmove > 0
            && (*parentPS).speed <= 200.0f32
        {
            qtrue
        } else {
            qfalse
        }
    }
}

/// Raven `FighterSuspended`.
///
/// The whole body is `#ifdef QAGAME` (reads the parent gentity's `spawnflags`);
/// the cgame `#elif CGAME` arm returns qfalse. Host-switched: the Game host reads
/// spawnflags via the `entity_spawnflags` upcall.
/// Source: `oracle/codemp/game/FighterNPC.c:340-355`
pub fn FighterSuspended(
    pmc: &mut PmoveContext,
    pVeh: *mut Vehicle_t,
    parentPS: *mut playerState_t,
) -> qboolean {
    unsafe {
        if pmc.bg.host != BgHost::Game {
            return qfalse;
        }
        if (*pVeh).m_pPilot.is_null()
            && (*parentPS).speed == 0.0f32
            && (*pVeh).m_ucmd.forwardmove <= 0
            && !(*pVeh).m_pParentEntity.is_null()
        {
            let parent_num = (*(*pVeh).m_pParentEntity).s.number;
            if (pmc.callbacks.entity_spawnflags(parent_num) & 2) != 0 {
                return qtrue;
            }
        }
        qfalse
    }
}

/// Raven `FighterWingMalfunctionCheck`.
///
/// Source: `oracle/codemp/game/FighterNPC.c:890-915`
pub fn FighterWingMalfunctionCheck(pVeh: *mut Vehicle_t, parentPS: *mut playerState_t) {
    unsafe {
        let mut mPitchOverride = 1.0f32;
        let mut mYawOverride = 1.0f32;
        BG_VehicleTurnRateForSpeed(
            pVeh,
            (*parentPS).speed,
            &mut mPitchOverride,
            &mut mYawOverride,
        );

        // `serverTime*0.001` is a double in C; sin is double; narrows once.
        // Source: oracle/codemp/game/FighterNPC.c:896-913
        if ((*parentPS).brokenLimbs & (1 << 6)) != 0 {
            // SHIPSURF_DAMAGE_RIGHT_HEAVY
            let ptr = (*pVeh).m_vOrientation.add(2);
            *ptr = (*ptr as f64
                + (((*pVeh).m_ucmd.serverTime as f64 * 0.001).sin() + 1.0)
                    * (*pVeh).m_fTimeModifier as f64
                    * mYawOverride as f64
                    * 50.0) as f32;
        } else if ((*parentPS).brokenLimbs & (1 << 2)) != 0 {
            // SHIPSURF_DAMAGE_RIGHT_LIGHT
            let ptr = (*pVeh).m_vOrientation.add(2);
            *ptr = (*ptr as f64
                + (((*pVeh).m_ucmd.serverTime as f64 * 0.001).sin() + 1.0)
                    * (*pVeh).m_fTimeModifier as f64
                    * mYawOverride as f64
                    * 12.5) as f32;
        }

        if ((*parentPS).brokenLimbs & (1 << 7)) != 0 {
            // SHIPSURF_DAMAGE_LEFT_HEAVY
            let ptr = (*pVeh).m_vOrientation.add(2);
            *ptr = (*ptr as f64
                - (((*pVeh).m_ucmd.serverTime as f64 * 0.001).sin() + 1.0)
                    * (*pVeh).m_fTimeModifier as f64
                    * mYawOverride as f64
                    * 50.0) as f32;
        } else if ((*parentPS).brokenLimbs & (1 << 3)) != 0 {
            // SHIPSURF_DAMAGE_LEFT_LIGHT
            let ptr = (*pVeh).m_vOrientation.add(2);
            *ptr = (*ptr as f64
                - (((*pVeh).m_ucmd.serverTime as f64 * 0.001).sin() + 1.0)
                    * (*pVeh).m_fTimeModifier as f64
                    * mYawOverride as f64
                    * 12.5) as f32;
        }
    }
}

/// Raven `FighterNoseMalfunctionCheck`.
///
/// Source: `oracle/codemp/game/FighterNPC.c:917-933`
pub fn FighterNoseMalfunctionCheck(pVeh: *mut Vehicle_t, parentPS: *mut playerState_t) {
    unsafe {
        let mut mPitchOverride = 1.0f32;
        let mut mYawOverride = 1.0f32;
        BG_VehicleTurnRateForSpeed(
            pVeh,
            (*parentPS).speed,
            &mut mPitchOverride,
            &mut mYawOverride,
        );

        if ((*parentPS).brokenLimbs & (1 << 4)) != 0 {
            // SHIPSURF_DAMAGE_FRONT_HEAVY
            let ptr = (*pVeh).m_vOrientation.add(0);
            *ptr = (*ptr as f64
                + ((*pVeh).m_ucmd.serverTime as f64 * 0.001).sin()
                    * (*pVeh).m_fTimeModifier as f64
                    * mPitchOverride as f64
                    * 50.0) as f32;
        } else if ((*parentPS).brokenLimbs & (1 << 0)) != 0 {
            // SHIPSURF_DAMAGE_FRONT_LIGHT
            let ptr = (*pVeh).m_vOrientation.add(0);
            *ptr = (*ptr as f64
                + ((*pVeh).m_ucmd.serverTime as f64 * 0.001).sin()
                    * (*pVeh).m_fTimeModifier as f64
                    * mPitchOverride as f64
                    * 20.0) as f32;
        }
    }
}

/// Raven `FighterDamageRoutine`.
///
/// The land-while-broken suicide is `#ifdef QAGAME` and reaches the parent's
/// `client->ps.origin` with a NULL attacker; host-switched through the
/// `veh_fighter_crash_suicide` upcall. `parent_num` is the parent's entity
/// number. Source: `oracle/codemp/game/FighterNPC.c:935-1089`
pub fn FighterDamageRoutine(
    pmc: &mut PmoveContext,
    pVeh: *mut Vehicle_t,
    parent_num: c_int,
    parentPS: *mut playerState_t,
    riderPS: *mut playerState_t,
    isDead: qboolean,
) {
    unsafe {
        if (*pVeh).m_iRemovedSurfaces == 0 {
            // Still in one piece
            if !(*pVeh).m_pParentEntity.is_null() && isDead != qfalse {
                // Death spiral
                (*pVeh).m_ucmd.upmove = 0;
                let num = parent_num;

                if num % 3 == 0 {
                    *(*pVeh).m_vOrientation.add(0) += (*pVeh).m_fTimeModifier;
                    if BG_UnrestrainedPitchRoll(riderPS, pVeh, &*pmc.bg) == qfalse {
                        if *(*pVeh).m_vOrientation.add(0) > 60.0f32 {
                            *(*pVeh).m_vOrientation.add(0) = 60.0f32;
                        }
                    }
                } else if num % 2 == 0 {
                    *(*pVeh).m_vOrientation.add(0) -= (*pVeh).m_fTimeModifier;
                    if BG_UnrestrainedPitchRoll(riderPS, pVeh, &*pmc.bg) == qfalse {
                        if *(*pVeh).m_vOrientation.add(0) > -60.0f32 {
                            *(*pVeh).m_vOrientation.add(0) = -60.0f32;
                        }
                    }
                }

                if num % 2 != 0 {
                    *(*pVeh).m_vOrientation.add(1) += (*pVeh).m_fTimeModifier;
                    *(*pVeh).m_vOrientation.add(2) += (*pVeh).m_fTimeModifier * 4.0f32;
                } else {
                    *(*pVeh).m_vOrientation.add(1) -= (*pVeh).m_fTimeModifier;
                    *(*pVeh).m_vOrientation.add(2) -= (*pVeh).m_fTimeModifier * 4.0f32;
                }
            }
            return;
        }

        // We have at least one broken piece
        (*pVeh).m_ucmd.upmove = 0;

        // If off the ground and not suspended, pitch down
        if (*pVeh).m_LandTrace.fraction >= 0.1f32 {
            if FighterSuspended(pmc, pVeh, parentPS) == qfalse {
                let num = parent_num;

                if num % 3 == 0 {
                    *(*pVeh).m_vOrientation.add(0) += (*pVeh).m_fTimeModifier;
                    if BG_UnrestrainedPitchRoll(riderPS, pVeh, &*pmc.bg) == qfalse {
                        if *(*pVeh).m_vOrientation.add(0) > 60.0f32 {
                            *(*pVeh).m_vOrientation.add(0) = 60.0f32;
                        }
                    }
                } else if num % 4 == 0 {
                    *(*pVeh).m_vOrientation.add(0) -= (*pVeh).m_fTimeModifier;
                    if BG_UnrestrainedPitchRoll(riderPS, pVeh, &*pmc.bg) == qfalse {
                        // Raven's own quirk: snaps to -60 whenever pitch is ABOVE it
                        if *(*pVeh).m_vOrientation.add(0) > -60.0f32 {
                            *(*pVeh).m_vOrientation.add(0) = -60.0f32;
                        }
                    }
                }
            }
        }

        // QAGAME: if you land at all while pieces of your ship are missing, then die.
        // Source: oracle/codemp/game/FighterNPC.c:1021-1032
        if (*pVeh).m_LandTrace.fraction < 1.0f32 {
            if pmc.bg.host == BgHost::Game {
                pmc.callbacks.veh_fighter_crash_suicide(parent_num);
            }
        }

        // Wing damage effects
        let c = SHIPSURF_BROKEN_C;
        let d = SHIPSURF_BROKEN_D;
        let e = SHIPSURF_BROKEN_E;
        let f = SHIPSURF_BROKEN_F;

        if (((*pVeh).m_iRemovedSurfaces & c) != 0 || ((*pVeh).m_iRemovedSurfaces & d) != 0)
            && (((*pVeh).m_iRemovedSurfaces & e) != 0 || ((*pVeh).m_iRemovedSurfaces & f) != 0)
        {
            // Wings on both sides broken
            let mut factor = 2.0f32;
            if ((*pVeh).m_iRemovedSurfaces & e) != 0
                && ((*pVeh).m_iRemovedSurfaces & f) != 0
                && ((*pVeh).m_iRemovedSurfaces & c) != 0
                && ((*pVeh).m_iRemovedSurfaces & d) != 0
            {
                factor *= 2.0f32;
            }
            let num = parent_num;
            if num % 2 == 0 || num % 6 == 0 {
                factor *= 4.0f32;
            }
            *(*pVeh).m_vOrientation.add(2) += (*pVeh).m_fTimeModifier * factor;
        } else if ((*pVeh).m_iRemovedSurfaces & c) != 0 || ((*pVeh).m_iRemovedSurfaces & d) != 0 {
            // Left wing broken
            let mut factor = 2.0f32;
            if ((*pVeh).m_iRemovedSurfaces & c) != 0 && ((*pVeh).m_iRemovedSurfaces & d) != 0 {
                factor *= 2.0f32;
            }
            let num = parent_num;
            if num % 2 == 0 || num % 6 == 0 {
                factor *= 4.0f32;
            }
            *(*pVeh).m_vOrientation.add(2) += factor * (*pVeh).m_fTimeModifier;
        } else if ((*pVeh).m_iRemovedSurfaces & e) != 0 || ((*pVeh).m_iRemovedSurfaces & f) != 0 {
            // Right wing broken
            let mut factor = 2.0f32;
            if ((*pVeh).m_iRemovedSurfaces & e) != 0 && ((*pVeh).m_iRemovedSurfaces & f) != 0 {
                factor *= 2.0f32;
            }
            let num = parent_num;
            if num % 2 == 0 || num % 6 == 0 {
                factor *= 4.0f32;
            }
            *(*pVeh).m_vOrientation.add(2) -= factor * (*pVeh).m_fTimeModifier;
        }
    }
}

/// Raven `FighterYawAdjust`.
///
/// Source: `oracle/codemp/game/FighterNPC.c:1299-1323`
pub fn FighterYawAdjust(
    pVeh: *mut Vehicle_t,
    riderPS: *mut playerState_t,
    parentPS: *mut playerState_t,
) {
    unsafe {
        let angDif = AngleSubtract(*(*pVeh).m_vOrientation.add(1), (*riderPS).viewangles[1]);

        if !parentPS.is_null() && (*parentPS).speed != 0.0f32 {
            let mut s = (*parentPS).speed;
            let maxDif = (*(*pVeh).m_pVehicleInfo).turningSpeed * 0.8f32;

            if s < 0.0f32 {
                s = -s;
            }
            let mut scaled = angDif * s / (*(*pVeh).m_pVehicleInfo).speedMax;
            if scaled > maxDif {
                scaled = maxDif;
            } else if scaled < -maxDif {
                scaled = -maxDif;
            }
            *(*pVeh).m_vOrientation.add(1) = AngleNormalize180(
                *(*pVeh).m_vOrientation.add(1) - scaled * ((*pVeh).m_fTimeModifier * 0.2f32),
            );
        }
    }
}

/// Raven `FighterPitchAdjust`.
///
/// Source: `oracle/codemp/game/FighterNPC.c:1325-1349`
pub fn FighterPitchAdjust(
    pVeh: *mut Vehicle_t,
    riderPS: *mut playerState_t,
    parentPS: *mut playerState_t,
) {
    unsafe {
        let angDif = AngleSubtract(*(*pVeh).m_vOrientation.add(0), (*riderPS).viewangles[0]);

        if !parentPS.is_null() && (*parentPS).speed != 0.0f32 {
            let mut s = (*parentPS).speed;
            let maxDif = (*(*pVeh).m_pVehicleInfo).turningSpeed * 0.8f32;

            if s < 0.0f32 {
                s = -s;
            }
            let mut scaled = angDif * s / (*(*pVeh).m_pVehicleInfo).speedMax;
            if scaled > maxDif {
                scaled = maxDif;
            } else if scaled < -maxDif {
                scaled = -maxDif;
            }
            *(*pVeh).m_vOrientation.add(0) = AngleNormalize360(
                *(*pVeh).m_vOrientation.add(0) - scaled * ((*pVeh).m_fTimeModifier * 0.2f32),
            );
        }
    }
}

/// Raven `ProcessMoveCommands` — move the fighter forward/back/up/down.
///
/// Raven: MP RULE — ALL PROCESSMOVECOMMANDS FUNCTIONS MUST BE BG-COMPATIBLE.
/// `curTime` is `pm->cmd.serverTime` (== `m_ucmd.serverTime`, the vehicle ucmd
/// copy). The `#ifdef QAGAME` islands (takeoff/turbo sound, the `FighterSuspended`
/// / `!Inhabited` speed branches, the strafe `Inhabited` clause, and the
/// pitch-speed/gravity block) host-switch on `pmc.bg.host`.
/// Source: `oracle/codemp/game/FighterNPC.c:370-887`;
/// cgame arm `oracle/codemp/cgame/JK2_cgame.vcproj` (`FighterNPC.c`)
pub fn ProcessMoveCommands(pmc: &mut PmoveContext, pVeh: *mut Vehicle_t) {
    unsafe {
        let host = pmc.bg.host;
        let parent = (*pVeh).m_pParentEntity;
        let parent_num = (*parent).s.number;
        let curTime: c_int = (*pVeh).m_ucmd.serverTime;
        let parentPS: *mut playerState_t = (*parent).playerState;
        let vi = (*pVeh).m_pVehicleInfo;

        // Going to Hyperspace: totally override movement.
        if (*parentPS).hyperSpaceTime != 0 && curTime - (*parentPS).hyperSpaceTime < HYPERSPACE_TIME
        {
            let timeFrac = (curTime - (*parentPS).hyperSpaceTime) as f32 / HYPERSPACE_TIME as f32;
            if timeFrac < HYPERSPACE_TELEPORT_FRAC {
                if (*parentPS).eFlags2 & EF2_HYPERSPACE == 0 {
                    (*parentPS).speed = 0.0;
                } else {
                    // QAGAME hyperspace sound is commented out in the oracle.
                    //TODO: Port FighterNPC CGAME hyperspace trap_S_StartSound arm
                    // Source: oracle/codemp/game/FighterNPC.c:414-424
                    // (client-only local sound; BgTraps carries no S_StartSound yet)
                    (*parentPS).speed = HYPERSPACE_SPEED;
                }
            } else {
                (*parentPS).speed = 200.0
                    + ((1.0 - timeFrac)
                        * (1.0 / HYPERSPACE_TELEPORT_FRAC)
                        * (HYPERSPACE_SPEED - 200.0));
                if VectorLength((*parentPS).velocity) < (*parentPS).speed {
                    _VectorScale(
                        (*parentPS).moveDir,
                        (*parentPS).speed,
                        &mut (*parentPS).velocity,
                    );
                }
            }
            return;
        }

        if (*pVeh).m_iDropTime >= curTime {
            (*parentPS).speed = 0.0;
            (*parentPS).gravity = 800;
            return;
        }

        let isLandingOrLaunching: qboolean = if FighterIsLanding(host, pVeh, parentPS) != qfalse
            || FighterIsLaunching(host, pVeh, parentPS) != qfalse
        {
            qtrue
        } else {
            qfalse
        };

        // If we are hitting the ground, just allow the fighter to go up and down.
        if isLandingOrLaunching != qfalse
            && ((*pVeh).m_ucmd.forwardmove <= 0
                || (*pVeh).m_LandTrace.fraction <= FIGHTER_MIN_TAKEOFF_FRACTION)
        {
            if (*pVeh).m_ucmd.upmove > 0 {
                if (*parentPS).velocity[2] <= 0.0 && (*vi).soundTakeOff != 0 {
                    // taking off for the first time (QAGAME sound island)
                    if host == BgHost::Game {
                        pmc.callbacks
                            .entity_sound(parent_num, CHAN_AUTO, (*vi).soundTakeOff);
                    }
                }
                (*parentPS).velocity[2] += (*vi).acceleration * (*pVeh).m_fTimeModifier;
            } else if (*pVeh).m_ucmd.upmove < 0 {
                (*parentPS).velocity[2] -= (*vi).acceleration * (*pVeh).m_fTimeModifier;
            } else if (*pVeh).m_ucmd.forwardmove < 0 {
                if (*pVeh).m_LandTrace.fraction != 0.0 {
                    (*parentPS).velocity[2] -= (*vi).acceleration * (*pVeh).m_fTimeModifier;
                }
                if (*pVeh).m_LandTrace.fraction <= FIGHTER_MIN_TAKEOFF_FRACTION {
                    (*parentPS).velocity[2] = PredictedAngularDecrement(
                        (*pVeh).m_LandTrace.fraction,
                        (*pVeh).m_fTimeModifier * 5.0,
                        (*parentPS).velocity[2],
                    );
                    (*parentPS).speed = 0.0;
                }
            }
            *(*pVeh).m_vOrientation.add(0) = PredictedAngularDecrement(
                0.7,
                (*pVeh).m_fTimeModifier * 10.0,
                *(*pVeh).m_vOrientation.add(0),
            );
            return;
        }

        if (*pVeh).m_ucmd.upmove > 0 && (*vi).turboSpeed != 0.0 {
            if (curTime - (*pVeh).m_iTurboTime) > (*vi).turboRecharge {
                (*pVeh).m_iTurboTime = curTime + (*vi).turboDuration;
                if (*vi).soundTurbo != 0 {
                    // QAGAME turbo sound island
                    if host == BgHost::Game {
                        pmc.callbacks
                            .entity_sound(parent_num, CHAN_AUTO, (*vi).soundTurbo);
                    }
                }
            }
        }

        let mut speedInc = (*vi).acceleration * (*pVeh).m_fTimeModifier;
        let mut speedMax;
        if curTime < (*pVeh).m_iTurboTime {
            speedMax = (*vi).turboSpeed;
            speedInc = ((*vi).acceleration * 2.0) * (*pVeh).m_fTimeModifier;
            (*pVeh).m_ucmd.forwardmove = 127;
            (*parentPS).eFlags |= EF_JETPACK_ACTIVE;
        } else {
            speedMax = (*vi).speedMax;
            if (*parentPS).eFlags & EF_JETPACK_ACTIVE != 0 {
                (*parentPS).eFlags &= !EF_JETPACK_ACTIVE;
            }
        }
        let mut speedIdleDec = (*vi).decelIdle * (*pVeh).m_fTimeModifier;
        let speedIdle = (*vi).speedIdle;
        let speedIdleAccel = (*vi).accelIdle * (*pVeh).m_fTimeModifier;
        let speedMin = (*vi).speedMin;

        if (*parentPS).brokenLimbs & (1 << 5) != 0 {
            // SHIPSURF_DAMAGE_BACK_HEAVY (=5)
            speedMax *= 0.8;
        } else if (*parentPS).brokenLimbs & (1 << 1) != 0 {
            // SHIPSURF_DAMAGE_BACK_LIGHT (=1)
            speedMax *= 0.6;
        }

        if (*pVeh).m_iRemovedSurfaces != 0 || (*parentPS).electrifyTime >= curTime {
            // go out of control
            (*parentPS).speed += speedInc;
            (*pVeh).m_ucmd.forwardmove = 127;
        } else if host == BgHost::Game && FighterSuspended(pmc, pVeh, parentPS) != qfalse {
            // #ifdef QAGAME
            (*parentPS).speed = 0.0;
            (*pVeh).m_ucmd.forwardmove = 0;
        } else if host == BgHost::Game && Inhabited(pVeh) == qfalse && (*parentPS).speed > 0.0 {
            // #ifdef QAGAME — pilot jumped out while moving forward, keep the throttle locked
            (*pVeh).m_ucmd.forwardmove = 127;
        } else if ((*parentPS).speed != 0.0
            || (*parentPS).groundEntityNum == ENTITYNUM_NONE
            || (*pVeh).m_ucmd.forwardmove != 0
            || (*pVeh).m_ucmd.upmove > 0)
            && (*pVeh).m_LandTrace.fraction >= 0.05
        {
            if (*pVeh).m_ucmd.forwardmove > 0 && speedInc != 0.0 {
                (*parentPS).speed += speedInc;
                (*pVeh).m_ucmd.forwardmove = 127;
            } else if (*pVeh).m_ucmd.forwardmove < 0 || (*pVeh).m_ucmd.upmove < 0 {
                if (*pVeh).m_ucmd.upmove < 0 {
                    if (*pVeh).m_ucmd.forwardmove != 0 {
                        speedInc += (*vi).braking;
                        speedIdleDec += (*vi).braking;
                    } else {
                        speedInc = (*vi).braking;
                        speedIdleDec = (*vi).braking;
                    }
                }
                if (*parentPS).speed > speedIdle {
                    (*parentPS).speed -= speedInc;
                } else if (*parentPS).speed > speedMin {
                    if FighterOverValidLandingSurface(pVeh) != qfalse {
                        (*parentPS).speed -= speedInc;
                    } else {
                        (*parentPS).speed -= speedIdleDec;
                        if (*parentPS).speed < MIN_LANDING_SPEED {
                            (*parentPS).speed = MIN_LANDING_SPEED;
                        }
                    }
                }
                if (*vi).r#type == vehicleType_t::VH_FIGHTER {
                    (*pVeh).m_ucmd.forwardmove = 127;
                } else if speedMin >= 0.0 {
                    (*pVeh).m_ucmd.forwardmove = 0;
                }
            } else if (*vi).throttleSticks != 0.0 {
                if (*parentPS).speed <= MIN_LANDING_SPEED {
                    if FighterOverValidLandingSurface(pVeh) != qfalse {
                        if (*parentPS).speed > 0.0 {
                            (*parentPS).speed -= speedIdleDec;
                        } else if (*parentPS).speed < 0.0 {
                            (*parentPS).speed += speedIdleDec;
                        }
                    } else if (*parentPS).speed < speedIdle {
                        (*parentPS).speed += speedIdleAccel;
                        if (*parentPS).speed > speedIdle {
                            (*parentPS).speed = speedIdle;
                        }
                    }
                }
            } else {
                if ((*pVeh).m_LandTrace.fraction >= 1.0
                    || (*pVeh).m_LandTrace.plane.normal[2] < MIN_LANDING_SLOPE)
                    && speedIdle > 0.0
                {
                    if (*parentPS).speed < speedIdle {
                        (*parentPS).speed += speedIdleAccel;
                        if (*parentPS).speed > speedIdle {
                            (*parentPS).speed = speedIdle;
                        }
                    } else if (*parentPS).speed > 0.0 {
                        (*parentPS).speed -= speedIdleDec;
                        if (*parentPS).speed < speedIdle {
                            (*parentPS).speed = speedIdle;
                        }
                    }
                } else if (*parentPS).speed > 0.0 {
                    (*parentPS).speed -= speedIdleDec;
                } else if (*parentPS).speed < 0.0 {
                    (*parentPS).speed += speedIdleDec;
                }
            }
        } else {
            if (*pVeh).m_ucmd.forwardmove < 0 {
                (*pVeh).m_ucmd.forwardmove = 0;
            }
            if (*pVeh).m_ucmd.upmove < 0 {
                (*pVeh).m_ucmd.upmove = 0;
            }
            // `#ifndef _JK2MP` strafe-clear is SP-only dead code (dropped §10)
        }

        // STRAFING
        if (*vi).strafePerc != 0.0
            && (host != BgHost::Game || Inhabited(pVeh) != qfalse)
            && (*pVeh).m_iRemovedSurfaces == 0
            && (*parentPS).electrifyTime < curTime
            && (*parentPS).vehTurnaroundTime < curTime
            && ((*pVeh).m_LandTrace.fraction >= 1.0
                || (*pVeh).m_LandTrace.plane.normal[2] < MIN_LANDING_SLOPE
                || (*parentPS).speed > MIN_LANDING_SPEED)
            && (*pVeh).m_ucmd.rightmove != 0
        {
            let mut vAngles: vec3_t = [
                *(*pVeh).m_vOrientation.add(0),
                *(*pVeh).m_vOrientation.add(1),
                *(*pVeh).m_vOrientation.add(2),
            ];
            let mut strafeSpeed = ((*vi).strafePerc * speedMax) * 5.0;
            vAngles[0] = 0.0; // PITCH
            vAngles[2] = 0.0; // ROLL
            let mut vRight: vec3_t = [0.0; 3];
            AngleVectors(vAngles, None, Some(&mut vRight), None);

            if (*pVeh).m_ucmd.rightmove > 0 {
                if ((*parentPS).hackingTime as f32) > -MAX_STRAFE_TIME {
                    let curStrafeSpeed = _DotProduct((*parentPS).velocity, vRight);
                    if curStrafeSpeed > 0.0 {
                        strafeSpeed -= curStrafeSpeed;
                    }
                    if strafeSpeed > 0.0 {
                        _VectorMA(
                            (*parentPS).velocity,
                            strafeSpeed * (*pVeh).m_fTimeModifier,
                            vRight,
                            &mut (*parentPS).velocity,
                        );
                    }
                    (*parentPS).hackingTime =
                        ((*parentPS).hackingTime as f32 - 50.0 * (*pVeh).m_fTimeModifier) as c_int;
                }
            } else if ((*parentPS).hackingTime as f32) < MAX_STRAFE_TIME {
                let curStrafeSpeed = _DotProduct((*parentPS).velocity, vRight);
                if curStrafeSpeed < 0.0 {
                    strafeSpeed += curStrafeSpeed;
                }
                if strafeSpeed > 0.0 {
                    _VectorMA(
                        (*parentPS).velocity,
                        -strafeSpeed * (*pVeh).m_fTimeModifier,
                        vRight,
                        &mut (*parentPS).velocity,
                    );
                }
                (*parentPS).hackingTime =
                    ((*parentPS).hackingTime as f32 + 50.0 * (*pVeh).m_fTimeModifier) as c_int;
            }
        } else if (*parentPS).hackingTime > 0 {
            (*parentPS).hackingTime =
                ((*parentPS).hackingTime as f32 - 50.0 * (*pVeh).m_fTimeModifier) as c_int;
            if (*parentPS).hackingTime < 0 {
                (*parentPS).hackingTime = 0;
            }
        } else if (*parentPS).hackingTime < 0 {
            (*parentPS).hackingTime =
                ((*parentPS).hackingTime as f32 + 50.0 * (*pVeh).m_fTimeModifier) as c_int;
            if (*parentPS).hackingTime > 0 {
                (*parentPS).hackingTime = 0;
            }
        }

        if (*parentPS).speed > speedMax {
            (*parentPS).speed = speedMax;
        } else if (*parentPS).speed < speedMin {
            (*parentPS).speed = speedMin;
        }

        // QAGAME pitch-speed + gravity block; the CGAME `#else` is `gravity = 0`.
        // Source: oracle/codemp/game/FighterNPC.c:813-882
        if host == BgHost::Game {
            if (*(*pVeh).m_vOrientation.add(0) * 0.1) > 10.0 {
                if pmc.callbacks.fighter_is_in_space(parent_num) != qfalse {
                    // in space, do nothing with speed based on pitch
                } else {
                    let mut mult = *(*pVeh).m_vOrientation.add(0) * 0.1;
                    if mult < 1.0 {
                        mult = 1.0;
                    }
                    (*parentPS).speed = PredictedAngularDecrement(
                        mult,
                        (*pVeh).m_fTimeModifier * 10.0,
                        (*parentPS).speed,
                    );
                }
            }

            if (*pVeh).m_iRemovedSurfaces != 0 || (*parentPS).electrifyTime >= curTime {
                // going down
                if pmc.callbacks.fighter_is_in_space(parent_num) != qfalse {
                    if parent_num & 3 == 0 {
                        (*parentPS).gravity = 0;
                    } else if parent_num & 2 == 0 {
                        (*parentPS).gravity = -500;
                        (*parentPS).velocity[2] = 80.0;
                    } else {
                        (*parentPS).gravity = 500;
                        (*parentPS).velocity[2] = -80.0;
                    }
                } else {
                    (*parentPS).gravity = 500;
                    (*parentPS).velocity[2] = -80.0;
                }
            } else if FighterSuspended(pmc, pVeh, parentPS) != qfalse {
                (*parentPS).gravity = 0;
            } else if ((*parentPS).speed == 0.0 || (*parentPS).speed < speedIdle)
                && (*pVeh).m_ucmd.upmove <= 0
            {
                if pmc.callbacks.fighter_is_in_space(parent_num) != qfalse {
                    if FighterOverValidLandingSurface(pVeh) != qfalse {
                        (*parentPS).gravity = ((speedIdle - (*parentPS).speed) / 4.0) as c_int;
                    }
                } else {
                    (*parentPS).gravity = ((speedIdle - (*parentPS).speed) / 4.0) as c_int;
                }
            } else {
                (*parentPS).gravity = 0;
            }
        } else {
            (*parentPS).gravity = 0;
        }
    }
}

/// Raven `ProcessOrientCommands` — keep the fighter properly oriented.
///
/// Raven: MP RULE — ALL PROCESSORIENTCOMMANDS FUNCTIONS MUST BE BG-COMPATIBLE.
/// The `VEH_CONTROL_SCHEME_4` and `#ifndef _JK2MP` (SP) paths are dropped (§10).
/// `curTime` uses `m_ucmd.serverTime`; the rider is resolved with
/// `PM_BGEntForNum` (the QAGAME `_JK2MP` pattern). The two `FighterDamageRoutine`
/// calls and the `FighterIsInSpace` / `FighterSuspended` gates host-switch.
/// Source: `oracle/codemp/game/FighterNPC.c:1381-1835`;
/// cgame arm `oracle/codemp/cgame/JK2_cgame.vcproj` (`FighterNPC.c`)
pub fn ProcessOrientCommands(pmc: &mut PmoveContext, pVeh: *mut Vehicle_t) {
    unsafe {
        let host = pmc.bg.host;
        let parent = (*pVeh).m_pParentEntity;
        let parent_num = (*parent).s.number;
        let vi = (*pVeh).m_pVehicleInfo;
        let groundFraction = 0.1f32;
        let mut curRoll;
        // QAGAME `curTime = level.time`; CGAME `curTime = pm->cmd.serverTime`.
        // Source: oracle/codemp/game/FighterNPC.c:1396-1402
        let curTime: c_int = if host == BgHost::Game {
            pmc.callbacks.get_time()
        } else {
            (*pmc.pm).cmd.serverTime
        };

        // Resolve the rider (`if owner != NONE: PM_BGEntForNum(owner); if !rider: parent`).
        let mut rider: *mut bgEntity_t = core::ptr::null_mut();
        if (*parent).s.owner != ENTITYNUM_NONE {
            rider = pmc.PM_BGEntForNum((*parent).s.owner);
        }
        if rider.is_null() {
            rider = parent;
        }

        let parentPS: *mut playerState_t = (*parent).playerState;
        let riderPS: *mut playerState_t = (*rider).playerState;
        let isDead: qboolean = if (*parentPS).eFlags & EF_DEAD != 0 {
            qtrue
        } else {
            qfalse
        };

        // Going to Hyperspace.
        if (*parentPS).hyperSpaceTime != 0
            && (curTime - (*parentPS).hyperSpaceTime) < HYPERSPACE_TIME
        {
            *(*pVeh).m_vOrientation.add(0) = (*riderPS).viewangles[0];
            *(*pVeh).m_vOrientation.add(1) = (*riderPS).viewangles[1];
            *(*pVeh).m_vOrientation.add(2) = (*riderPS).viewangles[2];
            (*parentPS).viewangles = (*riderPS).viewangles;
            return;
        }

        if (*pVeh).m_iDropTime >= curTime {
            let ry = (*riderPS).viewangles[1];
            *(*pVeh).m_vOrientation.add(1) = ry;
            (*parentPS).viewangles[1] = ry;
            return;
        }

        let angleTimeMod = (*pVeh).m_fTimeModifier;

        if isDead != qfalse
            || (*parentPS).electrifyTime >= curTime
            || ((*vi).surfDestruction != 0
                && (*pVeh).m_iRemovedSurfaces != 0
                && (*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_C != 0
                && (*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_D != 0
                && (*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_E != 0
                && (*pVeh).m_iRemovedSurfaces & SHIPSURF_BROKEN_F != 0)
        {
            // all wings torn off
            FighterDamageRoutine(pmc, pVeh, parent_num, parentPS, riderPS, isDead);
            *(*pVeh).m_vOrientation.add(2) = AngleNormalize180(*(*pVeh).m_vOrientation.add(2));
            return;
        }

        if BG_UnrestrainedPitchRoll(riderPS, pVeh, &*pmc.bg) == qfalse {
            *(*pVeh).m_vOrientation.add(2) =
                PredictedAngularDecrement(0.95, angleTimeMod * 2.0, *(*pVeh).m_vOrientation.add(2));
        }

        let isLandingOrLanded: qboolean = if FighterIsLanding(host, pVeh, parentPS) != qfalse
            || FighterIsLanded(pVeh, parentPS) != qfalse
        {
            qtrue
        } else {
            qfalse
        };

        if isLandingOrLanded == qfalse {
            FighterWingMalfunctionCheck(pVeh, parentPS);

            let mut m: usize = 0;
            while m < 3 {
                let aVelDif = (*pVeh).m_vFullAngleVelocity[m];
                if aVelDif != 0.0 {
                    let dForVel = (aVelDif * 0.1) * (*pVeh).m_fTimeModifier;
                    if dForVel > 1.0 || dForVel < -1.0 {
                        *(*pVeh).m_vOrientation.add(m) += dForVel;
                        *(*pVeh).m_vOrientation.add(m) =
                            AngleNormalize180(*(*pVeh).m_vOrientation.add(m));
                        if m == 0 {
                            if *(*pVeh).m_vOrientation.add(m) > 90.0
                                && (*(*pVeh).m_vOrientation.add(m) - dForVel) < 90.0
                            {
                                *(*pVeh).m_vOrientation.add(m) = 90.0;
                                (*pVeh).m_vFullAngleVelocity[m] = -(*pVeh).m_vFullAngleVelocity[m];
                            }
                        }
                        (*pVeh).m_vFullAngleVelocity[m] -= dForVel;
                    } else {
                        (*pVeh).m_vFullAngleVelocity[m] = 0.0;
                    }
                }
                m += 1;
            }
        } else {
            VectorClear(&mut (*pVeh).m_vFullAngleVelocity);
        }

        curRoll = *(*pVeh).m_vOrientation.add(2);

        // If landed, we can only take off.
        if isLandingOrLanded != qfalse
            && (*pVeh).m_iRemovedSurfaces == 0
            && (*parentPS).electrifyTime < curTime
        {
            if (*parentPS).speed > 0.0 {
                if (*pVeh).m_LandTrace.fraction < 0.3 {
                    *(*pVeh).m_vOrientation.add(0) = 0.0;
                } else {
                    *(*pVeh).m_vOrientation.add(0) = PredictedAngularDecrement(
                        0.83,
                        angleTimeMod * 10.0,
                        *(*pVeh).m_vOrientation.add(0),
                    );
                }
            }
            if (*pVeh).m_LandTrace.fraction > 0.1
                || (*pVeh).m_LandTrace.plane.normal[2] < MIN_LANDING_SLOPE
            {
                FighterYawAdjust(pVeh, riderPS, parentPS);
            }
        } else if ((*pVeh).m_iRemovedSurfaces != 0 || (*parentPS).electrifyTime >= curTime)
            && (parent_num % 2 == 0 || parent_num % 6 == 0)
        {
            // spiralling out of control: no yaw control
        } else if !(*pVeh).m_pPilot.is_null()
            && (*(*pVeh).m_pPilot).s.number < MAX_CLIENTS as c_int
            && (*parentPS).speed > 0.0
        {
            if BG_UnrestrainedPitchRoll(riderPS, pVeh, &*pmc.bg) != qfalse {
                *(*pVeh).m_vOrientation.add(0) = (*riderPS).viewangles[0];
                *(*pVeh).m_vOrientation.add(1) = (*riderPS).viewangles[1];
                *(*pVeh).m_vOrientation.add(2) = (*riderPS).viewangles[2];
                (*parentPS).viewangles = (*riderPS).viewangles;
                curRoll = *(*pVeh).m_vOrientation.add(2);
                FighterNoseMalfunctionCheck(pVeh, parentPS);
            } else {
                FighterYawAdjust(pVeh, riderPS, parentPS);

                if FighterOverValidLandingSurface(pVeh) == qfalse
                    || (*parentPS).speed > MIN_LANDING_SPEED
                {
                    FighterPitchAdjust(pVeh, riderPS, parentPS);
                    FighterNoseMalfunctionCheck(pVeh, parentPS);

                    let mut fYawDelta = AngleSubtract(
                        *(*pVeh).m_vOrientation.add(1),
                        (*pVeh).m_vPrevOrientation[1],
                    );
                    if fYawDelta > 8.0 {
                        fYawDelta = 8.0;
                    } else if fYawDelta < -8.0 {
                        fYawDelta = -8.0;
                    }
                    curRoll -= fYawDelta;
                    curRoll = PredictedAngularDecrement(0.93, angleTimeMod * 2.0, curRoll);

                    if (*vi).rollLimit != -1.0 {
                        if curRoll > (*vi).rollLimit {
                            curRoll = (*vi).rollLimit;
                        } else if curRoll < -(*vi).rollLimit {
                            curRoll = -(*vi).rollLimit;
                        }
                    }
                }
            }
        }

        // If directly impacting the ground, even out the pitch.
        if isLandingOrLanded != qfalse
            && isDead == qfalse
            && (*parentPS).electrifyTime < curTime
            && ((*vi).surfDestruction == 0 || (*pVeh).m_iRemovedSurfaces == 0)
        {
            if *(*pVeh).m_vOrientation.add(0) > 0.0 {
                *(*pVeh).m_vOrientation.add(0) = PredictedAngularDecrement(
                    0.2,
                    angleTimeMod * 10.0,
                    *(*pVeh).m_vOrientation.add(0),
                );
            } else {
                *(*pVeh).m_vOrientation.add(0) = PredictedAngularDecrement(
                    0.75,
                    angleTimeMod * 10.0,
                    *(*pVeh).m_vOrientation.add(0),
                );
            }
        }

        // No one aboard and up in the sky: pitch forward as it tumbles down (QAGAME).
        if host == BgHost::Game
            && Inhabited(pVeh) == qfalse
            && (*pVeh).m_LandTrace.fraction >= groundFraction
            && pmc.callbacks.fighter_is_in_space(parent_num) == qfalse
            && FighterSuspended(pmc, pVeh, parentPS) == qfalse
        {
            (*pVeh).m_ucmd.upmove = 0;
            *(*pVeh).m_vOrientation.add(0) += (*pVeh).m_fTimeModifier;
            if BG_UnrestrainedPitchRoll(riderPS, pVeh, &*pmc.bg) == qfalse
                && *(*pVeh).m_vOrientation.add(0) > 60.0
            {
                *(*pVeh).m_vOrientation.add(0) = 60.0;
            }
        }

        if (*parentPS).hackingTime == 0 {
            *(*pVeh).m_vOrientation.add(2) = curRoll;
            if *(*pVeh).m_vOrientation.add(2) != 0.0 {
                if ((*pVeh).m_iRemovedSurfaces != 0 || (*parentPS).electrifyTime >= curTime)
                    && (parent_num % 2 == 0 || parent_num % 6 == 0)
                {
                    // spiralling out of control: leave YAW alone
                } else if BG_UnrestrainedPitchRoll(riderPS, pVeh, &*pmc.bg) == qfalse {
                    *(*pVeh).m_vOrientation.add(1) -=
                        (*(*pVeh).m_vOrientation.add(2) * 0.05) * (*pVeh).m_fTimeModifier;
                }
            }
        } else {
            let strafeRoll = ((*parentPS).hackingTime as f32 / MAX_STRAFE_TIME) * (*vi).rollLimit;
            let strafeDif = AngleSubtract(strafeRoll, *(*pVeh).m_vOrientation.add(2));
            *(*pVeh).m_vOrientation.add(2) += (strafeDif * 0.1) * (*pVeh).m_fTimeModifier;
            if BG_UnrestrainedPitchRoll(riderPS, pVeh, &*pmc.bg) == qfalse
                && (*vi).rollLimit != -1.0
                && (*pVeh).m_iRemovedSurfaces == 0
                && (*parentPS).electrifyTime < curTime
            {
                if *(*pVeh).m_vOrientation.add(2) > (*vi).rollLimit {
                    *(*pVeh).m_vOrientation.add(2) = (*vi).rollLimit;
                } else if *(*pVeh).m_vOrientation.add(2) < -(*vi).rollLimit {
                    *(*pVeh).m_vOrientation.add(2) = -(*vi).rollLimit;
                }
            }
        }

        if (*vi).surfDestruction != 0 {
            FighterDamageRoutine(pmc, pVeh, parent_num, parentPS, riderPS, isDead);
        }
        *(*pVeh).m_vOrientation.add(2) = AngleNormalize180(*(*pVeh).m_vOrientation.add(2));
    }
}
