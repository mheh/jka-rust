//! `CgMarksState` — `cg_marks.c`'s mutable file-scope globals as one `CgWorld`
//! sub-struct.

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use mp_qshared::shared::{qhandle_t, vec3_t};

use crate::cg_marks::{MAX_PARTICLES, MAX_SHADER_ANIMS, MAX_SHADER_ANIM_FRAMES};

/// Raven `particle_type_t` — which shape/behavior a particle draws as.
///
/// Raven declares `cparticle_t.type` as a plain `int` and only ever stores
/// these constants (or the `memset` zero, `P_NONE`) into it, so the field is
/// typed with the enum here.
/// Type definition source: `oracle/codemp/cgame/cg_marks.c:340-358`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum particle_type_t {
    #[default]
    P_NONE = 0,
    P_WEATHER = 1,
    P_FLAT = 2,
    P_SMOKE = 3,
    P_ROTATE = 4,
    P_WEATHER_TURBULENT = 5,
    /// Ridah
    P_ANIM = 6,
    P_BAT = 7,
    P_BLEED = 8,
    P_FLAT_SCALEUP = 9,
    P_FLAT_SCALEUP_FADE = 10,
    P_WEATHER_FLURRY = 11,
    P_SMOKE_IMPACT = 12,
    P_BUBBLE = 13,
    P_BUBBLE_TURBULENT = 14,
    P_SPRITE = 15,
}

/// Raven `cparticle_t` — one entry of the fixed 1024-particle pool.
///
/// Raven's `struct particle_s *next` is an index into
/// [`CgMarksState::particles`] instead of a pointer: the pool is a fixed array
/// that both the free list and the active list chain through, so the link is a
/// slot number (§B5). `None` is Raven's `NULL` terminator. This is not the
/// DEC-46.3 slab — the particle pool's alloc *fails* when the free list is
/// empty rather than stealing the oldest, so the two chains stay explicit.
///
/// The type is file-local to `cg_marks.c` and never crosses the engine seam, so
/// it carries no layout obligation.
/// Type definition source: `oracle/codemp/cgame/cg_marks.c:300-338`
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct cparticle_t {
    /// Raven `struct particle_s *next` — free-list or active-list successor.
    pub next: Option<usize>,

    pub time: f32,
    pub endtime: f32,

    pub org: vec3_t,
    pub vel: vec3_t,
    pub accel: vec3_t,
    pub color: c_int,
    pub colorvel: f32,
    pub alpha: f32,
    pub alphavel: f32,
    pub r#type: particle_type_t,
    pub pshader: qhandle_t,

    pub height: f32,
    pub width: f32,

    pub endheight: f32,
    pub endwidth: f32,

    pub start: f32,
    pub end: f32,

    pub startfade: f32,
    pub rotate: bool,
    pub snum: c_int,

    pub link: bool,

    // Ridah
    pub shaderAnim: c_int,
    pub roll: c_int,

    pub accumroll: c_int,
}

/// `cg_marks.c`'s mutable file-scope globals, grouped by owning `.c` file
/// (§B3: file-scope globals become owned state, they never become Rust
/// globals).
///
/// Fields fold in as the waves transcribe `cg_marks.c`'s file-scope statics
/// (DEC-46.1), so a wave transcriber only ever touches its own TU's two files
/// — the function file and this one — and never `cg_world.rs`. Raven's
/// read-only tables beside them are compiled-in data, not state; they land as
/// `const`s beside the functions that read them (§C8).
///
/// The mark pool itself is not here — it is `CgWorld.cg_markPolys`, the DEC-46.3
/// slab.
///
/// Source: `oracle/codemp/cgame/cg_marks.c:367,374,380-388`
#[derive(Debug, Clone)]
pub struct CgMarksState {
    /// Raven `cparticle_t particles[MAX_PARTICLES]` — the pool both lists chain
    /// through. Boxed: 1024 entries never transit the stack.
    /// Source: `oracle/codemp/cgame/cg_marks.c:381`
    pub particles: Box<[cparticle_t]>,

    /// Raven `cparticle_t *active_particles` — head of the live list, newest
    /// first (`CG_AddParticles` rebuilds it every frame).
    /// Source: `oracle/codemp/cgame/cg_marks.c:380`
    pub active_particles: Option<usize>,

