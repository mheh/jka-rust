//! Port of `oracle/codemp/cgame/cg_marks.c` — wall marks, their pool, and the shader-animation tables. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case, non_upper_case_globals)]

use core::ffi::c_int;

use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::shared::q_math::{
    _VectorCopy, _VectorMA, vectoangles, AngleVectors, Distance, VectorClear, VectorLength,
    VectorSet, ROLL,
};
use mp_qshared::shared::{qhandle_t, vec3_t};
use native_string::Q_stricmp;

use crate::cg_main::{CG_Error, CG_Printf};
use crate::trap;
use crate::world::cg_context::CgContext;
use crate::world::cg_marks_state::particle_type_t::{
    P_ANIM, P_BAT, P_BLEED, P_BUBBLE, P_BUBBLE_TURBULENT, P_FLAT, P_FLAT_SCALEUP,
    P_FLAT_SCALEUP_FADE, P_NONE, P_SMOKE, P_SMOKE_IMPACT, P_SPRITE, P_WEATHER, P_WEATHER_FLURRY,
    P_WEATHER_TURBULENT,
};
use crate::world::cg_marks_state::CgMarksState;
use crate::world::cg_world::CgWorld;
use crate::world::effect_handle::EffectHandle;

// FILE-SCOPE CONSTANTS
// Source: `oracle/codemp/cgame/cg_marks.c:107-108,218-219,296-298,360-372,377-378,1904,2008-2009`

/// Raven `MAX_MARK_FRAGMENTS` — `CG_ImpactMark`'s fragment-buffer size.
/// Source: `oracle/codemp/cgame/cg_marks.c:107`
pub const MAX_MARK_FRAGMENTS: usize = 128;

/// Raven `MAX_MARK_POINTS` — `CG_ImpactMark`'s point-buffer size.
/// Source: `oracle/codemp/cgame/cg_marks.c:108`
pub const MAX_MARK_POINTS: usize = 384;

/// Raven `MARK_TOTAL_TIME` — how long a mark lives, in msec.
/// Source: `oracle/codemp/cgame/cg_marks.c:218`
pub const MARK_TOTAL_TIME: c_int = 10000;

/// Raven `MARK_FADE_TIME` — the fade-out tail of [`MARK_TOTAL_TIME`], in msec.
/// Source: `oracle/codemp/cgame/cg_marks.c:219`
pub const MARK_FADE_TIME: c_int = 1000;

/// Raven `BLOODRED` — a `cparticle_t.color` selector, not an RGB value.
/// Source: `oracle/codemp/cgame/cg_marks.c:296`
pub const BLOODRED: c_int = 2;

/// Raven `EMISIVEFADE` — `cparticle_t.color` selector (sic; Raven's spelling).
/// Source: `oracle/codemp/cgame/cg_marks.c:297`
pub const EMISIVEFADE: c_int = 3;

/// Raven `GREY75` — `cparticle_t.color` selector: grey that brightens as the
/// viewer closes in.
/// Source: `oracle/codemp/cgame/cg_marks.c:298`
pub const GREY75: c_int = 4;

/// Raven `MAX_SHADER_ANIMS`.
/// Source: `oracle/codemp/cgame/cg_marks.c:360`
pub const MAX_SHADER_ANIMS: usize = 32;

/// Raven `MAX_SHADER_ANIM_FRAMES`.
/// Source: `oracle/codemp/cgame/cg_marks.c:361`
pub const MAX_SHADER_ANIM_FRAMES: usize = 64;

/// Raven `static char *shaderAnimNames[MAX_SHADER_ANIMS]` — the animated-shader
/// name table, NULL-terminated after its one entry.
/// [`CG_ParticleExplosion`] walks it until the `None`, so the terminator is
/// carried rather than collapsed to a one-element table.
/// Source: `oracle/codemp/cgame/cg_marks.c:363-366`
pub const shaderAnimNames: [Option<&str>; MAX_SHADER_ANIMS] = {
    let mut names = [None; MAX_SHADER_ANIMS];
    names[0] = Some("explode1");
    names
};

/// Raven `static int shaderAnimCounts[MAX_SHADER_ANIMS]` — frames per animated
/// shader. Read-only data, so it is a `const`, not state.
/// Source: `oracle/codemp/cgame/cg_marks.c:368-370`
pub const shaderAnimCounts: [c_int; MAX_SHADER_ANIMS] = {
    let mut counts = [0; MAX_SHADER_ANIMS];
    // "explode1" has 23 frames; C's initializer zero-fills the rest
    counts[0] = 23;
    counts
};

/// Raven `static float shaderAnimSTRatio[MAX_SHADER_ANIMS]` — per-animation
/// s/t aspect ratio. Nothing in the shipped tree reads it.
/// Source: `oracle/codemp/cgame/cg_marks.c:371-373`
pub const shaderAnimSTRatio: [f32; MAX_SHADER_ANIMS] = {
    let mut ratio = [0.0; MAX_SHADER_ANIMS];
    ratio[0] = 1.0;
    ratio
};

/// Raven `PARTICLE_GRAVITY`.
/// Source: `oracle/codemp/cgame/cg_marks.c:377`
pub const PARTICLE_GRAVITY: c_int = 40;

/// Raven `MAX_PARTICLES` — the particle pool's fixed size.
/// Source: `oracle/codemp/cgame/cg_marks.c:378`
pub const MAX_PARTICLES: usize = 1024;

/// Raven `EXTRUDE_DIST` — how far a mark's projection box extends either side
/// of the surface.
/// Source: `oracle/codemp/cgame/cg_marks.c:1904`
pub const EXTRUDE_DIST: f32 = 0.5;

/// Raven `NORMALSIZE` — the small particle footprint.
/// Source: `oracle/codemp/cgame/cg_marks.c:2008`
pub const NORMALSIZE: c_int = 16;

/// Raven `LARGESIZE` — the big particle footprint.
/// Source: `oracle/codemp/cgame/cg_marks.c:2009`
pub const LARGESIZE: c_int = 32;

/// Raven `CG_InitMarkPolys` — empties the mark pool.
///
/// Raven: "This is called at startup and for tournement restarts".
///
/// The `memset` plus the free-list/active-list relinking IS
/// [`EffectPool::clear`](crate::world::effect_pool::EffectPool::clear) — the
/// links dissolved into the slab under DEC-46.3.
///
/// Source: `oracle/codemp/cgame/cg_marks.c:28-39`
pub fn CG_InitMarkPolys(world: &mut CgWorld) {
    world.cg_markPolys.clear();
}

/// Raven `CG_ClearParticles` — rebuilds the whole particle pool as one free
/// list and drops everything that was live.
///
/// Raven's shaderAnims registration loop right below is commented out, which is
/// why `numShaderAnims` ends at 0 and the anim table stays empty in the shipped
/// build.
///
/// Source: `oracle/codemp/cgame/cg_marks.c:395-428`
pub fn CG_ClearParticles(world: &mut CgWorld) {
    let cgTime = world.cg.time;
    let marks = &mut world.marks;

    for p in marks.particles.iter_mut() {
        *p = Default::default();
    }

    marks.free_particles = Some(0);
    marks.active_particles = None;

    let cl_numparticles = marks.cl_numparticles as usize;
    for i in 0..cl_numparticles {
        // PORT-NOTE: the last pass writes a link to `particles[1024]`, one past
        // the array — Raven's address arithmetic is never dereferenced and the
        // line below overwrites it immediately.
        marks.particles[i].next = Some(i + 1);
        marks.particles[i].r#type = P_NONE;
    }
    marks.particles[cl_numparticles - 1].next = None;

    marks.oldtime = cgTime as f32;

    marks.numShaderAnims = 0;
    // done.

    marks.initparticles = true;
}

