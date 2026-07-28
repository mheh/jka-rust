//! Port of `oracle/codemp/cgame/cg_effects.c` — client-side effect spawners (puffs, debris, glass, chunks). Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_qshared::common::mp::gentity::{
    material_t, MAT_CRATE1, MAT_CRATE2, MAT_DRK_STONE, MAT_ELECTRICAL, MAT_ELEC_METAL, MAT_GLASS,
    MAT_GLASS_METAL, MAT_GRATE1, MAT_GREY_STONE, MAT_LT_STONE, MAT_METAL, MAT_METAL2, MAT_METAL3,
    MAT_ROPE, MAT_SNOWY_ROCK, MAT_WHITE_METAL,
};
use mp_qshared::common::mp::qcommon::saber::saber_colors::{
    SABER_BLUE, SABER_GREEN, SABER_ORANGE, SABER_PURPLE, SABER_RED, SABER_YELLOW,
};
use mp_qshared::shared::q_math::{
    _VectorAdd, _VectorMA, _VectorScale, _VectorSubtract, CrossProduct, VectorNormalize, VectorSet,
};
use mp_qshared::shared::{addpolyArgStruct_t, vec2_t, vec3_t};

use crate::trap;
use crate::world::cg_context::CgContext;
use crate::world::cg_world::CgWorld;

// FILE-SCOPE CONSTANTS
// Source: `oracle/codemp/cgame/cg_effects.c:257-259,452-454,756-764,1384-1386`

/// Raven `FX_ALPHA_NONLINEAR` — `addpolyArgStruct_t.flags` bit: fade the alpha ramp non-linearly.
/// Source: `oracle/codemp/cgame/cg_effects.c:257`
pub const FX_ALPHA_NONLINEAR: c_int = 0x00000004;

/// Raven `FX_APPLY_PHYSICS` — `addpolyArgStruct_t.flags` bit: integrate `vel`/`accel`/bounce each frame.
/// Source: `oracle/codemp/cgame/cg_effects.c:258`
pub const FX_APPLY_PHYSICS: c_int = 0x02000000;

/// Raven `FX_USE_ALPHA` — `addpolyArgStruct_t.flags` bit: honor `alpha1`/`alpha2`/`alphaParm`.
/// Source: `oracle/codemp/cgame/cg_effects.c:259`
pub const FX_USE_ALPHA: c_int = 0x08000000;

/// Raven `TIME_DECAY_SLOW`.
/// Source: `oracle/codemp/cgame/cg_effects.c:452`
pub const TIME_DECAY_SLOW: f32 = 0.1;

/// Raven `TIME_DECAY_MED`.
/// Source: `oracle/codemp/cgame/cg_effects.c:453`
pub const TIME_DECAY_MED: f32 = 0.04;

/// Raven `TIME_DECAY_FAST`.
/// Source: `oracle/codemp/cgame/cg_effects.c:454`
pub const TIME_DECAY_FAST: f32 = 0.009;

/// Raven `DEBRIS_SPECIALCASE_ROCK` — negative sentinel debris-model index.
/// Source: `oracle/codemp/cgame/cg_effects.c:756`
pub const DEBRIS_SPECIALCASE_ROCK: c_int = -1;

/// Raven `DEBRIS_SPECIALCASE_CHUNKS` — negative sentinel debris-model index.
/// Source: `oracle/codemp/cgame/cg_effects.c:757`
pub const DEBRIS_SPECIALCASE_CHUNKS: c_int = -2;

/// Raven `DEBRIS_SPECIALCASE_WOOD` — negative sentinel debris-model index.
/// Source: `oracle/codemp/cgame/cg_effects.c:758`
pub const DEBRIS_SPECIALCASE_WOOD: c_int = -3;

/// Raven `DEBRIS_SPECIALCASE_GLASS` — negative sentinel debris-model index.
/// Source: `oracle/codemp/cgame/cg_effects.c:759`
pub const DEBRIS_SPECIALCASE_GLASS: c_int = -4;

/// Raven `NUM_DEBRIS_MODELS_GLASS`.
/// Source: `oracle/codemp/cgame/cg_effects.c:761`
pub const NUM_DEBRIS_MODELS_GLASS: usize = 8;

