//! Raven `tr_surfacesprites.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_surfacesprites.cpp`

// Raven-named functions/types keep their original casing across this
// transcription, matching the rest of the renderer/engine crates.
#![allow(non_snake_case)]
// Raven's own dead stores are transcribed as written (porting-rules §A2/§C10).
#![allow(unused_assignments)]

use mp_qshared::common::mp::cgame::color4ub_t::color4ub_t;
use mp_qshared::shared::{vec2_t, vec3_t, vec4_t};

use crate::tr_quicksprite::CQuickSpriteSystem;

/// Per-subsystem owned state for `tr_surfacesprites.cpp`'s render-thread
/// file-scope statics — named by this wave (DEC-37 A13.3): these are
/// computed once per frame by writers not yet in this wave's fn list (wind/
/// view-vector setup upstream in the same TU) and only read by the four
/// functions below. No row in R2 `## State ownership` — not a `trGlobals_t`
/// member — so this wave threads a dedicated carrier rather than inventing a
/// field on `RenderAssets`/`FrameState`.
///
/// Type definition source: writer bodies are outside this wave's packet;
/// fields below cover exactly what `RB_VerticalSurfaceSprite`/
/// `RB_VerticalSurfaceSpriteWindPoint`/`RB_OrientedSurfaceSprite`/
/// `RB_EffectSurfaceSprite` read. A later wave porting the writers extends
/// this struct rather than forking a second one.
pub struct SurfaceSpriteState {
    /// `curWindGrassDir` — current per-frame wind direction used to sway
    /// vertical sprites (grass, foliage).
    pub cur_wind_grass_dir: vec3_t,
    /// `curWindSpeed` — current per-frame wind speed magnitude.
    pub cur_wind_speed: f32,
    /// `rightvectorcount` — index into `ss_right_vectors` selecting this
    /// sprite's billboard right vector.
    pub right_vector_count: i32,
    /// `ssfwdvector` — per-frame forward vector, used to skew vertical
    /// sprites' top edge toward the camera.
    pub ss_fwd_vector: vec3_t,
    /// `ssrightvectors` — per-frame table of candidate billboard right
    /// vectors, indexed by `right_vector_count`.
    pub ss_right_vectors: Vec<vec3_t>,
    /// `ssViewRight` — per-frame view right vector for oriented/effect
    /// sprites.
    pub ss_view_right: vec3_t,
    /// `ssViewUp` — per-frame view up vector for oriented/effect sprites.
    pub ss_view_up: vec3_t,
}

/// Raven `RB_VerticalSurfaceSprite`.
///
/// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:314-409`
///
/// PORT-NOTE: `tr.refdef.time` is homed at `FrameState::refdef.time` per R2's
/// SPLIT row for `tr` (frontend scratch), but `TrRefdef`
/// (`crate::render_state::placeholders::TrRefdef`) is still an empty
/// placeholder — its fields land with the `tr_main` R3 wave. Threaded here as
/// a plain `refdef_time` parameter rather than through `&FrameState` so this
/// file doesn't add a field to a type it doesn't own; the caller passes
/// `frame.refdef.time` once that field exists.
pub fn RB_VerticalSurfaceSprite(
    quick_sprite: &mut CQuickSpriteSystem,
    state: &SurfaceSpriteState,
    refdef_time: i32,
    loc: vec3_t,
    width: f32,
    height: f32,
    light: u8,
    alpha: u8,
    wind: f32,
    windidle: f32,
    fog: vec2_t,
    hangdown: bool,
    skew: vec2_t,
) {
    let angle = (loc[0] + loc[1]) * 0.02 + (refdef_time as f32 * 0.0015);

    let mut loc2: vec3_t = [0.0; 3];

    if windidle > 0.0 {
        let windsway = height * windidle * 0.075;
        loc2[0] = loc[0] + skew[0] + angle.cos() * windsway;
        loc2[1] = loc[1] + skew[1] + angle.sin() * windsway;

        loc2[2] = if hangdown {
            loc[2] - height
        } else {
            loc[2] + height
        };
    } else {
        loc2[0] = loc[0] + skew[0];
        loc2[1] = loc[1] + skew[1];
        loc2[2] = if hangdown {
            loc[2] - height
        } else {
            loc[2] + height
        };
    }

    if wind > 0.0 && state.cur_wind_speed > 0.001 {
        let mut windsway = height * wind * 0.075;

        // Add the angle
        // VectorMA(loc2, height*wind, curWindGrassDir, loc2);
        let scale = height * wind;
        loc2[0] += scale * state.cur_wind_grass_dir[0];
        loc2[1] += scale * state.cur_wind_grass_dir[1];
        loc2[2] += scale * state.cur_wind_grass_dir[2];

        // Bob up and down
        if state.cur_wind_speed < 40.0 {
            windsway *= state.cur_wind_speed * (1.0 / 100.0);
        } else {
            windsway *= 0.4;
        }
        loc2[2] += (angle * 2.5).sin() * windsway;
    }

    // VectorScale(ssrightvectors[rightvectorcount], width*0.5, right);
    let sv = state.ss_right_vectors[state.right_vector_count as usize];
    let s = width * 0.5;
    let right: vec3_t = [sv[0] * s, sv[1] * s, sv[2] * s];

    let color: color4ub_t = [light, light, light, alpha];

    let points: [vec4_t; 4] = [
        // Bottom right
        [loc[0] + right[0], loc[1] + right[1], loc[2] + right[2], 0.0],
        // Top right
        [
            loc2[0] + right[0],
            loc2[1] + right[1],
            loc2[2] + right[2],
            0.0,
        ],
        // Top left
        [
            loc2[0] - right[0] + state.ss_fwd_vector[0] * width * 0.2,
            loc2[1] - right[1] + state.ss_fwd_vector[1] * width * 0.2,
            loc2[2] - right[2],
            0.0,
        ],
        // Bottom left
        [loc[0] - right[0], loc[1] - right[1], loc[2] - right[2], 0.0],
    ];

    // Add the sprite to the render list.
    quick_sprite.add(points, color, Some(fog));
}