/// Raven `CG_AddParticleToScene` — turns one live particle into the renderer
/// poly its type calls for.
///
/// Raven's `cparticle_t *p` is the particle's pool slot number here: the
/// particle lives in `world.marks.particles`, which this call also has to reach
/// for the shader-anim table (§B5).
///
/// Raven's `alpha` parameter is dead — every branch either hardcodes the
/// modulate or reads `p->alpha` directly.
///
/// The slot is copied out, mutated, and written back at every exit; Raven
/// mutates `*p` in place and the callers see those writes.
///
/// PORT-NOTE: the `255 * <float>` modulate stores are C float-to-byte
/// conversions; C narrows the float through `int` before landing in the
/// `unsigned char` field (wraps mod 256), not Rust's saturating `as u8` -
/// truncate through `i32` first to match (`cg_localents.rs`'s `CG_AddFadeRGB`
/// needed the same fix).
///
/// Source: `oracle/codemp/cgame/cg_marks.c:436-1071`
pub fn CG_AddParticleToScene(ctx: &mut CgContext, pnum: usize, org: vec3_t, _alpha: f32) {
    let engine = ctx.engine;
    let cgTime = ctx.world.cg.time;
    // Raven errors out on a null `cg.snap` before any of this runs
    // (`cg_snapshot.c:402-403`); dereferencing it anyway would be UB, so the
    // unreachable arm answers with the zero origin rather than diverging
    // further (§F19).
    let snapOrigin = ctx
        .world
        .cg
        .snap_ref()
        .map(|snap| snap.ps.origin)
        .unwrap_or([0.0; 3]);
    let viewaxis0 = ctx.world.cg.refdef.viewaxis[0];
    let pvright = ctx.world.marks.pvright;
    let pvup = ctx.world.marks.pvup;
    let rforward = ctx.world.marks.rforward;
    let rright = ctx.world.marks.rright;
    let rup = ctx.world.marks.rup;

    let mut point: vec3_t = [0.0; 3];
    let mut verts = [polyVert_t {
        xyz: [0.0; 3],
        st: [0.0; 2],
        modulate: [0; 4],
    }; 4];
    let mut TRIverts = [polyVert_t {
        xyz: [0.0; 3],
        st: [0.0; 2],
        modulate: [0; 4],
    }; 3];
    let mut rright2: vec3_t = [0.0; 3];
    let mut rup2: vec3_t = [0.0; 3];

    let mut p = ctx.world.marks.particles[pnum];

    if p.r#type == P_WEATHER
        || p.r#type == P_WEATHER_TURBULENT
        || p.r#type == P_WEATHER_FLURRY
        || p.r#type == P_BUBBLE
        || p.r#type == P_BUBBLE_TURBULENT
    {
        // create a front facing polygon
        if p.r#type != P_WEATHER_FLURRY {
            if p.r#type == P_BUBBLE || p.r#type == P_BUBBLE_TURBULENT {
                if org[2] > p.end {
                    p.time = cgTime as f32;
                    // Ridah, fixes rare snow flakes that flicker on the ground
                    _VectorCopy(org, &mut p.org);

                    let jitter = ctx.world.bg_state.rng.crandom() * 4.0;
                    p.org[2] = (p.start as f64 + jitter) as f32;

                    if p.r#type == P_BUBBLE_TURBULENT {
                        p.vel[0] = (ctx.world.bg_state.rng.crandom() * 4.0) as f32;
                        p.vel[1] = (ctx.world.bg_state.rng.crandom() * 4.0) as f32;
                    }
                }
            } else if org[2] < p.end {
                p.time = cgTime as f32;
                // Ridah, fixes rare snow flakes that flicker on the ground
                _VectorCopy(org, &mut p.org);

                while p.org[2] < p.end {
                    p.org[2] += p.start - p.end;
                }

                if p.r#type == P_WEATHER_TURBULENT {
                    p.vel[0] = (ctx.world.bg_state.rng.crandom() * 16.0) as f32;
                    p.vel[1] = (ctx.world.bg_state.rng.crandom() * 16.0) as f32;
                }
            }

            // Rafael snow pvs check
            if !p.link {
                ctx.world.marks.particles[pnum] = p;
                return;
            }

            p.alpha = 1.0;
        }

        // Ridah, had to do this or MAX_POLYS is being exceeded in village1.bsp
        if Distance(snapOrigin, org) > 1024.0 {
            ctx.world.marks.particles[pnum] = p;
            return;
        }
        // done.

        if p.r#type == P_BUBBLE || p.r#type == P_BUBBLE_TURBULENT {
            _VectorMA(org, -p.height, pvup, &mut point);
            _VectorMA(point, -p.width, pvright, &mut point);
            _VectorCopy(point, &mut verts[0].xyz);
            verts[0].st[0] = 0.0;
            verts[0].st[1] = 0.0;
            verts[0].modulate[0] = 255;
            verts[0].modulate[1] = 255;
            verts[0].modulate[2] = 255;
            verts[0].modulate[3] = (255.0 * p.alpha) as i32 as u8;

            _VectorMA(org, -p.height, pvup, &mut point);
            _VectorMA(point, p.width, pvright, &mut point);
            _VectorCopy(point, &mut verts[1].xyz);
            verts[1].st[0] = 0.0;
            verts[1].st[1] = 1.0;
            verts[1].modulate[0] = 255;
            verts[1].modulate[1] = 255;
            verts[1].modulate[2] = 255;
            verts[1].modulate[3] = (255.0 * p.alpha) as i32 as u8;

            _VectorMA(org, p.height, pvup, &mut point);
            _VectorMA(point, p.width, pvright, &mut point);
            _VectorCopy(point, &mut verts[2].xyz);
            verts[2].st[0] = 1.0;
            verts[2].st[1] = 1.0;
            verts[2].modulate[0] = 255;
            verts[2].modulate[1] = 255;
            verts[2].modulate[2] = 255;
            verts[2].modulate[3] = (255.0 * p.alpha) as i32 as u8;

            _VectorMA(org, p.height, pvup, &mut point);
            _VectorMA(point, -p.width, pvright, &mut point);
            _VectorCopy(point, &mut verts[3].xyz);
            verts[3].st[0] = 1.0;
            verts[3].st[1] = 0.0;
            verts[3].modulate[0] = 255;
            verts[3].modulate[1] = 255;
            verts[3].modulate[2] = 255;
            verts[3].modulate[3] = (255.0 * p.alpha) as i32 as u8;
        } else {
            _VectorMA(org, -p.height, pvup, &mut point);
            _VectorMA(point, -p.width, pvright, &mut point);
            _VectorCopy(point, &mut TRIverts[0].xyz);
            TRIverts[0].st[0] = 1.0;
            TRIverts[0].st[1] = 0.0;
            TRIverts[0].modulate[0] = 255;
            TRIverts[0].modulate[1] = 255;
            TRIverts[0].modulate[2] = 255;
            TRIverts[0].modulate[3] = (255.0 * p.alpha) as i32 as u8;

            _VectorMA(org, p.height, pvup, &mut point);
            _VectorMA(point, -p.width, pvright, &mut point);
            _VectorCopy(point, &mut TRIverts[1].xyz);
            TRIverts[1].st[0] = 0.0;
            TRIverts[1].st[1] = 0.0;
            TRIverts[1].modulate[0] = 255;
            TRIverts[1].modulate[1] = 255;
            TRIverts[1].modulate[2] = 255;
            TRIverts[1].modulate[3] = (255.0 * p.alpha) as i32 as u8;

            _VectorMA(org, p.height, pvup, &mut point);
            _VectorMA(point, p.width, pvright, &mut point);
            _VectorCopy(point, &mut TRIverts[2].xyz);
            TRIverts[2].st[0] = 0.0;
            TRIverts[2].st[1] = 1.0;
            TRIverts[2].modulate[0] = 255;
            TRIverts[2].modulate[1] = 255;
            TRIverts[2].modulate[2] = 255;
            TRIverts[2].modulate[3] = (255.0 * p.alpha) as i32 as u8;
        }
    } else if p.r#type == P_SPRITE {
        let mut rr: vec3_t = [0.0; 3];
        let mut ru: vec3_t = [0.0; 3];
        let mut rotate_ang: vec3_t = [0.0; 3];
        let mut color: vec3_t = [0.0; 3];

        VectorSet(&mut color, 1.0, 1.0, 0.5);
        let time = (cgTime as f32) - p.time;
        let time2 = p.endtime - p.time;
        let ratio = time / time2;

        let width = p.width + (ratio * (p.endwidth - p.width));
        let height = p.height + (ratio * (p.endheight - p.height));

        if p.roll != 0 {
            vectoangles(viewaxis0, &mut rotate_ang);
            rotate_ang[ROLL] += p.roll as f32;
            AngleVectors(rotate_ang, None, Some(&mut rr), Some(&mut ru));
        }

        if p.roll != 0 {
            _VectorMA(org, -height, ru, &mut point);
            _VectorMA(point, -width, rr, &mut point);
        } else {
            _VectorMA(org, -height, pvup, &mut point);
            _VectorMA(point, -width, pvright, &mut point);
        }
        _VectorCopy(point, &mut verts[0].xyz);
        verts[0].st[0] = 0.0;
        verts[0].st[1] = 0.0;
        verts[0].modulate[0] = 255;
        verts[0].modulate[1] = 255;
        verts[0].modulate[2] = 255;
        verts[0].modulate[3] = 255;

        if p.roll != 0 {
            _VectorMA(point, 2.0 * height, ru, &mut point);
        } else {
            _VectorMA(point, 2.0 * height, pvup, &mut point);
        }
        _VectorCopy(point, &mut verts[1].xyz);
        verts[1].st[0] = 0.0;
        verts[1].st[1] = 1.0;
        verts[1].modulate[0] = 255;
        verts[1].modulate[1] = 255;
        verts[1].modulate[2] = 255;
        verts[1].modulate[3] = 255;

        if p.roll != 0 {
            _VectorMA(point, 2.0 * width, rr, &mut point);
        } else {
            _VectorMA(point, 2.0 * width, pvright, &mut point);
        }
        _VectorCopy(point, &mut verts[2].xyz);
        verts[2].st[0] = 1.0;
        verts[2].st[1] = 1.0;
        verts[2].modulate[0] = 255;
        verts[2].modulate[1] = 255;
        verts[2].modulate[2] = 255;
        verts[2].modulate[3] = 255;

        if p.roll != 0 {
            _VectorMA(point, -2.0 * height, ru, &mut point);
        } else {
            _VectorMA(point, -2.0 * height, pvup, &mut point);
        }
        _VectorCopy(point, &mut verts[3].xyz);
        verts[3].st[0] = 1.0;
        verts[3].st[1] = 0.0;
        verts[3].modulate[0] = 255;
        verts[3].modulate[1] = 255;
        verts[3].modulate[2] = 255;
        verts[3].modulate[3] = 255;
    } else if p.r#type == P_SMOKE || p.r#type == P_SMOKE_IMPACT {
        // create a front rotating facing polygon
        let mut color: vec3_t = [0.0; 3];

        if p.r#type == P_SMOKE_IMPACT && Distance(snapOrigin, org) > 1024.0 {
            ctx.world.marks.particles[pnum] = p;
            return;
        }

        if p.color == BLOODRED {
            VectorSet(&mut color, 0.22, 0.0, 0.0);
        } else if p.color == GREY75 {
            let mut len = Distance(snapOrigin, org);
            if len == 0.0 {
                len = 1.0;
            }

            let val = 4096.0 / len;
            let mut greyit = (0.25 * val as f64) as f32;
            if greyit > 0.5 {
                greyit = 0.5;
            }

            VectorSet(&mut color, greyit, greyit, greyit);
        } else {
            VectorSet(&mut color, 1.0, 1.0, 1.0);
        }

        let time = (cgTime as f32) - p.time;
        let time2 = p.endtime - p.time;
        let ratio = time / time2;

        let mut invratio: f32;
        if (cgTime as f32) > p.startfade {
            invratio = 1.0 - (((cgTime as f32) - p.startfade) / (p.endtime - p.startfade));

            if p.color == EMISIVEFADE {
                let mut fval = invratio * invratio;
                if fval < 0.0 {
                    fval = 0.0;
                }
                VectorSet(&mut color, fval, fval, fval);
            }
            invratio *= p.alpha;
        } else {
            invratio = 1.0 * p.alpha;
        }

        if invratio > 1.0 {
            invratio = 1.0;
        }

        let width = p.width + (ratio * (p.endwidth - p.width));
        let height = p.height + (ratio * (p.endheight - p.height));

        if p.r#type != P_SMOKE_IMPACT {
            let mut temp: vec3_t = [0.0; 3];

            vectoangles(rforward, &mut temp);
            p.accumroll += p.roll;
            temp[ROLL] = (temp[ROLL] as f64 + p.accumroll as f64 * 0.1) as f32;
            AngleVectors(temp, None, Some(&mut rright2), Some(&mut rup2));
        } else {
            _VectorCopy(rright, &mut rright2);
            _VectorCopy(rup, &mut rup2);
        }

        if p.rotate {
            _VectorMA(org, -height, rup2, &mut point);
            _VectorMA(point, -width, rright2, &mut point);
        } else {
            _VectorMA(org, -p.height, pvup, &mut point);
            _VectorMA(point, -p.width, pvright, &mut point);
        }
        _VectorCopy(point, &mut verts[0].xyz);
        verts[0].st[0] = 0.0;
        verts[0].st[1] = 0.0;
        verts[0].modulate[0] = (255.0 * color[0]) as i32 as u8;
        verts[0].modulate[1] = (255.0 * color[1]) as i32 as u8;
        verts[0].modulate[2] = (255.0 * color[2]) as i32 as u8;
        verts[0].modulate[3] = (255.0 * invratio) as i32 as u8;

        if p.rotate {
            _VectorMA(org, -height, rup2, &mut point);
            _VectorMA(point, width, rright2, &mut point);
        } else {
            _VectorMA(org, -p.height, pvup, &mut point);
            _VectorMA(point, p.width, pvright, &mut point);
        }
        _VectorCopy(point, &mut verts[1].xyz);
        verts[1].st[0] = 0.0;
        verts[1].st[1] = 1.0;
        verts[1].modulate[0] = (255.0 * color[0]) as i32 as u8;
        verts[1].modulate[1] = (255.0 * color[1]) as i32 as u8;
        verts[1].modulate[2] = (255.0 * color[2]) as i32 as u8;
        verts[1].modulate[3] = (255.0 * invratio) as i32 as u8;

        if p.rotate {
            _VectorMA(org, height, rup2, &mut point);
            _VectorMA(point, width, rright2, &mut point);
        } else {
            _VectorMA(org, p.height, pvup, &mut point);
            _VectorMA(point, p.width, pvright, &mut point);
        }
        _VectorCopy(point, &mut verts[2].xyz);
        verts[2].st[0] = 1.0;
        verts[2].st[1] = 1.0;
        verts[2].modulate[0] = (255.0 * color[0]) as i32 as u8;
        verts[2].modulate[1] = (255.0 * color[1]) as i32 as u8;
        verts[2].modulate[2] = (255.0 * color[2]) as i32 as u8;
        verts[2].modulate[3] = (255.0 * invratio) as i32 as u8;

        if p.rotate {
            _VectorMA(org, height, rup2, &mut point);
            _VectorMA(point, -width, rright2, &mut point);
        } else {
            _VectorMA(org, p.height, pvup, &mut point);
            _VectorMA(point, -p.width, pvright, &mut point);
        }
        _VectorCopy(point, &mut verts[3].xyz);
        verts[3].st[0] = 1.0;
        verts[3].st[1] = 0.0;
        verts[3].modulate[0] = (255.0 * color[0]) as i32 as u8;
        verts[3].modulate[1] = (255.0 * color[1]) as i32 as u8;
        verts[3].modulate[2] = (255.0 * color[2]) as i32 as u8;
        verts[3].modulate[3] = (255.0 * invratio) as i32 as u8;
    } else if p.r#type == P_BLEED {
        let mut rr: vec3_t = [0.0; 3];
        let mut ru: vec3_t = [0.0; 3];
        let mut rotate_ang: vec3_t = [0.0; 3];

        let alpha = p.alpha;

        if p.roll != 0 {
            vectoangles(viewaxis0, &mut rotate_ang);
            rotate_ang[ROLL] += p.roll as f32;
            AngleVectors(rotate_ang, None, Some(&mut rr), Some(&mut ru));
        } else {
            _VectorCopy(pvup, &mut ru);
            _VectorCopy(pvright, &mut rr);
        }

        _VectorMA(org, -p.height, ru, &mut point);
        _VectorMA(point, -p.width, rr, &mut point);
        _VectorCopy(point, &mut verts[0].xyz);
        verts[0].st[0] = 0.0;
        verts[0].st[1] = 0.0;
        verts[0].modulate[0] = 111;
        verts[0].modulate[1] = 19;
        verts[0].modulate[2] = 9;
        verts[0].modulate[3] = (255.0 * alpha) as i32 as u8;

        _VectorMA(org, -p.height, ru, &mut point);
        _VectorMA(point, p.width, rr, &mut point);
        _VectorCopy(point, &mut verts[1].xyz);
        verts[1].st[0] = 0.0;
        verts[1].st[1] = 1.0;
        verts[1].modulate[0] = 111;
        verts[1].modulate[1] = 19;
        verts[1].modulate[2] = 9;
        verts[1].modulate[3] = (255.0 * alpha) as i32 as u8;

        _VectorMA(org, p.height, ru, &mut point);
        _VectorMA(point, p.width, rr, &mut point);
        _VectorCopy(point, &mut verts[2].xyz);
        verts[2].st[0] = 1.0;
        verts[2].st[1] = 1.0;
        verts[2].modulate[0] = 111;
        verts[2].modulate[1] = 19;
        verts[2].modulate[2] = 9;
        verts[2].modulate[3] = (255.0 * alpha) as i32 as u8;

        _VectorMA(org, p.height, ru, &mut point);
        _VectorMA(point, -p.width, rr, &mut point);
        _VectorCopy(point, &mut verts[3].xyz);
        verts[3].st[0] = 1.0;
        verts[3].st[1] = 0.0;
        verts[3].modulate[0] = 111;
        verts[3].modulate[1] = 19;
        verts[3].modulate[2] = 9;
        verts[3].modulate[3] = (255.0 * alpha) as i32 as u8;
    } else if p.r#type == P_FLAT_SCALEUP {
        let mut color: vec3_t = [0.0; 3];

        if p.color == BLOODRED {
            VectorSet(&mut color, 1.0, 1.0, 1.0);
        } else {
            VectorSet(&mut color, 0.5, 0.5, 0.5);
        }

        let time = (cgTime as f32) - p.time;
        let time2 = p.endtime - p.time;
        let ratio = time / time2;

        let mut width = p.width + (ratio * (p.endwidth - p.width));
        let mut height = p.height + (ratio * (p.endheight - p.height));

        if width > p.endwidth {
            width = p.endwidth;
        }

        if height > p.endheight {
            height = p.endheight;
        }

        let rad = (p.roll as f64 * std::f64::consts::PI) / 180.0;
        let sinR = (height as f64 * rad.sin() * (2.0f64).sqrt()) as f32;
        let cosR = (width as f64 * rad.cos() * (2.0f64).sqrt()) as f32;

        _VectorCopy(org, &mut verts[0].xyz);
        verts[0].xyz[0] -= sinR;
        verts[0].xyz[1] -= cosR;
        verts[0].st[0] = 0.0;
        verts[0].st[1] = 0.0;
        verts[0].modulate[0] = (255.0 * color[0]) as i32 as u8;
        verts[0].modulate[1] = (255.0 * color[1]) as i32 as u8;
        verts[0].modulate[2] = (255.0 * color[2]) as i32 as u8;
        verts[0].modulate[3] = 255;

        _VectorCopy(org, &mut verts[1].xyz);
        verts[1].xyz[0] -= cosR;
        verts[1].xyz[1] += sinR;
        verts[1].st[0] = 0.0;
        verts[1].st[1] = 1.0;
        verts[1].modulate[0] = (255.0 * color[0]) as i32 as u8;
        verts[1].modulate[1] = (255.0 * color[1]) as i32 as u8;
        verts[1].modulate[2] = (255.0 * color[2]) as i32 as u8;
        verts[1].modulate[3] = 255;

        _VectorCopy(org, &mut verts[2].xyz);
        verts[2].xyz[0] += sinR;
        verts[2].xyz[1] += cosR;
        verts[2].st[0] = 1.0;
        verts[2].st[1] = 1.0;
        verts[2].modulate[0] = (255.0 * color[0]) as i32 as u8;
        verts[2].modulate[1] = (255.0 * color[1]) as i32 as u8;
        verts[2].modulate[2] = (255.0 * color[2]) as i32 as u8;
        verts[2].modulate[3] = 255;

        _VectorCopy(org, &mut verts[3].xyz);
        verts[3].xyz[0] += cosR;
        verts[3].xyz[1] -= sinR;
        verts[3].st[0] = 1.0;
        verts[3].st[1] = 0.0;
        verts[3].modulate[0] = (255.0 * color[0]) as i32 as u8;
        verts[3].modulate[1] = (255.0 * color[1]) as i32 as u8;
        verts[3].modulate[2] = (255.0 * color[2]) as i32 as u8;
        verts[3].modulate[3] = 255;
    } else if p.r#type == P_FLAT {
        _VectorCopy(org, &mut verts[0].xyz);
        verts[0].xyz[0] -= p.height;
        verts[0].xyz[1] -= p.width;
        verts[0].st[0] = 0.0;
        verts[0].st[1] = 0.0;
        verts[0].modulate[0] = 255;
        verts[0].modulate[1] = 255;
        verts[0].modulate[2] = 255;
        verts[0].modulate[3] = 255;

        _VectorCopy(org, &mut verts[1].xyz);
        verts[1].xyz[0] -= p.height;
        verts[1].xyz[1] += p.width;
        verts[1].st[0] = 0.0;
        verts[1].st[1] = 1.0;
        verts[1].modulate[0] = 255;
        verts[1].modulate[1] = 255;
        verts[1].modulate[2] = 255;
        verts[1].modulate[3] = 255;

        _VectorCopy(org, &mut verts[2].xyz);
        verts[2].xyz[0] += p.height;
        verts[2].xyz[1] += p.width;
        verts[2].st[0] = 1.0;
        verts[2].st[1] = 1.0;
        verts[2].modulate[0] = 255;
        verts[2].modulate[1] = 255;
        verts[2].modulate[2] = 255;
        verts[2].modulate[3] = 255;

        _VectorCopy(org, &mut verts[3].xyz);
        verts[3].xyz[0] += p.height;
        verts[3].xyz[1] -= p.width;
        verts[3].st[0] = 1.0;
        verts[3].st[1] = 0.0;
        verts[3].modulate[0] = 255;
        verts[3].modulate[1] = 255;
        verts[3].modulate[2] = 255;
        verts[3].modulate[3] = 255;
    }
    // Ridah
    else if p.r#type == P_ANIM {
        let mut rr: vec3_t = [0.0; 3];
        let mut ru: vec3_t = [0.0; 3];
        let mut rotate_ang: vec3_t = [0.0; 3];

        let time = (cgTime as f32) - p.time;
        let time2 = p.endtime - p.time;
        let mut ratio = time / time2;
        if ratio >= 1.0 {
            ratio = 0.9999;
        }

        let width = p.width + (ratio * (p.endwidth - p.width));
        let height = p.height + (ratio * (p.endheight - p.height));

        // if we are "inside" this sprite, don't draw
        if (Distance(snapOrigin, org) as f64) < (width as f64 / 1.5) {
            ctx.world.marks.particles[pnum] = p;
            return;
        }

        let i = p.shaderAnim;
        let j = ((ratio * shaderAnimCounts[p.shaderAnim as usize] as f32) as f64).floor() as c_int;
        p.pshader = ctx.world.marks.shaderAnims[i as usize][j as usize];

        if p.roll != 0 {
            vectoangles(viewaxis0, &mut rotate_ang);
            rotate_ang[ROLL] += p.roll as f32;
            AngleVectors(rotate_ang, None, Some(&mut rr), Some(&mut ru));
        }

        if p.roll != 0 {
            _VectorMA(org, -height, ru, &mut point);
            _VectorMA(point, -width, rr, &mut point);
        } else {
            _VectorMA(org, -height, pvup, &mut point);
            _VectorMA(point, -width, pvright, &mut point);
        }
        _VectorCopy(point, &mut verts[0].xyz);
        verts[0].st[0] = 0.0;
        verts[0].st[1] = 0.0;
        verts[0].modulate[0] = 255;
        verts[0].modulate[1] = 255;
        verts[0].modulate[2] = 255;
        verts[0].modulate[3] = 255;

        if p.roll != 0 {
            _VectorMA(point, 2.0 * height, ru, &mut point);
        } else {
            _VectorMA(point, 2.0 * height, pvup, &mut point);
        }
        _VectorCopy(point, &mut verts[1].xyz);
        verts[1].st[0] = 0.0;
        verts[1].st[1] = 1.0;
        verts[1].modulate[0] = 255;
        verts[1].modulate[1] = 255;
        verts[1].modulate[2] = 255;
        verts[1].modulate[3] = 255;

        if p.roll != 0 {
            _VectorMA(point, 2.0 * width, rr, &mut point);
        } else {
            _VectorMA(point, 2.0 * width, pvright, &mut point);
        }
        _VectorCopy(point, &mut verts[2].xyz);
        verts[2].st[0] = 1.0;
        verts[2].st[1] = 1.0;
        verts[2].modulate[0] = 255;
        verts[2].modulate[1] = 255;
        verts[2].modulate[2] = 255;
        verts[2].modulate[3] = 255;

        if p.roll != 0 {
            _VectorMA(point, -2.0 * height, ru, &mut point);
        } else {
            _VectorMA(point, -2.0 * height, pvup, &mut point);
        }
        _VectorCopy(point, &mut verts[3].xyz);
        verts[3].st[0] = 1.0;
        verts[3].st[1] = 0.0;
        verts[3].modulate[0] = 255;
        verts[3].modulate[1] = 255;
        verts[3].modulate[2] = 255;
        verts[3].modulate[3] = 255;
    }
    // done.

    ctx.world.marks.particles[pnum] = p;

    if p.pshader == 0 {
        // (SA) temp commented out for DM
        // CG_Printf ("CG_AddParticleToScene type %d p->pshader == ZERO\n", p->type);
        return;
    }

    if p.r#type == P_WEATHER || p.r#type == P_WEATHER_TURBULENT || p.r#type == P_WEATHER_FLURRY {
        trap::R_AddPolyToScene(engine, p.pshader, &TRIverts);
    } else {
        trap::R_AddPolyToScene(engine, p.pshader, &verts);
    }
}

