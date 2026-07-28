//! Port of `oracle/codemp/cgame/cg_marks.c` — wall marks, their pool, and the shader-animation tables. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case, non_upper_case_globals)]

use core::ffi::c_int;

use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::shared::q_math::{
    _VectorCopy, _VectorMA, vectoangles, AngleVectors, Distance, VectorClear, VectorLength,
    VectorSet, ROLL,
};
use mp_qshared::shared::vec3_t;

use crate::trap;
use crate::world::cg_context::CgContext;
use crate::world::cg_marks_state::particle_type_t::{
    P_ANIM, P_BLEED, P_BUBBLE, P_BUBBLE_TURBULENT, P_FLAT, P_FLAT_SCALEUP, P_NONE, P_SMOKE,
    P_SMOKE_IMPACT, P_SPRITE, P_WEATHER, P_WEATHER_FLURRY, P_WEATHER_TURBULENT,
};
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
