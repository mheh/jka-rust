//! Port of `oracle/codemp/cgame/cg_turret.c` — turret entity rendering and aim tracking. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use mp_qshared::shared::q_math::{PITCH, YAW};
use mp_qshared::shared::vec3_t;

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