/// Raven `CG_ParticleBulletDebris` — one short-lived falling smoke fleck, for
/// bullet impacts.
///
/// Source: `oracle/codemp/cgame/cg_marks.c:1449-1488`
pub fn CG_ParticleBulletDebris(world: &mut CgWorld, org: vec3_t, vel: vec3_t, duration: c_int) {
    let cgTime = world.cg.time;

    let Some(pnum) = world.marks.free_particles else {
        return;
    };
    world.marks.free_particles = world.marks.particles[pnum].next;
    world.marks.particles[pnum].next = world.marks.active_particles;
    world.marks.active_particles = Some(pnum);

    let p = &mut world.marks.particles[pnum];
    p.time = cgTime as f32;

    p.endtime = (cgTime + duration) as f32;
    p.startfade = (cgTime + duration / 2) as f32;

    p.color = EMISIVEFADE;
    p.alpha = 1.0;
    p.alphavel = 0.0;

    p.height = 0.5;
    p.width = 0.5;
    p.endheight = 0.5;
    p.endwidth = 0.5;

    p.pshader = 0; //cgs.media.tracerShader;

    p.r#type = P_SMOKE;

    _VectorCopy(org, &mut p.org);

    p.vel[0] = vel[0];
    p.vel[1] = vel[1];
    p.vel[2] = vel[2];
    p.accel[0] = 0.0;
    p.accel[1] = 0.0;
    p.accel[2] = 0.0;

    p.accel[2] = -60.0;
    p.vel[2] += -20.0;
}

