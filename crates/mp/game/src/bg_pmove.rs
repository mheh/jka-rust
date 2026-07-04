// PORT-COMPLETE: bg_pmove.c 11/91
//! FAITHFUL signature skeleton for `oracle/oracle/codemp/game/bg_pmove.c`.
//!
//! Bodies filled per the settled fork rulings. The vast majority of this file
//! is built on the file-static pmove working set (`pmove_t *pm`, `pml_t pml`,
//! `bgEntity_t *pm_entSelf`, `pm_entVeh`, `pm_flying`, `gPMDoSlowFall`,
//! `pm_cancelOutZoom`). Porting-rules §B3 forbids `static mut`/hidden globals,
//! but the faithful no-arg C signatures here thread no `pm`/engine context, so
//! the representation of that working set is a genuine unsettled design fork.
//! Every function that reads/writes it — and every function whose skeleton
//! signature passes a mutated `vec3_t` out-param BY VALUE (`[f32;3]` is `Copy`,
//! so in-place writes cannot propagate) — is parked with a `PORT-ESCALATION`.
//! The clean, pointer-parameterized / pure functions are ported.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

// Raven `qboolean` is `c_int`; keep the source spelling at assignment sites.
// Source: `oracle/oracle/codemp/game/q_shared.h`
const qtrue: qboolean = 1;
const qfalse: qboolean = 0;
use crate::g_strap::strap_G2API_SetBoneAngles;
use crate::q_math::{AngleMod, AngleSubtract};
use crate::q_math::{AngleVectors, Q_fabs, vectoangles};
use crate::q_math::{PITCH, ROLL, YAW};
use mp_bg::public::anim_number::animNumber_t;
use mp_bg::vehicles::MIN_LANDING_SLOPE;

// Pass-3 bg state channel (fork rulings 12-16): the per-call working set + the
// two seam traits + the session state. `PmoveContext` replaces the file-static
// pmove working set the skeletons parked on.
use crate::bg_channel::{BgState, BgTraps, GameCallbacks, PmoveContext};
// Vehicle-type discriminants for the `PM_Friction` vehicle-friction branch.
use mp_bg::vehicles::vehicle_type_t::vehicleType_t;

// --- `bg_pmove.c` file-scope movement parameters (globals 41-55). These are
// read-only tunables, so they stay module `const`s (post-mega-pass ruling 15:
// "the pm_* float constants can stay consts").
// Source: `oracle/oracle/codemp/game/bg_pmove.c:41-55`
pub const pm_stopspeed: f32 = 100.0;
pub const pm_friction: f32 = 6.0;
pub const pm_waterfriction: f32 = 1.0;
pub const pm_spectatorfriction: f32 = 5.0;

// --- `bg_pmove.c` local `FLY_*` enum (bg_pmove.c:441-444). Mirrors `pm_flying`.
// Source: `oracle/oracle/codemp/game/bg_pmove.c:441-444`
pub const FLY_NONE: c_int = 0;
pub const FLY_NORMAL: c_int = 1;
pub const FLY_VEHICLE: c_int = 2;
pub const FLY_HOVER: c_int = 3;

// --- Constants the pmove slice reads that are not (yet) centrally exported;
// defined here per the codebase's per-file `#define` convention (cf. w_force.rs
// defining its own `PMF_STUCK_TO_WALL`). Each cites its Raven `#define`.
/// `MINS_Z`. Source: `oracle/oracle/codemp/game/bg_public.h:46`
pub const MINS_Z: c_int = -24;
/// `MIN_WALK_NORMAL`. Source: `oracle/oracle/codemp/game/bg_local.h:5`
pub const MIN_WALK_NORMAL: f32 = 0.7;
/// `SURF_SLICK`. Source: `oracle/oracle/codemp/game/surfaceflags.h:39`
const SURF_SLICK: c_int = 0x0000_4000;
/// `CONTENTS_LAVA|WATER|SLIME`. Source: `oracle/oracle/codemp/game/surfaceflags.h:11,12,30`
const MASK_WATER: c_int = 0x0000_0002 | 0x0000_0004 | 0x0002_0000;
/// `PMF_STUCK_TO_WALL`. Source: `oracle/oracle/codemp/game/bg_public.h:417`
const PMF_STUCK_TO_WALL: c_int = 16384;
/// `PMF_TIME_KNOCKBACK`. Source: `oracle/oracle/codemp/game/bg_public.h:409`
const PMF_TIME_KNOCKBACK: c_int = 64;
/// `PMF_JUMP_HELD`. Source: `oracle/oracle/codemp/game/bg_public.h:404`
const PMF_JUMP_HELD: c_int = 2;
/// `PS_PMOVEFRAMECOUNTBITS`. Source: `oracle/oracle/codemp/game/q_shared.h:2141`
pub const PS_PMOVEFRAMECOUNTBITS: c_int = 6;
/// `BUTTON_ATTACK`. Source: `oracle/oracle/codemp/game/q_shared.h:2451`
const BUTTON_ATTACK: c_int = 1;
/// `BUTTON_ALT_ATTACK`. Source: `oracle/oracle/codemp/game/q_shared.h:2462`
const BUTTON_ALT_ATTACK: c_int = 128;

/// Raven `BONE_ANGLES_POSTMULT` (ghoul2 bone-angle apply mode).
/// Source: `oracle/oracle/code/game/ghoul2_shared.h:54`
const BONE_ANGLES_POSTMULT: c_int = 0x0002;


/// Raven `PM_BGEntForNum`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:172-199`
// PORT-ESCALATION(pmove-working-state): how is the file-static `pmove_t *pm` (and
// `pml`/`pm_entSelf`) threaded into these no-arg C-signature fns without a §B3 static?
pub fn PM_BGEntForNum(
    num: c_int,
) -> *mut bgEntity_t {
    todo!("Port PM_BGEntForNum — parked: pmove-working-state")
}

/// Raven `BG_SabersOff`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:201-216`
pub fn BG_SabersOff(
    ps: *mut playerState_t,
) -> qboolean {
    unsafe {
        if (*ps).saberHolstered == 0 {
            return qfalse;
        }
        if (*ps).fd.saberAnimLevelBase as c_int == saber_styles_t::SS_DUAL as c_int
            || (*ps).fd.saberAnimLevelBase as c_int == saber_styles_t::SS_STAFF as c_int
        {
            if (*ps).saberHolstered < 2 {
                return qfalse;
            }
        }
        qtrue
    }
}

/// Raven `BG_KnockDownable`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:218-237`
pub fn BG_KnockDownable(
    ps: *mut playerState_t,
) -> qboolean {
    unsafe {
        if ps.is_null() {
            // just for safety
            return qfalse;
        }
        if (*ps).m_iVehicleNum != 0 {
            // riding a vehicle, don't knock me down
            return qfalse;
        }
        if (*ps).emplacedIndex != 0 {
            // using emplaced gun or eweb, can't be knocked down
            return qfalse;
        }
        // ok, I guess?
        qtrue
    }
}

/// Raven `PM_IsRocketTrooper`.
///
/// Raven: hacky assumption check — the humanoid/siege check is commented out in
/// the oracle; the live path always returns qfalse.
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:247-259`
pub fn PM_IsRocketTrooper() -> qboolean {
    qfalse
}

/// Raven `PM_GetSaberStance`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:261-319`
// PORT-ESCALATION(pmove-working-state): reads `pm->ps->...`; needs the pmove working-set threading decision.
pub fn PM_GetSaberStance() -> c_int {
    todo!("Port PM_GetSaberStance — parked: pmove-working-state")
}

/// Raven `PM_DoSlowFall`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:321-329`
// PORT-ESCALATION(pmove-working-state): reads `pm->ps`; needs the pmove working-set threading decision.
pub fn PM_DoSlowFall() -> qboolean {
    todo!("Port PM_DoSlowFall — parked: pmove-working-state")
}

/// Raven `PM_pitch_roll_for_slope`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:346-439`
// PORT-ESCALATION(pmove-working-state): writes `pm`; `storeAngles` is also a by-value vec3_t out-param.
pub fn PM_pitch_roll_for_slope(
    forwhom: *mut bgEntity_t,
    pass_slope: vec3_t,
    storeAngles: vec3_t,
) {
    todo!("Port PM_pitch_roll_for_slope — parked: pmove-working-state")
}

/// Raven `PM_SetSpecialMoveValues`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:447-480`
// PORT-ESCALATION(pmove-working-state): reads `pm`/`pm_entSelf`, writes `pm_flying`.
pub fn PM_SetSpecialMoveValues() {
    todo!("Port PM_SetSpecialMoveValues — parked: pmove-working-state")
}

/// Raven `PM_SetVehicleAngles`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:482-635`
// PORT-ESCALATION(pmove-working-state): reads `pm`/`pm_entSelf`/`pml`.
pub fn PM_SetVehicleAngles(
    normal: vec3_t,
) {
    todo!("Port PM_SetVehicleAngles — parked: pmove-working-state")
}

/// Raven `BG_ExternThisSoICanRecompileInDebug`.
///
/// Raven: the entire body is commented out in the oracle (a debug-recompile
/// hook); it is a no-op.
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:641-674`
pub fn BG_ExternThisSoICanRecompileInDebug(
    pVeh: *mut Vehicle_t,
    riderPS: *mut playerState_t,
) {
    // No-op: the oracle body is entirely `/* ... */`-commented.
}

/// Raven `BG_VehicleTurnRateForSpeed`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:676-706`
pub fn BG_VehicleTurnRateForSpeed(
    pVeh: *mut Vehicle_t,
    speed: f32,
    mPitchOverride: *mut f32,
    mYawOverride: *mut f32,
) {
    unsafe {
        if !pVeh.is_null() && !(*pVeh).m_pVehicleInfo.is_null() {
            let info = (*pVeh).m_pVehicleInfo;
            let mut speedFrac: f32 = 1.0;
            if (*info).speedDependantTurning != 0 {
                if (*pVeh).m_LandTrace.fraction >= 1.0
                    || (*pVeh).m_LandTrace.plane.normal[2] < MIN_LANDING_SLOPE
                {
                    speedFrac = speed / ((*info).speedMax * 0.75);
                    if speedFrac < 0.25 {
                        speedFrac = 0.25;
                    } else if speedFrac > 1.0 {
                        speedFrac = 1.0;
                    }
                }
            }
            if (*info).mousePitch != 0.0 {
                *mPitchOverride = (*info).mousePitch * speedFrac;
            }
            if (*info).mouseYaw != 0.0 {
                *mYawOverride = (*info).mouseYaw * speedFrac;
            }
        }
    }
}