/// Raven `NUM_DEBRIS_MODELS_WOOD`.
/// Source: `oracle/codemp/cgame/cg_effects.c:762`
pub const NUM_DEBRIS_MODELS_WOOD: usize = 8;

/// Raven `NUM_DEBRIS_MODELS_CHUNKS`.
/// Source: `oracle/codemp/cgame/cg_effects.c:763`
pub const NUM_DEBRIS_MODELS_CHUNKS: usize = 3;

/// Raven `NUM_DEBRIS_MODELS_ROCKS` (Raven comment: `//12` — the table shipped
/// with 4 entries, not the originally planned 12).
/// Source: `oracle/codemp/cgame/cg_effects.c:764`
pub const NUM_DEBRIS_MODELS_ROCKS: usize = 4;

/// Raven `NUM_SPARKS`.
/// Source: `oracle/codemp/cgame/cg_effects.c:1384`
pub const NUM_SPARKS: usize = 12;

/// Raven `NUM_PUFFS`.
/// Source: `oracle/codemp/cgame/cg_effects.c:1385`
pub const NUM_PUFFS: usize = 1;

/// Raven `NUM_EXPLOSIONS`.
/// Source: `oracle/codemp/cgame/cg_effects.c:1386`
pub const NUM_EXPLOSIONS: usize = 4;

/// Raven `CGDEBUG_SaberColor` — debug-draw color for a saber blade color enum,
/// packed `0x00bbggrr`.
///
/// Source: `oracle/codemp/cgame/cg_effects.c:135-161`
pub fn CGDEBUG_SaberColor(saberColor: c_int) -> c_int {
    match saberColor {
        SABER_RED => 0x000000ff,
        SABER_ORANGE => 0x000088ff,
        SABER_YELLOW => 0x0000ffff,
        SABER_GREEN => 0x0000ff00,
        SABER_BLUE => 0x00ff0000,
        SABER_PURPLE => 0x00ff00ff,
        _ => saberColor,
    }
}

/// Raven `CG_DoGlassQuad` — spawns one glass-shard poly via `trap_FX_AddPoly`,
/// tumbling under gravity with a bit of bounce.
///
/// Raven: "rww - this is dirty." — the ideal `FX_AddPoly`/`CPoly` path above
/// this function is dead/commented-out in the oracle; this transcribes only
/// the live `addpolyArgStruct_t` path Raven actually ships.
///
/// Source: `oracle/codemp/cgame/cg_effects.c:261-365`
pub fn CG_DoGlassQuad(
    ctx: &mut CgContext,
    p: &[vec3_t; 4],
    uv: &[vec2_t; 4],
    stick: bool,
    time: c_int,
    dmgDir: &vec3_t,
) {
    let mut vel = [0.0f32; 3];
    VectorSet(
        &mut vel,
        (ctx.world.bg_state.rng.crandom() * 12.0) as f32,
        (ctx.world.bg_state.rng.crandom() * 12.0) as f32,
        -1.0,
    );

    if !stick {
        // We aren't a motion delayed chunk, so let us move quickly
        _VectorMA(vel, 0.3, *dmgDir, &mut vel);
    }

    // Set up acceleration due to gravity, 800 is standard QuakeIII gravity, so let's use something close
    let mut accel = [0.0f32; 3];
    VectorSet(
        &mut accel,
        0.0,
        0.0,
        -(600.0 + ctx.world.bg_state.rng.random() * 100.0),
    );

    // We are using an additive shader, so let's set the RGB low so we look more like transparent glass
    let rgb1: vec3_t = [1.0, 1.0, 1.0];

    // Being glass, we don't want to bounce much
    let bounce = ctx.world.bg_state.rng.random() * 0.2 + 0.15;

    // Set up our random rotate, we only do PITCH and YAW, not ROLL. This is something like degrees per second
    let mut rotationDelta = [0.0f32; 3];
    VectorSet(
        &mut rotationDelta,
        (ctx.world.bg_state.rng.crandom() * 40.0) as f32,
        (ctx.world.bg_state.rng.crandom() * 40.0) as f32,
        0.0,
    );

    let mut apArgs = addpolyArgStruct_t {
        p: *p,
        ev: *uv,
        numVerts: 4,
        vel,
        accel,
        alpha1: 0.15,
        alpha2: 0.0,
        alphaParm: 85.0,
        rgb1,
        rgb2: rgb1,
        rgbParm: 0.0,
        rotationDelta,
        bounce,
        motionDelay: time,
        killTime: 6000,
        shader: ctx.world.cgs.media.glassShardShader,
        flags: FX_APPLY_PHYSICS | FX_ALPHA_NONLINEAR | FX_USE_ALPHA,
    };

    trap::FX_AddPoly(ctx.engine, &mut apArgs);
}