/// Raven `CG_AddParticleShrapnel` — a no-op; Raven's body is a bare `return`.
///
/// Source: `oracle/codemp/cgame/cg_marks.c:1550-1553`
pub fn CG_AddParticleShrapnel(_le: EffectHandle) {}

/// Raven `CG_SnowLink` — turns a weather emitter's flakes on or off.
///
/// The emitter entity's `currentState.frame` is the snow-system id the
/// particles carry in `snum`.
///
/// Source: `oracle/codemp/cgame/cg_marks.c:1629-1652`
pub fn CG_SnowLink(world: &mut CgWorld, centNum: usize, particleOn: bool) {
    let id = world.entity(centNum).currentState.frame;

    let mut walk = world.marks.active_particles;
    while let Some(pnum) = walk {
        let p = &mut world.marks.particles[pnum];
        walk = p.next;

        if p.r#type == P_WEATHER || p.r#type == P_WEATHER_TURBULENT {
            if p.snum == id {
                p.link = particleOn;
            }
        }
    }
}

/// Raven `CG_ParticleBloodCloud` — marches a line of big red smoke puffs along
/// `dir`, one per 32 units of its length.
///
/// PORT-NOTE: Raven marches `point` down the line but spawns every puff at
/// `origin`, so the whole cloud stacks on the impact point. Preserved.
///
/// Source: `oracle/codemp/cgame/cg_marks.c:2011-2086`
pub fn CG_ParticleBloodCloud(world: &mut CgWorld, _centNum: usize, origin: vec3_t, dir: vec3_t) {
    let mut angles: vec3_t = [0.0; 3];
    let mut forward: vec3_t = [0.0; 3];
    let mut point: vec3_t = [0.0; 3];

    let mut dist: f32 = 0.0;

    let length = VectorLength(dir);
    vectoangles(dir, &mut angles);
    AngleVectors(angles, Some(&mut forward), None, None);

    let crittersize = LARGESIZE as f32;

    if length != 0.0 {
        dist = length / crittersize;
    }

    if dist < 1.0 {
        dist = 1.0;
    }

    _VectorCopy(origin, &mut point);

    let cgTime = world.cg.time;
    let mut i = 0;
    while (i as f32) < dist {
        _VectorMA(point, crittersize, forward, &mut point);

        let Some(pnum) = world.marks.free_particles else {
            return;
        };

        world.marks.free_particles = world.marks.particles[pnum].next;
        world.marks.particles[pnum].next = world.marks.active_particles;
        world.marks.active_particles = Some(pnum);

        // the two draws, in Raven's order: `endtime`'s crandom, then `roll`'s rand
        let endfuzz = world.bg_state.rng.crandom();
        let rollrand = world.bg_state.rng.rand();

        let p = &mut world.marks.particles[pnum];

        p.time = cgTime as f32;
        p.alpha = 1.0;
        p.alphavel = 0.0;
        p.roll = 0;

        p.pshader = 0; //cgs.media.smokePuffShader;

        p.endtime = ((cgTime + 350) as f64 + endfuzz * 100.0) as f32;

        p.startfade = cgTime as f32;

        p.width = LARGESIZE as f32;
        p.height = LARGESIZE as f32;
        p.endheight = LARGESIZE as f32;
        p.endwidth = LARGESIZE as f32;

        p.r#type = P_SMOKE;

        _VectorCopy(origin, &mut p.org);

        p.vel[0] = 0.0;
        p.vel[1] = 0.0;
        p.vel[2] = -1.0;

        VectorClear(&mut p.accel);

        p.rotate = false;

        p.roll = rollrand % 179;

        p.color = BLOODRED;

        p.alpha = 0.75;

        i += 1;
    }
}