/// Raven `PM_HoverTrace`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:719-901`
// PORT-ESCALATION(pmove-working-state): reads `pm_entSelf`, writes `pm`/`pml`.
pub fn PM_HoverTrace() {
    todo!("Port PM_HoverTrace — parked: pmove-working-state")
}

/// Raven `PM_AddEvent`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:910-912`
// PORT-ESCALATION(pmove-working-state): reads `pm`.
pub fn PM_AddEvent(
    newEvent: c_int,
) {
    todo!("Port PM_AddEvent — parked: pmove-working-state")
}

/// Raven `PM_AddEventWithParm`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:914-917`
// PORT-ESCALATION(pmove-working-state): reads `pm`.
pub fn PM_AddEventWithParm(
    newEvent: c_int,
    parm: c_int,
) {
    todo!("Port PM_AddEventWithParm — parked: pmove-working-state")
}

/// Raven `PM_AddTouchEnt`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:924-944`
// PORT-ESCALATION(pmove-working-state): writes `pm->touchents`.
pub fn PM_AddTouchEnt(
    entityNum: c_int,
) {
    todo!("Port PM_AddTouchEnt — parked: pmove-working-state")
}

// The pmove pipeline as `PmoveContext` methods (pass-3 rulings 12/8a). Each was
// a no-arg C function reaching the file-static working set; the set now lives in
// `self` (`self.pm`/`self.pml`/`self.pm_entSelf`/… + `self.bg`/`self.traps`).
// The `unsafe` that dereferences the faithful `pm`/entity pointers is confined
// to these bodies (porting-rules §D11; ruling 14).
impl PmoveContext<'_> {
    /// Raven `PM_BGEntForNum` — the faithful `baseEnt`/`entSize` head-overlay
    /// (ruling 14). Returns the `bgEntity_t` at index `num` in the base array
    /// the engine handed us. Raven's `assert`s become defensive null/zero
    /// returns (out-of-pmove calls / unset base are the UB cases §19 covers).
    /// Source: `oracle/oracle/codemp/game/bg_pmove.c:172-199`
    pub fn PM_BGEntForNum(&self, num: c_int) -> *mut bgEntity_t {
        unsafe {
            if self.pm.is_null() {
                // "You cannot call PM_BGEntForNum outside of pm functions!"
                return core::ptr::null_mut();
            }
            let pm = &*self.pm;
            if pm.baseEnt.is_null() {
                // "Base entity address not set"
                return core::ptr::null_mut();
            }
            if pm.entSize == 0 {
                // "sizeof(ent) is 0, impossible (not set?)"
                return core::ptr::null_mut();
            }
            debug_assert!(num >= 0 && num < MAX_GENTITIES as c_int);
            // ent = (bgEntity_t *)((byte *)pm->baseEnt + pm->entSize*(num));
            (pm.baseEnt as *mut byte).offset((pm.entSize * num) as isize) as *mut bgEntity_t
        }
    }

    /// Raven `PM_ClipVelocity` — slide `in` off the impacting surface `normal`
    /// into `out` (§C7 out-param shape; `in`/`normal` by value permit the
    /// `PM_ClipVelocity(pml.forward, …, pml.forward, …)` self-aliasing callers).
    /// Source: `oracle/oracle/codemp/game/bg_pmove.c:954-988`
    pub fn PM_ClipVelocity(
        &self,
        r#in: vec3_t,
        normal: vec3_t,
        out: &mut vec3_t,
        overbounce: f32,
    ) {
        unsafe {
            let ps = &*(*self.pm).ps;
            if ps.pm_flags & PMF_STUCK_TO_WALL != 0 {
                // no sliding!
                *out = r#in; // VectorCopy( in, out )
                return;
            }
            let oldInZ = r#in[2];

            // backoff = DotProduct (in, normal);
            let mut backoff = r#in[0] * normal[0] + r#in[1] * normal[1] + r#in[2] * normal[2];

            if backoff < 0.0 {
                backoff *= overbounce;
            } else {
                backoff /= overbounce;
            }

            for i in 0..3 {
                let change = normal[i] * backoff;
                out[i] = r#in[i] - change;
            }
            if (*self.pm).stepSlideFix != 0
                && ps.clientNum < MAX_CLIENTS as c_int// normal player
                && ps.groundEntityNum != ENTITYNUM_NONE // on the ground
                && normal[2] < MIN_WALK_NORMAL
            {
                // sliding against a steep slope: don't slide up slopes too steep to walk on
                out[2] = oldInZ;
            }
        }
    }

    /// Raven `PM_Friction` — ground + water friction on `pm->ps->velocity`.
    /// Source: `oracle/oracle/codemp/game/bg_pmove.c:998-1123`
    pub fn PM_Friction(&mut self) {
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;

            // vec = velocity, ignoring slope movement while walking.
            let mut vec = (*ps).velocity;
            if self.pml.walking != 0 {
                vec[2] = 0.0;
            }

            // speed = VectorLength(vec);
            let speed = (vec[0] * vec[0] + vec[1] * vec[1] + vec[2] * vec[2]).sqrt();
            if speed < 1.0 {
                (*ps).velocity[0] = 0.0;
                (*ps).velocity[1] = 0.0; // allow sinking underwater
                if (*ps).pm_type == PM_SPECTATOR as c_int {
                    (*ps).velocity[2] = 0.0;
                }
                return;
            }

            let mut drop: f32 = 0.0;

            let mut pEnt: *mut bgEntity_t = core::ptr::null_mut();
            if (*ps).clientNum >= MAX_CLIENTS as c_int{
                pEnt = self.pm_entSelf;
            }

            // apply ground friction, even if on ladder
            if self.pm_flying != FLY_VEHICLE
                && !pEnt.is_null()
                && (*pEnt).s.NPC_class == CLASS_VEHICLE as c_int
                && !(*pEnt).m_pVehicle.is_null()
                && (*(*(*pEnt).m_pVehicle).m_pVehicleInfo).r#type as c_int
                    != vehicleType_t::VH_ANIMAL as c_int
                && (*(*(*pEnt).m_pVehicle).m_pVehicleInfo).r#type as c_int
                    != vehicleType_t::VH_WALKER as c_int
                && (*(*(*pEnt).m_pVehicle).m_pVehicleInfo).friction != 0.0
            {
                let friction = (*(*(*pEnt).m_pVehicle).m_pVehicleInfo).friction;
                if (*ps).pm_flags & PMF_TIME_KNOCKBACK == 0 {
                    let control = if speed < pm_stopspeed { pm_stopspeed } else { speed };
                    drop += control * friction * self.pml.frametime;
                }
            } else if self.pm_flying != FLY_NORMAL && self.pm_flying != FLY_VEHICLE {
                // apply ground friction
                if (*pm).waterlevel <= 1
                    && self.pml.walking != 0
                    && self.pml.groundTrace.surfaceFlags & SURF_SLICK == 0
                    && (*ps).pm_flags & PMF_TIME_KNOCKBACK == 0
                {
                    // if getting knocked back, no friction
                    let control = if speed < pm_stopspeed { pm_stopspeed } else { speed };
                    drop += control * pm_friction * self.pml.frametime;
                }
            }

            if self.pm_flying == FLY_VEHICLE && (*ps).pm_flags & PMF_TIME_KNOCKBACK == 0 {
                let control = speed;
                drop += control * pm_friction * self.pml.frametime;
            }

            // apply water friction even if just wading
            if (*pm).waterlevel != 0 {
                drop += speed * pm_waterfriction * (*pm).waterlevel as f32 * self.pml.frametime;
            }
            // If on a client then there is no friction
            else if (*ps).groundEntityNum < MAX_CLIENTS as c_int{
                drop = 0.0;
            }

            if (*ps).pm_type == PM_SPECTATOR as c_int || (*ps).pm_type == PM_FLOAT as c_int {
                if (*ps).pm_type == PM_FLOAT as c_int {
                    // almost no friction while floating (Raven's `0.1` is a
                    // `double` literal; compute in f64 to preserve parity).
                    drop = (drop as f64
                        + speed as f64 * 0.1 * self.pml.frametime as f64)
                        as f32;
                } else {
                    drop += speed * pm_spectatorfriction * self.pml.frametime;
                }
            }

            // scale the velocity
            let mut newspeed = speed - drop;
            if newspeed < 0.0 {
                newspeed = 0.0;
            }
            newspeed /= speed;

            (*ps).velocity[0] *= newspeed;
            (*ps).velocity[1] *= newspeed;
            (*ps).velocity[2] *= newspeed;
        }
    }

    /// Raven `PM_SetWaterLevel` — set `pm->waterlevel`/`watertype` by sampling
    /// point contents at three heights (accounting for ducking). Exercises the
    /// `BgTraps::pointcontents` seam (ruling 13).
    /// Source: `oracle/oracle/codemp/game/bg_pmove.c:4285-4320`
    pub fn PM_SetWaterLevel(&mut self) {
        unsafe {
            let pm = self.pm;
            let ps = (*pm).ps;

            // get waterlevel, accounting for ducking
            (*pm).waterlevel = 0;
            (*pm).watertype = 0;

            let mut point: vec3_t = [
                (*ps).origin[0],
                (*ps).origin[1],
                (*ps).origin[2] + MINS_Z as f32 + 1.0,
            ];
            let mut cont = self
                .traps
                .pointcontents(core::ptr::addr_of!(point), (*ps).clientNum);

            if cont & MASK_WATER != 0 {
                let sample2 = (*ps).viewheight - MINS_Z;
                let sample1 = sample2 / 2;

                (*pm).watertype = cont;
                (*pm).waterlevel = 1;
                point[2] = (*ps).origin[2] + MINS_Z as f32 + sample1 as f32;
                cont = self
                    .traps
                    .pointcontents(core::ptr::addr_of!(point), (*ps).clientNum);
                if cont & MASK_WATER != 0 {
                    (*pm).waterlevel = 2;
                    point[2] = (*ps).origin[2] + MINS_Z as f32 + sample2 as f32;
                    cont = self
                        .traps
                        .pointcontents(core::ptr::addr_of!(point), (*ps).clientNum);
                    if cont & MASK_WATER != 0 {
                        (*pm).waterlevel = 3;
                    }
                }
            }
        }
    }

    /// Raven `PM_DoSlowFall` — reads `pm->ps`; ported in the pass-3 remainder.
    //TODO: Port PM_DoSlowFall
    // Source: oracle/oracle/codemp/game/bg_pmove.c:321-329
    pub fn PM_DoSlowFall(&mut self) -> qboolean {
        todo!("Port PM_DoSlowFall — pass 3 (bg_pmove.c:321-329)")
    }

    /// Raven `PmoveSingle` — one fixed-timestep move. The opening (proxy button
    /// fix-up, working-set entity setup, slow-fall latch, result clear, rocket-
    /// trooper crouch clamp) is ported faithfully; the ~930-line move pipeline
    /// remainder is the pass-3 target (single `todo!` per task).
    /// Source: `oracle/oracle/codemp/game/bg_pmove.c:10174-11157`
    pub fn PmoveSingle(&mut self, pmove: *mut pmove_t) {
        unsafe {
            self.pm = pmove;
            let pm = self.pm;

            if (*(*pm).ps).emplacedIndex != 0 && (*pm).cmd.buttons & BUTTON_ALT_ATTACK != 0 {
                // hackerrific.
                (*pm).cmd.buttons &= !BUTTON_ALT_ATTACK;
                (*pm).cmd.buttons |= BUTTON_ATTACK;
            }

            // set up these "global" bg ents
            self.pm_entSelf = self.PM_BGEntForNum((*(*pm).ps).clientNum);
            if (*(*pm).ps).m_iVehicleNum != 0 {
                if (*(*pm).ps).clientNum < MAX_CLIENTS as c_int{
                    // player riding vehicle
                    self.pm_entVeh = self.PM_BGEntForNum((*(*pm).ps).m_iVehicleNum);
                } else {
                    // vehicle with player pilot
                    self.pm_entVeh = self.PM_BGEntForNum((*(*pm).ps).m_iVehicleNum - 1);
                }
            } else {
                // no vehicle ent
                self.pm_entVeh = core::ptr::null_mut();
            }

            self.gPMDoSlowFall = self.PM_DoSlowFall();

            // this counter lets us debug movement problems with a journal
            self.bg.c_pmove += 1;

            // clear results
            (*pm).numtouch = 0;
            (*pm).watertype = 0;
            (*pm).waterlevel = 0;

            if PM_IsRocketTrooper() != 0 {
                // don't let a nonhumanoid (probably a rockettrooper) crouch
                if (*pm).cmd.upmove < 0 {
                    (*pm).cmd.upmove = 0;
                }
            }

            //TODO: Port PmoveSingle remainder
            // Source: oracle/oracle/codemp/game/bg_pmove.c:10228-11157
            todo!("Port PmoveSingle remainder — pass 3")
        }
    }
}

