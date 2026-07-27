//! Raven `tr_surfacesprites.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_surfacesprites.cpp`

// Raven-named functions/types keep their original casing across this
// transcription, matching the rest of the renderer/engine crates.
#![allow(non_snake_case)]
// Raven's own dead stores are transcribed as written (porting-rules §A2/§C10).
#![allow(unused_assignments)]

use core::f64::consts::PI;

use mp_engine_qcommon::common::{com_printf, Common};
use mp_qshared::common::mp::cgame::color4ub_t::color4ub_t;
use mp_qshared::shared::{vec2_t, vec3_t, vec4_t};
use native_math::qmath::{vectoangles, AngleVectors, CrossProduct, PITCH, ROLL, YAW};

use crate::render_state::placeholders::RefEntity;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::tr_local::shader_stage_t::shaderStage_t;
use crate::tr_quicksprite::CQuickSpriteSystem;
use crate::tr_worldeffects::world_effects::{R_IsPuffing, WindZoneState, WorldEffectsState};

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

    // --- fields below landed by the `R_SurfaceSpriteFrameUpdate` wave-1
    // transcription. The file-scope declarations for all of them sit above
    // this wave's packet slice (`tr_surfacesprites.cpp:1-86`, not
    // transcribed); each type below is inferred from its usage in the
    // packet's function bodies, per this struct's own "NAMED BY THIS WAVE"
    // charter (DEC-37 A13.3).
    /// `curWeatherAmount` — current per-frame weather intensity (rain/puff
    /// scale) driving weather-affected surface-sprite density.
    ///
    /// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:170,174`
    pub cur_weather_amount: f32,
    /// `curWindBlowVect` — current per-frame wind-blow vector (direction *
    /// speed), smoothed toward `target_wind_blow_vect` each update.
    ///
    /// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:263-264`
    pub cur_wind_blow_vect: vec3_t,
    /// `curWindGust` — current per-frame gust magnitude (seconds-to-next
    /// -gust divisor, or a direct cvar-sourced gust strength).
    ///
    /// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:100-193`
    pub cur_wind_gust: f32,
    /// `curWindPoint` — current per-frame point-wind source location (XY
    /// from cvars, Z pinned to 0).
    ///
    /// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:280-282`
    pub cur_wind_point: vec3_t,
    /// `curWindPointActive` — whether a point-wind source is currently in
    /// effect (`curWindPointForce >= 0.01`).
    ///
    /// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:275,279`
    pub cur_wind_point_active: bool,
    /// `curWindPointForce` — current per-frame point-wind force, smoothed
    /// toward `r_windPointForce`'s cvar value.
    ///
    /// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:272`
    pub cur_wind_point_force: f32,
    /// `gustLeft` — seconds remaining in the current wind gust.
    ///
    /// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:103,198-213`
    pub gust_left: f32,
    /// `lastSSUpdateTime` — `backEnd.refdef.time` at the last update, used to
    /// detect a new frame and to derive `dtime`/gust decay.
    ///
    /// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:95,270`
    pub last_ss_update_time: i32,
    /// `nextGustTime` — `backEnd.refdef.time` at which the next gust may
    /// begin.
    ///
    /// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:102,206,209`
    pub next_gust_time: f32,
    /// `rangescalefactor` — multiplies shader `fadeMax`/`fadeDist` ranges to
    /// compensate for a non-standard FOV.
    ///
    /// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:125-140`
    pub range_scale_factor: f32,
    /// `ssLastEntityDrawn` — the last entity a surface sprite was drawn for,
    /// reset to `None` at the top of every frame.
    ///
    /// PORT-NOTE: Raven compares this by pointer identity (`trRefEntity_t
    /// *`); this wave only writes `None` (the reset), no read site is in its
    /// packet — whichever wave adds the read must settle how by-value
    /// `RefEntity` equality substitutes for pointer identity.
    ///
    /// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:107`
    pub ss_last_entity_drawn: Option<RefEntity>,
    /// `sssurfaces` — per-frame surface-sprite-bearing-surface counter,
    /// printed by the `r_surfaceSprites >= 2` debug path and reset each
    /// frame.
    ///
    /// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:287,291`
    pub ss_surfaces: i32,
    /// `standardfovinitialized` — whether `standard_fov_x`/`standard_scale_x`
    /// have been captured from the first rendered view yet.
    ///
    /// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:112,118`
    pub standard_fov_initialized: bool,
    /// `standardfovx` — the FOV of the first view rendered, used as the
    /// baseline for `range_scale_factor`.
    ///
    /// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:116,122,127`
    pub standard_fov_x: f32,
    /// `standardscalex` — `tan(standard_fov_x/2)`, the baseline scale factor.
    ///
    /// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:117,123`
    pub standard_scale_x: f32,
    /// `targetWindBlowVect` — this frame's target wind-blow vector, before
    /// smoothing into `cur_wind_blow_vect`.
    ///
    /// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:248,262-264`
    pub target_wind_blow_vect: vec3_t,
    /// `targetWindGrassDir` — this frame's target grass-sway direction,
    /// before smoothing into `cur_wind_grass_dir`.
    ///
    /// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:242-243,267-268`
    pub target_wind_grass_dir: vec3_t,
    /// `totalsurfsprites` — per-frame surface-sprite draw counter, printed by
    /// the `r_surfaceSprites >= 2` debug path and reset each frame.
    ///
    /// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:287,290`
    pub total_surf_sprites: i32,
}