/// Raven `CG_ParticleSparks` — one emissive smoke fleck kicked upward, jittered
/// by `x`/`y` around `org`.
///
/// Source: `oracle/codemp/cgame/cg_marks.c:2088-2134`
pub fn CG_ParticleSparks(
    world: &mut CgWorld,
    org: vec3_t,
    vel: vec3_t,
    duration: c_int,
    x: f32,
    y: f32,
    speed: f32,
) {
    let cgTime = world.cg.time;

    let Some(pnum) = world.marks.free_particles else {
        return;
    };

    world.marks.free_particles = world.marks.particles[pnum].next;
    world.marks.particles[pnum].next = world.marks.active_particles;
    world.marks.active_particles = Some(pnum);

    // the draws, hoisted in Raven's order: org x/y, then vel x/y/z, then accel x/y
    let orgx = world.bg_state.rng.crandom();
    let orgy = world.bg_state.rng.crandom();
    let velx = world.bg_state.rng.crandom();
    let vely = world.bg_state.rng.crandom();
    let velz = world.bg_state.rng.crandom();
    let accelx = world.bg_state.rng.crandom();
    let accely = world.bg_state.rng.crandom();

    let p = &mut world.marks.particles[pnum];
    p.time = cgTime as f32;

    p.endtime = (cgTime + duration) as f32;
    p.startfade = (cgTime + duration / 2) as f32;

    p.color = EMISIVEFADE;
    p.alpha = 0.4;
    p.alphavel = 0.0;

    p.height = 0.5;
    p.width = 0.5;
    p.endheight = 0.5;
    p.endwidth = 0.5;

    p.pshader = 0; //cgs.media.tracerShader;

    p.r#type = P_SMOKE;

    _VectorCopy(org, &mut p.org);

    p.org[0] = (p.org[0] as f64 + orgx * x as f64) as f32;
    p.org[1] = (p.org[1] as f64 + orgy * y as f64) as f32;

    p.vel[0] = vel[0];
    p.vel[1] = vel[1];
    p.vel[2] = vel[2];

    p.accel[0] = 0.0;
    p.accel[1] = 0.0;
    p.accel[2] = 0.0;

    p.vel[0] = (p.vel[0] as f64 + velx * 4.0) as f32;
    p.vel[1] = (p.vel[1] as f64 + vely * 4.0) as f32;
    p.vel[2] = (p.vel[2] as f64 + (20.0 + velz * 10.0) * speed as f64) as f32;

    p.accel[0] = (accelx * 4.0) as f32;
    p.accel[1] = (accely * 4.0) as f32;
}

/// Raven `CG_ParticleDust` — the dust plume a landing or an impact kicks up,
/// one puff per 32 units of `dir`'s length.
///
/// PORT-NOTE: Raven negates the caller's `dir` in place, so the vector comes
/// back flipped — kept, callers depend on whatever they do with it next.
///
/// PORT-NOTE: the three `p->accel` writes are dead — `VectorClear` on the next
/// line wipes them. Kept as Raven has them.
///
/// Source: `oracle/codemp/cgame/cg_marks.c:2136-2230`
pub fn CG_ParticleDust(world: &mut CgWorld, _centNum: usize, origin: vec3_t, dir: &mut vec3_t) {
    let mut angles: vec3_t = [0.0; 3];
    let mut forward: vec3_t = [0.0; 3];
    let mut point: vec3_t = [0.0; 3];

    let mut dist: f32 = 0.0;

    // VectorNegate( dir, dir ) — no ported helper for the macro
    dir[0] = -dir[0];
    dir[1] = -dir[1];
    dir[2] = -dir[2];

    let length = VectorLength(*dir);
    vectoangles(*dir, &mut angles);
    AngleVectors(angles, Some(&mut forward), None, None);

    let crittersize = LARGESIZE as f32;

    if length != 0.0 {
        dist = length / crittersize;
    }

    if dist < 1.0 {
        dist = 1.0;
    }

    _VectorCopy(origin, &mut point);

    let cgTime = world.cg.time;
    let mut i = 0;
    while (i as f32) < dist {
        _VectorMA(point, crittersize, forward, &mut point);

        let Some(pnum) = world.marks.free_particles else {
            return;
        };

        world.marks.free_particles = world.marks.particles[pnum].next;
        world.marks.particles[pnum].next = world.marks.active_particles;
        world.marks.active_particles = Some(pnum);

        // the draws, in Raven's order: endtime, vel x/y/z, accel x/y, then roll
        let endfuzz = world.bg_state.rng.crandom();
        let velx = world.bg_state.rng.crandom();
        let vely = world.bg_state.rng.crandom();
        let velz = world.bg_state.rng.random();
        let accelx = world.bg_state.rng.crandom();
        let accely = world.bg_state.rng.crandom();
        let rollrand = world.bg_state.rng.rand();

        let p = &mut world.marks.particles[pnum];

        p.time = cgTime as f32;
        p.alpha = 5.0;
        p.alphavel = 0.0;
        p.roll = 0;

        p.pshader = 0; //cgs.media.smokePuffShader;

        // RF, stay around for long enough to expand and dissipate naturally
        if length != 0.0 {
            p.endtime = ((cgTime + 4500) as f64 + endfuzz * 3500.0) as f32;
        } else {
            p.endtime = ((cgTime + 750) as f64 + endfuzz * 500.0) as f32;
        }

        p.startfade = cgTime as f32;

        p.width = LARGESIZE as f32;
        p.height = LARGESIZE as f32;

        // RF, expand while falling
        p.endheight = LARGESIZE as f32 * 3.0;
        p.endwidth = LARGESIZE as f32 * 3.0;

        if length == 0.0 {
            p.width *= 0.2;
            p.height *= 0.2;

            p.endheight = NORMALSIZE as f32;
            p.endwidth = NORMALSIZE as f32;
        }

        p.r#type = P_SMOKE;

        _VectorCopy(point, &mut p.org);

        p.vel[0] = (velx * 6.0) as f32;
        p.vel[1] = (vely * 6.0) as f32;
        p.vel[2] = velz * 20.0;

        // RF, add some gravity/randomness
        p.accel[0] = (accelx * 3.0) as f32;
        p.accel[1] = (accely * 3.0) as f32;
        p.accel[2] = (-PARTICLE_GRAVITY as f64 * 0.4) as f32;

        VectorClear(&mut p.accel);

        p.rotate = false;

        p.roll = rollrand % 179;

        p.alpha = 0.75;

        i += 1;
    }
}

/// Raven `CG_FreeMarkPoly` — hands one mark back to the pool.
///
/// The doubly-linked unlink plus the free-list push IS
/// [`EffectPool::free`](crate::world::effect_pool::EffectPool::free) — the links
/// dissolved into the slab under DEC-46.3, and its `false` return is Raven's
/// `!le->prevMark` "not active" case.
///
/// PORT-NOTE: Raven's error text says `CG_FreeLocalEntity` — a copy-paste from
/// `cg_localents.c`. Kept as-is.
///
/// Source: `oracle/codemp/cgame/cg_marks.c:47-59`
pub fn CG_FreeMarkPoly(ctx: &mut CgContext, mark: EffectHandle) {
    if !ctx.world.cg_markPolys.free(mark) {
        CG_Error(ctx, "CG_FreeLocalEntity: not active");
    }
}

/// The "put this particle back and blank it" tail `CG_AddParticles` repeats at
/// each of its four expiry branches.
///
/// Source: `oracle/codemp/cgame/cg_marks.c:1119-1123`
fn free_particle(marks: &mut CgMarksState, pnum: usize) {
    marks.particles[pnum].next = marks.free_particles;
    marks.free_particles = Some(pnum);

    let p = &mut marks.particles[pnum];
    p.r#type = P_NONE;
    p.color = 0;
    p.alpha = 0.0;
}