/// Raven `PM_Accelerate`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1133-1186`
// PORT-ESCALATION(pmove-working-state): reads `pml`, writes `pm`.
pub fn PM_Accelerate(
    wishdir: vec3_t,
    wishspeed: f32,
    accel: f32,
) {
    todo!("Port PM_Accelerate — parked: pmove-working-state")
}

/// Raven `PM_CmdScale`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1199-1222`
// PORT-ESCALATION(pmove-working-state): reads `pm`.
pub fn PM_CmdScale(
    cmd: *mut usercmd_t,
) -> f32 {
    todo!("Port PM_CmdScale — parked: pmove-working-state")
}

/// Raven `PM_SetMovementDir`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1233-1262`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_SetMovementDir() {
    todo!("Port PM_SetMovementDir — parked: pmove-working-state")
}

/// Raven `PM_ForceJumpingUp`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1266-1306`
// PORT-ESCALATION(pmove-working-state): reads `pm`.
pub fn PM_ForceJumpingUp() -> qboolean {
    todo!("Port PM_ForceJumpingUp — parked: pmove-working-state")
}

/// Raven `PM_JumpForDir`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1308-1340`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_JumpForDir() {
    todo!("Port PM_JumpForDir — parked: pmove-working-state")
}

/// Raven `PM_SetPMViewAngle`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1342-1354`
pub fn PM_SetPMViewAngle(
    ps: *mut playerState_t,
    angle: vec3_t,
    ucmd: *mut usercmd_t,
) {
    unsafe {
        for i in 0..3 {
            // set the delta angle. Raven `ANGLE2SHORT(x)` == `((int)((x)*65536/360) & 65535)`.
            let cmdAngle: c_int = ((angle[i] * 65536.0 / 360.0) as c_int) & 65535;
            (*ps).delta_angles[i] = cmdAngle - (*ucmd).angles[i];
        }
        (*ps).viewangles = angle;
    }
}

/// Raven `PM_AdjustAngleForWallRun`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1356-1462`
// PORT-ESCALATION(pmove-working-state): reads `pm`.
pub fn PM_AdjustAngleForWallRun(
    ps: *mut playerState_t,
    ucmd: *mut usercmd_t,
    doMove: qboolean,
) -> qboolean {
    todo!("Port PM_AdjustAngleForWallRun — parked: pmove-working-state")
}

/// Raven `PM_AdjustAnglesForWallRunUpFlipAlt`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1464-1470`
// PORT-ESCALATION(pmove-working-state): reads `pm`.
pub fn PM_AdjustAnglesForWallRunUpFlipAlt(
    ucmd: *mut usercmd_t,
) -> qboolean {
    todo!("Port PM_AdjustAnglesForWallRunUpFlipAlt — parked: pmove-working-state")
}

/// Raven `PM_AdjustAngleForWallRunUp`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1472-1598`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_AdjustAngleForWallRunUp(
    ps: *mut playerState_t,
    ucmd: *mut usercmd_t,
    doMove: qboolean,
) -> qboolean {
    todo!("Port PM_AdjustAngleForWallRunUp — parked: pmove-working-state")
}

/// Raven `BG_ForceWallJumpStrength`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1602-1605`
// PORT-ESCALATION(missing-global-table): returns `forceJumpStrength[FORCE_LEVEL_3]/2.5f`. The
// backfill supplies only `JUMP_VELOCITY` (element 0); the full `forceJumpStrength` file-scope
// table (a bg_pmove.c global, also parked in NPC_AI_Jedi.rs) is unresolved — no invention allowed.
pub fn BG_ForceWallJumpStrength() -> f32 {
    todo!("Port BG_ForceWallJumpStrength — parked: missing-global-table")
}

/// Raven `PM_AdjustAngleForWallJump`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1607-1756`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_AdjustAngleForWallJump(
    ps: *mut playerState_t,
    ucmd: *mut usercmd_t,
    doMove: qboolean,
) -> qboolean {
    todo!("Port PM_AdjustAngleForWallJump — parked: pmove-working-state")
}

/// Raven `PM_SetForceJumpZStart`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1759-1766`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_SetForceJumpZStart(
    value: f32,
) {
    todo!("Port PM_SetForceJumpZStart — parked: pmove-working-state")
}

/// Raven `PM_GrabWallForJump`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1776-1781`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_GrabWallForJump(
    anim: c_int,
) {
    todo!("Port PM_GrabWallForJump — parked: pmove-working-state")
}

/// Raven `PM_CheckJump`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:1788-2775`
// PORT-ESCALATION(pmove-working-state): reads/writes `pm`/`pml`/`pm_entSelf`.
pub fn PM_CheckJump() -> qboolean {
    todo!("Port PM_CheckJump — parked: pmove-working-state")
}

/// Raven `PM_CheckWaterJump`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:2781-2821`
// PORT-ESCALATION(pmove-working-state): reads `pml`, writes `pm`.
pub fn PM_CheckWaterJump() -> qboolean {
    todo!("Port PM_CheckWaterJump — parked: pmove-working-state")
}

/// Raven `PM_WaterJumpMove`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:2833-2844`
// PORT-ESCALATION(pmove-working-state): reads `pml`, writes `pm`.
pub fn PM_WaterJumpMove() {
    todo!("Port PM_WaterJumpMove — parked: pmove-working-state")
}

/// Raven `PM_WaterMove`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:2852-2916`
// PORT-ESCALATION(pmove-working-state): reads `pm`/`pml` + movement-param globals.
pub fn PM_WaterMove() {
    todo!("Port PM_WaterMove — parked: pmove-working-state")
}

/// Raven `PM_FlyVehicleMove`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:2924-3012`
// PORT-ESCALATION(pmove-working-state): reads `pml`, writes `pm`.
pub fn PM_FlyVehicleMove() {
    todo!("Port PM_FlyVehicleMove — parked: pmove-working-state")
}

/// Raven `PM_FlyMove`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:3021-3059`
// PORT-ESCALATION(pmove-working-state): reads `pm`/`pml`.
pub fn PM_FlyMove() {
    todo!("Port PM_FlyMove — parked: pmove-working-state")
}

/// Raven `PM_AirMove`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:3068-3297`
// PORT-ESCALATION(pmove-working-state): reads `gPMDoSlowFall`/`pm`/`pm_entSelf`, writes `pml`.
pub fn PM_AirMove() {
    todo!("Port PM_AirMove — parked: pmove-working-state")
}

/// Raven `PM_WalkMove`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:3305-3484`
// PORT-ESCALATION(pmove-working-state): reads/writes `pm`/`pml`/`pm_entSelf` + movement-param globals.
pub fn PM_WalkMove() {
    todo!("Port PM_WalkMove — parked: pmove-working-state")
}

/// Raven `PM_DeadMove`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:3492-3509`
// PORT-ESCALATION(pmove-working-state): reads `pm`/`pml`.
pub fn PM_DeadMove() {
    todo!("Port PM_DeadMove — parked: pmove-working-state")
}

/// Raven `PM_NoclipMove`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:3517-3576`
// PORT-ESCALATION(pmove-working-state): reads `pml`, writes `pm`.
pub fn PM_NoclipMove() {
    todo!("Port PM_NoclipMove — parked: pmove-working-state")
}