/// Raven `WIND_DAMP_INTERVAL`.
///
/// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:57`
const WIND_DAMP_INTERVAL: i32 = 50;

/// Raven `WIND_GUST_TIME`.
///
/// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:58`
const WIND_GUST_TIME: f64 = 2500.0;

/// Raven `WIND_GUST_DECAY` (`1.0 / WIND_GUST_TIME`) — a C `double`, so every
/// expression it feeds is evaluated in `f64` (ruling 12).
///
/// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:59`
const WIND_GUST_DECAY: f64 = 1.0 / WIND_GUST_TIME;

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

/// Raven `R_SurfaceSpriteFrameUpdate` — once-per-frame wind/FOV bookkeeping
/// for the surface-sprite subsystem: rebuilds the billboard right-vector
/// table, updates the smoothed wind vectors/gust state, and resets the
/// per-frame sprite counters.
///
/// `backEnd.refdef.time`/`.fov_x` are threaded as bare scalars
/// (`refdef_time`/`refdef_fov_x`) rather than through `&FrameState` — this
/// wave needs `.time`, which is not a landed `TrRefdef` field yet, so both
/// refdef reads stay bare scalars for one consistent source rather than
/// splitting across a landed/unlanded seam (matches `tr_main.rs`
/// `R_SetupProjection`'s `SetFarClip` precedent).
///
/// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:87-292`
// Deferred `todo!()` escalation sites (cited above) diverge, leaving the rest
// of this body statically unreachable and its inputs unread until the value
// they wait on lands.
#[allow(unreachable_code, unused_variables)]
pub fn R_SurfaceSpriteFrameUpdate(
    state: &mut SurfaceSpriteState,
    wind: &WindZoneState,
    effects: &WorldEffectsState,
    common: &mut Common,
    cvars: &RendererCvars,
    refdef_time: i32,
    refdef_fov_x: f32,
) {
    if refdef_time == state.last_ss_update_time {
        return;
    }

    if refdef_time < state.last_ss_update_time {
        // Time is BEFORE the last update time, so reset everything.
        state.cur_wind_gust = 5.0;
        state.cur_wind_speed = common.cvar(cvars.r_windSpeed).value;
        state.next_gust_time = 0.0;
        state.gust_left = 0.0;
    }

    // Reset the last entity drawn, since this is a new frame.
    state.ss_last_entity_drawn = None;

    // Adjust for an FOV.  If things look twice as wide on the screen, pretend the shaders have twice the range.
    // ASSUMPTION HERE IS THAT "standard" fov is the first one rendered.

    if !state.standard_fov_initialized {
        // This isn't initialized yet.
        if refdef_fov_x > 50.0 && refdef_fov_x < 135.0 {
            // I don't consider anything below 50 or above 135 to be "normal".
            state.standard_fov_x = refdef_fov_x;
            // C promotes to double (M_PI, tan()); f64 intermediate per
            // wave-0 ruling 12, rounded to f32 once at the assignment (C's
            // own narrowing point).
            state.standard_scale_x =
                (state.standard_fov_x as f64 * 0.5 * (PI / 180.0)).tan() as f32;
            state.standard_fov_initialized = true;
        } else {
            state.standard_fov_x = 90.0;
            state.standard_scale_x =
                (state.standard_fov_x as f64 * 0.5 * (PI / 180.0)).tan() as f32;
        }
        state.range_scale_factor = 1.0; // Don't multiply the shader range by anything.
    } else if state.standard_fov_x == refdef_fov_x {
        // This is the standard FOV (or higher), don't multiply the shader range.
        state.range_scale_factor = 1.0;
    } else {
        // We are using a non-standard FOV.  We need to multiply the range of the shader by a scale factor.
        if refdef_fov_x > 135.0 {
            state.range_scale_factor =
                (state.standard_scale_x as f64 / (135.0_f64 * 0.5 * (PI / 180.0)).tan()) as f32;
        } else {
            state.range_scale_factor = (state.standard_scale_x as f64
                / (refdef_fov_x as f64 * 0.5 * (PI / 180.0)).tan())
                as f32;
        }
    }

    // Create a set of four right vectors so that vertical sprites aren't always facing the same way.
    // First generate a HORIZONTAL forward vector (important).
    let up: vec3_t = [0.0, 0.0, 1.0];
    CrossProduct(state.ss_view_right, up, &mut state.ss_fwd_vector);

    // Right Zero has a nudge forward (10 degrees).
    let right0: vec3_t = [
        state.ss_view_right[0] * 0.985 + 0.174 * state.ss_fwd_vector[0],
        state.ss_view_right[1] * 0.985 + 0.174 * state.ss_fwd_vector[1],
        state.ss_view_right[2] * 0.985 + 0.174 * state.ss_fwd_vector[2],
    ];

    // Right One has a big nudge back (30 degrees).
    let right1: vec3_t = [
        state.ss_view_right[0] * 0.866 + -0.5 * state.ss_fwd_vector[0],
        state.ss_view_right[1] * 0.866 + -0.5 * state.ss_fwd_vector[1],
        state.ss_view_right[2] * 0.866 + -0.5 * state.ss_fwd_vector[2],
    ];

    // Right two has a big nudge forward (30 degrees).
    let right2: vec3_t = [
        state.ss_view_right[0] * 0.866 + 0.5 * state.ss_fwd_vector[0],
        state.ss_view_right[1] * 0.866 + 0.5 * state.ss_fwd_vector[1],
        state.ss_view_right[2] * 0.866 + 0.5 * state.ss_fwd_vector[2],
    ];

    // Right three has a nudge back (10 degrees).
    let right3: vec3_t = [
        state.ss_view_right[0] * 0.985 + -0.174 * state.ss_fwd_vector[0],
        state.ss_view_right[1] * 0.985 + -0.174 * state.ss_fwd_vector[1],
        state.ss_view_right[2] * 0.985 + -0.174 * state.ss_fwd_vector[2],
    ];

    state.ss_right_vectors = vec![right0, right1, right2, right3];

    // Update the wind.
    // If it is raining, get the windspeed from the rain system rather than the cvar
    if effects.R_IsRaining() || R_IsPuffing() {
        state.cur_weather_amount = 1.0;
    } else {
        state.cur_weather_amount = common.cvar(cvars.r_surfaceWeather).value;
    }

    let (got_wind_speed, mut targetspeed) = wind.R_GetWindSpeed();
    if got_wind_speed {
        // We successfully got a speed from the rain system.
        // Set the windgust to 5, since that looks pretty good.
        targetspeed *= 0.3;
        if targetspeed >= 1.0 {
            state.cur_wind_gust = 300.0 / targetspeed;
        } else {
            state.cur_wind_gust = 0.0;
        }
    } else {
        // Use the cvar.
        targetspeed = common.cvar(cvars.r_windSpeed).value; // Minimum gust delay, in seconds.
        state.cur_wind_gust = common.cvar(cvars.r_windGust).value;
    }

    if targetspeed > 0.0 && state.cur_wind_gust != 0.0 {
        if state.gust_left > 0.0 {
            // We are gusting
            // Add an amount to the target wind speed
            targetspeed *= 1.0 + state.gust_left;

            // `WIND_GUST_DECAY` is a C `double`, so the compound assignment
            // runs in f64 and rounds once on store (ruling 12).
            state.gust_left = (state.gust_left as f64
                - (refdef_time - state.last_ss_update_time) as f32 as f64 * WIND_GUST_DECAY)
                as f32;
            if state.gust_left <= 0.0 {
                // DEFERRED: flrand — the renderer's own engine LCG; R2
                // assigns the renderer no rand-family receiver (this
                // packet's rand-family note, DEC-37 A13.3). Wire a live
                // source when one is threaded to this fn.
                // Source: oracle/codemp/renderer/tr_surfacesprites.cpp:206
                state.next_gust_time = todo!(
                    "DEFERRED: flrand receiver — oracle/codemp/renderer/tr_surfacesprites.cpp:206"
                );
            }
        } else if refdef_time as f32 >= state.next_gust_time {
            // See if there is another right now
            // Gust next time, mano
            // DEFERRED: flrand — see the note above.
            // Source: oracle/codemp/renderer/tr_surfacesprites.cpp:212
            state.gust_left = todo!(
                "DEFERRED: flrand receiver — oracle/codemp/renderer/tr_surfacesprites.cpp:212"
            );
        }
    }

    // See if there is a weather system that will tell us a windspeed.
    let mut ang: vec3_t = [0.0, 0.0, 0.0];
    let (got_wind_vector, mut retwindvec) = wind.R_GetWindVector();
    if got_wind_vector {
        retwindvec[2] = 0.0;
        retwindvec = [-retwindvec[0], -retwindvec[1], -retwindvec[2]];
        vectoangles(retwindvec, &mut ang);
    } else {
        // Calculate the target wind vector based off cvars
        ang[YAW] = common.cvar(cvars.r_windAngle).value;
    }

    ang[PITCH] = -90.0 + targetspeed;
    if ang[PITCH] > -45.0 {
        ang[PITCH] = -45.0;
    }
    ang[ROLL] = 0.0;

    // Raven: both statements in this branch are commented out in the oracle
    // (`//ang[YAW] += cos(...)`, `//ang[PITCH] += sin(...)`), so the branch
    // is a no-op; dropped rather than transcribed as an empty `if` for
    // fidelity with zero behavioral difference (porting-rules §A2/§C10).
    // Source: oracle/codemp/renderer/tr_surfacesprites.cpp:235-239

    // Get the grass wind vector first
    AngleVectors(ang, Some(&mut state.target_wind_grass_dir), None, None);
    state.target_wind_grass_dir[2] -= 1.0;

    // Now get the general wind vector (no pitch)
    ang[PITCH] = 0.0;
    AngleVectors(ang, Some(&mut state.target_wind_blow_vect), None, None);

    // Start calculating a smoothing factor so wind doesn't change abruptly between speeds.
    let dampfactor = 1.0 - common.cvar(cvars.r_windDampFactor).value; // We must exponent the amount LEFT rather than the amount bled off
                                                                      // The `1.0` literal is a C `double`, so the whole product is
                                                                      // evaluated in f64 and rounds once on store (ruling 12).
    let dtime = ((refdef_time - state.last_ss_update_time) as f32 as f64
        * (1.0 / WIND_DAMP_INTERVAL as f32 as f64)) as f32; // Our dampfactor is geared towards a time interval equal to "1".

    // Note that since there are a finite number of "practical" delta millisecond values possible,
    // the ratio should be initialized into a chart ultimately.
    // C promotes to double (pow()); f64 intermediate per wave-0 ruling 12.
    let ratio = (dampfactor as f64).powf(dtime as f64) as f32;

    // Apply this ratio to the windspeed...
    state.cur_wind_speed = targetspeed - (ratio * (targetspeed - state.cur_wind_speed));

    // Use the curWindSpeed to calculate the final target wind vector (with speed)
    state.target_wind_blow_vect = [
        state.target_wind_blow_vect[0] * state.cur_wind_speed,
        state.target_wind_blow_vect[1] * state.cur_wind_speed,
        state.target_wind_blow_vect[2] * state.cur_wind_speed,
    ];
    let diff: vec3_t = [
        state.target_wind_blow_vect[0] - state.cur_wind_blow_vect[0],
        state.target_wind_blow_vect[1] - state.cur_wind_blow_vect[1],
        state.target_wind_blow_vect[2] - state.cur_wind_blow_vect[2],
    ];
    state.cur_wind_blow_vect = [
        state.target_wind_blow_vect[0] + -ratio * diff[0],
        state.target_wind_blow_vect[1] + -ratio * diff[1],
        state.target_wind_blow_vect[2] + -ratio * diff[2],
    ];

    // Update the grass vector now
    let diff2: vec3_t = [
        state.target_wind_grass_dir[0] - state.cur_wind_grass_dir[0],
        state.target_wind_grass_dir[1] - state.cur_wind_grass_dir[1],
        state.target_wind_grass_dir[2] - state.cur_wind_grass_dir[2],
    ];
    state.cur_wind_grass_dir = [
        state.target_wind_grass_dir[0] + -ratio * diff2[0],
        state.target_wind_grass_dir[1] + -ratio * diff2[1],
        state.target_wind_grass_dir[2] + -ratio * diff2[2],
    ];

    state.last_ss_update_time = refdef_time;

    let wind_point_force_cvar = common.cvar(cvars.r_windPointForce).value;
    state.cur_wind_point_force =
        wind_point_force_cvar - (ratio * (wind_point_force_cvar - state.cur_wind_point_force));
    if state.cur_wind_point_force < 0.01 {
        state.cur_wind_point_active = false;
    } else {
        state.cur_wind_point_active = true;
        state.cur_wind_point[0] = common.cvar(cvars.r_windPointX).value;
        state.cur_wind_point[1] = common.cvar(cvars.r_windPointY).value;
        state.cur_wind_point[2] = 0.0;
    }

    if common.cvar(cvars.r_surfaceSprites).integer >= 2 {
        com_printf(
            common,
            &format!(
                "Surfacesprites Drawn: {}, on {} surfaces\n",
                state.total_surf_sprites, state.ss_surfaces
            ),
        );
    }

    state.total_surf_sprites = 0;
    state.ss_surfaces = 0;
}