/// Raven `CG_AddParticles` — ages every live particle, drops the expired ones
/// on the free list, and hands the survivors to [`CG_AddParticleToScene`].
///
/// Raven rebuilds the active list from scratch each frame through the local
/// `active`/`tail` pair, so a particle that expires simply never gets relinked.
///
/// PORT-NOTE: Raven's `color` and `type` locals are written and never read —
/// dropped rather than carried as dead bindings.
///
/// Source: `oracle/codemp/cgame/cg_marks.c:1081-1208`
pub fn CG_AddParticles(ctx: &mut CgContext) {
    let mut org: vec3_t = [0.0; 3];
    let mut rotate_ang: vec3_t = [0.0; 3];

    if !ctx.world.marks.initparticles {
        CG_ClearParticles(ctx.world);
    }

    let cgTime = ctx.world.cg.time;
    let viewaxis = ctx.world.cg.refdef.viewaxis;

    _VectorCopy(viewaxis[0], &mut ctx.world.marks.pvforward);
    _VectorCopy(viewaxis[1], &mut ctx.world.marks.pvright);
    _VectorCopy(viewaxis[2], &mut ctx.world.marks.pvup);

    vectoangles(viewaxis[0], &mut rotate_ang);
    ctx.world.marks.roll = (ctx.world.marks.roll as f64
        + (cgTime as f32 - ctx.world.marks.oldtime) as f64 * 0.1) as f32;
    rotate_ang[ROLL] = (rotate_ang[ROLL] as f64 + ctx.world.marks.roll as f64 * 0.9) as f32;
    AngleVectors(
        rotate_ang,
        Some(&mut ctx.world.marks.rforward),
        Some(&mut ctx.world.marks.rright),
        Some(&mut ctx.world.marks.rup),
    );

    ctx.world.marks.oldtime = cgTime as f32;

    let mut active: Option<usize> = None;
    let mut tail: Option<usize> = None;

    let mut walk = ctx.world.marks.active_particles;
    while let Some(pnum) = walk {
        let p = ctx.world.marks.particles[pnum];

        walk = p.next;

        let time = ((cgTime as f32 - p.time) as f64 * 0.001) as f32;

        let mut alpha = p.alpha + time * p.alphavel;
        if alpha <= 0.0 {
            // faded out
            free_particle(&mut ctx.world.marks, pnum);
            continue;
        }

        if p.r#type == P_SMOKE
            || p.r#type == P_ANIM
            || p.r#type == P_BLEED
            || p.r#type == P_SMOKE_IMPACT
        {
            if (cgTime as f32) > p.endtime {
                free_particle(&mut ctx.world.marks, pnum);
                continue;
            }
        }

        if p.r#type == P_WEATHER_FLURRY {
            if (cgTime as f32) > p.endtime {
                free_particle(&mut ctx.world.marks, pnum);
                continue;
            }
        }

        if p.r#type == P_FLAT_SCALEUP_FADE {
            if (cgTime as f32) > p.endtime {
                free_particle(&mut ctx.world.marks, pnum);
                continue;
            }
        }

        if (p.r#type == P_BAT || p.r#type == P_SPRITE) && p.endtime < 0.0 {
            // temporary sprite
            CG_AddParticleToScene(ctx, pnum, p.org, alpha);
            free_particle(&mut ctx.world.marks, pnum);
            continue;
        }

        ctx.world.marks.particles[pnum].next = None;
        match tail {
            None => {
                active = Some(pnum);
                tail = Some(pnum);
            }
            Some(t) => {
                ctx.world.marks.particles[t].next = Some(pnum);
                tail = Some(pnum);
            }
        }

        if alpha > 1.0 {
            alpha = 1.0;
        }

        let time2 = time * time;

        org[0] = p.org[0] + p.vel[0] * time + p.accel[0] * time2;
        org[1] = p.org[1] + p.vel[1] * time + p.accel[1] * time2;
        org[2] = p.org[2] + p.vel[2] * time + p.accel[2] * time2;

        CG_AddParticleToScene(ctx, pnum, org, alpha);
    }

    ctx.world.marks.active_particles = active;
}

/// Raven `CG_ParticleSnowFlurry` — one wind-blown flake from a weather emitter
/// entity, whose `currentState` carries the whole spawn recipe.
///
/// PORT-NOTE: `turb` is a local `qtrue` Raven never varies, so the `p->vel[2] =
/// -20` above it is dead and the two accel jitters always happen. Kept, and the
/// rng draws hoist in Raven's order on that basis.
///
/// PORT-NOTE: the three `p->org[n] = p->org[n]` self-assignments are no-ops —
/// dropped.
///
/// Source: `oracle/codemp/cgame/cg_marks.c:1215-1281`
pub fn CG_ParticleSnowFlurry(ctx: &mut CgContext, pshader: qhandle_t, centNum: usize) {
    let turb = true;

    if pshader == 0 {
        CG_Printf(ctx, "CG_ParticleSnowFlurry pshader == ZERO!\n");
    }

    let Some(pnum) = ctx.world.marks.free_particles else {
        return;
    };

    let cgTime = ctx.world.cg.time;
    let cs = ctx.world.entity(centNum).currentState;

    // the draws, in Raven's order: the size roll, then vel x/y, then accel x/y
    let sizeRoll = ctx.world.bg_state.rng.rand();
    let velx = ctx.world.bg_state.rng.crandom();
    let vely = ctx.world.bg_state.rng.crandom();
    let accelx = ctx.world.bg_state.rng.crandom();
    let accely = ctx.world.bg_state.rng.crandom();

    ctx.world.marks.free_particles = ctx.world.marks.particles[pnum].next;
    ctx.world.marks.particles[pnum].next = ctx.world.marks.active_particles;
    ctx.world.marks.active_particles = Some(pnum);

    let p = &mut ctx.world.marks.particles[pnum];
    p.time = cgTime as f32;
    p.color = 0;
    p.alpha = 0.90;
    p.alphavel = 0.0;

    p.start = cs.origin2[0];
    p.end = cs.origin2[1];

    p.endtime = (cgTime + cs.time) as f32;
    p.startfade = (cgTime + cs.time2) as f32;

    p.pshader = pshader;

    if sizeRoll % 100 > 90 {
        p.height = 32.0;
        p.width = 32.0;
        p.alpha = 0.10;
    } else {
        p.height = 1.0;
        p.width = 1.0;
    }

    p.vel[2] = -20.0;

    p.r#type = P_WEATHER_FLURRY;

    if turb {
        p.vel[2] = -10.0;
    }

    _VectorCopy(cs.origin, &mut p.org);

    p.vel[0] = 0.0;
    p.vel[1] = 0.0;

    p.accel[0] = 0.0;
    p.accel[1] = 0.0;
    p.accel[2] = 0.0;

    p.vel[0] = (p.vel[0] as f64 + (cs.angles[0] * 32.0) as f64 + velx * 16.0) as f32;
    p.vel[1] = (p.vel[1] as f64 + (cs.angles[1] * 32.0) as f64 + vely * 16.0) as f32;
    p.vel[2] += cs.angles[2];

    if turb {
        p.accel[0] = (accelx * 16.0) as f32;
        p.accel[1] = (accely * 16.0) as f32;
    }
}

/// Raven `CG_ParticleSnow` — one falling flake somewhere in the `range`-wide
/// column between `origin` and `origin2`.
///
/// `snum` is the emitter's snow-system id, which [`CG_SnowLink`] matches on.
///
/// Source: `oracle/codemp/cgame/cg_marks.c:1283-1338`
pub fn CG_ParticleSnow(
    ctx: &mut CgContext,
    pshader: qhandle_t,
    origin: vec3_t,
    origin2: vec3_t,
    turb: c_int,
    range: f32,
    snum: c_int,
) {
    if pshader == 0 {
        CG_Printf(ctx, "CG_ParticleSnow pshader == ZERO!\n");
    }

    let Some(pnum) = ctx.world.marks.free_particles else {
        return;
    };

    let cgTime = ctx.world.cg.time;

    // the draws, in Raven's order: org x/y/z, then the turbulent vel x/y
    let orgx = ctx.world.bg_state.rng.crandom();
    let orgy = ctx.world.bg_state.rng.crandom();
    let orgz = ctx.world.bg_state.rng.crandom();
    let (velx, vely) = if turb != 0 {
        (
            ctx.world.bg_state.rng.crandom(),
            ctx.world.bg_state.rng.crandom(),
        )
    } else {
        (0.0, 0.0)
    };

    ctx.world.marks.free_particles = ctx.world.marks.particles[pnum].next;
    ctx.world.marks.particles[pnum].next = ctx.world.marks.active_particles;
    ctx.world.marks.active_particles = Some(pnum);

    let p = &mut ctx.world.marks.particles[pnum];
    p.time = cgTime as f32;
    p.color = 0;
    p.alpha = 0.40;
    p.alphavel = 0.0;
    p.start = origin[2];
    p.end = origin2[2];
    p.pshader = pshader;
    p.height = 1.0;
    p.width = 1.0;

    p.vel[2] = -50.0;

    if turb != 0 {
        p.r#type = P_WEATHER_TURBULENT;
        p.vel[2] = (-50.0 * 1.3) as f32;
    } else {
        p.r#type = P_WEATHER;
    }

    _VectorCopy(origin, &mut p.org);

    p.org[0] = (p.org[0] as f64 + orgx * range as f64) as f32;
    p.org[1] = (p.org[1] as f64 + orgy * range as f64) as f32;
    p.org[2] = (p.org[2] as f64 + orgz * (p.start - p.end) as f64) as f32;

    p.vel[0] = 0.0;
    p.vel[1] = 0.0;

    p.accel[0] = 0.0;
    p.accel[1] = 0.0;
    p.accel[2] = 0.0;

    if turb != 0 {
        p.vel[0] = (velx * 16.0) as f32;
        p.vel[1] = (vely * 16.0) as f32;
    }

    // Rafael snow pvs check
    p.snum = snum;
    p.link = true;
}