/// Raven `PM_FootstepForSurface`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:3587-3594`
// PORT-ESCALATION(pmove-working-state): reads `pml`.
pub fn PM_FootstepForSurface() -> c_int {
    todo!("Port PM_FootstepForSurface — parked: pmove-working-state")
}

/// Raven `PM_TryRoll`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:3597-3681`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_TryRoll() -> c_int {
    todo!("Port PM_TryRoll — parked: pmove-working-state")
}

/// Raven `PM_CrashLandEffect`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:3684-3722`
// PORT-ESCALATION(pmove-working-state): reads `pm`/`pml`.
pub fn PM_CrashLandEffect() {
    todo!("Port PM_CrashLandEffect — parked: pmove-working-state")
}

/// Raven `PM_CrashLand`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:3731-4002`
// PORT-ESCALATION(pmove-working-state): reads `pml`/`WeaponReadyAnim`, writes `pm`.
pub fn PM_CrashLand() {
    todo!("Port PM_CrashLand — parked: pmove-working-state")
}

/// Raven `PM_CorrectAllSolid`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4009-4044`
// PORT-ESCALATION(pmove-working-state): reads `c_pmove`, writes `pm`/`pml`.
pub fn PM_CorrectAllSolid(
    trace: *mut trace_t,
) -> c_int {
    todo!("Port PM_CorrectAllSolid — parked: pmove-working-state")
}

/// Raven `PM_GroundTraceMissed`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4053-4133`
// PORT-ESCALATION(pmove-working-state): writes `pm`/`pml`.
pub fn PM_GroundTraceMissed() {
    todo!("Port PM_GroundTraceMissed — parked: pmove-working-state")
}

/// Raven `PM_GroundTrace`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4141-4277`
// PORT-ESCALATION(pmove-working-state): reads `g_entities`/`g_gametype`/`pm_entSelf`, writes `pm`/`pml`.
pub fn PM_GroundTrace() {
    todo!("Port PM_GroundTrace — parked: pmove-working-state")
}

// `PM_SetWaterLevel` is a `PmoveContext` method above (it reads the pmove
// working set and drives `BgTraps::pointcontents`).

/// Raven `PM_CheckDualForwardJumpDuck`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4322-4339`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_CheckDualForwardJumpDuck() -> qboolean {
    todo!("Port PM_CheckDualForwardJumpDuck — parked: pmove-working-state")
}

/// Raven `PM_CheckFixMins`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4341-4401`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_CheckFixMins() {
    todo!("Port PM_CheckFixMins — parked: pmove-working-state")
}

/// Raven `PM_CheckDuck`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4410-4542`
// PORT-ESCALATION(pmove-working-state): reads `g_entities`/`level`/`pm_entVeh`, writes `pm`.
pub fn PM_CheckDuck() {
    todo!("Port PM_CheckDuck — parked: pmove-working-state")
}

/// Raven `PM_Use`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4559-4577`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_Use() {
    todo!("Port PM_Use — parked: pmove-working-state")
}

/// Raven `PM_WalkingAnim`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4579-4598`
pub fn PM_WalkingAnim(
    anim: c_int,
) -> qboolean {
    use animNumber_t::*;
    const ANIMS: &[animNumber_t] = &[
        BOTH_WALK1,          //# Normal walk
        BOTH_WALK2,          //# Normal walk with saber
        BOTH_WALK_STAFF,     //# Normal walk with staff
        BOTH_WALK_DUAL,      //# Normal walk with staff
        BOTH_WALK5,          //# Tavion taunting Kyle (cin 22)
        BOTH_WALK6,          //# Slow walk for Luke (cin 12)
        BOTH_WALK7,          //# Fast walk
        BOTH_WALKBACK1,      //# Walk1 backwards
        BOTH_WALKBACK2,      //# Walk2 backwards
        BOTH_WALKBACK_STAFF, //# Walk backwards with staff
        BOTH_WALKBACK_DUAL,  //# Walk backwards with dual
    ];
    if ANIMS.iter().any(|&a| a as c_int == anim) {
        qtrue
    } else {
        qfalse
    }
}

/// Raven `PM_RunningAnim`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4600-4620`
pub fn PM_RunningAnim(
    anim: c_int,
) -> qboolean {
    use animNumber_t::*;
    const ANIMS: &[animNumber_t] = &[
        BOTH_RUN1,
        BOTH_RUN2,
        BOTH_RUN_STAFF,
        BOTH_RUN_DUAL,
        BOTH_RUNBACK1,
        BOTH_RUNBACK2,
        BOTH_RUNBACK_STAFF,
        BOTH_RUNBACK_DUAL,
        BOTH_RUN1START,        //# Start into full run1
        BOTH_RUN1STOP,         //# Stop from full run1
        BOTH_RUNSTRAFE_LEFT1,  //# Sidestep left: should loop
        BOTH_RUNSTRAFE_RIGHT1, //# Sidestep right: should loop
    ];
    if ANIMS.iter().any(|&a| a as c_int == anim) {
        qtrue
    } else {
        qfalse
    }
}

/// Raven `PM_SwimmingAnim`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4622-4633`
pub fn PM_SwimmingAnim(
    anim: c_int,
) -> qboolean {
    use animNumber_t::*;
    const ANIMS: &[animNumber_t] = &[
        BOTH_SWIM_IDLE1, //# Swimming Idle 1
        BOTH_SWIMFORWARD,  //# Swim forward loop
        BOTH_SWIMBACKWARD, //# Swim backward loop
    ];
    if ANIMS.iter().any(|&a| a as c_int == anim) {
        qtrue
    } else {
        qfalse
    }
}

/// Raven `PM_RollingAnim`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4635-4647`
pub fn PM_RollingAnim(
    anim: c_int,
) -> qboolean {
    use animNumber_t::*;
    const ANIMS: &[animNumber_t] = &[
        BOTH_ROLL_F, //# Roll forward
        BOTH_ROLL_B, //# Roll backward
        BOTH_ROLL_L, //# Roll left
        BOTH_ROLL_R, //# Roll right
    ];
    if ANIMS.iter().any(|&a| a as c_int == anim) {
        qtrue
    } else {
        qfalse
    }
}

/// Raven `PM_AnglesForSlope`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4649-4675`
// fork-9: `angles` is a written-through out-param → `&mut [f32;3]`; `slope` stays a
// read-only by-value input. Cross-file callers are updated by the fixer.
pub fn PM_AnglesForSlope(
    yaw: f32,
    slope: vec3_t,
    angles: &mut [f32; 3],
) {
    let mut nvf: vec3_t = [0.0; 3];
    let mut ovf: vec3_t = [0.0; 3];
    let mut ovr: vec3_t = [0.0; 3];
    let mut new_angles: vec3_t = [0.0; 3];

    // VectorSet( angles, 0, yaw, 0 )
    angles[0] = 0.0;
    angles[1] = yaw;
    angles[2] = 0.0;
    AngleVectors(*angles, Some(&mut ovf), Some(&mut ovr), None);

    vectoangles(slope, &mut new_angles);
    let pitch = new_angles[PITCH] + 90.0;
    new_angles[ROLL] = 0.0;
    new_angles[PITCH] = 0.0;

    AngleVectors(new_angles, Some(&mut nvf), None, None);

    // mod = DotProduct( nvf, ovr )
    let mut r#mod = nvf[0] * ovr[0] + nvf[1] * ovr[1] + nvf[2] * ovr[2];
    if r#mod < 0.0 {
        r#mod = -1.0;
    } else {
        r#mod = 1.0;
    }

    // dot = DotProduct( nvf, ovf )
    let dot = nvf[0] * ovf[0] + nvf[1] * ovf[1] + nvf[2] * ovf[2];

    angles[YAW] = 0.0;
    angles[PITCH] = dot * pitch;
    angles[ROLL] = (1.0 - Q_fabs(dot)) * pitch * r#mod;
}

/// Raven `PM_FootSlopeTrace`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4677-4740`
// PORT-ESCALATION(pmove-working-state): reads `pm` (ghoul2, trace, modelScale, mins, ...).
pub fn PM_FootSlopeTrace(
    pDiff: *mut f32,
    pInterval: *mut f32,
) {
    todo!("Port PM_FootSlopeTrace — parked: pmove-working-state")
}

/// Raven `BG_InSlopeAnim`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4742-4800`
pub fn BG_InSlopeAnim(
    anim: c_int,
) -> qboolean {
    use animNumber_t::*;
    const ANIMS: &[animNumber_t] = &[
        LEGS_LEFTUP1,  //# On a slope with left foot 4 higher than right
        LEGS_LEFTUP2,  //# On a slope with left foot 8 higher than right
        LEGS_LEFTUP3,  //# On a slope with left foot 12 higher than right
        LEGS_LEFTUP4,  //# On a slope with left foot 16 higher than right
        LEGS_LEFTUP5,  //# On a slope with left foot 20 higher than right
        LEGS_RIGHTUP1, //# On a slope with RIGHT foot 4 higher than left
        LEGS_RIGHTUP2, //# On a slope with RIGHT foot 8 higher than left
        LEGS_RIGHTUP3, //# On a slope with RIGHT foot 12 higher than left
        LEGS_RIGHTUP4, //# On a slope with RIGHT foot 16 higher than left
        LEGS_RIGHTUP5, //# On a slope with RIGHT foot 20 higher than left
        LEGS_S1_LUP1, LEGS_S1_LUP2, LEGS_S1_LUP3, LEGS_S1_LUP4, LEGS_S1_LUP5,
        LEGS_S1_RUP1, LEGS_S1_RUP2, LEGS_S1_RUP3, LEGS_S1_RUP4, LEGS_S1_RUP5,
        LEGS_S3_LUP1, LEGS_S3_LUP2, LEGS_S3_LUP3, LEGS_S3_LUP4, LEGS_S3_LUP5,
        LEGS_S3_RUP1, LEGS_S3_RUP2, LEGS_S3_RUP3, LEGS_S3_RUP4, LEGS_S3_RUP5,
        LEGS_S4_LUP1, LEGS_S4_LUP2, LEGS_S4_LUP3, LEGS_S4_LUP4, LEGS_S4_LUP5,
        LEGS_S4_RUP1, LEGS_S4_RUP2, LEGS_S4_RUP3, LEGS_S4_RUP4, LEGS_S4_RUP5,
        LEGS_S5_LUP1, LEGS_S5_LUP2, LEGS_S5_LUP3, LEGS_S5_LUP4, LEGS_S5_LUP5,
        LEGS_S5_RUP1, LEGS_S5_RUP2, LEGS_S5_RUP3, LEGS_S5_RUP4, LEGS_S5_RUP5,
    ];
    if ANIMS.iter().any(|&a| a as c_int == anim) {
        qtrue
    } else {
        qfalse
    }
}

