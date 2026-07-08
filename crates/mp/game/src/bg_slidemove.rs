// PORT-COMPLETE: bg_slidemove.c 5/5
//! Ported `oracle/oracle/codemp/game/bg_slidemove.c` bodies (pass 3).
//!
//! All five functions are built on the pmove working set that `bg_pmove.rs`
//! threads as `PmoveContext`: `self.pm`/`self.pml`/
//! `self.pm_entSelf`, `self.bg` (session state), `self.traps` (`BgTraps`)
//! and `self.callbacks` (`GameCallbacks`) replace the
//! Raven file statics and the `QAGAME`-only game-tier calls. They are `impl
//! PmoveContext<'_>` methods, mirroring the proven shape already landed for
//! `PM_BGEntForNum`/`PM_ClipVelocity`/`PM_Friction` in `bg_pmove.rs`.
//!
//! `PM_VehicleImpact` is `#ifdef QAGAME`/`#else` duplicated in the oracle (game
//! vs. cgame-prediction bodies, porting-rules §20). This crate is the game
//! (`jampgame`) tier, so the `QAGAME` branch is the one transcribed here; the
//! `#else` cgame-prediction branch is out of scope for `mp_game` and is not
//! ported in this file (a future cgame port re-derives it per §20).
#![allow(non_snake_case, unused, clippy::all)]

use crate::g_main::Com_Printf;
use crate::prelude::*;
use crate::q_math::{AngleNormalize180, AnglesSubtract, VectorLength, VectorNormalize2};
use mp_bg::vehicles::MIN_LANDING_SLOPE;
use mp_qshared::shared::trajectory::trType_t;

// --- file-local `#defines` from `bg_slidemove.c` (porting-rules per-file
// convention, cf. `bg_pmove.rs`'s `MIN_WALK_NORMAL`/`PMF_STUCK_TO_WALL`). ---

/// `MAX_IMPACT_TURN_ANGLE`. Source: `oracle/oracle/codemp/game/bg_slidemove.c:48`
pub const MAX_IMPACT_TURN_ANGLE: f32 = 45.0;
/// `MAX_CLIP_PLANES`. Source: `oracle/oracle/codemp/game/bg_slidemove.c:633`
pub const MAX_CLIP_PLANES: usize = 5;
/// `MIN_LANDING_SPEED`. Source: `oracle/oracle/codemp/game/bg_vehicles.h:399`
pub const MIN_LANDING_SPEED: f32 = 200.0;
/// `OVERCLIP`. Source: `oracle/oracle/codemp/game/bg_local.h:10`
pub const OVERCLIP: f32 = 1.001;
/// `STEPSIZE`. Source: `oracle/oracle/codemp/game/bg_public.h:22`
pub const STEPSIZE: f32 = 18.0;
/// `MIN_WALK_NORMAL`. Source: `oracle/oracle/codemp/game/bg_local.h:5`
const MIN_WALK_NORMAL: f32 = 0.7;
// `PMF_STUCK_TO_WALL` local shadow removed in the const sweep — the qshared
// `pm_flags` canonical (value 16384) reaches this file via `crate::prelude::*`.
// `SOLID_BMODEL` (`q_shared.h:2642`) canonical in `mp_qshared::shared::surface_flags`,
// reaches this file via `crate::prelude::*`.

