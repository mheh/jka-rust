// PORT-COMPLETE: bg_g2_utils.c 2/2
//! Raven `bg_g2_utils.c` functions ported to Rust.
//!
//! Source: `oracle/oracle/codemp/game/bg_g2_utils.c:25-122`
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

/// Raven `BG_AttachToRancor`.
///
/// Source: `oracle/oracle/codemp/game/bg_g2_utils.c:25-98`
pub fn BG_AttachToRancor(
    ghoul2: *mut c_void,
    rancYaw: f32,
    rancOrigin: vec3_t,
    time: c_int,
    modelList: *mut qhandle_t,
    modelScale: vec3_t,
    inMouth: qboolean,
    out_origin: &mut vec3_t,
    out_angles: &mut vec3_t,
    out_axis: *mut vec3_t,
    bg: &BgState,
    traps: &dyn BgTraps,
) {
    let mut boltMatrix: mdxaBone_t = unsafe { core::mem::zeroed() };
    let boltIndex: c_int;
    let mut rancAngles: vec3_t = [0.0, rancYaw, 0.0];
    let mut temp_angles: vec3_t = [0.0, 0.0, 0.0];

    // Getting the bolt here
    if inMouth != 0 {
        // in mouth
        let bolt_name = cstr("jaw_bone");
        boltIndex = traps.g2api_add_bolt(ghoul2, 0, bolt_name.as_ptr());
    } else {
        // in right hand
        let bolt_name = cstr("*r_hand");
        boltIndex = traps.g2api_add_bolt(ghoul2, 0, bolt_name.as_ptr());
    }

    traps.g2api_get_bolt_matrix(
        ghoul2,
        0,
        boltIndex,
        &mut boltMatrix,
        &rancAngles,
        &rancOrigin,
        time,
        modelList,
        &modelScale,
    );

    // Storing ent position, bolt position, and bolt axis.
    // out_origin/out_angles are C `vec3_t` out-params (nullable in C, but the sole
    // live caller G_HeldByMonster always passes non-NULL), ported as `&mut vec3_t`.
    {
        BG_GiveMeVectorFromMatrix(
            &boltMatrix as *const mdxaBone_t,
            Eorientations::ORIGIN as c_int,
            out_origin,
        );
    }

    if !out_axis.is_null() {
        if inMouth != 0 {
            // in mouth
            BG_GiveMeVectorFromMatrix(
                &boltMatrix as *const mdxaBone_t,
                Eorientations::POSITIVE_Z as c_int,
                unsafe { &mut *out_axis.add(0) },
            );
            BG_GiveMeVectorFromMatrix(
                &boltMatrix as *const mdxaBone_t,
                Eorientations::NEGATIVE_Y as c_int,
                unsafe { &mut *out_axis.add(1) },
            );
            BG_GiveMeVectorFromMatrix(
                &boltMatrix as *const mdxaBone_t,
                Eorientations::NEGATIVE_X as c_int,
                unsafe { &mut *out_axis.add(2) },
            );
        } else {
            // in hand
            BG_GiveMeVectorFromMatrix(
                &boltMatrix as *const mdxaBone_t,
                Eorientations::NEGATIVE_Y as c_int,
                unsafe { &mut *out_axis.add(0) },
            );
            BG_GiveMeVectorFromMatrix(
                &boltMatrix as *const mdxaBone_t,
                Eorientations::POSITIVE_X as c_int,
                unsafe { &mut *out_axis.add(1) },
            );
            BG_GiveMeVectorFromMatrix(
                &boltMatrix as *const mdxaBone_t,
                Eorientations::POSITIVE_Z as c_int,
                unsafe { &mut *out_axis.add(2) },
            );
        }

        // FIXME: this is messing up our axis and turning us inside-out?
        {
            vectoangles(unsafe { *out_axis.add(0) }, out_angles);
            vectoangles(unsafe { *out_axis.add(2) }, &mut temp_angles);
            out_angles[2] = -temp_angles[0]; // ROLL = -PITCH
        }
    } else {
        let mut temp_axis: [vec3_t; 3] = [[0.0, 0.0, 0.0]; 3];
        if inMouth != 0 {
            // in mouth
            BG_GiveMeVectorFromMatrix(
                &boltMatrix as *const mdxaBone_t,
                Eorientations::POSITIVE_Z as c_int,
                &mut temp_axis[0],
            );
            BG_GiveMeVectorFromMatrix(
                &boltMatrix as *const mdxaBone_t,
                Eorientations::NEGATIVE_X as c_int,
                &mut temp_axis[2],
            );
        } else {
            // in hand
            BG_GiveMeVectorFromMatrix(
                &boltMatrix as *const mdxaBone_t,
                Eorientations::NEGATIVE_Y as c_int,
                &mut temp_axis[0],
            );
            BG_GiveMeVectorFromMatrix(
                &boltMatrix as *const mdxaBone_t,
                Eorientations::POSITIVE_Z as c_int,
                &mut temp_axis[2],
            );
        }

        // FIXME: this is messing up our axis and turning us inside-out?
        vectoangles(temp_axis[0], out_angles);
        vectoangles(temp_axis[2], &mut temp_angles);
        out_angles[2] = -temp_angles[0]; // ROLL = -PITCH
    }
}

/// Raven `BG_GetRootSurfNameWithVariant`.
///
/// Source: `oracle/oracle/codemp/game/bg_g2_utils.c:101-122`
pub fn BG_GetRootSurfNameWithVariant(
    ghoul2: *mut c_void,
    rootSurfName: *const c_char,
    returnSurfName: *mut c_char,
    returnSize: c_int,
    bg: &BgState,
    traps: &dyn BgTraps,
) -> qboolean {
    // Raven file-local `#define MAX_VARIANTS 8`.
    // Source: `oracle/oracle/codemp/game/bg_g2_utils.c:100`
    const MAX_VARIANTS: c_int = 8;

    if ghoul2.is_null() || traps.g2api_get_surface_render_status(ghoul2, 0, rootSurfName) == qfalse
    {
        // see if the basic name without variants is on
        unsafe {
            Q_strncpyz(returnSurfName, rootSurfName, returnSize);
        }
        return qtrue;
    } else {
        // check variants
        for i in 0..MAX_VARIANTS {
            let variant_name = unsafe {
                format!(
                    "{}{}",
                    cstr_to_str(rootSurfName),
                    ((b'a' + i as u8) as char)
                )
            };
            let variant_cstr = cstr(&variant_name);
            unsafe {
                Q_strncpyz(returnSurfName, variant_cstr.as_ptr(), returnSize);
            }
            if traps.g2api_get_surface_render_status(ghoul2, 0, returnSurfName) == qfalse {
                return qtrue;
            }
        }
    }

    // Fall back to root surface name
    unsafe {
        Q_strncpyz(returnSurfName, rootSurfName, returnSize);
    }
    qfalse
}