/// Raven `PM_AdjustStandAnimForSlope`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:4804-5102`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_AdjustStandAnimForSlope() -> qboolean {
    todo!("Port PM_AdjustStandAnimForSlope — parked: pmove-working-state")
}

/// Raven `PM_LegsSlopeBackTransition`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:5107-5168`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_LegsSlopeBackTransition(
    desiredAnim: c_int,
) -> c_int {
    todo!("Port PM_LegsSlopeBackTransition — parked: pmove-working-state")
}

/// Raven `PM_Footsteps`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:5175-5661`
// PORT-ESCALATION(pmove-working-state): reads `WeaponReadyLegsAnim`/`pm_entSelf`/`pml`, writes `pm`.
pub fn PM_Footsteps() {
    todo!("Port PM_Footsteps — parked: pmove-working-state")
}

/// Raven `PM_WaterEvents`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:5670-5748`
// PORT-ESCALATION(pmove-working-state): reads `pm`/`pml`.
pub fn PM_WaterEvents() {
    todo!("Port PM_WaterEvents — parked: pmove-working-state")
}

/// Raven `BG_ClearRocketLock`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:5750-5759`
pub fn BG_ClearRocketLock(
    ps: *mut playerState_t,
) {
    unsafe {
        if !ps.is_null() {
            (*ps).rocketLockIndex = ENTITYNUM_NONE;
            (*ps).rocketLastValidTime = 0.0;
            (*ps).rocketLockTime = -1.0;
            (*ps).rocketTargetTime = 0.0;
        }
    }
}

/// Raven `PM_BeginWeaponChange`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:5766-5793`
// PORT-ESCALATION(pmove-working-state): reads/writes `pm->ps`.
pub fn PM_BeginWeaponChange(
    weapon: c_int,
) {
    todo!("Port PM_BeginWeaponChange — parked: pmove-working-state")
}

/// Raven `PM_FinishWeaponChange`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:5801-5825`
// PORT-ESCALATION(pmove-working-state): reads/writes `pm->ps`/`pm->cmd`.
pub fn PM_FinishWeaponChange() {
    todo!("Port PM_FinishWeaponChange — parked: pmove-working-state")
}

/// Raven `BG_VehTraceFromCamPos`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:5833-5872`
// PORT-ESCALATION(engine-trap-threading): the QAGAME path calls `trap_Trace`, whose SEAM-D13
// wrapper needs an engine handle (`trap::Trace(engine, ..)`), but this C-signature fn threads none.
pub fn BG_VehTraceFromCamPos(
    camTrace: *mut trace_t,
    bgEnt: *mut bgEntity_t,
    entOrg: vec3_t,
    shotStart: vec3_t,
    end: vec3_t,
    newEnd: vec3_t,
    shotDir: vec3_t,
    bestDist: f32,
) -> c_int {
    todo!("Port BG_VehTraceFromCamPos — parked: engine-trap-threading")
}

/// Raven `PM_RocketLock`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:5874-5977`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_RocketLock(
    lockDist: f32,
    vehicleLock: qboolean,
) {
    todo!("Port PM_RocketLock — parked: pmove-working-state")
}

/// Raven `PM_DoChargedWeapons`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:5980-6233`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_DoChargedWeapons(
    vehicleRocketLock: qboolean,
    veh: *mut bgEntity_t,
) -> qboolean {
    todo!("Port PM_DoChargedWeapons — parked: pmove-working-state")
}

/// Raven `PM_ItemUsable`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:6239-6366`
// PORT-ESCALATION(pmove-working-state): reads `pm`/`bg_itemlist`.
pub fn PM_ItemUsable(
    ps: *mut playerState_t,
    forcedUse: c_int,
) -> c_int {
    todo!("Port PM_ItemUsable — parked: pmove-working-state")
}

/// Raven `PM_CanSetWeaponAnims`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:6369-6377`
// PORT-ESCALATION(pmove-working-state): reads `pm`.
pub fn PM_CanSetWeaponAnims() -> qboolean {
    todo!("Port PM_CanSetWeaponAnims — parked: pmove-working-state")
}

/// Raven `PM_VehicleWeaponAnimate`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:6381-6631`
// PORT-ESCALATION(pmove-working-state): reads `pm_entVeh`, writes `pm`.
pub fn PM_VehicleWeaponAnimate() {
    todo!("Port PM_VehicleWeaponAnimate — parked: pmove-working-state")
}

/// Raven `PM_Weapon`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:6641-7672`
// PORT-ESCALATION(pmove-working-state): reads `pm_entSelf`/`pm_entVeh`/`pml`/many tables, writes `pm`.
pub fn PM_Weapon() {
    todo!("Port PM_Weapon — parked: pmove-working-state")
}

/// Raven `PM_Animate`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:7680-7740`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_Animate() {
    todo!("Port PM_Animate — parked: pmove-working-state")
}

/// Raven `PM_DropTimers`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:7748-7773`
// PORT-ESCALATION(pmove-working-state): reads `pml`, writes `pm`.
pub fn PM_DropTimers() {
    todo!("Port PM_DropTimers — parked: pmove-working-state")
}

/// Raven `BG_UnrestrainedPitchRoll`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:7784-7798`
// PORT-ESCALATION(bg-global): reads the extern cvar `bg_fighterAltControl.integer`, which is
// not resolved in the packet (a bg-tier vmCvar global; ruling 1 places it in GameCvars, no threading here).
pub fn BG_UnrestrainedPitchRoll(
    ps: *mut playerState_t,
    pVeh: *mut Vehicle_t,
) -> qboolean {
    todo!("Port BG_UnrestrainedPitchRoll — parked: bg-global")
}

/// Raven `PM_UpdateViewAngles`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:7813-7894`
// PORT-ESCALATION(pmove-working-state): reads `pm_entVeh`.
pub fn PM_UpdateViewAngles(
    ps: *mut playerState_t,
    cmd: *const usercmd_t,
) {
    todo!("Port PM_UpdateViewAngles — parked: pmove-working-state")
}

/// Raven `PM_AdjustAttackStates`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8031-8199`
// PORT-ESCALATION(pmove-working-state): reads `pm_entSelf`/`pm_entVeh`/`weaponData` globals.
pub fn PM_AdjustAttackStates(
    pm: *mut pmove_t,
) {
    todo!("Port PM_AdjustAttackStates — parked: pmove-working-state")
}

/// Raven `BG_CmdForRoll`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8201-8327`
pub fn BG_CmdForRoll(
    ps: *mut playerState_t,
    anim: c_int,
    pCmd: *mut usercmd_t,
) {
    use animNumber_t::*;
    use crate::bg_panimate::PM_AnimLength;
    unsafe {
        if anim == BOTH_ROLL_F as c_int {
            (*pCmd).forwardmove = 127;
            (*pCmd).rightmove = 0;
        } else if anim == BOTH_ROLL_B as c_int {
            (*pCmd).forwardmove = -127;
            (*pCmd).rightmove = 0;
        } else if anim == BOTH_ROLL_R as c_int {
            (*pCmd).forwardmove = 0;
            (*pCmd).rightmove = 127;
        } else if anim == BOTH_ROLL_L as c_int {
            (*pCmd).forwardmove = 0;
            (*pCmd).rightmove = -127;
        } else if anim == BOTH_GETUP_BROLL_R as c_int {
            (*pCmd).forwardmove = 0;
            (*pCmd).rightmove = 48;
            //NOTE: speed is 400
        } else if anim == BOTH_GETUP_FROLL_R as c_int {
            if (*ps).legsTimer <= 250 {
                //end of anim
                (*pCmd).forwardmove = 0;
                (*pCmd).rightmove = 0;
            } else {
                (*pCmd).forwardmove = 0;
                (*pCmd).rightmove = 48;
            }
        } else if anim == BOTH_GETUP_BROLL_L as c_int {
            (*pCmd).forwardmove = 0;
            (*pCmd).rightmove = -48;
        } else if anim == BOTH_GETUP_FROLL_L as c_int {
            if (*ps).legsTimer <= 250 {
                (*pCmd).forwardmove = 0;
                (*pCmd).rightmove = 0;
            } else {
                (*pCmd).forwardmove = 0;
                (*pCmd).rightmove = -48;
            }
        } else if anim == BOTH_GETUP_BROLL_B as c_int {
            if (*ps).torsoTimer <= 250 {
                //end of anim
                (*pCmd).forwardmove = 0;
                (*pCmd).rightmove = 0;
            } else if PM_AnimLength(0, (*ps).legsAnim) - (*ps).torsoTimer < 350 {
                //beginning of anim
                (*pCmd).forwardmove = 0;
                (*pCmd).rightmove = 0;
            } else {
                (*pCmd).forwardmove = -64;
                (*pCmd).rightmove = 0;
            }
        } else if anim == BOTH_GETUP_FROLL_B as c_int {
            if (*ps).torsoTimer <= 100 {
                (*pCmd).forwardmove = 0;
                (*pCmd).rightmove = 0;
            } else if PM_AnimLength(0, (*ps).legsAnim) - (*ps).torsoTimer < 200 {
                (*pCmd).forwardmove = 0;
                (*pCmd).rightmove = 0;
            } else {
                (*pCmd).forwardmove = -64;
                (*pCmd).rightmove = 0;
            }
        } else if anim == BOTH_GETUP_BROLL_F as c_int {
            if (*ps).torsoTimer <= 550 {
                (*pCmd).forwardmove = 0;
                (*pCmd).rightmove = 0;
            } else if PM_AnimLength(0, (*ps).legsAnim) - (*ps).torsoTimer < 150 {
                (*pCmd).forwardmove = 0;
                (*pCmd).rightmove = 0;
            } else {
                (*pCmd).forwardmove = 64;
                (*pCmd).rightmove = 0;
            }
        } else if anim == BOTH_GETUP_FROLL_F as c_int {
            if (*ps).torsoTimer <= 100 {
                (*pCmd).forwardmove = 0;
                (*pCmd).rightmove = 0;
            } else {
                (*pCmd).forwardmove = 64;
                (*pCmd).rightmove = 0;
            }
        }
        (*pCmd).upmove = 0;
    }
}