/// Raven `CG_CalcBiLerp` — bilinearly interpolates the four corner `verts` at
/// each of `uv`'s four sample points into `subVerts` (Raven's out-param
/// becomes the return per §C7).
///
/// Source: `oracle/codemp/cgame/cg_effects.c:367-399`
pub fn CG_CalcBiLerp(verts: &[vec3_t; 4], uv: &[vec2_t; 4]) -> [vec3_t; 4] {
    let mut subVerts = [[0.0f32; 3]; 4];
    let mut temp = [0.0f32; 3];

    // Nasty crap
    for i in 0..4 {
        _VectorScale(verts[0], 1.0 - uv[i][0], &mut subVerts[i]);
        _VectorMA(subVerts[i], uv[i][0], verts[1], &mut subVerts[i]);
        _VectorScale(subVerts[i], 1.0 - uv[i][1], &mut temp);
        _VectorScale(verts[3], 1.0 - uv[i][0], &mut subVerts[i]);
        _VectorMA(subVerts[i], uv[i][0], verts[2], &mut subVerts[i]);
        _VectorMA(temp, uv[i][1], subVerts[i], &mut subVerts[i]);
    }

    subVerts
}

/// Raven `CG_CalcHeightWidth` — the average cross-product-derived height and
/// width of the `verts` quad (Raven's two out-params become the `(height,
/// width)` return per §C7, matching the declared param order).
///
/// PORT-NOTE: Raven normalizes `dir1` in place on the first cross product of
/// each pair, then reuses the now-unit-length `dir1` for the second cross
/// product — not a fresh direction vector. Preserved verbatim; it's what the
/// oracle actually computes.
///
/// Source: `oracle/codemp/cgame/cg_effects.c:404-425`
pub fn CG_CalcHeightWidth(verts: &[vec3_t; 4]) -> (f32, f32) {
    let mut dir1 = [0.0f32; 3];
    let mut dir2 = [0.0f32; 3];
    let mut cross = [0.0f32; 3];

    _VectorSubtract(verts[3], verts[0], &mut dir1); // v
    _VectorSubtract(verts[1], verts[0], &mut dir2); // p-a
    CrossProduct(dir1, dir2, &mut cross);
    let mut width = VectorNormalize(&mut cross) / VectorNormalize(&mut dir1); // v
    _VectorSubtract(verts[2], verts[0], &mut dir2); // p-a
    CrossProduct(dir1, dir2, &mut cross);
    width += VectorNormalize(&mut cross) / VectorNormalize(&mut dir1); // v
    width *= 0.5;

    _VectorSubtract(verts[1], verts[0], &mut dir1); // v
    _VectorSubtract(verts[2], verts[0], &mut dir2); // p-a
    CrossProduct(dir1, dir2, &mut cross);
    let mut height = VectorNormalize(&mut cross) / VectorNormalize(&mut dir1); // v
    _VectorSubtract(verts[3], verts[0], &mut dir2); // p-a
    CrossProduct(dir1, dir2, &mut cross);
    height += VectorNormalize(&mut cross) / VectorNormalize(&mut dir1); // v
    height *= 0.5;

    (height, width)
}

/// Raven `CG_InitGlass` — builds the 20x20 random crack-offset table once up
/// front, so the glass tesselation looks less predictable without paying the
/// random cost per-shatter.
///
/// Source: `oracle/codemp/cgame/cg_effects.c:430-444`
pub fn CG_InitGlass(world: &mut CgWorld) {
    // Build a table first, so that we can do a more unpredictable crack scheme
    // do it once, up front to save a bit of time.
    for i in 0..20usize {
        for t in 0..20usize {
            world.effects.offX[t][i] = (world.bg_state.rng.crandom() * 0.03) as f32;
            world.effects.offZ[i][t] = (world.bg_state.rng.crandom() * 0.03) as f32;
        }
    }
}