/// Raven `CG_ParticleBubble` — [`CG_ParticleSnow`]'s underwater twin: it rises
/// instead of falling and gets a randomized size.
///
/// PORT-NOTE: Raven's zero-shader complaint says `CG_ParticleSnow`, a
/// copy-paste. Kept.
///
/// Source: `oracle/codemp/cgame/cg_marks.c:1340-1399`
pub fn CG_ParticleBubble(
    ctx: &mut CgContext,
    pshader: qhandle_t,
    origin: vec3_t,
    origin2: vec3_t,
    turb: c_int,
    range: f32,
    snum: c_int,
) {
    if pshader == 0 {
        CG_Printf(ctx, "CG_ParticleSnow pshader == ZERO!\n");
    }

    let Some(pnum) = ctx.world.marks.free_particles else {
        return;
    };

    let cgTime = ctx.world.cg.time;

    // the draws, in Raven's order: the size, vel z, org x/y/z, then the
    // turbulent vel x/y
    let sizefuzz = ctx.world.bg_state.rng.crandom();
    let velz = ctx.world.bg_state.rng.crandom();
    let orgx = ctx.world.bg_state.rng.crandom();
    let orgy = ctx.world.bg_state.rng.crandom();
    let orgz = ctx.world.bg_state.rng.crandom();
    let (velx, vely) = if turb != 0 {
        (
            ctx.world.bg_state.rng.crandom(),
            ctx.world.bg_state.rng.crandom(),
        )
    } else {
        (0.0, 0.0)
    };

    ctx.world.marks.free_particles = ctx.world.marks.particles[pnum].next;
    ctx.world.marks.particles[pnum].next = ctx.world.marks.active_particles;
    ctx.world.marks.active_particles = Some(pnum);

    let p = &mut ctx.world.marks.particles[pnum];
    p.time = cgTime as f32;
    p.color = 0;
    p.alpha = 0.40;
    p.alphavel = 0.0;
    p.start = origin[2];
    p.end = origin2[2];
    p.pshader = pshader;

    let randsize = (1.0 + (sizefuzz * 0.5)) as f32;

    p.height = randsize;
    p.width = randsize;

    p.vel[2] = (50.0 + (velz * 10.0)) as f32;

    if turb != 0 {
        p.r#type = P_BUBBLE_TURBULENT;
        p.vel[2] = (50.0 * 1.3) as f32;
    } else {
        p.r#type = P_BUBBLE;
    }

    _VectorCopy(origin, &mut p.org);

    p.org[0] = (p.org[0] as f64 + orgx * range as f64) as f32;
    p.org[1] = (p.org[1] as f64 + orgy * range as f64) as f32;
    p.org[2] = (p.org[2] as f64 + orgz * (p.start - p.end) as f64) as f32;

    p.vel[0] = 0.0;
    p.vel[1] = 0.0;

    p.accel[0] = 0.0;
    p.accel[1] = 0.0;
    p.accel[2] = 0.0;

    if turb != 0 {
        p.vel[0] = (velx * 4.0) as f32;
        p.vel[1] = (vely * 4.0) as f32;
    }

    // Rafael snow pvs check
    p.snum = snum;
    p.link = true;
}

/// Raven `CG_ParticleSmoke` — the rising smoke column a smoke-emitter entity
/// puffs out, one particle per call.
///
/// Raven: "using cent->density = enttime, cent->frame = startfade" — the
/// comment is stale, the times come off `currentState.time`/`time2` and `frame`
/// is the reverse-gravity flag.
///
/// Source: `oracle/codemp/cgame/cg_marks.c:1401-1446`
pub fn CG_ParticleSmoke(ctx: &mut CgContext, pshader: qhandle_t, centNum: usize) {
    if pshader == 0 {
        CG_Printf(ctx, "CG_ParticleSmoke == ZERO!\n");
    }

    let Some(pnum) = ctx.world.marks.free_particles else {
        return;
    };

    let cgTime = ctx.world.cg.time;
    let cs = ctx.world.entity(centNum).currentState;

    let rollfuzz = ctx.world.bg_state.rng.crandom();

    ctx.world.marks.free_particles = ctx.world.marks.particles[pnum].next;
    ctx.world.marks.particles[pnum].next = ctx.world.marks.active_particles;
    ctx.world.marks.active_particles = Some(pnum);

    let p = &mut ctx.world.marks.particles[pnum];
    p.time = cgTime as f32;

    p.endtime = (cgTime + cs.time) as f32;
    p.startfade = (cgTime + cs.time2) as f32;

    p.color = 0;
    p.alpha = 1.0;
    p.alphavel = 0.0;
    p.start = cs.origin[2];
    p.end = cs.origin2[2];
    p.pshader = pshader;
    p.rotate = false;
    p.height = 8.0;
    p.width = 8.0;
    p.endheight = 32.0;
    p.endwidth = 32.0;
    p.r#type = P_SMOKE;

    _VectorCopy(cs.origin, &mut p.org);

    p.vel[0] = 0.0;
    p.vel[1] = 0.0;
    p.accel[0] = 0.0;
    p.accel[1] = 0.0;
    p.accel[2] = 0.0;

    p.vel[2] = 5.0;

    // reverse gravity
    if cs.frame == 1 {
        p.vel[2] *= -1.0;
    }

    p.roll = (8.0 + (rollfuzz * 4.0)) as c_int;
}

/// Raven `CG_ParticleExplosion` — a shader-animated explosion sprite that
/// scales from `sizeStart` to `sizeEnd` over `duration`.
///
/// A negative `duration` means "no random roll", and its magnitude is the real
/// lifetime.
///
/// PORT-NOTE: Raven's `animStr < (char *)10` guard catches a caller passing an
/// index where a string belongs — there is no Rust analog for it, so it is
/// dropped rather than faked.
///
/// Source: `oracle/codemp/cgame/cg_marks.c:1496-1547`
pub fn CG_ParticleExplosion(
    ctx: &mut CgContext,
    animStr: &str,
    origin: vec3_t,
    vel: vec3_t,
    duration: c_int,
    sizeStart: c_int,
    sizeEnd: c_int,
) {
    let mut duration = duration;

    // find the animation string
    let mut anim = 0usize;
    while let Some(name) = shaderAnimNames[anim] {
        if Q_stricmp(animStr, name) == 0 {
            break;
        }
        anim += 1;
    }
    if shaderAnimNames[anim].is_none() {
        CG_Error(
            ctx,
            &format!("CG_ParticleExplosion: unknown animation string: {animStr}\n"),
        );
        return;
    }

    let Some(pnum) = ctx.world.marks.free_particles else {
        return;
    };

    let cgTime = ctx.world.cg.time;
    // the roll draw only happens on the positive-duration arm, so it hoists
    // under the same test rather than unconditionally
    let rollfuzz = if duration < 0 {
        0.0
    } else {
        ctx.world.bg_state.rng.crandom()
    };

    ctx.world.marks.free_particles = ctx.world.marks.particles[pnum].next;
    ctx.world.marks.particles[pnum].next = ctx.world.marks.active_particles;
    ctx.world.marks.active_particles = Some(pnum);

    let p = &mut ctx.world.marks.particles[pnum];
    p.time = cgTime as f32;
    p.alpha = 0.5;
    p.alphavel = 0.0;

    if duration < 0 {
        duration *= -1;
        p.roll = 0;
    } else {
        p.roll = (rollfuzz * 179.0) as c_int;
    }

    p.shaderAnim = anim as c_int;

    // for sprites that are stretch in either direction
    p.width = sizeStart as f32;
    p.height = sizeStart as f32 * shaderAnimSTRatio[anim];

    p.endheight = sizeEnd as f32;
    p.endwidth = sizeEnd as f32 * shaderAnimSTRatio[anim];

    p.endtime = (cgTime + duration) as f32;

    p.r#type = P_ANIM;

    _VectorCopy(origin, &mut p.org);
    _VectorCopy(vel, &mut p.vel);
    VectorClear(&mut p.accel);
}

/// Raven `CG_ParticleImpactSmokePuff` — the little rotating puff an impact
/// leaves behind.
///
/// PORT-NOTE: Raven sets `endtime` twice, 1000 then 500 msec; the second wins.
/// Kept.
///
/// Source: `oracle/codemp/cgame/cg_marks.c:1654-1692`
pub fn CG_ParticleImpactSmokePuff(ctx: &mut CgContext, pshader: qhandle_t, origin: vec3_t) {
    if pshader == 0 {
        CG_Printf(ctx, "CG_ParticleImpactSmokePuff pshader == ZERO!\n");
    }

    let Some(pnum) = ctx.world.marks.free_particles else {
        return;
    };

    let cgTime = ctx.world.cg.time;

    // the draws, in Raven's order: the roll, then width, then height
    let rollfuzz = ctx.world.bg_state.rng.crandom();
    let widthRoll = ctx.world.bg_state.rng.rand();
    let heightRoll = ctx.world.bg_state.rng.rand();

    ctx.world.marks.free_particles = ctx.world.marks.particles[pnum].next;
    ctx.world.marks.particles[pnum].next = ctx.world.marks.active_particles;
    ctx.world.marks.active_particles = Some(pnum);

    let p = &mut ctx.world.marks.particles[pnum];
    p.time = cgTime as f32;
    p.alpha = 0.25;
    p.alphavel = 0.0;
    p.roll = (rollfuzz * 179.0) as c_int;

    p.pshader = pshader;

    p.endtime = (cgTime + 1000) as f32;
    p.startfade = (cgTime + 100) as f32;

    p.width = (widthRoll % 4 + 8) as f32;
    p.height = (heightRoll % 4 + 8) as f32;

    p.endheight = p.height * 2.0;
    p.endwidth = p.width * 2.0;

    p.endtime = (cgTime + 500) as f32;

    p.r#type = P_SMOKE_IMPACT;

    _VectorCopy(origin, &mut p.org);
    VectorSet(&mut p.vel, 0.0, 0.0, 20.0);
    VectorSet(&mut p.accel, 0.0, 0.0, 20.0);

    p.rotate = true;
}