/// Raven `BG_AdjustClientSpeed`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8331-8510`
// PORT-ESCALATION(pmove-working-state): reads `pm`/`pm_entSelf`.
pub fn BG_AdjustClientSpeed(
    ps: *mut playerState_t,
    cmd: *mut usercmd_t,
    svTime: c_int,
) {
    todo!("Port BG_AdjustClientSpeed — parked: pmove-working-state")
}

/// Raven `BG_InRollAnim`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8512-8523`
pub fn BG_InRollAnim(
    cent: *mut entityState_t,
) -> qboolean {
    use animNumber_t::*;
    unsafe {
        let a = (*cent).legsAnim;
        if a == BOTH_ROLL_F as c_int
            || a == BOTH_ROLL_B as c_int
            || a == BOTH_ROLL_R as c_int
            || a == BOTH_ROLL_L as c_int
        {
            return qtrue;
        }
        qfalse
    }
}

/// Raven `BG_InKnockDown`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8525-8560`
pub fn BG_InKnockDown(
    anim: c_int,
) -> qboolean {
    use animNumber_t::*;
    if anim == BOTH_KNOCKDOWN1 as c_int
        || anim == BOTH_KNOCKDOWN2 as c_int
        || anim == BOTH_KNOCKDOWN3 as c_int
        || anim == BOTH_KNOCKDOWN4 as c_int
        || anim == BOTH_KNOCKDOWN5 as c_int
    {
        return qtrue;
    }
    if anim == BOTH_GETUP1 as c_int
        || anim == BOTH_GETUP2 as c_int
        || anim == BOTH_GETUP3 as c_int
        || anim == BOTH_GETUP4 as c_int
        || anim == BOTH_GETUP5 as c_int
        || anim == BOTH_FORCE_GETUP_F1 as c_int
        || anim == BOTH_FORCE_GETUP_F2 as c_int
        || anim == BOTH_FORCE_GETUP_B1 as c_int
        || anim == BOTH_FORCE_GETUP_B2 as c_int
        || anim == BOTH_FORCE_GETUP_B3 as c_int
        || anim == BOTH_FORCE_GETUP_B4 as c_int
        || anim == BOTH_FORCE_GETUP_B5 as c_int
        || anim == BOTH_GETUP_BROLL_B as c_int
        || anim == BOTH_GETUP_BROLL_F as c_int
        || anim == BOTH_GETUP_BROLL_L as c_int
        || anim == BOTH_GETUP_BROLL_R as c_int
        || anim == BOTH_GETUP_FROLL_B as c_int
        || anim == BOTH_GETUP_FROLL_F as c_int
        || anim == BOTH_GETUP_FROLL_L as c_int
        || anim == BOTH_GETUP_FROLL_R as c_int
    {
        return qtrue;
    }
    qfalse
}

/// Raven `BG_InRollES`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8562-8574`
pub fn BG_InRollES(
    ps: *mut entityState_t,
    anim: c_int,
) -> qboolean {
    use animNumber_t::*;
    // Raven's `ps` param is unreferenced; the switch keys off `anim`.
    let _ = ps;
    if anim == BOTH_ROLL_F as c_int
        || anim == BOTH_ROLL_B as c_int
        || anim == BOTH_ROLL_R as c_int
        || anim == BOTH_ROLL_L as c_int
    {
        return qtrue;
    }
    qfalse
}

/// Raven `BG_IK_MoveArm`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8576-8730`
// PORT-ESCALATION(bg-global): indexes the extern `bgHumanoidAnimations[basePose]` table, which is
// not resolved in the packet (a bg_panimate.c global).
pub fn BG_IK_MoveArm(
    ghoul2: *mut c_void,
    lHandBolt: c_int,
    time: c_int,
    ent: *mut entityState_t,
    basePose: c_int,
    desiredPos: vec3_t,
    ikInProgress: *mut qboolean,
    origin: vec3_t,
    angles: vec3_t,
    scale: vec3_t,
    blendTime: c_int,
    forceHalt: qboolean,
) {
    todo!("Port BG_IK_MoveArm — parked: bg-global")
}

/// Raven `BG_UpdateLookAngles`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8733-8787`
// fork-9: `lastHeadAngles`/`lookAngles` are written in place → `&mut vec3_t`
// (never NULL in the oracle callers).
pub fn BG_UpdateLookAngles(
    lookingDebounceTime: c_int,
    lastHeadAngles: &mut vec3_t,
    time: c_int,
    lookAngles: &mut vec3_t,
    lookSpeed: f32,
    minPitch: f32,
    maxPitch: f32,
    minYaw: f32,
    maxYaw: f32,
    minRoll: f32,
    maxRoll: f32,
) {
    let fFrameInter: f32 = 0.1;
    // Raven's function-scope `static` scratch (oldLookAngles/lookAnglesDiff/ang)
    // are single-call temporaries (ruling 5) → plain locals.
    let mut oldLookAngles: vec3_t = [0.0; 3];
    let mut lookAnglesDiff: vec3_t = [0.0; 3];

    if lookingDebounceTime > time {
        //clamp so don't get "Exorcist" effect
        if lookAngles[PITCH] > maxPitch {
            lookAngles[PITCH] = maxPitch;
        } else if lookAngles[PITCH] < minPitch {
            lookAngles[PITCH] = minPitch;
        }
        if lookAngles[YAW] > maxYaw {
            lookAngles[YAW] = maxYaw;
        } else if lookAngles[YAW] < minYaw {
            lookAngles[YAW] = minYaw;
        }
        if lookAngles[ROLL] > maxRoll {
            lookAngles[ROLL] = maxRoll;
        } else if lookAngles[ROLL] < minRoll {
            lookAngles[ROLL] = minRoll;
        }

        //slowly lerp to this new value; remember last headAngles
        oldLookAngles = *lastHeadAngles;
        for i in 0..3 {
            lookAnglesDiff[i] = lookAngles[i] - oldLookAngles[i];
        }

        for ang in 0..3 {
            lookAnglesDiff[ang] = crate::q_math::AngleNormalize180(lookAnglesDiff[ang]);
        }

        if crate::q_math::VectorLengthSquared(lookAnglesDiff) != 0.0 {
            lookAngles[PITCH] = crate::q_math::AngleNormalize180(
                oldLookAngles[PITCH] + (lookAnglesDiff[PITCH] * fFrameInter * lookSpeed),
            );
            lookAngles[YAW] = crate::q_math::AngleNormalize180(
                oldLookAngles[YAW] + (lookAnglesDiff[YAW] * fFrameInter * lookSpeed),
            );
            lookAngles[ROLL] = crate::q_math::AngleNormalize180(
                oldLookAngles[ROLL] + (lookAnglesDiff[ROLL] * fFrameInter * lookSpeed),
            );
        }
    }
    //Remember current lookAngles next time
    *lastHeadAngles = *lookAngles;
}

/// Raven `BG_G2ClientNeckAngles`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8790-8866`
// fork-9: `headAngles`/`neckAngles`/`thoracicAngles` are written in place → `&mut vec3_t`;
// `lookAngles`/`headClampMin/MaxAngles` are read-only → keep by-value `vec3_t`.
pub fn BG_G2ClientNeckAngles(
    ghoul2: *mut c_void,
    time: c_int,
    lookAngles: vec3_t,
    headAngles: &mut vec3_t,
    neckAngles: &mut vec3_t,
    thoracicAngles: &mut vec3_t,
    headClampMinAngles: vec3_t,
    headClampMaxAngles: vec3_t,
) {
    let mut lA: vec3_t = lookAngles;
    //clamp the headangles (which should now be relative to the cervical (neck) angles
    if lA[PITCH] < headClampMinAngles[PITCH] {
        lA[PITCH] = headClampMinAngles[PITCH];
    } else if lA[PITCH] > headClampMaxAngles[PITCH] {
        lA[PITCH] = headClampMaxAngles[PITCH];
    }
    if lA[YAW] < headClampMinAngles[YAW] {
        lA[YAW] = headClampMinAngles[YAW];
    } else if lA[YAW] > headClampMaxAngles[YAW] {
        lA[YAW] = headClampMaxAngles[YAW];
    }
    if lA[ROLL] < headClampMinAngles[ROLL] {
        lA[ROLL] = headClampMinAngles[ROLL];
    } else if lA[ROLL] > headClampMaxAngles[ROLL] {
        lA[ROLL] = headClampMaxAngles[ROLL];
    }

    //split it up between the neck and cranium
    if thoracicAngles[PITCH] != 0.0 {
        //already been set above, blend them
        thoracicAngles[PITCH] = (thoracicAngles[PITCH] + (lA[PITCH] * 0.4)) * 0.5;
    } else {
        thoracicAngles[PITCH] = lA[PITCH] * 0.4;
    }
    if thoracicAngles[YAW] != 0.0 {
        thoracicAngles[YAW] = (thoracicAngles[YAW] + (lA[YAW] * 0.1)) * 0.5;
    } else {
        thoracicAngles[YAW] = lA[YAW] * 0.1;
    }
    if thoracicAngles[ROLL] != 0.0 {
        thoracicAngles[ROLL] = (thoracicAngles[ROLL] + (lA[ROLL] * 0.1)) * 0.5;
    } else {
        thoracicAngles[ROLL] = lA[ROLL] * 0.1;
    }

    neckAngles[PITCH] = lA[PITCH] * 0.2;
    neckAngles[YAW] = lA[YAW] * 0.3;
    neckAngles[ROLL] = lA[ROLL] * 0.3;

    headAngles[PITCH] = lA[PITCH] * 0.4;
    headAngles[YAW] = lA[YAW] * 0.6;
    headAngles[ROLL] = lA[ROLL] * 0.6;

    unsafe {
        strap_G2API_SetBoneAngles(
            ghoul2,
            0,
            b"cranium\0".as_ptr() as *const c_char,
            *headAngles,
            BONE_ANGLES_POSTMULT,
            POSITIVE_X as c_int,
            NEGATIVE_Y as c_int,
            NEGATIVE_Z as c_int,
            core::ptr::null_mut(),
            0,
            time,
        );
        strap_G2API_SetBoneAngles(
            ghoul2,
            0,
            b"cervical\0".as_ptr() as *const c_char,
            *neckAngles,
            BONE_ANGLES_POSTMULT,
            POSITIVE_X as c_int,
            NEGATIVE_Y as c_int,
            NEGATIVE_Z as c_int,
            core::ptr::null_mut(),
            0,
            time,
        );
        strap_G2API_SetBoneAngles(
            ghoul2,
            0,
            b"thoracic\0".as_ptr() as *const c_char,
            *thoracicAngles,
            BONE_ANGLES_POSTMULT,
            POSITIVE_X as c_int,
            NEGATIVE_Y as c_int,
            NEGATIVE_Z as c_int,
            core::ptr::null_mut(),
            0,
            time,
        );
    }
}