    /// Raven `cparticle_t *free_particles` — head of the singly linked free
    /// list; `NULL`/`None` means every particle is in use and the spawners bail.
    /// Source: `oracle/codemp/cgame/cg_marks.c:380`
    pub free_particles: Option<usize>,

    /// Raven `int cl_numparticles = MAX_PARTICLES` — how much of `particles` the
    /// free list is built over. A global, not a `#define`, so it folds in here.
    /// Source: `oracle/codemp/cgame/cg_marks.c:382`
    pub cl_numparticles: c_int,

    /// Raven `qboolean initparticles` — set by `CG_ClearParticles`, checked by
    /// `CG_AddParticles` so the pool self-initializes on first use.
    /// Source: `oracle/codemp/cgame/cg_marks.c:384`
    pub initparticles: bool,

    /// Raven `vec3_t pvforward` — the view axis `CG_AddParticles` latches each
    /// frame for the front-facing particle quads.
    /// Source: `oracle/codemp/cgame/cg_marks.c:385`
    pub pvforward: vec3_t,

    /// Raven `vec3_t pvright`.
    /// Source: `oracle/codemp/cgame/cg_marks.c:385`
    pub pvright: vec3_t,

    /// Raven `vec3_t pvup`.
    /// Source: `oracle/codemp/cgame/cg_marks.c:385`
    pub pvup: vec3_t,

    /// Raven `vec3_t rforward` — the same axis rolled by the per-frame `roll`
    /// accumulator, for the rotating smoke quads.
    /// Source: `oracle/codemp/cgame/cg_marks.c:386`
    pub rforward: vec3_t,

    /// Raven `vec3_t rright`.
    /// Source: `oracle/codemp/cgame/cg_marks.c:386`
    pub rright: vec3_t,

    /// Raven `vec3_t rup`.
    /// Source: `oracle/codemp/cgame/cg_marks.c:386`
    pub rup: vec3_t,

    /// Raven `float oldtime` — `cg.time` at the last `CG_AddParticles`, the
    /// delta the roll accumulator advances on.
    /// Source: `oracle/codemp/cgame/cg_marks.c:388`
    pub oldtime: f32,

    /// Raven `static float roll` — the ever-growing roll `CG_AddParticles` adds
    /// to the view angles before building `rforward`/`rright`/`rup`. Declared
    /// down beside `CG_AddParticles` rather than up with the rest.
    /// Source: `oracle/codemp/cgame/cg_marks.c:1074`
    pub roll: f32,

    /// Raven `static qhandle_t shaderAnims[MAX_SHADER_ANIMS][MAX_SHADER_ANIM_FRAMES]`
    /// — the animated-shader frame table. Raven's registration loop in
    /// `CG_ClearParticles` is commented out, so this stays all-zero in the
    /// shipped build.
    /// Source: `oracle/codemp/cgame/cg_marks.c:367`
    pub shaderAnims: [[qhandle_t; MAX_SHADER_ANIM_FRAMES]; MAX_SHADER_ANIMS],

    /// Raven `static int numShaderAnims` — how many rows of `shaderAnims` were
    /// registered; zero in the shipped build, same reason.
    /// Source: `oracle/codemp/cgame/cg_marks.c:374`
    pub numShaderAnims: c_int,
}

impl Default for CgMarksState {
    /// Raven's BSS zero, plus `cl_numparticles`'s compile-time initializer.
    ///
    /// The pool is built straight into a heap slice — 1024 particles is far too
    /// much to hand back by value.
    fn default() -> Self {
        CgMarksState {
            particles: vec![cparticle_t::default(); MAX_PARTICLES].into_boxed_slice(),
            active_particles: None,
            free_particles: None,
            cl_numparticles: MAX_PARTICLES as c_int,
            initparticles: false,
            pvforward: [0.0; 3],
            pvright: [0.0; 3],
            pvup: [0.0; 3],
            rforward: [0.0; 3],
            rright: [0.0; 3],
            rup: [0.0; 3],
            oldtime: 0.0,
            roll: 0.0,
            shaderAnims: [[0; MAX_SHADER_ANIM_FRAMES]; MAX_SHADER_ANIMS],
            numShaderAnims: 0,
        }
    }
}