/// Raven `CG_Particle_Bleed` — the dark red smoke fleck a wound sheds.
///
/// A nonzero `fleshEntityNum` means the blood starts fading immediately rather
/// than after 100 msec.
///
/// PORT-NOTE: `dir` is unused, `p->roll` is written twice (0, then the random
/// one), and the type is `P_SMOKE` rather than `P_BLEED` — all Raven's, kept.
///
/// Source: `oracle/codemp/cgame/cg_marks.c:1694-1742`
pub fn CG_Particle_Bleed(
    ctx: &mut CgContext,
    pshader: qhandle_t,
    start: vec3_t,
    _dir: vec3_t,
    fleshEntityNum: c_int,
    duration: c_int,
) {
    if pshader == 0 {
        CG_Printf(ctx, "CG_Particle_Bleed pshader == ZERO!\n");
    }

    let Some(pnum) = ctx.world.marks.free_particles else {
        return;
    };

    let cgTime = ctx.world.cg.time;

    // the draws, in Raven's order: the end size, then the roll
    let sizeRoll = ctx.world.bg_state.rng.rand();
    let rollRoll = ctx.world.bg_state.rng.rand();

    ctx.world.marks.free_particles = ctx.world.marks.particles[pnum].next;
    ctx.world.marks.particles[pnum].next = ctx.world.marks.active_particles;
    ctx.world.marks.active_particles = Some(pnum);

    let p = &mut ctx.world.marks.particles[pnum];
    p.time = cgTime as f32;
    p.alpha = 1.0;
    p.alphavel = 0.0;
    p.roll = 0;

    p.pshader = pshader;

    p.endtime = (cgTime + duration) as f32;

    if fleshEntityNum != 0 {
        p.startfade = cgTime as f32;
    } else {
        p.startfade = (cgTime + 100) as f32;
    }

    p.width = 4.0;
    p.height = 4.0;

    p.endheight = (4 + sizeRoll % 3) as f32;
    p.endwidth = p.endheight;

    p.r#type = P_SMOKE;

    _VectorCopy(start, &mut p.org);
    p.vel[0] = 0.0;
    p.vel[1] = 0.0;
    p.vel[2] = -20.0;
    VectorClear(&mut p.accel);

    p.rotate = false;

    p.roll = rollRoll % 179;

    p.color = BLOODRED;
    p.alpha = 0.75;
}

/// Raven `CG_Particle_OilParticle` — a drip flung off an oil-spraying entity,
/// its speed falling off as the emitter ages.
///
/// PORT-NOTE: `p->snum` is an int and Raven assigns the float literal `1.0f` to
/// it, so the tag is 1 — the same tag [`CG_OilSlickRemove`] matches on. Kept.
///
/// Source: `oracle/codemp/cgame/cg_marks.c:1744-1805`
pub fn CG_Particle_OilParticle(ctx: &mut CgContext, pshader: qhandle_t, centNum: usize) {
    let duration: f32 = 1500.0;

    let cgTime = ctx.world.cg.time;
    let cs = ctx.world.entity(centNum).currentState;

    let time = cgTime;
    let time2 = cgTime + cs.time;

    let ratio = 1.0f32 - (time as f32 / time2 as f32);

    if pshader == 0 {
        CG_Printf(ctx, "CG_Particle_OilParticle == ZERO!\n");
    }

    let Some(pnum) = ctx.world.marks.free_particles else {
        return;
    };

    let rollRoll = ctx.world.bg_state.rng.rand();

    ctx.world.marks.free_particles = ctx.world.marks.particles[pnum].next;
    ctx.world.marks.particles[pnum].next = ctx.world.marks.active_particles;
    ctx.world.marks.active_particles = Some(pnum);

    let p = &mut ctx.world.marks.particles[pnum];
    p.time = cgTime as f32;
    p.alpha = 1.0;
    p.alphavel = 0.0;
    p.roll = 0;

    p.pshader = pshader;

    p.endtime = cgTime as f32 + duration;

    p.startfade = p.endtime;

    p.width = 1.0;
    p.height = 3.0;

    p.endheight = 3.0;
    p.endwidth = 1.0;

    p.r#type = P_SMOKE;

    _VectorCopy(cs.origin, &mut p.org);

    p.vel[0] = cs.origin2[0] * (16.0 * ratio);
    p.vel[1] = cs.origin2[1] * (16.0 * ratio);
    p.vel[2] = cs.origin2[2];

    p.snum = 1;

    VectorClear(&mut p.accel);

    p.accel[2] = -20.0;

    p.rotate = false;

    p.roll = rollRoll % 179;

    p.alpha = 0.75;
}

/// Raven `CG_Particle_OilSlick` — the flat pool an oil leak leaves on the
/// ground, sized and timed off the emitter's `angles2`.
///
/// PORT-NOTE: `p->snum` is an int taking the float literal `1.0`, so the tag is
/// 1 — [`CG_OilSlickRemove`]'s match. Kept.
///
/// Source: `oracle/codemp/cgame/cg_marks.c:1808-1872`
pub fn CG_Particle_OilSlick(ctx: &mut CgContext, pshader: qhandle_t, centNum: usize) {
    if pshader == 0 {
        CG_Printf(ctx, "CG_Particle_OilSlick == ZERO!\n");
    }

    let Some(pnum) = ctx.world.marks.free_particles else {
        return;
    };

    let cgTime = ctx.world.cg.time;
    let cs = ctx.world.entity(centNum).currentState;

    // the draws, in Raven's order: the org z lift, then the roll
    let liftfuzz = ctx.world.bg_state.rng.crandom();
    let rollRoll = ctx.world.bg_state.rng.rand();

    ctx.world.marks.free_particles = ctx.world.marks.particles[pnum].next;
    ctx.world.marks.particles[pnum].next = ctx.world.marks.active_particles;
    ctx.world.marks.active_particles = Some(pnum);

    let p = &mut ctx.world.marks.particles[pnum];
    p.time = cgTime as f32;

    if cs.angles2[2] != 0.0 {
        p.endtime = cgTime as f32 + cs.angles2[2];
    } else {
        p.endtime = (cgTime + 60000) as f32;
    }

    p.startfade = p.endtime;

    p.alpha = 1.0;
    p.alphavel = 0.0;
    p.roll = 0;

    p.pshader = pshader;

    if cs.angles2[0] != 0.0 || cs.angles2[1] != 0.0 {
        p.width = cs.angles2[0];
        p.height = cs.angles2[0];

        p.endheight = cs.angles2[1];
        p.endwidth = cs.angles2[1];
    } else {
        p.width = 8.0;
        p.height = 8.0;

        p.endheight = 16.0;
        p.endwidth = 16.0;
    }

    p.r#type = P_FLAT_SCALEUP;

    p.snum = 1;

    _VectorCopy(cs.origin, &mut p.org);

    p.org[2] = (p.org[2] as f64 + 0.55 + (liftfuzz * 0.5)) as f32;

    p.vel[0] = 0.0;
    p.vel[1] = 0.0;
    p.vel[2] = 0.0;
    VectorClear(&mut p.accel);

    p.rotate = false;

    p.roll = rollRoll % 179;

    p.alpha = 0.75;
}

/// Raven `CG_OilSlickRemove` — starts every oil slick fading out.
///
/// PORT-NOTE: `cent` is unused and `id` is the constant 1 (Raven writes the
/// float literal `1.0f` into an int), so this matches every `P_FLAT_SCALEUP`
/// particle and the `!id` complaint below can never fire. Kept.
///
/// Source: `oracle/codemp/cgame/cg_marks.c:1874-1900`
pub fn CG_OilSlickRemove(ctx: &mut CgContext, _centNum: usize) {
    let id: c_int = 1;

    if id == 0 {
        CG_Printf(ctx, "CG_OilSlickRevove NULL id\n");
    }

    let cgTime = ctx.world.cg.time;

    let mut walk = ctx.world.marks.active_particles;
    while let Some(pnum) = walk {
        let p = &mut ctx.world.marks.particles[pnum];
        walk = p.next;

        if p.r#type == P_FLAT_SCALEUP && p.snum == id {
            p.endtime = (cgTime + 100) as f32;
            p.startfade = p.endtime;
            p.r#type = P_FLAT_SCALEUP_FADE;
        }
    }
}

/// Raven `CG_ParticleMisc` — the generic one-off sprite: a square of `size`
/// that neither grows nor shrinks.
///
/// A `duration` of zero or less lands in `endtime` raw, which is how the
/// negative-endtime "temporary sprite" path in [`CG_AddParticles`] gets fed.
///
/// PORT-NOTE: the `alpha` parameter is dead — Raven hardcodes `p->alpha = 1.0`
/// over it — and the zero-shader complaint names
/// `CG_ParticleImpactSmokePuff`. Both kept.
///
/// Source: `oracle/codemp/cgame/cg_marks.c:2232-2271`
pub fn CG_ParticleMisc(
    ctx: &mut CgContext,
    pshader: qhandle_t,
    origin: vec3_t,
    size: c_int,
    duration: c_int,
    _alpha: f32,
) {
    if pshader == 0 {
        CG_Printf(ctx, "CG_ParticleImpactSmokePuff pshader == ZERO!\n");
    }

    let Some(pnum) = ctx.world.marks.free_particles else {
        return;
    };

    let cgTime = ctx.world.cg.time;
    let rollRoll = ctx.world.bg_state.rng.rand();

    ctx.world.marks.free_particles = ctx.world.marks.particles[pnum].next;
    ctx.world.marks.particles[pnum].next = ctx.world.marks.active_particles;
    ctx.world.marks.active_particles = Some(pnum);

    let p = &mut ctx.world.marks.particles[pnum];
    p.time = cgTime as f32;
    p.alpha = 1.0;
    p.alphavel = 0.0;
    p.roll = rollRoll % 179;

    p.pshader = pshader;

    if duration > 0 {
        p.endtime = (cgTime + duration) as f32;
    } else {
        p.endtime = duration as f32;
    }

    p.startfade = cgTime as f32;

    p.width = size as f32;
    p.height = size as f32;

    p.endheight = size as f32;
    p.endwidth = size as f32;

    p.r#type = P_SPRITE;

    _VectorCopy(origin, &mut p.org);

    p.rotate = false;
}