/// Raven `BG_G2ClientSpineAngles`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8869-8990`
// fork-9: `viewAngles`/`thoracicAngles`/`ulAngles`/`llAngles` are written in place → `&mut vec3_t`;
// `cent_lerpOrigin`/`cent_lerpAngles`/`angles`/`modelScale` are read-only → keep by-value `vec3_t`.
// Only Raven's active `#if 1` correction path is ported (the `#else` branch is dead); with `#if 1`,
// `tPitchAngle`/`tYawAngle`/`corrTime` are unreferenced.
pub fn BG_G2ClientSpineAngles(
    ghoul2: *mut c_void,
    motionBolt: c_int,
    cent_lerpOrigin: vec3_t,
    cent_lerpAngles: vec3_t,
    cent: *mut entityState_t,
    time: c_int,
    viewAngles: &mut vec3_t,
    ciLegs: c_int,
    ciTorso: c_int,
    angles: vec3_t,
    thoracicAngles: &mut vec3_t,
    ulAngles: &mut vec3_t,
    llAngles: &mut vec3_t,
    modelScale: vec3_t,
    tPitchAngle: *mut f32,
    tYawAngle: *mut f32,
    corrTime: *mut c_int,
) {
    use crate::bg_panimate::{
        BG_FlippingAnim, BG_InDeathAnim, BG_InSpecialJump, BG_SaberInSpecial,
        BG_SaberInSpecialAttack, BG_SpinningSaberAnim,
    };
    use crate::g_strap::strap_G2API_GetBoltMatrix_NoRecNoRot;
    unsafe {
        let mut doCorr = qfalse;

        //*tPitchAngle = viewAngles[PITCH];
        viewAngles[YAW] = crate::q_math::AngleDelta(cent_lerpAngles[YAW], angles[YAW]);
        //*tYawAngle = viewAngles[YAW];

        if BG_FlippingAnim((*cent).legsAnim) == qfalse
            && BG_SpinningSaberAnim((*cent).legsAnim) == qfalse
            && BG_SpinningSaberAnim((*cent).torsoAnim) == qfalse
            && BG_InSpecialJump((*cent).legsAnim) == qfalse
            && BG_InSpecialJump((*cent).torsoAnim) == qfalse
            && BG_InDeathAnim((*cent).legsAnim) == qfalse
            && BG_InDeathAnim((*cent).torsoAnim) == qfalse
            && BG_InRollES(cent, (*cent).legsAnim) == qfalse
            && BG_InRollAnim(cent) == qfalse
            && BG_SaberInSpecial((*cent).saberMove) == qfalse
            && BG_SaberInSpecialAttack((*cent).torsoAnim) == qfalse
            && BG_SaberInSpecialAttack((*cent).legsAnim) == qfalse
            && BG_InKnockDown((*cent).torsoAnim) == qfalse
            && BG_InKnockDown((*cent).legsAnim) == qfalse
            && BG_InKnockDown(ciTorso) == qfalse
            && BG_InKnockDown(ciLegs) == qfalse
            && BG_FlippingAnim(ciLegs) == qfalse
            && BG_SpinningSaberAnim(ciLegs) == qfalse
            && BG_SpinningSaberAnim(ciTorso) == qfalse
            && BG_InSpecialJump(ciLegs) == qfalse
            && BG_InSpecialJump(ciTorso) == qfalse
            && BG_InDeathAnim(ciLegs) == qfalse
            && BG_InDeathAnim(ciTorso) == qfalse
            && BG_SaberInSpecialAttack(ciTorso) == qfalse
            && BG_SaberInSpecialAttack(ciLegs) == qfalse
            && ((*cent).eFlags & EF_DEAD) == 0
            && (*cent).legsAnim != (*cent).torsoAnim
            && ciLegs != ciTorso
            && (*cent).m_iVehicleNum == 0
        {
            doCorr = qtrue;
        }

        if doCorr == qtrue {
            //FIXME: no need to do this if legs and torso on are same frame
            //adjust for motion offset
            let mut boltMatrix = mdxaBone_t { matrix: [[0.0; 4]; 3] };
            let mut motionFwd: vec3_t = [0.0; 3];
            let mut motionAngles: vec3_t = [0.0; 3];
            let mut motionRt: vec3_t = [0.0; 3];
            let mut tempAng: vec3_t = [0.0; 3];

            strap_G2API_GetBoltMatrix_NoRecNoRot(
                ghoul2,
                0,
                motionBolt,
                &mut boltMatrix,
                crate::q_math::vec3_origin,
                cent_lerpOrigin,
                time,
                core::ptr::null_mut(),
                modelScale,
            );
            //BG_GiveMeVectorFromMatrix( &boltMatrix, NEGATIVE_Y, motionFwd );
            motionFwd[0] = -boltMatrix.matrix[0][1];
            motionFwd[1] = -boltMatrix.matrix[1][1];
            motionFwd[2] = -boltMatrix.matrix[2][1];

            vectoangles(motionFwd, &mut motionAngles);

            //BG_GiveMeVectorFromMatrix( &boltMatrix, NEGATIVE_X, motionRt );
            motionRt[0] = -boltMatrix.matrix[0][0];
            motionRt[1] = -boltMatrix.matrix[1][0];
            motionRt[2] = -boltMatrix.matrix[2][0];

            vectoangles(motionRt, &mut tempAng);
            motionAngles[ROLL] = -tempAng[PITCH];

            for ang in 0..3 {
                viewAngles[ang] = crate::q_math::AngleNormalize180(
                    viewAngles[ang] - crate::q_math::AngleNormalize180(motionAngles[ang]),
                );
            }
        }

        //distribute the angles differently up the spine
        //NOTE: each of these distributions must add up to 1.0f
        thoracicAngles[PITCH] = viewAngles[PITCH] * 0.20;
        llAngles[PITCH] = viewAngles[PITCH] * 0.40;
        ulAngles[PITCH] = viewAngles[PITCH] * 0.40;

        thoracicAngles[YAW] = viewAngles[YAW] * 0.20;
        ulAngles[YAW] = viewAngles[YAW] * 0.35;
        llAngles[YAW] = viewAngles[YAW] * 0.45;

        thoracicAngles[ROLL] = viewAngles[ROLL] * 0.20;
        ulAngles[ROLL] = viewAngles[ROLL] * 0.35;
        llAngles[ROLL] = viewAngles[ROLL] * 0.45;
    }
}

/// Raven `BG_SwingAngles`.
///
/// Raven: `CG_SwingAngles` — swing an angle towards a destination, modifying
/// speed by the delta and clamping to tolerance.
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:8997-9053`
pub fn BG_SwingAngles(
    destination: f32,
    swingTolerance: f32,
    clampTolerance: f32,
    speed: f32,
    angle: *mut f32,
    swinging: *mut qboolean,
    frametime: c_int,
) -> f32 {
    unsafe {
        let mut swing: f32 = 0.0;
        let mut r#move: f32;
        let mut scale: f32;

        if *swinging == qfalse {
            // see if a swing should be started
            swing = AngleSubtract(*angle, destination);
            if swing > swingTolerance || swing < -swingTolerance {
                *swinging = qtrue;
            }
        }

        if *swinging == qfalse {
            return 0.0;
        }

        // modify the speed depending on the delta so it doesn't seem so linear
        swing = AngleSubtract(destination, *angle);
        scale = swing.abs();
        if scale < swingTolerance * 0.5 {
            scale = 0.5;
        } else if scale < swingTolerance {
            scale = 1.0;
        } else {
            scale = 2.0;
        }

        // swing towards the destination angle
        if swing >= 0.0 {
            r#move = frametime as f32 * scale * speed;
            if r#move >= swing {
                r#move = swing;
                *swinging = qfalse;
            }
            *angle = AngleMod(*angle + r#move);
        } else if swing < 0.0 {
            r#move = frametime as f32 * scale * -speed;
            if r#move <= swing {
                r#move = swing;
                *swinging = qfalse;
            }
            *angle = AngleMod(*angle + r#move);
        }

        // clamp to no more than tolerance
        swing = AngleSubtract(destination, *angle);
        if swing > clampTolerance {
            *angle = AngleMod(destination - (clampTolerance - 1.0));
        } else if swing < -clampTolerance {
            *angle = AngleMod(destination + (clampTolerance - 1.0));
        }

        swing
    }
}

/// Raven `BG_InRoll2`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9058-9078`
pub fn BG_InRoll2(
    es: *mut entityState_t,
) -> qboolean {
    use animNumber_t::*;
    unsafe {
        let a = (*es).legsAnim;
        if a == BOTH_GETUP_BROLL_B as c_int
            || a == BOTH_GETUP_BROLL_F as c_int
            || a == BOTH_GETUP_BROLL_L as c_int
            || a == BOTH_GETUP_BROLL_R as c_int
            || a == BOTH_GETUP_FROLL_B as c_int
            || a == BOTH_GETUP_FROLL_F as c_int
            || a == BOTH_GETUP_FROLL_L as c_int
            || a == BOTH_GETUP_FROLL_R as c_int
            || a == BOTH_ROLL_F as c_int
            || a == BOTH_ROLL_B as c_int
            || a == BOTH_ROLL_R as c_int
            || a == BOTH_ROLL_L as c_int
        {
            return qtrue;
        }
        qfalse
    }
}