/// Raven `RB_DrawVerticalSurfaceSprites`.
///
/// DEFERRED: R4 — every observable effect (the `RB_VerticalSurfaceSprite`/
/// `RB_VerticalSurfaceSpriteWindPoint` calls) lives inside a loop keyed on
/// `input` (`shaderCommands_t *`), the same R4-dissolved type as `tess` (R2
/// `## State ownership` row `tess`: "dissolved into R4's
/// tessellation/vertex-building pipeline ... no single global scratch
/// buffer survives the new topology"); no R3 type exists for
/// `numVertexes`/`xyz`/`indexes`/`normal`/`vertexColors`, nor for
/// `tess.svars.texcoords`/`tess.SSInitializedWind`. The pre-loop setup
/// (`cutdist`/`fadedist`/`inv_fadediff`/`faderange`) also needs the
/// unresolved `FADE_RANGE` `#define` (not in this wave's packet) and feeds
/// nothing but that same loop — computing it here would be dead code with
/// zero behavioral effect once deferred (same treatment as `tr_shade.rs`'s
/// `DrawNormals`).
///
/// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:497-793`
pub fn RB_DrawVerticalSurfaceSprites(
    _stage: &shaderStage_t,
    _state: &mut SurfaceSpriteState,
    _quick_sprite: &mut CQuickSpriteSystem,
) {
    // DEFERRED: R4 — RB_DrawVerticalSurfaceSprites (see doc comment above)
    // Source: oracle/codemp/renderer/tr_surfacesprites.cpp:497-793
}