/// Raven `Vector2Set` — writes `(b, c)` into `a`. Kept as an out-param
/// receiver (not a return) to match `VectorSet`'s established 3D twin in this
/// tree.
///
/// Source: `oracle/codemp/cgame/cg_effects.c:446-450`
pub fn Vector2Set(a: &mut vec2_t, b: f32, c: f32) {
    a[0] = b;
    a[1] = c;
}

/// Raven `CG_MiscModelExplosion` — spawns a `material_t`-flavored debris burst
/// of registered particle effects roughly inside `[mins, maxs]`, shot outward
/// from the box center.
///
/// Source: `oracle/codemp/cgame/cg_effects.c:949-1051`
pub fn CG_MiscModelExplosion(
    ctx: &mut CgContext,
    mins: vec3_t,
    maxs: vec3_t,
    size: c_int,
    chunkType: material_t,
) {
    let mut ct: c_int = 13;
    let mut effect: Option<&str> = None;
    let mut effect2: Option<&str> = None;

    let mut mid = [0.0f32; 3];
    _VectorAdd(mins, maxs, &mut mid);
    _VectorScale(mid, 0.5, &mut mid);

    match chunkType {
        MAT_GLASS => {
            effect = Some("chunks/glassbreak");
            ct = 5;
        }
        MAT_GLASS_METAL => {
            effect = Some("chunks/glassbreak");
            effect2 = Some("chunks/metalexplode");
            ct = 5;
        }
        MAT_ELECTRICAL | MAT_ELEC_METAL => {
            effect = Some("chunks/sparkexplode");
            ct = 5;
        }
        MAT_METAL | MAT_METAL2 | MAT_METAL3 | MAT_CRATE1 | MAT_CRATE2 => {
            effect = Some("chunks/metalexplode");
            ct = 2;
        }
        MAT_GRATE1 => {
            effect = Some("chunks/grateexplode");
            ct = 8;
        }
        MAT_ROPE => {
            ct = 20;
            effect = Some("chunks/ropebreak");
        }
        // not sure what this crap is really supposed to be..
        MAT_WHITE_METAL | MAT_DRK_STONE | MAT_LT_STONE | MAT_GREY_STONE | MAT_SNOWY_ROCK => {
            effect = Some(match size {
                2 => "chunks/rockbreaklg",
                // 1 and default
                _ => "chunks/rockbreakmed",
            });
        }
        _ => {}
    }

    let Some(effect) = effect else {
        return;
    };

    ct += 7 * size;

    // FIXME: real precache .. VERify that these need to be here...don't think they would because
    // the effects should be registered in g_breakable
    // rww - No they don't.. indexed effects gameside get precached on load clientside, as server
    // objects are setup before client asset load time. However, we need to index them, so..
    let eID1 = trap::FX_RegisterEffect(ctx.engine, effect);

    let has_effect2 = effect2.is_some_and(|e| !e.is_empty());
    let eID2 = if has_effect2 {
        // FIXME: real precache
        trap::FX_RegisterEffect(ctx.engine, effect2.unwrap())
    } else {
        0
    };

    // spawn chunk roughly in the bbox of the thing..
    for _ in 0..ct {
        let mut org = [0.0f32; 3];
        for j in 0..3 {
            let r = ctx.world.bg_state.rng.random() * 0.8 + 0.1;
            org[j] = r * mins[j] + (1.0 - r) * maxs[j];
        }

        // shoot effect away from center
        let mut dir = [0.0f32; 3];
        _VectorSubtract(org, mid, &mut dir);
        VectorNormalize(&mut dir);

        if has_effect2 && (ctx.world.bg_state.rng.rand() & 1) != 0 {
            trap::FX_PlayEffectID(ctx.engine, eID2, &org, &dir, -1, -1);
        } else {
            trap::FX_PlayEffectID(ctx.engine, eID1, &org, &dir, -1, -1);
        }
    }
}