/// Raven `BG_G2PlayerAngles`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9082-9457`
// PORT-ESCALATION(bg-boundary): indexes the bg-owned runtime `WeaponReadyAnim[cent->weapon]` table
// (ruling 11: bg-owned state threaded per 8a), but this bg-tier C signature carries no threading
// channel (no `ctx`/`PmoveContext`). fork-9 out-param reshape is otherwise settled.
pub fn BG_G2PlayerAngles(
    ghoul2: *mut c_void,
    motionBolt: c_int,
    cent: *mut entityState_t,
    time: c_int,
    cent_lerpOrigin: vec3_t,
    cent_lerpAngles: vec3_t,
    legs: *mut vec3_t,
    legsAngles: vec3_t,
    tYawing: *mut qboolean,
    tPitching: *mut qboolean,
    lYawing: *mut qboolean,
    tYawAngle: *mut f32,
    tPitchAngle: *mut f32,
    lYawAngle: *mut f32,
    frametime: c_int,
    turAngles: vec3_t,
    modelScale: vec3_t,
    ciLegs: c_int,
    ciTorso: c_int,
    corrTime: *mut c_int,
    lookAngles: vec3_t,
    lastHeadAngles: vec3_t,
    lookTime: c_int,
    emplaced: *mut entityState_t,
    crazySmoothFactor: *mut c_int,
) {
    todo!("Port BG_G2PlayerAngles — parked: bg-boundary")
}

/// Raven `BG_G2ATSTAngles`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9459-9462`
pub fn BG_G2ATSTAngles(
    ghoul2: *mut c_void,
    time: c_int,
    cent_lerpAngles: vec3_t,
) {
    unsafe {
        // up = POSITIVE_X, right = NEGATIVE_Y, fwd = NEGATIVE_Z
        strap_G2API_SetBoneAngles(
            ghoul2,
            0,
            b"thoracic\0".as_ptr() as *const c_char,
            cent_lerpAngles,
            BONE_ANGLES_POSTMULT,
            POSITIVE_X as c_int,
            NEGATIVE_Y as c_int,
            NEGATIVE_Z as c_int,
            core::ptr::null_mut(),
            0,
            time,
        );
    }
}

/// Raven `PM_AdjustAnglesForDualJumpAttack`.
///
/// Raven: the pitch/yaw ucmd override is commented out in the oracle; the live
/// path unconditionally returns qtrue.
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9464-9469`
pub fn PM_AdjustAnglesForDualJumpAttack(
    ps: *mut playerState_t,
    ucmd: *mut usercmd_t,
) -> qboolean {
    qtrue
}

/// Raven `PM_CmdForSaberMoves`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9474-9639`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_CmdForSaberMoves(
    ucmd: *mut usercmd_t,
) {
    todo!("Port PM_CmdForSaberMoves — parked: pmove-working-state")
}

/// Raven `PM_VehicleViewAngles`.
///
/// Raven: constrain the rider's viewangles based on the vehicle's caps (or leave
/// a turret-operating passenger unclamped). `VEH_CONTROL_SCHEME_4` is undefined,
/// so the `#else` (BG_UnrestrainedPitchRoll) branch is the compiled one.
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9642-9713`
pub fn PM_VehicleViewAngles(
    ps: *mut playerState_t,
    veh: *mut bgEntity_t,
    ucmd: *mut usercmd_t,
) {
    unsafe {
        let pVeh: *mut Vehicle_t = (*veh).m_pVehicle;
        let mut setAngles: qboolean = qtrue;
        let mut clampMin: vec3_t = [0.0; 3];
        let mut clampMax: vec3_t = [0.0; 3];

        if !(*(*veh).m_pVehicle).m_pPilot.is_null()
            && (*(*(*veh).m_pVehicle).m_pPilot).s.number == (*ps).clientNum
        {
            // set the pilot's viewangles to the vehicle's viewangles, but only if
            // not doing special free-roll/pitch control
            if BG_UnrestrainedPitchRoll(ps, (*veh).m_pVehicle) == qfalse {
                setAngles = qtrue;
                clampMin[PITCH as usize] = -(*(*pVeh).m_pVehicleInfo).lookPitch;
                clampMax[PITCH as usize] = (*(*pVeh).m_pVehicleInfo).lookPitch;
                clampMin[YAW as usize] = 0.0;
                clampMax[YAW as usize] = 0.0;
                clampMin[ROLL as usize] = -1.0;
                clampMax[ROLL as usize] = -1.0;
            }
        } else {
            // passengers can look around freely, UNLESS they're controlling a turret!
            for i in 0..MAX_VEHICLE_TURRETS {
                if (*(*(*veh).m_pVehicle).m_pVehicleInfo).turret[i as usize].passengerNum
                    == (*ps).generic1
                {
                    // this turret is my station — don't clamp
                    return;
                }
            }
        }

        if setAngles == qtrue {
            for i in 0..3usize {
                // clamp viewangles
                if clampMin[i] == -1.0 || clampMax[i] == -1.0 {
                    // no clamp
                } else if clampMin[i] == 0.0 && clampMax[i] == 0.0 {
                    // no allowance
                } else {
                    // allowance
                    if (*ps).viewangles[i] > clampMax[i] {
                        (*ps).viewangles[i] = clampMax[i];
                    } else if (*ps).viewangles[i] < clampMin[i] {
                        (*ps).viewangles[i] = clampMin[i];
                    }
                }
            }

            PM_SetPMViewAngle(ps, (*ps).viewangles, ucmd);
        }
    }
}

/// Raven `PM_WeaponOkOnVehicle`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9745-9759`
pub fn PM_WeaponOkOnVehicle(
    weapon: c_int,
) -> qboolean {
    // FIXME (Raven): check g_vehicleInfo for our vehicle?
    if weapon == WP_MELEE as c_int
        || weapon == WP_SABER as c_int
        || weapon == WP_BLASTER as c_int
    {
        return qtrue;
    }
    qfalse
}

/// Raven `PM_GetOkWeaponForVehicle`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9762-9780`
// PORT-ESCALATION(pmove-working-state): reads `pm`.
pub fn PM_GetOkWeaponForVehicle() -> c_int {
    todo!("Port PM_GetOkWeaponForVehicle — parked: pmove-working-state")
}

/// Raven `PM_VehForcedTurning`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9783-9830`
// PORT-ESCALATION(pmove-working-state): reads `pml`, writes `pm`.
pub fn PM_VehForcedTurning(
    veh: *mut bgEntity_t,
) {
    todo!("Port PM_VehForcedTurning — parked: pmove-working-state")
}

/// Raven `PM_VehFaceHyperspacePoint`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9916-9989`
// PORT-ESCALATION(pmove-working-state): reads `pml`, writes `pm`.
pub fn PM_VehFaceHyperspacePoint(
    veh: *mut bgEntity_t,
) {
    todo!("Port PM_VehFaceHyperspacePoint — parked: pmove-working-state")
}

/// Raven `BG_VehicleAdjustBBoxForOrientation`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:9993-10076`
// PORT-ESCALATION(fn-pointer-param): `localTrace` is a raw C function-pointer param whose type is
// unported (`void (*)(trace_t*, const vec_t*, const vec_t*, const vec_t*, const vec_t*, int, int)`);
// the body invokes it, so it needs the trap/dispatch shape settled first.
pub fn BG_VehicleAdjustBBoxForOrientation(
    veh: *mut Vehicle_t,
    origin: vec3_t,
    mins: vec3_t,
    maxs: vec3_t,
    clientNum: c_int,
    tracemask: c_int,
    //TODO: Port void ()(trace_t , vec_t , vec_t , vec_t , vec_t , int, int)  (C: `void (*)(trace_t *, const vec_t *, const vec_t *, const vec_t *, const vec_t *, int, int)`)
    localTrace: *mut c_void,
) {
    todo!("Port BG_VehicleAdjustBBoxForOrientation — parked: fn-pointer-param")
}

/// Raven `PM_MoveForKata`.
///
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:10092-10172`
// PORT-ESCALATION(pmove-working-state): writes `pm`.
pub fn PM_MoveForKata(
    ucmd: *mut usercmd_t,
) {
    todo!("Port PM_MoveForKata — parked: pmove-working-state")
}

// `PmoveSingle` is a `PmoveContext` method above (it owns the per-call working
// set the C file-statics used to hold).

/// Raven `Pmove` — the public pmove entrypoint. Constructs one `PmoveContext`
/// per call (ruling 12) from the bg channel handles the game tier supplies,
/// then chops the move into fixed timesteps and runs `PmoveSingle` for each.
/// Source: `oracle/oracle/codemp/game/bg_pmove.c:11167-11215`
pub fn Pmove(
    pmove: *mut pmove_t,
    bg: &mut BgState,
    traps: &dyn BgTraps,
    callbacks: &mut dyn GameCallbacks,
) {
    unsafe {
        let finalTime = (*pmove).cmd.serverTime;

        if finalTime < (*(*pmove).ps).commandTime {
            return; // should not happen
        }

        if finalTime > (*(*pmove).ps).commandTime + 1000 {
            (*(*pmove).ps).commandTime = finalTime - 1000;
        }

        if (*(*pmove).ps).fallingToDeath != 0 {
            (*pmove).cmd.forwardmove = 0;
            (*pmove).cmd.rightmove = 0;
            (*pmove).cmd.upmove = 0;
            (*pmove).cmd.buttons = 0;
        }

        (*(*pmove).ps).pmove_framecount =
            ((*(*pmove).ps).pmove_framecount + 1) & ((1 << PS_PMOVEFRAMECOUNTBITS) - 1);

        // One working-set context for the whole (possibly multi-step) move.
        let mut pmc = PmoveContext::new(bg, traps, callbacks);

        // chop the move up if it is too long, to prevent framerate-dependent behavior
        while (*(*pmove).ps).commandTime != finalTime {
            let mut msec = finalTime - (*(*pmove).ps).commandTime;

            if (*pmove).pmove_fixed != 0 {
                if msec > (*pmove).pmove_msec {
                    msec = (*pmove).pmove_msec;
                }
            } else if msec > 66 {
                msec = 66;
            }
            (*pmove).cmd.serverTime = (*(*pmove).ps).commandTime + msec;

            pmc.PmoveSingle(pmove);

            if (*(*pmove).ps).pm_flags & PMF_JUMP_HELD != 0 {
                (*pmove).cmd.upmove = 20;
            }
        }
    }
}