impl PmoveContext<'_> {
    /// Raven `PM_VehicleImpact` — vehicle-vs-world/entity impact damage,
    /// bounce and turn-away reaction (`QAGAME` branch only, see module doc).
    ///
    /// Source: `oracle/oracle/codemp/game/bg_slidemove.c:49-557`
    pub fn PM_VehicleImpact(&mut self, pEnt: *mut bgEntity_t, trace: *mut trace_t) {
        unsafe {
            let pSelfVeh = ((*pEnt).m_pVehicle as *mut Vehicle_t);
            let pSelfVehInfo = (*pSelfVeh).m_pVehicleInfo;
            let ps = (*self.pm).ps;
            let velocity = (*ps).velocity;
            let mut magnitude = VectorLength(velocity) * (*pSelfVehInfo).mass as f32 / 50.0;
            let mut forceSurfDestruction: qboolean = QFALSE;

            let hitEnt: *mut bgEntity_t = if !trace.is_null() {
                self.PM_BGEntForNum((*trace).entityNum as c_int)
            } else {
                core::ptr::null_mut()
            };

            // PORT-NOTE(bg-tier-gap): Raven's `hitEnt` here is a full
            // `gentity_t*` (inuse/r.ownerNum/client/…), which the bg tier's
            // `bgEntity_t` overlay does not expose. Fields below
            // that only exist on `gentity_t` are referenced literally and
            // reported as missing symbols; a fixer must widen the bg-visible
            // surface (new `BgTraps`/`GameCallbacks` accessors) to close them.
            if hitEnt.is_null()
                || (!pSelfVeh.is_null()
                    && !(*pSelfVeh).m_pPilot.is_null()
                    && (*hitEnt).s.eType == ET_MISSILE as c_int
                    && (*hitEnt).inuse != 0
                    && (*hitEnt).r.ownerNum == (*(*pSelfVeh).m_pPilot).s.number)
            {
                // don't hit it
                return;
            }

            if !pSelfVeh.is_null() && (*pSelfVeh).m_iRemovedSurfaces != 0 {
                // spiralling to our deaths, explode on any solid impact
                if (*hitEnt).s.NPC_class == CLASS_VEHICLE as c_int {
                    // Give credit to whoever got me into this death spiral state
                    self.callbacks.damage_from_killer(
                        (*pEnt).s.number,
                        (*(*pSelfVeh).m_pParentEntity).s.number,
                        (*hitEnt).s.number,
                        // Raven `G_DamageFromKiller` initializes `killer = attacker`.
                        (*hitEnt).s.number,
                        core::ptr::null(),
                        core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                        999999,
                        DAMAGE_NO_ARMOR,
                        MOD_COLLISION as c_int,
                    );
                    return;
                } else if (*trace).plane.normal != [0.0, 0.0, 0.0]
                    && ((*trace).entityNum as c_int == ENTITYNUM_WORLD as c_int
                        || (*hitEnt).r.bmodel != 0)
                {
                    // have a valid hit plane and we hit a solid brush
                    let mut moveDir = (*ps).velocity;
                    VectorNormalize(&mut moveDir);
                    let impactDot = moveDir[0] * (*trace).plane.normal[0]
                        + moveDir[1] * (*trace).plane.normal[1]
                        + moveDir[2] * (*trace).plane.normal[2];
                    if impactDot <= -0.7 {
                        // hit rather head-on and hard: just DIE now
                        self.callbacks.damage_from_killer(
                            (*pEnt).s.number,
                            (*(*pSelfVeh).m_pParentEntity).s.number,
                            (*hitEnt).s.number,
                            // Raven `G_DamageFromKiller` initializes `killer = attacker`.
                            (*hitEnt).s.number,
                            core::ptr::null(),
                            core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                            999999,
                            DAMAGE_NO_ARMOR,
                            MOD_FALLING as c_int,
                        );
                        return;
                    }
                }
            }

            if ((*trace).entityNum as c_int) < ENTITYNUM_WORLD as c_int
                && (*hitEnt).s.eType == ET_MOVER as c_int
                && (*hitEnt).s.apos.trType as c_int != trType_t::TR_STATIONARY as c_int
                && ((*hitEnt).spawnflags & 16) != 0
                && Q_stricmp(
                    (*hitEnt).classname,
                    c"func_rotating".as_ptr() as *const c_char,
                ) == 0
            {
                // hit a func_rotating that is supposed to destroy anything it touches!
                forceSurfDestruction = QTRUE;
            } else if (Q_fabs((*ps).velocity[0]) + Q_fabs((*ps).velocity[1])) < 100.0
                && (*ps).velocity[2] > -100.0
            {
                // we're landing, we're cool
                if !(!hitEnt.is_null()
                    && ((*hitEnt).s.eType == ET_PLAYER as c_int
                        || (*hitEnt).s.eType == ET_NPC as c_int)
                    && (*pSelfVehInfo).r#type as c_int == VH_FIGHTER as c_int)
                {
                    return;
                }
            }

            if !pSelfVeh.is_null()
                && ((*pSelfVehInfo).r#type as c_int == VH_SPEEDER as c_int
                    || (*pSelfVehInfo).r#type as c_int == VH_FIGHTER as c_int)
                && (magnitude >= 100.0 || forceSurfDestruction != 0)
            {
                if (*((*pEnt).m_pVehicle as *mut Vehicle_t)).m_iHitDebounce
                    < (*self.pm).cmd.serverTime
                    || forceSurfDestruction != 0
                {
                    let mut noDamage: qboolean = QFALSE;

                    if !trace.is_null()
                        && (*pSelfVeh).m_iRemovedSurfaces == 0
                        && forceSurfDestruction == 0
                    {
                        let mut turnFromImpact: qboolean = QFALSE;
                        let mut turnHitEnt: qboolean = QFALSE;
                        let l0 = (*ps).speed * 0.5;
                        let mut bounceDir: vec3_t = [0.0; 3];

                        if ((*trace).entityNum as c_int == ENTITYNUM_WORLD as c_int
                            || (*hitEnt).s.solid == SOLID_BMODEL as c_int)
                            && (*trace).plane.normal != [0.0, 0.0, 0.0]
                        {
                            // bounce off in the opposite direction of the impact
                            if (*pSelfVehInfo).r#type as c_int == VH_SPEEDER as c_int {
                                (*ps).speed *= self.pml.frametime;
                                bounceDir = (*trace).plane.normal;
                            } else if (*trace).plane.normal[2] >= MIN_LANDING_SLOPE
                                && (*pSelfVeh).m_LandTrace.fraction < 1.0
                                && (*ps).speed <= MIN_LANDING_SPEED
                            {
                                // could land here, don't bounce off, return altogether!
                                return;
                            } else {
                                if (*pSelfVehInfo).r#type as c_int == VH_FIGHTER as c_int {
                                    turnFromImpact = QTRUE;
                                }
                                bounceDir = (*trace).plane.normal;
                            }
                        } else if (*pSelfVehInfo).r#type as c_int == VH_FIGHTER as c_int {
                            // check for impact with another fighter
                            if (*hitEnt).s.NPC_class == CLASS_VEHICLE as c_int
                                && !((*hitEnt).m_pVehicle as *mut Vehicle_t).is_null()
                                && !(*((*hitEnt).m_pVehicle as *mut Vehicle_t))
                                    .m_pVehicleInfo
                                    .is_null()
                                && (*(*((*hitEnt).m_pVehicle as *mut Vehicle_t)).m_pVehicleInfo)
                                    .r#type as c_int
                                    == VH_FIGHTER as c_int
                            {
                                // two vehicles hit each other, turn away from the impact
                                turnFromImpact = QTRUE;
                                turnHitEnt = QTRUE;
                                for i in 0..3 {
                                    bounceDir[i] = (*ps).origin[i] - (*hitEnt).r.currentOrigin[i];
                                }
                                VectorNormalize(&mut bounceDir);
                            }
                        }

                        if turnFromImpact != 0 {
                            // bounce off impact surf and turn away
                            let mut pushDir: vec3_t = [0.0; 3];
                            let mut turnAwayAngles: vec3_t = [0.0; 3];
                            let mut turnDelta: vec3_t = [0.0; 3];
                            let mut moveDir: vec3_t = [0.0; 3];

                            if turnHitEnt == 0 {
                                // hit wall
                                let scale = (*ps).speed * 0.25 / (*pSelfVehInfo).mass as f32;
                                for i in 0..3 {
                                    pushDir[i] = bounceDir[i] * scale;
                                }
                            } else {
                                // hit another fighter
                                let hitSpeed = if !((*hitEnt).client as *mut gclient_t).is_null() {
                                    (*((*hitEnt).client as *mut gclient_t)).ps.speed
                                } else {
                                    (*hitEnt).s.speed
                                };
                                // QAGAME side (bg_slidemove.c:221-231): all three VectorScales
                                // write pushDir; bounceDir stays intact for bounceDot below.
                                let scale1 = ((*ps).speed + hitSpeed) * 0.5;
                                let scale2 = l0 / (*pSelfVehInfo).mass as f32;
                                for i in 0..3 {
                                    pushDir[i] = bounceDir[i] * scale1 * scale2 * 0.1;
                                }
                            }
                            VectorNormalize2((*ps).velocity, &mut moveDir);
                            let mut bounceDot = -(moveDir[0] * bounceDir[0]
                                + moveDir[1] * bounceDir[1]
                                + moveDir[2] * bounceDir[2]);
                            if bounceDot < 0.1 {
                                bounceDot = 0.1;
                            }
                            for i in 0..3 {
                                pushDir[i] *= bounceDot;
                            }
                            for i in 0..3 {
                                (*ps).velocity[i] += pushDir[i];
                            }
                            // turn
                            let mut turnDivider = (*pSelfVehInfo).mass as f32 / 400.0;
                            if turnHitEnt != 0 {
                                turnDivider *= 4.0;
                            }
                            if turnDivider < 0.5 {
                                turnDivider = 0.5;
                            }
                            let mut turnStrength = magnitude / 2000.0;
                            if turnStrength < 0.1 {
                                turnStrength = 0.1;
                            } else if turnStrength > 2.0 {
                                turnStrength = 2.0;
                            }
                            vectoangles(bounceDir, &mut turnAwayAngles);
                            AnglesSubtract(
                                turnAwayAngles,
                                *(*pSelfVeh).m_vOrientation.cast::<vec3_t>(),
                                &mut turnDelta,
                            );
                            let orientation = &mut *(*pSelfVeh).m_vOrientation.cast::<vec3_t>();
                            if bounceDir[2] != 0.0 {
                                let mut pitchTurnStrength =
                                    turnStrength * turnDelta[PITCH as usize];
                                if pitchTurnStrength > MAX_IMPACT_TURN_ANGLE {
                                    pitchTurnStrength = MAX_IMPACT_TURN_ANGLE;
                                } else if pitchTurnStrength < -MAX_IMPACT_TURN_ANGLE {
                                    pitchTurnStrength = -MAX_IMPACT_TURN_ANGLE;
                                }
                                (*pSelfVeh).m_vFullAngleVelocity[PITCH as usize] =
                                    AngleNormalize180(
                                        orientation[PITCH as usize]
                                            + pitchTurnStrength / turnDivider
                                                * (*pSelfVeh).m_fTimeModifier,
                                    );
                            }
                            if bounceDir[0] != 0.0 || bounceDir[1] != 0.0 {
                                let mut yawTurnStrength = turnStrength * turnDelta[YAW as usize];
                                if yawTurnStrength > MAX_IMPACT_TURN_ANGLE {
                                    yawTurnStrength = MAX_IMPACT_TURN_ANGLE;
                                } else if yawTurnStrength < -MAX_IMPACT_TURN_ANGLE {
                                    yawTurnStrength = -MAX_IMPACT_TURN_ANGLE;
                                }
                                (*pSelfVeh).m_vFullAngleVelocity[ROLL as usize] = AngleNormalize180(
                                    orientation[ROLL as usize]
                                        - yawTurnStrength / turnDivider
                                            * (*pSelfVeh).m_fTimeModifier,
                                );
                            }

                            // PORT-NOTE(fn-ptr-skip): the `#ifdef QAGAME` block that
                            // turns/pushes `hitEnt` (the other ship we hit) away too
                            // needs the same gentity_t-only fields as above
                            // (client/spawnflags/m_pVehicle). Transcribed literally
                            // below; several field accesses are bg-tier gaps.
                            if turnHitEnt != 0
                                && !((*hitEnt).client as *mut gclient_t).is_null()
                                && FighterIsLanded(
                                    ((*hitEnt).m_pVehicle as *mut Vehicle_t),
                                    core::ptr::addr_of_mut!(
                                        (*((*hitEnt).client as *mut gclient_t)).ps
                                    ),
                                ) == 0
                                && ((*hitEnt).spawnflags & 2) == 0
                            {
                                let l = (*((*hitEnt).client as *mut gclient_t)).ps.speed;
                                for i in 0..3 {
                                    bounceDir[i] = -bounceDir[i];
                                }
                                let mut pushDir2: vec3_t = [0.0; 3];
                                let scale = ((*ps).speed + l) * 0.5;
                                for i in 0..3 {
                                    pushDir2[i] = bounceDir[i] * scale;
                                }
                                let hitVehInfo =
                                    (*((*hitEnt).m_pVehicle as *mut Vehicle_t)).m_pVehicleInfo;
                                let scale2 = l * 0.5 / (*hitVehInfo).mass as f32;
                                for i in 0..3 {
                                    pushDir2[i] *= scale2;
                                }
                                let mut moveDir2: vec3_t = [0.0; 3];
                                VectorNormalize2(
                                    (*((*hitEnt).client as *mut gclient_t)).ps.velocity,
                                    &mut moveDir2,
                                );
                                let mut bounceDot2 = -(moveDir2[0] * bounceDir[0]
                                    + moveDir2[1] * bounceDir[1]
                                    + moveDir2[2] * bounceDir[2]);
                                if bounceDot2 < 0.1 {
                                    bounceDot2 = 0.1;
                                }
                                for i in 0..3 {
                                    pushDir2[i] *= bounceDot2;
                                }
                                for i in 0..3 {
                                    (*((*hitEnt).client as *mut gclient_t)).ps.velocity[i] +=
                                        pushDir2[i];
                                }
                                let mut turnDivider2 = (*hitVehInfo).mass as f32 / 400.0;
                                if turnHitEnt != 0 {
                                    turnDivider2 *= 4.0;
                                }
                                if turnDivider2 < 0.5 {
                                    turnDivider2 = 0.5;
                                }
                                let mut turnAwayAngles2: vec3_t = [0.0; 3];
                                let mut turnDelta2: vec3_t = [0.0; 3];
                                vectoangles(bounceDir, &mut turnAwayAngles2);
                                let hitOrient = &mut *(*((*hitEnt).m_pVehicle as *mut Vehicle_t))
                                    .m_vOrientation
                                    .cast::<vec3_t>();
                                AnglesSubtract(turnAwayAngles2, *hitOrient, &mut turnDelta2);
                                if bounceDir[2] != 0.0 {
                                    let mut pitchTurnStrength2 =
                                        turnStrength * turnDelta2[PITCH as usize];
                                    if pitchTurnStrength2 > MAX_IMPACT_TURN_ANGLE {
                                        pitchTurnStrength2 = MAX_IMPACT_TURN_ANGLE;
                                    } else if pitchTurnStrength2 < -MAX_IMPACT_TURN_ANGLE {
                                        pitchTurnStrength2 = -MAX_IMPACT_TURN_ANGLE;
                                    }
                                    (*((*hitEnt).m_pVehicle as *mut Vehicle_t))
                                        .m_vFullAngleVelocity
                                        [PITCH as usize] = AngleNormalize180(
                                        hitOrient[PITCH as usize]
                                            + pitchTurnStrength2 / turnDivider2
                                                * (*pSelfVeh).m_fTimeModifier,
                                    );
                                }
                                if bounceDir[0] != 0.0 || bounceDir[1] != 0.0 {
                                    let mut yawTurnStrength2 =
                                        turnStrength * turnDelta2[YAW as usize];
                                    if yawTurnStrength2 > MAX_IMPACT_TURN_ANGLE {
                                        yawTurnStrength2 = MAX_IMPACT_TURN_ANGLE;
                                    } else if yawTurnStrength2 < -MAX_IMPACT_TURN_ANGLE {
                                        yawTurnStrength2 = -MAX_IMPACT_TURN_ANGLE;
                                    }
                                    (*((*hitEnt).m_pVehicle as *mut Vehicle_t))
                                        .m_vFullAngleVelocity
                                        [ROLL as usize] = AngleNormalize180(
                                        hitOrient[ROLL as usize]
                                            - yawTurnStrength2 / turnDivider2
                                                * (*pSelfVeh).m_fTimeModifier,
                                    );
                                }
                            }
                        }
                    }

                    if hitEnt.is_null() {
                        return;
                    }

                    let mut vehUp: vec3_t = [0.0; 3];
                    AngleVectors(
                        *(*pSelfVeh).m_vOrientation.cast::<vec3_t>(),
                        None,
                        None,
                        Some(&mut vehUp),
                    );
                    if (*pSelfVehInfo).iImpactFX != 0 {
                        // tempent use bad! (Raven comment)
                        self.callbacks.add_event(
                            (*pEnt).s.number,
                            EV_PLAY_EFFECT_ID as c_int,
                            (*pSelfVehInfo).iImpactFX,
                        );
                    }
                    (*((*pEnt).m_pVehicle as *mut Vehicle_t)).m_iHitDebounce =
                        (*self.pm).cmd.serverTime + 200;
                    magnitude /= (*pSelfVehInfo).toughness * 50.0;

                    if (*hitEnt).s.eType != ET_TERRAIN as c_int
                        || ((*hitEnt).spawnflags & 1) == 0
                        || (*pSelfVehInfo).r#type as c_int == VH_FIGHTER as c_int
                    {
                        // don't damage the vehicle from terrain that doesn't want to damage vehicles
                        let mut killerNum: c_int = (*pEnt).s.number;
                        let mut haveKiller = false;
                        if (*pSelfVehInfo).r#type as c_int == VH_FIGHTER as c_int {
                            let mut mult = (*(*pSelfVeh).m_vOrientation.cast::<vec3_t>())
                                [PITCH as usize]
                                * 0.1;
                            if mult < 1.0 {
                                mult = 1.0;
                            }
                            if (*hitEnt).inuse != 0 && (*hitEnt).takedamage != 0 {
                                if (*hitEnt).s.eType == ET_NPC as c_int
                                    && (*hitEnt).s.NPC_class == CLASS_VEHICLE as c_int
                                    && !((*hitEnt).m_pVehicle as *mut Vehicle_t).is_null()
                                {
                                    mult = 1.5;
                                } else {
                                    mult = 0.5;
                                }
                            }
                            magnitude *= mult;
                        }
                        (*pSelfVeh).m_iLastImpactDmg = magnitude as c_int;
                        if (*hitEnt).s.eType == ET_MISSILE as c_int {
                            // FIX: NEVER do or take impact damage from a missile...
                            noDamage = QTRUE;
                            if ((*hitEnt).s.eFlags & EF_JETPACK_ACTIVE) != 0
                                && (*hitEnt).r.ownerNum < MAX_CLIENTS as c_int
                            {
                                killerNum = (*hitEnt).r.ownerNum;
                                haveKiller = true;
                            }
                        }
                        if noDamage == 0 {
                            let mod_ = if (*hitEnt).s.NPC_class == CLASS_VEHICLE as c_int {
                                MOD_COLLISION as c_int
                            } else {
                                MOD_FALLING as c_int
                            };
                            let attacker = if haveKiller {
                                killerNum
                            } else {
                                (*hitEnt).s.number
                            };
                            // Oracle bg_slidemove.c:466: targ = pEnt (the vehicle damages
                            // ITSELF on impact); inflictor = hitEnt.
                            self.callbacks.damage(
                                (*pEnt).s.number,
                                (*hitEnt).s.number,
                                attacker,
                                core::ptr::null(),
                                core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                                (magnitude * 5.0) as c_int,
                                DAMAGE_NO_ARMOR,
                                mod_,
                            );
                        }

                        if (*pSelfVehInfo).surfDestruction != 0 {
                            // Oracle bg_slidemove.c:472: pass the live impact `trace` and
                            // the `forceSurfDestruction` flag (both in scope here).
                            self.callbacks.flyveh_surface_destruction(
                                (*pEnt).s.number,
                                trace,
                                magnitude as c_int,
                                forceSurfDestruction,
                            );
                        }

                        (*pSelfVeh).m_ulFlags |= VEH_CRASHING as u64;
                    }

                    if (*hitEnt).inuse != 0 && (*hitEnt).takedamage != 0 {
                        // damage this guy because we hit him
                        let mut pmult: f32 = 1.0;
                        if ((*hitEnt).s.eType == ET_PLAYER as c_int
                            && (*hitEnt).s.number < MAX_CLIENTS as c_int)
                            || ((*hitEnt).s.eType == ET_NPC as c_int
                                && (*hitEnt).s.NPC_class != CLASS_VEHICLE as c_int)
                        {
                            // probably a humanoid, or something
                            if (*pSelfVehInfo).r#type as c_int == VH_FIGHTER as c_int {
                                pmult = 2000.0;
                            } else {
                                pmult = 40.0;
                            }

                            if !((*hitEnt).client as *mut gclient_t).is_null()
                                && BG_KnockDownable(core::ptr::addr_of_mut!(
                                    (*((*hitEnt).client as *mut gclient_t)).ps
                                )) != 0
                                && self
                                    .callbacks
                                    .can_be_enemy((*pEnt).s.number, (*hitEnt).s.number)
                                    != 0
                            {
                                // smash!
                                if (*((*hitEnt).client as *mut gclient_t)).ps.forceHandExtend
                                    != HANDEXTEND_KNOCKDOWN as c_int
                                {
                                    (*((*hitEnt).client as *mut gclient_t)).ps.forceHandExtend =
                                        HANDEXTEND_KNOCKDOWN as c_int;
                                    (*((*hitEnt).client as *mut gclient_t))
                                        .ps
                                        .forceHandExtendTime = (*self.pm).cmd.serverTime + 1100;
                                    (*((*hitEnt).client as *mut gclient_t)).ps.forceDodgeAnim = 0;
                                }
                                (*((*hitEnt).client as *mut gclient_t)).ps.otherKiller =
                                    (*pEnt).s.number;
                                (*((*hitEnt).client as *mut gclient_t)).ps.otherKillerTime =
                                    (*self.pm).cmd.serverTime + 5000;
                                (*((*hitEnt).client as *mut gclient_t))
                                    .ps
                                    .otherKillerDebounceTime = (*self.pm).cmd.serverTime + 100;
                                (*((*hitEnt).client as *mut gclient_t)).otherKillerMOD =
                                    MOD_COLLISION as c_int;
                                (*((*hitEnt).client as *mut gclient_t)).otherKillerVehWeapon = 0;
                                (*((*hitEnt).client as *mut gclient_t)).otherKillerWeaponType =
                                    WP_NONE as c_int;
                                for i in 0..3 {
                                    (*((*hitEnt).client as *mut gclient_t)).ps.velocity[i] +=
                                        (*ps).velocity[i];
                                }
                                (*((*hitEnt).client as *mut gclient_t)).ps.velocity[2] += 200.0;
                            }
                        }

                        let attackNum = if !(*pSelfVeh).m_pPilot.is_null() {
                            (*(*pSelfVeh).m_pPilot).s.number
                        } else {
                            (*pEnt).s.number
                        };

                        let mut finalD = (magnitude * pmult) as c_int;
                        if finalD < 1 {
                            finalD = 1;
                        }
                        if noDamage == 0 {
                            let mod_ = if (*hitEnt).s.NPC_class == CLASS_VEHICLE as c_int {
                                MOD_COLLISION as c_int
                            } else {
                                MOD_FALLING as c_int
                            };
                            self.callbacks.damage(
                                (*hitEnt).s.number,
                                attackNum,
                                attackNum,
                                core::ptr::null(),
                                core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                                finalD,
                                0,
                                mod_,
                            );
                        }
                    }
                }
            }
        }
    }

    /// Raven `PM_GroundSlideOkay`.
    ///
    /// Source: `oracle/oracle/codemp/game/bg_slidemove.c:559-580`
    pub fn PM_GroundSlideOkay(&self, zNormal: f32) -> qboolean {
        unsafe {
            let ps = &*(*self.pm).ps;
            if zNormal > 0.0 && ps.velocity[2] > 0.0 {
                let legsAnim = ps.legsAnim;
                if legsAnim == BOTH_WALL_RUN_RIGHT as c_int
                    || legsAnim == BOTH_WALL_RUN_LEFT as c_int
                    || legsAnim == BOTH_WALL_RUN_RIGHT_STOP as c_int
                    || legsAnim == BOTH_WALL_RUN_LEFT_STOP as c_int
                    || legsAnim == BOTH_FORCEWALLRUNFLIP_START as c_int
                    || legsAnim == BOTH_FORCELONGLEAP_START as c_int
                    || legsAnim == BOTH_FORCELONGLEAP_ATTACK as c_int
                    || legsAnim == BOTH_FORCELONGLEAP_LAND as c_int
                    || BG_InReboundJump(legsAnim) != 0
                {
                    return QFALSE;
                }
            }
            QTRUE
        }
    }

    /// Raven `PM_ClientImpact`.
    ///
    /// Source: `oracle/oracle/codemp/game/bg_slidemove.c:590-623`
    pub fn PM_ClientImpact(&mut self, trace: *mut trace_t) -> qboolean {
        unsafe {
            let otherEntityNum = (*trace).entityNum as c_int;

            if self.pm_entSelf.is_null() {
                return QFALSE;
            }

            if otherEntityNum >= ENTITYNUM_WORLD as c_int {
                return QFALSE;
            }

            let ps = &*(*self.pm).ps;
            if VectorLength(ps.velocity) >= 100.0
                && (*self.pm_entSelf).s.NPC_class != CLASS_VEHICLE as c_int
                && ps.lastOnGround + 100 < self.callbacks.get_time()
            {
                self.callbacks
                    .client_check_impact_bbrush((*self.pm_entSelf).s.number, otherEntityNum);
            }

            // PORT-NOTE(bg-tier-gap): `traceEnt->r.contents` is a `gentity_t`-only
            // field not on the bg-visible `bgEntity_t` overlay; referenced
            // literally and reported as a missing symbol.
            let traceEnt = self.PM_BGEntForNum(otherEntityNum);
            if traceEnt.is_null() || ((*traceEnt).r.contents & (*self.pm).tracemask) == 0 {
                // it's dead or not in my way anymore, don't clip against it
                return QTRUE;
            }

            QFALSE
        }
    }

    /// Raven `PM_SlideMove`.
    ///
    /// Source: `oracle/oracle/codemp/game/bg_slidemove.c:634-853`
    pub fn PM_SlideMove(&mut self, gravity: qboolean) -> qboolean {
        unsafe {
            let numbumps = 4;
            let ps = (*self.pm).ps;
            let mut primal_velocity = (*ps).velocity;
            let mut endVelocity: vec3_t = [0.0; 3];

            if gravity != 0 {
                endVelocity = (*ps).velocity;
                endVelocity[2] -= (*ps).gravity as f32 * self.pml.frametime;
                (*ps).velocity[2] = ((*ps).velocity[2] + endVelocity[2]) * 0.5;
                primal_velocity[2] = endVelocity[2];
                if self.pml.groundPlane != 0 {
                    if self.PM_GroundSlideOkay(self.pml.groundTrace.plane.normal[2]) != 0 {
                        // slide along the ground plane
                        let normal = self.pml.groundTrace.plane.normal;
                        let mut out = (*ps).velocity;
                        self.PM_ClipVelocity((*ps).velocity, normal, &mut out, OVERCLIP);
                        (*ps).velocity = out;
                    }
                }
            }

            let mut time_left = self.pml.frametime;

            let mut planes: [vec3_t; MAX_CLIP_PLANES] = [[0.0; 3]; MAX_CLIP_PLANES];
            let mut numplanes: usize;

            // never turn against the ground plane
            if self.pml.groundPlane != 0 {
                numplanes = 1;
                planes[0] = self.pml.groundTrace.plane.normal;
                if self.PM_GroundSlideOkay(planes[0][2]) == 0 {
                    planes[0][2] = 0.0;
                    VectorNormalize(&mut planes[0]);
                }
            } else {
                numplanes = 0;
            }

            // never turn against original velocity
            VectorNormalize2((*ps).velocity, &mut planes[numplanes]);
            numplanes += 1;

            let mut bumpcount = 0;
            while bumpcount < numbumps {
                // calculate position we are trying to move to
                let mut end: vec3_t = [0.0; 3];
                for i in 0..3 {
                    end[i] = (*ps).origin[i] + time_left * (*ps).velocity[i];
                }

                // see if we can make it there
                let mut trace: trace_t = core::mem::zeroed();
                self.traps.trace(
                    &mut trace,
                    core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                    core::ptr::addr_of!((*self.pm).mins) as *const vec3_t,
                    core::ptr::addr_of!((*self.pm).maxs) as *const vec3_t,
                    core::ptr::addr_of!(end) as *const vec3_t,
                    (*ps).clientNum,
                    (*self.pm).tracemask,
                );

                if trace.allsolid != 0 {
                    // entity is completely trapped in another solid
                    (*ps).velocity[2] = 0.0; // don't build up falling damage, but allow sideways acceleration
                    return QTRUE;
                }

                if trace.fraction > 0.0 {
                    // actually covered some distance
                    (*ps).origin = trace.endpos;
                }

                if trace.fraction == 1.0 {
                    break; // moved the entire distance
                }

                // save entity for contact
                self.PM_AddTouchEnt(trace.entityNum as c_int);

                if (*ps).clientNum >= MAX_CLIENTS as c_int {
                    let pEnt = self.pm_entSelf;
                    if !pEnt.is_null()
                        && (*pEnt).s.eType == ET_NPC as c_int
                        && (*pEnt).s.NPC_class == CLASS_VEHICLE as c_int
                        && !((*pEnt).m_pVehicle as *mut Vehicle_t).is_null()
                    {
                        // do vehicle impact stuff then
                        self.PM_VehicleImpact(pEnt, &mut trace);
                    }
                } else if self.PM_ClientImpact(&mut trace) != 0 {
                    bumpcount += 1;
                    continue;
                }

                time_left -= time_left * trace.fraction;

                if numplanes >= MAX_CLIP_PLANES {
                    // this shouldn't really happen
                    (*ps).velocity = [0.0; 3];
                    return QTRUE;
                }

                let mut normal = trace.plane.normal;

                if self.PM_GroundSlideOkay(normal[2]) == 0 {
                    // wall-running: never push up off a sloped wall
                    normal[2] = 0.0;
                    VectorNormalize(&mut normal);
                }

                // if this is the same plane we hit before, nudge velocity out
                // along it, which fixes some epsilon issues with non-axial planes
                let mut same_plane_continue = false;
                if (*ps).pm_flags & PMF_STUCK_TO_WALL == 0 {
                    // no sliding if stuck to wall!
                    let mut i = 0;
                    while i < numplanes {
                        if normal == planes[i] {
                            for k in 0..3 {
                                (*ps).velocity[k] += normal[k];
                            }
                            break;
                        }
                        i += 1;
                    }
                    if i < numplanes {
                        same_plane_continue = true;
                    }
                }
                if same_plane_continue {
                    bumpcount += 1;
                    continue;
                }

                planes[numplanes] = normal;
                numplanes += 1;

                // modify velocity so it parallels all of the clip planes; find a
                // plane that it enters
                let mut i = 0;
                while i < numplanes {
                    let into = (*ps).velocity[0] * planes[i][0]
                        + (*ps).velocity[1] * planes[i][1]
                        + (*ps).velocity[2] * planes[i][2];
                    if into >= 0.1 {
                        i += 1;
                        continue; // move doesn't interact with the plane
                    }

                    // see how hard we are hitting things
                    if -into > self.pml.impactSpeed {
                        self.pml.impactSpeed = -into;
                    }

                    // slide along the plane
                    let mut clipVelocity: vec3_t = [0.0; 3];
                    self.PM_ClipVelocity((*ps).velocity, planes[i], &mut clipVelocity, OVERCLIP);

                    let mut endClipVelocity: vec3_t = [0.0; 3];
                    self.PM_ClipVelocity(endVelocity, planes[i], &mut endClipVelocity, OVERCLIP);

                    // see if there is a second plane that the new move enters
                    let mut j = 0;
                    let mut triple_stop = false;
                    while j < numplanes {
                        if j == i {
                            j += 1;
                            continue;
                        }
                        if clipVelocity[0] * planes[j][0]
                            + clipVelocity[1] * planes[j][1]
                            + clipVelocity[2] * planes[j][2]
                            >= 0.1
                        {
                            j += 1;
                            continue; // move doesn't interact with the plane
                        }

                        // try clipping the move to the plane
                        let cv = clipVelocity;
                        self.PM_ClipVelocity(cv, planes[j], &mut clipVelocity, OVERCLIP);
                        let ecv = endClipVelocity;
                        self.PM_ClipVelocity(ecv, planes[j], &mut endClipVelocity, OVERCLIP);

                        // see if it goes back into the first clip plane
                        if clipVelocity[0] * planes[i][0]
                            + clipVelocity[1] * planes[i][1]
                            + clipVelocity[2] * planes[i][2]
                            >= 0.0
                        {
                            j += 1;
                            continue;
                        }

                        // slide the original velocity along the crease
                        let mut dir: vec3_t = [0.0; 3];
                        CrossProduct(planes[i], planes[j], &mut dir);
                        VectorNormalize(&mut dir);
                        let mut d = dir[0] * (*ps).velocity[0]
                            + dir[1] * (*ps).velocity[1]
                            + dir[2] * (*ps).velocity[2];
                        for k in 0..3 {
                            clipVelocity[k] = dir[k] * d;
                        }

                        CrossProduct(planes[i], planes[j], &mut dir);
                        VectorNormalize(&mut dir);
                        d = dir[0] * endVelocity[0]
                            + dir[1] * endVelocity[1]
                            + dir[2] * endVelocity[2];
                        for k in 0..3 {
                            endClipVelocity[k] = dir[k] * d;
                        }

                        // see if there is a third plane the new move enters
                        let mut k_idx = 0;
                        while k_idx < numplanes {
                            if k_idx == i || k_idx == j {
                                k_idx += 1;
                                continue;
                            }
                            if clipVelocity[0] * planes[k_idx][0]
                                + clipVelocity[1] * planes[k_idx][1]
                                + clipVelocity[2] * planes[k_idx][2]
                                >= 0.1
                            {
                                k_idx += 1;
                                continue; // move doesn't interact with the plane
                            }

                            // stop dead at a triple plane interaction
                            (*ps).velocity = [0.0; 3];
                            triple_stop = true;
                            break;
                        }
                        if triple_stop {
                            break;
                        }
                        j += 1;
                    }
                    if triple_stop {
                        return QTRUE;
                    }

                    // if we have fixed all interactions, try another move
                    (*ps).velocity = clipVelocity;
                    endVelocity = endClipVelocity;
                    break;
                }

                bumpcount += 1;
            }

            if gravity != 0 {
                (*ps).velocity = endVelocity;
            }

            // don't change velocity if in a timer (FIXME: is this correct?)
            if (*ps).pm_time != 0 {
                (*ps).velocity = primal_velocity;
            }

            if bumpcount != 0 {
                QTRUE
            } else {
                QFALSE
            }
        }
    }

    /// Raven `PM_StepSlideMove`.
    ///
    /// Source: `oracle/oracle/codemp/game/bg_slidemove.c:861-1073`
    pub fn PM_StepSlideMove(&mut self, mut gravity: qboolean) {
        unsafe {
            let ps = (*self.pm).ps;
            let start_o = (*ps).origin;
            let start_v = (*ps).velocity;

            if BG_InReboundHold((*ps).legsAnim) != 0 {
                gravity = QFALSE;
            }

            if self.PM_SlideMove(gravity) == 0 {
                return; // we got exactly where we wanted to go first try
            }

            let pEnt = self.pm_entSelf;

            if (*ps).clientNum >= MAX_CLIENTS as c_int
                && !pEnt.is_null()
                && (*pEnt).s.NPC_class == CLASS_VEHICLE as c_int
                && !((*pEnt).m_pVehicle as *mut Vehicle_t).is_null()
                && (*(*((*pEnt).m_pVehicle as *mut Vehicle_t)).m_pVehicleInfo).hoverHeight > 0.0
            {
                return;
            }

            let mut down = start_o;
            down[2] -= STEPSIZE;
            let mut trace: trace_t = core::mem::zeroed();
            self.traps.trace(
                &mut trace,
                core::ptr::addr_of!(start_o) as *const vec3_t,
                core::ptr::addr_of!((*self.pm).mins) as *const vec3_t,
                core::ptr::addr_of!((*self.pm).maxs) as *const vec3_t,
                core::ptr::addr_of!(down) as *const vec3_t,
                (*ps).clientNum,
                (*self.pm).tracemask,
            );
            let up_test: vec3_t = [0.0, 0.0, 1.0];
            // never step up when you still have up velocity
            if (*ps).velocity[2] > 0.0
                && (trace.fraction == 1.0
                    || (trace.plane.normal[0] * up_test[0]
                        + trace.plane.normal[1] * up_test[1]
                        + trace.plane.normal[2] * up_test[2])
                        < 0.7)
            {
                return;
            }

            let down_o = (*ps).origin;
            let down_v = (*ps).velocity;

            let mut up = start_o;

            let mut isGiant: qboolean = QFALSE;
            if (*ps).clientNum >= MAX_CLIENTS as c_int {
                // apply ground friction, even if on ladder
                // Raven's `&&`-over-`||` precedence leaves the CLASS_VEHICLE clause
                // dereferencing pEnt with no null check; the added guard hardens that
                // deref (pEnt is never null here, so behavior is unchanged).
                if (!pEnt.is_null() && (*pEnt).s.NPC_class == CLASS_ATST as c_int)
                    || (!pEnt.is_null()
                        && (*pEnt).s.NPC_class == CLASS_VEHICLE as c_int
                        && !((*pEnt).m_pVehicle as *mut Vehicle_t).is_null()
                        && (*(*((*pEnt).m_pVehicle as *mut Vehicle_t)).m_pVehicleInfo).r#type
                            as c_int
                            == VH_WALKER as c_int)
                {
                    // AT-STs can step high
                    up[2] += 66.0;
                    isGiant = QTRUE;
                } else if !pEnt.is_null() && (*pEnt).s.NPC_class == CLASS_RANCOR as c_int {
                    // also can step up high
                    up[2] += 64.0;
                    isGiant = QTRUE;
                } else {
                    up[2] += STEPSIZE;
                }
            } else {
                up[2] += STEPSIZE;
            }

            // test the player position if they were a stepheight higher
            self.traps.trace(
                &mut trace,
                core::ptr::addr_of!(start_o) as *const vec3_t,
                core::ptr::addr_of!((*self.pm).mins) as *const vec3_t,
                core::ptr::addr_of!((*self.pm).maxs) as *const vec3_t,
                core::ptr::addr_of!(up) as *const vec3_t,
                (*ps).clientNum,
                (*self.pm).tracemask,
            );
            if trace.allsolid != 0 {
                if (*self.pm).debugLevel != 0 {
                    // Com_Printf("%i:bend can't step\n", c_pmove);
                    let msg = format!("{}:bend can't step\n", self.bg.c_pmove);
                    Com_Printf(msg.as_ptr() as *const c_char);
                }
                return; // can't step up
            }

            let stepSize = trace.endpos[2] - start_o[2];
            // try slidemove from this position
            (*ps).origin = trace.endpos;
            (*ps).velocity = start_v;

            self.PM_SlideMove(gravity);

            // push down the final amount
            down = (*ps).origin;
            down[2] -= stepSize;
            self.traps.trace(
                &mut trace,
                core::ptr::addr_of!((*ps).origin) as *const vec3_t,
                core::ptr::addr_of!((*self.pm).mins) as *const vec3_t,
                core::ptr::addr_of!((*self.pm).maxs) as *const vec3_t,
                core::ptr::addr_of!(down) as *const vec3_t,
                (*ps).clientNum,
                (*self.pm).tracemask,
            );

            let mut skipStep: qboolean = QFALSE;
            if (*self.pm).stepSlideFix != 0
                && (*ps).clientNum < MAX_CLIENTS as c_int
                && trace.plane.normal[2] < MIN_WALK_NORMAL
            {
                // normal players cannot step up slopes that are too steep to walk on!
                // BUT: figure the slope of the whole move (A)->(B); if that's
                // walk-upable, it's still okay.
                let mut stepVec: vec3_t = [0.0; 3];
                for i in 0..3 {
                    stepVec[i] = trace.endpos[i] - down_o[i];
                }
                VectorNormalize(&mut stepVec);
                if stepVec[2] > (1.0 - MIN_WALK_NORMAL) {
                    skipStep = QTRUE;
                }
            }

            if trace.allsolid == 0 && skipStep == 0 {
                if (*ps).clientNum >= MAX_CLIENTS as c_int
                    && isGiant != 0
                    && (trace.entityNum as c_int) < MAX_CLIENTS as c_int
                    && !pEnt.is_null()
                    && (*pEnt).s.NPC_class == CLASS_RANCOR as c_int
                {
                    // Rancor don't step on clients
                    if (*self.pm).stepSlideFix != 0 {
                        (*ps).origin = down_o;
                        (*ps).velocity = down_v;
                    } else {
                        (*ps).origin = start_o;
                        (*ps).velocity = start_v;
                    }
                } else {
                    (*ps).origin = trace.endpos;
                    if (*self.pm).stepSlideFix != 0 && trace.fraction < 1.0 {
                        let normal = trace.plane.normal;
                        let mut out = (*ps).velocity;
                        self.PM_ClipVelocity((*ps).velocity, normal, &mut out, OVERCLIP);
                        (*ps).velocity = out;
                    }
                }
            } else if (*self.pm).stepSlideFix != 0 {
                (*ps).origin = down_o;
                (*ps).velocity = down_v;
            }

            if (*self.pm).stepSlideFix == 0 && trace.fraction < 1.0 {
                let normal = trace.plane.normal;
                let mut out = (*ps).velocity;
                self.PM_ClipVelocity((*ps).velocity, normal, &mut out, OVERCLIP);
                (*ps).velocity = out;
            }

            // use the step move
            let delta = (*ps).origin[2] - start_o[2];
            if delta > 2.0 {
                let ent = if delta < 7.0 {
                    EV_STEP_4 as c_int
                } else if delta < 11.0 {
                    EV_STEP_8 as c_int
                } else if delta < 15.0 {
                    EV_STEP_12 as c_int
                } else {
                    EV_STEP_16 as c_int
                };
                self.PM_AddEvent(ent);
            }
            if (*self.pm).debugLevel != 0 {
                // Com_Printf("%i:stepped\n", c_pmove);
                let msg = format!("{}:stepped\n", self.bg.c_pmove);
                Com_Printf(msg.as_ptr() as *const c_char);
            }
        }
    }
}