/// Raven `RB_VerticalSurfaceSpriteWindPoint`.
///
/// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:411-495`
///
/// PORT-NOTE: `tr.refdef.time` threaded as `refdef_time` — see
/// `RB_VerticalSurfaceSprite`'s PORT-NOTE.
pub fn RB_VerticalSurfaceSpriteWindPoint(
    quick_sprite: &mut CQuickSpriteSystem,
    state: &SurfaceSpriteState,
    refdef_time: i32,
    loc: vec3_t,
    width: f32,
    height: f32,
    light: u8,
    alpha: u8,
    wind: f32,
    windidle: f32,
    fog: vec2_t,
    hangdown: bool,
    skew: vec2_t,
    winddiff: vec2_t,
    mut windforce: f32,
) {
    if windforce > 1.0 {
        windforce = 1.0;
    }

    // wind += 1.0-windforce;

    let angle = (loc[0] + loc[1]) * 0.02 + (refdef_time as f32 * 0.0015);

    let mut loc2: vec3_t = [0.0; 3];

    if state.cur_wind_speed < 80.0 {
        let windsway = (height * windidle * 0.1) * (1.0 + windforce);
        loc2[0] = loc[0] + skew[0] + angle.cos() * windsway;
        loc2[1] = loc[1] + skew[1] + angle.sin() * windsway;
    } else {
        loc2[0] = loc[0] + skew[0];
        loc2[1] = loc[1] + skew[1];
    }

    loc2[2] = if hangdown {
        loc[2] - height
    } else {
        loc[2] + height
    };

    if state.cur_wind_speed > 0.001 {
        // Add the angle
        // VectorMA(loc2, height*wind, curWindGrassDir, loc2);
        let scale = height * wind;
        loc2[0] += scale * state.cur_wind_grass_dir[0];
        loc2[1] += scale * state.cur_wind_grass_dir[1];
        loc2[2] += scale * state.cur_wind_grass_dir[2];
    }

    loc2[0] += height * winddiff[0] * windforce;
    loc2[1] += height * winddiff[1] * windforce;
    loc2[2] -= height
        * windforce
        * (0.75 + 0.15 * ((refdef_time as f32 + 500.0 * windforce) * 0.01).sin());

    // VectorScale(ssrightvectors[rightvectorcount], width*0.5, right);
    let sv = state.ss_right_vectors[state.right_vector_count as usize];
    let s = width * 0.5;
    let right: vec3_t = [sv[0] * s, sv[1] * s, sv[2] * s];

    let color: color4ub_t = [light, light, light, alpha];

    let points: [vec4_t; 4] = [
        // Bottom right
        [loc[0] + right[0], loc[1] + right[1], loc[2] + right[2], 0.0],
        // Top right
        [
            loc2[0] + right[0],
            loc2[1] + right[1],
            loc2[2] + right[2],
            0.0,
        ],
        // Top left
        [
            loc2[0] - right[0] + state.ss_fwd_vector[0] * width * 0.15,
            loc2[1] - right[1] + state.ss_fwd_vector[1] * width * 0.15,
            loc2[2] - right[2],
            0.0,
        ],
        // Bottom left
        [loc[0] - right[0], loc[1] - right[1], loc[2] - right[2], 0.0],
    ];

    // Add the sprite to the render list.
    quick_sprite.add(points, color, Some(fog));
}