/// Raven `RB_DrawOrientedSurfaceSprites`.
///
/// DEFERRED: R4 — same reasoning as `RB_DrawVerticalSurfaceSprites`: every
/// observable effect (the `RB_OrientedSurfaceSprite` calls) lives inside a
/// loop keyed on `input` (`shaderCommands_t *`, R4-dissolved, R2 `##
/// State ownership` row `tess`); the pre-loop setup needs the unresolved
/// `FADE_RANGE` `#define` and feeds nothing but that same loop.
///
/// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:881-1068`
pub fn RB_DrawOrientedSurfaceSprites(
    _stage: &shaderStage_t,
    _state: &mut SurfaceSpriteState,
    _quick_sprite: &mut CQuickSpriteSystem,
) {
    // DEFERRED: R4 — RB_DrawOrientedSurfaceSprites (see doc comment above)
    // Source: oracle/codemp/renderer/tr_surfacesprites.cpp:881-1068
}

/// Raven `RB_DrawEffectSurfaceSprites`.
///
/// DEFERRED: R4 — same reasoning as `RB_DrawVerticalSurfaceSprites`: every
/// observable effect (the `RB_EffectSurfaceSprite` calls) lives inside a
/// loop keyed on `input` (`shaderCommands_t *`, R4-dissolved, R2 `##
/// State ownership` row `tess`); the pre-loop setup needs the unresolved
/// `FADE_RANGE` `#define` and feeds nothing but that same loop. The early
/// `SURFSPRITE_WEATHERFX` return (`curWeatherAmount < 0.01`) is real
/// per-frame state (`SurfaceSpriteState::cur_weather_amount`) but has no
/// observable effect independent of the deferred loop either, so it is not
/// split out.
///
/// Source: `oracle/codemp/renderer/tr_surfacesprites.cpp:1156-1387`
pub fn RB_DrawEffectSurfaceSprites(
    _stage: &shaderStage_t,
    _state: &mut SurfaceSpriteState,
    _quick_sprite: &mut CQuickSpriteSystem,
) {
    // DEFERRED: R4 — RB_DrawEffectSurfaceSprites (see doc comment above)
    // Source: oracle/codemp/renderer/tr_surfacesprites.cpp:1156-1387
}
