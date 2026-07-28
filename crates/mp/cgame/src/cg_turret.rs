//! Port of `oracle/codemp/cgame/cg_turret.c` — turret entity rendering and aim tracking. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};

use mp_bg::bg_misc::BG_GiveMeVectorFromMatrix;
use mp_bg::public::configstring::CS_MODELS;
use mp_bg::weapons::weapon_t::WP_TURRET;
use mp_qshared::common::mp::ghoul2::bone_flags::{
    BONE_ANGLES_POSTMULT, BONE_ANGLES_REPLACE, BONE_ANIM_OVERRIDE_FREEZE,
};
use mp_qshared::shared::q_math::{
    _VectorSubtract, vec3_origin, vectoangles, VectorNormalize, PITCH, ROLL, YAW,
};
use mp_qshared::shared::{mdxaBone_t, qfalse, vec3_t, Eorientations, ENTITYNUM_NONE};

use crate::cg_main::CG_ConfigString;
use crate::cg_weaponinit::CG_RegisterWeapon;
use crate::trap;
use crate::world::cg_context::CgContext;

/// Raven `CreepToPosition` — steps `current`'s YAW and PITCH 90 degrees at a time toward `ideal`, picking whichever
/// rotation direction (negative/positive) is the shorter arc, and snaps to `ideal` once within the 180-degree step.
///
/// Source: `oracle/codemp/cgame/cg_turret.c:7-123`
pub fn CreepToPosition(ideal: &mut vec3_t, current: &mut vec3_t) {
    let max_degree_switch: f32 = 90.0;
    let mut degrees_negative;
    let mut degrees_positive;
    let mut doNegative;

    let mut angle_ideal = ideal[YAW] as i32;
    let mut angle_current = current[YAW] as i32;

    if angle_ideal <= angle_current {
        degrees_negative = angle_current - angle_ideal;
        degrees_positive = (360 - angle_current) + angle_ideal;
    } else {
        degrees_negative = angle_current + (360 - angle_ideal);
        degrees_positive = angle_ideal - angle_current;
    }

    doNegative = degrees_negative < degrees_positive;

    if doNegative {
        current[YAW] -= max_degree_switch;

        if current[YAW] < ideal[YAW] && (current[YAW] + (max_degree_switch * 2.0)) >= ideal[YAW] {
            current[YAW] = ideal[YAW];
        }

        if current[YAW] < 0.0 {
            current[YAW] += 361.0;
        }
    } else {
        current[YAW] += max_degree_switch;

        if current[YAW] > ideal[YAW] && (current[YAW] - (max_degree_switch * 2.0)) <= ideal[YAW] {
            current[YAW] = ideal[YAW];
        }

        if current[YAW] > 360.0 {
            current[YAW] -= 361.0;
        }
    }

    if ideal[PITCH] < 0.0 {
        ideal[PITCH] += 360.0;
    }

    angle_ideal = ideal[PITCH] as i32;
    angle_current = current[PITCH] as i32;

    if angle_ideal <= angle_current {
        degrees_negative = angle_current - angle_ideal;
        degrees_positive = (360 - angle_current) + angle_ideal;
    } else {
        degrees_negative = angle_current + (360 - angle_ideal);
        degrees_positive = angle_ideal - angle_current;
    }

    doNegative = degrees_negative < degrees_positive;

    if doNegative {
        current[PITCH] -= max_degree_switch;

        if current[PITCH] < ideal[PITCH]
            && (current[PITCH] + (max_degree_switch * 2.0)) >= ideal[PITCH]
        {
            current[PITCH] = ideal[PITCH];
        }

        if current[PITCH] < 0.0 {
            current[PITCH] += 361.0;
        }
    } else {
        current[PITCH] += max_degree_switch;

        if current[PITCH] > ideal[PITCH]
            && (current[PITCH] - (max_degree_switch * 2.0)) <= ideal[PITCH]
        {
            current[PITCH] = ideal[PITCH];
        }

        if current[PITCH] > 360.0 {
            current[PITCH] -= 361.0;
        }
    }
}