/// Raven `RB_OrientedSurfaceSprite`.
///
/// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:799-879`
pub fn RB_OrientedSurfaceSprite(
    quick_sprite: &mut CQuickSpriteSystem,
    state: &SurfaceSpriteState,
    loc: vec3_t,
    mut width: f32,
    mut height: f32,
    light: u8,
    alpha: u8,
    fog: vec2_t,
    faceup: bool,
) {
    let color: color4ub_t = [light, light, light, alpha];

    let points: [vec4_t; 4] = if faceup {
        width *= 0.5;
        height *= 0.5;

        [
            // Bottom right
            [loc[0] + width, loc[1] - width, loc[2] + 1.0, 0.0],
            // Top right
            [loc[0] + width, loc[1] + width, loc[2] + 1.0, 0.0],
            // Top left
            [loc[0] - width, loc[1] + width, loc[2] + 1.0, 0.0],
            // Bottom left
            [loc[0] - width, loc[1] - width, loc[2] + 1.0, 0.0],
        ]
    } else {
        // VectorMA(loc, height, ssViewUp, loc2);
        let loc2: vec3_t = [
            loc[0] + height * state.ss_view_up[0],
            loc[1] + height * state.ss_view_up[1],
            loc[2] + height * state.ss_view_up[2],
        ];
        // VectorScale(ssViewRight, width*0.5, right);
        let s = width * 0.5;
        let right: vec3_t = [
            state.ss_view_right[0] * s,
            state.ss_view_right[1] * s,
            state.ss_view_right[2] * s,
        ];

        [
            // Bottom right
            [loc[0] + right[0], loc[1] + right[1], loc[2] + right[2], 0.0],
            // Top right
            [
                loc2[0] + right[0],
                loc2[1] + right[1],
                loc2[2] + right[2],
                0.0,
            ],
            // Top left
            [
                loc2[0] - right[0],
                loc2[1] - right[1],
                loc2[2] - right[2],
                0.0,
            ],
            // Bottom left
            [loc[0] - right[0], loc[1] - right[1], loc[2] - right[2], 0.0],
        ]
    };

    // Add the sprite to the render list.
    quick_sprite.add(points, color, Some(fog));
}

/// Raven `RB_EffectSurfaceSprite`.
///
/// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:1074-1154`
pub fn RB_EffectSurfaceSprite(
    quick_sprite: &mut CQuickSpriteSystem,
    state: &SurfaceSpriteState,
    loc: vec3_t,
    mut width: f32,
    mut height: f32,
    light: u8,
    alpha: u8,
    _life: f32,
    faceup: bool,
) {
    let color: color4ub_t = [light, light, light, alpha]; // light; alpha;

    let points: [vec4_t; 4] = if faceup {
        width *= 0.5;
        height *= 0.5;

        [
            // Bottom right
            [loc[0] + width, loc[1] - width, loc[2] + 1.0, 0.0],
            // Top right
            [loc[0] + width, loc[1] + width, loc[2] + 1.0, 0.0],
            // Top left
            [loc[0] - width, loc[1] + width, loc[2] + 1.0, 0.0],
            // Bottom left
            [loc[0] - width, loc[1] - width, loc[2] + 1.0, 0.0],
        ]
    } else {
        // VectorMA(loc, height, ssViewUp, loc2);
        let loc2: vec3_t = [
            loc[0] + height * state.ss_view_up[0],
            loc[1] + height * state.ss_view_up[1],
            loc[2] + height * state.ss_view_up[2],
        ];
        // VectorScale(ssViewRight, width*0.5, right);
        let s = width * 0.5;
        let right: vec3_t = [
            state.ss_view_right[0] * s,
            state.ss_view_right[1] * s,
            state.ss_view_right[2] * s,
        ];

        [
            // Bottom right
            [loc[0] + right[0], loc[1] + right[1], loc[2] + right[2], 0.0],
            // Top right
            [
                loc2[0] + right[0],
                loc2[1] + right[1],
                loc2[2] + right[2],
                0.0,
            ],
            // Top left
            [
                loc2[0] - right[0],
                loc2[1] - right[1],
                loc2[2] - right[2],
                0.0,
            ],
            // Bottom left
            [loc[0] - right[0], loc[1] - right[1], loc[2] - right[2], 0.0],
        ]
    };

    // Add the sprite to the render list.
    quick_sprite.add(points, color, None);
}