/// Raven `TurretClientRun` — client-side turret think. Lazily inits the
/// ghoul2 instance and its three tracking bolts the first time it sees this
/// entity, plays the muzzle flash effect on a fresh fire flag, then either
/// turns the turret to track `bolt2`'s entity or idles it in a slow spin
/// before re-applying the hinge bone angles once per frame.
///
/// Source: `oracle/codemp/cgame/cg_turret.c:125-242`
pub fn TurretClientRun(ctx: &mut CgContext, centNum: usize) {
    if ctx.world.entity(centNum).ghoul2.is_null() {
        let modelindex = ctx.world.entity(centNum).currentState.modelindex;
        let modelName = CG_ConfigString(ctx, CS_MODELS + modelindex);

        let mut ghoul2 = ctx.world.entity(centNum).ghoul2;
        trap::G2API_InitGhoul2Model(
            ctx.engine,
            &mut ghoul2 as *mut *mut c_void,
            &modelName,
            0,
            0,
            0,
            0,
            0,
        );
        ctx.world.entity_mut(centNum).ghoul2 = ghoul2;

        if ctx.world.entity(centNum).ghoul2.is_null() {
            // bad
            return;
        }

        let ghoul2 = ctx.world.entity(centNum).ghoul2;
        let torsoBolt = trap::G2API_AddBolt(ctx.engine, ghoul2, 0, "*flash02");
        ctx.world.entity_mut(centNum).torsoBolt = torsoBolt;

        let time = ctx.world.cg.time;
        let up = Eorientations::POSITIVE_Y as c_int;
        let right = Eorientations::POSITIVE_Z as c_int;
        let forward = Eorientations::POSITIVE_X as c_int;
        trap::G2API_SetBoneAngles(
            ctx.engine,
            ghoul2,
            0,
            "bone_hinge",
            &vec3_origin,
            BONE_ANGLES_POSTMULT,
            up,
            right,
            forward,
            None,
            100,
            time,
        );
        trap::G2API_SetBoneAngles(
            ctx.engine,
            ghoul2,
            0,
            "bone_gback",
            &vec3_origin,
            BONE_ANGLES_POSTMULT,
            up,
            right,
            forward,
            None,
            100,
            time,
        );
        trap::G2API_SetBoneAngles(
            ctx.engine,
            ghoul2,
            0,
            "bone_barrel",
            &vec3_origin,
            BONE_ANGLES_POSTMULT,
            up,
            right,
            forward,
            None,
            100,
            time,
        );

        trap::G2API_SetBoneAnim(
            ctx.engine,
            ghoul2,
            0,
            "model_root",
            0,
            11,
            BONE_ANIM_OVERRIDE_FREEZE,
            0.8,
            time,
            0.0,
            0,
        );

        let ent = ctx.world.entity_mut(centNum);
        ent.turAngles[ROLL] = 0.0;
        ent.turAngles[PITCH] = 90.0;
        ent.turAngles[YAW] = 0.0;

        if ctx.world.cg_weapons[WP_TURRET as usize].registered == qfalse {
            CG_RegisterWeapon(ctx, WP_TURRET);
        }
    }

    if ctx.world.entity(centNum).currentState.fireflag == 2 {
        // I'm about to blow
        // Raven's `if (ent->turAngles)` tests a fixed-size array's decayed
        // pointer, which C never treats as null - always true. Preserved as
        // an unconditional block.
        let ghoul2 = ctx.world.entity(centNum).ghoul2;
        let turAngles = ctx.world.entity(centNum).turAngles;
        let time = ctx.world.cg.time;
        trap::G2API_SetBoneAngles(
            ctx.engine,
            ghoul2,
            0,
            "bone_hinge",
            &turAngles,
            BONE_ANGLES_REPLACE,
            Eorientations::NEGATIVE_Y as c_int,
            Eorientations::NEGATIVE_Z as c_int,
            Eorientations::NEGATIVE_X as c_int,
            None,
            100,
            time,
        );
        return;
    } else if ctx.world.entity(centNum).currentState.fireflag != 0
        && ctx.world.entity(centNum).bolt4 != ctx.world.entity(centNum).currentState.fireflag
    {
        let mut boltMatrix = mdxaBone_t {
            matrix: [[0.0; 4]; 3],
        };
        let mut muzzleOrg: vec3_t = [0.0; 3];
        let mut muzzleDir: vec3_t = [0.0; 3];

        let ghoul2 = ctx.world.entity(centNum).ghoul2;
        let torsoBolt = ctx.world.entity(centNum).torsoBolt;
        let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
        let time = ctx.world.cg.time;
        let modelScale = ctx.world.entity(centNum).modelScale;

        trap::G2API_GetBoltMatrix(
            ctx.engine,
            ghoul2,
            0,
            torsoBolt,
            &mut boltMatrix,
            &vec3_origin,
            &lerpOrigin,
            time,
            Some(&mut ctx.world.cgs.gameModels[0]),
            &modelScale,
        );
        BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, &mut muzzleOrg);
        BG_GiveMeVectorFromMatrix(
            &boltMatrix,
            Eorientations::NEGATIVE_X as c_int,
            &mut muzzleDir,
        );

        let mTurretMuzzleFlash = ctx.world.cgs.effects.mTurretMuzzleFlash;
        trap::FX_PlayEffectID(
            ctx.engine,
            mTurretMuzzleFlash,
            &muzzleOrg,
            &muzzleDir,
            -1,
            -1,
        );

        let fireflag = ctx.world.entity(centNum).currentState.fireflag;
        ctx.world.entity_mut(centNum).bolt4 = fireflag;
    } else if ctx.world.entity(centNum).currentState.fireflag == 0 {
        ctx.world.entity_mut(centNum).bolt4 = 0;
    }

    if ctx.world.entity(centNum).currentState.bolt2 != ENTITYNUM_NONE {
        // turn toward the enemy
        // Raven's `if (enemy)` tests `&cg_entities[idx]`'s address, never
        // null in C - always true. Preserved as an unconditional block.
        let bolt2 = ctx.world.entity(centNum).currentState.bolt2 as usize;
        let enPos = ctx.world.entity(bolt2).currentState.pos.trBase;
        let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;

        let mut enAng: vec3_t = [0.0; 3];
        _VectorSubtract(enPos, lerpOrigin, &mut enAng);
        VectorNormalize(&mut enAng);

        let mut enAngles: vec3_t = [0.0; 3];
        vectoangles(enAng, &mut enAngles);
        enAngles[ROLL] = 0.0;
        enAngles[PITCH] += 90.0;

        let ent = ctx.world.entity_mut(centNum);
        CreepToPosition(&mut enAngles, &mut ent.turAngles);
    } else {
        let time = ctx.world.cg.time;
        let ent = ctx.world.entity_mut(centNum);

        if ent.turAngles[YAW] > 360.0 {
            ent.turAngles[YAW] -= 361.0;
        }

        if ent.dustTrailTime == 0 {
            ent.dustTrailTime = time;
        }

        let mut turnAmount = (time - ent.dustTrailTime) as f32 * 0.03;

        if turnAmount > 360.0 {
            turnAmount = 360.0;
        }

        let mut idleAng: vec3_t = [0.0; 3];
        idleAng[PITCH] = 90.0;
        idleAng[ROLL] = 0.0;
        idleAng[YAW] = ent.turAngles[YAW] + turnAmount;
        ent.dustTrailTime = time;

        CreepToPosition(&mut idleAng, &mut ent.turAngles);
    }

    let time = ctx.world.cg.time;
    if time < ctx.world.entity(centNum).frame_minus1_refreshed {
        ctx.world.entity_mut(centNum).frame_minus1_refreshed = time;
        return;
    }

    ctx.world.entity_mut(centNum).frame_minus1_refreshed = time;
    let ghoul2 = ctx.world.entity(centNum).ghoul2;
    let turAngles = ctx.world.entity(centNum).turAngles;
    trap::G2API_SetBoneAngles(
        ctx.engine,
        ghoul2,
        0,
        "bone_hinge",
        &turAngles,
        BONE_ANGLES_REPLACE,
        Eorientations::NEGATIVE_Y as c_int,
        Eorientations::NEGATIVE_Z as c_int,
        Eorientations::NEGATIVE_X as c_int,
        None,
        100,
        time,
    );
}
