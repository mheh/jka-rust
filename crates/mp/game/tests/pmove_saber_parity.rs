//! Pmove SABER-wielding differential parity test against the Raven oracle.
//!
//! Drives the ported `mp_bg::bg_pmove::Pmove` with `weapon = WP_SABER` over the
//! `fixtures/pmove_saber/` scenarios and the same synthetic `animation.cfg` as
//! the C dumper `tools/jampgame-oracle/main_pmove_saber.c`, and byte-compares to
//! the committed golden `golden/pmove_saber.txt`. It reproduces the melee slice's
//! world stub, RNG tripwire, anim mirror, fixture grammar, and dump format
//! (`tests/pmove_parity.rs`), extended with the saber attack/stance chain that
//! `PM_Weapon` dispatches to `PM_WeaponLightsaber` when the weapon is WP_SABER.
//!
//! `TestTraps` (BgTraps) is the world: an axial-brush trace/pointcontents +
//! `rintf` snap_vector — verbatim from `pmworld.h`. `TestCallbacks`
//! (GameCallbacks) panics on everything but the two anim restart-check reads,
//! served from the prior-frame anim mirror.
//!
//! Saber determinism (mirrored from `main_pmove_saber.c`): `g_entities`/the
//! `bgEntity_t` arena are zeroed, so `BG_MySaber` returns NULL on both sides —
//! no per-saber `saberInfo` data is read and every saber-object override path is
//! skipped identically, staying off the known xbox-residue divergence classes in
//! `oracle/discrepancies/bg_saber.md`. `bg_saber.c` makes no G2API/effect/sound
//! calls on the reachable path, and the only holdrand draw in the chain (the
//! saber-lock super-break) is unreachable here — so `rng` holds `89abcdef` in
//! every scenario. See `tools/jampgame-oracle/main_pmove_saber.c` for the full
//! provenance and the exact list of divergences from `main_pmove.c`.
#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use std::cell::Cell;
use std::fmt::Write as _;
use std::path::PathBuf;

use mp_bg::bg_misc::snap_vector;
use mp_bg::bg_panimate::BG_ParseAnimationFile;
use mp_bg::bg_pmove::Pmove;
use mp_game::bg_channel::{BgState, BgTraps, GameCallbacks};
use mp_game::prelude::*;
use testkit::{compare, oracle_dir};

fn fixture_dir() -> PathBuf {
    oracle_dir(env!("CARGO_MANIFEST_DIR")).join("fixtures/pmove_saber")
}

// ============================ the axial-brush world ============================
// Verbatim transcription of tools/jampgame-oracle/pmworld.h. Bit-identity rules:
// every literal is f32 (`0.125f32`), only + - * / and compares — no libm.

const PMW_SURFACE_CLIP_EPSILON: f32 = 0.125f32;

#[derive(Clone, Copy)]
struct Brush {
    mins: [f32; 3],
    maxs: [f32; 3],
    surface_flags: c_int,
}

fn zero_trace() -> trace_t {
    let mut tr = trace_t::zeroed();
    tr.fraction = 1.0f32;
    tr
}

// The six outward axial faces of one AABB: normal is exactly (0,±1); dist is the
// world plane distance so `normal · p == dist` on the face.
fn brush_plane(b: &Brush, face: usize) -> ([f32; 3], f32, usize) {
    let a = face >> 1;
    let positive = (face & 1) == 0;
    let mut normal = [0.0f32; 3];
    if positive {
        normal[a] = 1.0f32;
        (normal, b.maxs[a], a)
    } else {
        normal[a] = -1.0f32;
        (normal, -b.mins[a], a)
    }
}

// Q3 CM_ClipBoxToBrush for a single AABB brush; updates `tr` in place.
fn clip_box_to_brush(
    tr: &mut trace_t,
    start: &[f32; 3],
    end: &[f32; 3],
    tw_mins: &[f32; 3],
    tw_maxs: &[f32; 3],
    brush: &Brush,
) {
    let mut enter_frac = -1.0f32;
    let mut leave_frac = 1.0f32;
    let mut clip_normal = [0.0f32; 3];
    let mut clip_dist = 0.0f32;
    let mut clip_axis = 0usize;
    let mut getout = 0;
    let mut startout = 0;

    for face in 0..6 {
        let (normal, plane_dist, axis) = brush_plane(brush, face);

        let ofs = [
            if normal[0] < 0.0f32 {
                tw_maxs[0]
            } else {
                tw_mins[0]
            },
            if normal[1] < 0.0f32 {
                tw_maxs[1]
            } else {
                tw_mins[1]
            },
            if normal[2] < 0.0f32 {
                tw_maxs[2]
            } else {
                tw_mins[2]
            },
        ];

        let dist = plane_dist - (ofs[0] * normal[0] + ofs[1] * normal[1] + ofs[2] * normal[2]);

        let d1 = (start[0] * normal[0] + start[1] * normal[1] + start[2] * normal[2]) - dist;
        let d2 = (end[0] * normal[0] + end[1] * normal[1] + end[2] * normal[2]) - dist;

        if d2 > 0.0f32 {
            getout = 1;
        }
        if d1 > 0.0f32 {
            startout = 1;
        }

        if d1 > 0.0f32 && (d2 >= PMW_SURFACE_CLIP_EPSILON || d2 >= d1) {
            return;
        }
        if d1 <= 0.0f32 && d2 <= 0.0f32 {
            continue;
        }

        if d1 > d2 {
            // entering
            let mut f = (d1 - PMW_SURFACE_CLIP_EPSILON) / (d1 - d2);
            if f < 0.0f32 {
                f = 0.0f32;
            }
            if f > enter_frac {
                enter_frac = f;
                clip_normal = normal;
                clip_dist = plane_dist;
                clip_axis = axis;
            }
        } else {
            // leaving
            let mut f = (d1 + PMW_SURFACE_CLIP_EPSILON) / (d1 - d2);
            if f > 1.0f32 {
                f = 1.0f32;
            }
            if f < leave_frac {
                leave_frac = f;
            }
        }
    }

    if startout == 0 {
        tr.startsolid = 1;
        if getout == 0 {
            tr.allsolid = 1;
            tr.fraction = 0.0f32;
            tr.contents = CONTENTS_SOLID;
        }
        return;
    }

    if enter_frac < leave_frac && enter_frac > -1.0f32 && enter_frac < tr.fraction {
        if enter_frac < 0.0f32 {
            enter_frac = 0.0f32;
        }
        tr.fraction = enter_frac;
        tr.plane.normal = clip_normal;
        tr.plane.dist = clip_dist;
        tr.plane.r#type = clip_axis as u8;
        tr.plane.signbits = (if clip_normal[0] < 0.0f32 { 1 } else { 0 })
            | (if clip_normal[1] < 0.0f32 { 2 } else { 0 })
            | (if clip_normal[2] < 0.0f32 { 4 } else { 0 });
        tr.surfaceFlags = brush.surface_flags;
        tr.contents = CONTENTS_SOLID;
    }
}

// Sweep an AABB [mins,maxs] from start to end through all brushes.
fn trace_world(
    brushes: &[Brush],
    start: &[f32; 3],
    mins: &[f32; 3],
    maxs: &[f32; 3],
    end: &[f32; 3],
) -> trace_t {
    let mut tr = zero_trace();
    for b in brushes {
        clip_box_to_brush(&mut tr, start, end, mins, maxs, b);
        if tr.allsolid != 0 {
            break;
        }
    }
    if tr.allsolid != 0 {
        tr.startsolid = 1;
    }
    tr.endpos[0] = start[0] + tr.fraction * (end[0] - start[0]);
    tr.endpos[1] = start[1] + tr.fraction * (end[1] - start[1]);
    tr.endpos[2] = start[2] + tr.fraction * (end[2] - start[2]);
    if tr.fraction < 1.0f32 {
        tr.entityNum = ENTITYNUM_WORLD as i16;
        tr.contents = CONTENTS_SOLID;
    } else {
        tr.entityNum = ENTITYNUM_NONE as i16;
    }
    tr
}

fn point_in_brushes(brushes: &[Brush], p: &[f32; 3]) -> c_int {
    for b in brushes {
        if p[0] >= b.mins[0]
            && p[0] <= b.maxs[0]
            && p[1] >= b.mins[1]
            && p[1] <= b.maxs[1]
            && p[2] >= b.mins[2]
            && p[2] <= b.maxs[2]
        {
            return CONTENTS_SOLID;
        }
    }
    0
}

// ================================ fixture parse ===============================

fn parse_float(tok: &str) -> f32 {
    let b = tok.as_bytes();
    if b.len() >= 2 && b[0] == b'0' && (b[1] == b'x' || b[1] == b'X') {
        let u = u32::from_str_radix(&tok[2..], 16).unwrap();
        f32::from_bits(u)
    } else {
        tok.parse::<i64>().unwrap() as f32
    }
}

fn parse_int(tok: &str) -> c_int {
    let b = tok.as_bytes();
    if b.len() >= 2 && b[0] == b'0' && (b[1] == b'x' || b[1] == b'X') {
        i64::from_str_radix(&tok[2..], 16).unwrap() as c_int
    } else {
        tok.parse::<i64>().unwrap() as c_int
    }
}

fn parse_surf(tok: &str) -> c_int {
    match tok.find('=') {
        Some(i) => {
            let v = &tok[i + 1..];
            let b = v.as_bytes();
            if b.len() >= 2 && b[0] == b'0' && (b[1] == b'x' || b[1] == b'X') {
                i64::from_str_radix(&v[2..], 16).unwrap() as c_int
            } else {
                v.parse::<i64>().unwrap() as c_int
            }
        }
        None => 0,
    }
}

fn strip_comment(line: &str) -> &str {
    match line.find('#') {
        Some(i) => &line[..i],
        None => line,
    }
}

// ================================ TestTraps ===================================

struct TestTraps {
    brushes: Vec<Brush>,
    trace_count: Cell<i64>,
    fixdir: PathBuf,
    files: std::cell::RefCell<Vec<Option<(Vec<u8>, usize)>>>, // handle -> (bytes,pos)
}

impl TestTraps {
    fn new(fixdir: PathBuf) -> Self {
        Self {
            brushes: Vec::new(),
            trace_count: Cell::new(0),
            fixdir,
            files: std::cell::RefCell::new(vec![None]),
        }
    }
}

impl BgTraps for TestTraps {
    fn com_printf(&self, _msg: &str) {}
    fn com_error(&self, error_level: c_int, msg: &str) {
        panic!("com_error({}) in test: {:?}", error_level, msg);
    }
    fn trace(
        &self,
        results: *mut trace_t,
        start: *const vec3_t,
        mins: *const vec3_t,
        maxs: *const vec3_t,
        end: *const vec3_t,
        _passEntityNum: c_int,
        _contentMask: c_int,
    ) {
        self.trace_count.set(self.trace_count.get() + 1);
        unsafe {
            let tr = trace_world(&self.brushes, &*start, &*mins, &*maxs, &*end);
            *results = tr;
        }
    }

    fn pointcontents(&self, point: *const vec3_t, _passEntityNum: c_int) -> c_int {
        unsafe { point_in_brushes(&self.brushes, &*point) }
    }

    fn snap_vector(&self, v: *mut f32) {
        // Canonical `SnapVector`; the trait hands a raw `*mut f32`, reborrowed as
        // the `&mut vec3_t` the shared impl takes (test-mock seam).
        snap_vector(unsafe { &mut *(v as *mut vec3_t) });
    }

    // --- FS: only the animation.cfg load path uses these ---
    fn fs_fopen(&self, qpath: &str, f: *mut fileHandle_t, mode: fsMode_t) -> c_int {
        // FS_READ == 0; only reads are served.
        if mode as c_int != 0 {
            unsafe {
                if !f.is_null() {
                    *f = 0;
                }
            }
            return -1;
        }
        // Safety: `qpath` is the NUL-terminated vpath literal the animation.cfg
        // load path passes through `BG_ParseAnimationFile` / the `trap_FS_*` seam.
        let vpath = qpath;
        let base = vpath.rsplit('/').next().unwrap_or(vpath);
        let real = self.fixdir.join(base);
        match std::fs::read(&real) {
            Ok(bytes) => {
                let len = bytes.len() as c_int;
                let mut files = self.files.borrow_mut();
                let h = files.len() as fileHandle_t;
                files.push(Some((bytes, 0)));
                unsafe {
                    if !f.is_null() {
                        *f = h;
                    }
                }
                len
            }
            Err(_) => {
                unsafe {
                    if !f.is_null() {
                        *f = 0;
                    }
                }
                -1
            }
        }
    }

    fn fs_read(&self, buffer: *mut c_void, len: c_int, f: fileHandle_t) {
        let mut files = self.files.borrow_mut();
        let idx = f as usize;
        if idx == 0 || idx >= files.len() {
            return;
        }
        if let Some((bytes, pos)) = files[idx].as_mut() {
            let want = len as usize;
            let avail = bytes.len().saturating_sub(*pos);
            let n = want.min(avail);
            unsafe {
                std::ptr::copy_nonoverlapping(bytes[*pos..].as_ptr(), buffer as *mut u8, n);
            }
            *pos += n;
        }
    }

    fn fs_write(&self, _buffer: *const c_void, _len: c_int, _f: fileHandle_t) {}

    fn fs_fclose(&self, f: fileHandle_t) {
        let mut files = self.files.borrow_mut();
        let idx = f as usize;
        if idx != 0 && idx < files.len() {
            files[idx] = None;
        }
    }

    fn fs_getfilelist(
        &self,
        _path: &str,
        _extension: &str,
        _listbuf: *mut c_char,
        _bufsize: c_int,
    ) -> c_int {
        0
    }

    // --- everything below is off the basic saber path ---
    fn r_register_skin(&self, _name: &str) -> qhandle_t {
        unreachable!("r_register_skin off the basic pmove saber path")
    }
    fn g2api_init_ghoul2_model(
        &self,
        _a: *mut *mut c_void,
        _b: &str,
        _c: c_int,
        _d: qhandle_t,
        _e: qhandle_t,
        _f: c_int,
        _g: c_int,
    ) -> c_int {
        unreachable!()
    }
    fn g2api_clean_ghoul2_models(&self, _a: *mut *mut c_void) {
        unreachable!()
    }
    fn g2api_add_bolt(&self, _a: *mut c_void, _b: c_int, _c: &str) -> c_int {
        unreachable!()
    }
    fn g2api_get_bolt_matrix(
        &self,
        _a: *mut c_void,
        _b: c_int,
        _c: c_int,
        _d: *mut mdxaBone_t,
        _e: *const vec3_t,
        _f: *const vec3_t,
        _g: c_int,
        _h: *mut qhandle_t,
        _i: *const vec3_t,
    ) -> qboolean {
        unreachable!()
    }
    fn g2api_get_bolt_matrix_no_reconstruct(
        &self,
        _a: *mut c_void,
        _b: c_int,
        _c: c_int,
        _d: *mut mdxaBone_t,
        _e: *const vec3_t,
        _f: *const vec3_t,
        _g: c_int,
        _h: *mut qhandle_t,
        _i: *const vec3_t,
    ) -> qboolean {
        unreachable!()
    }
    fn g2api_get_bolt_matrix_no_rec_no_rot(
        &self,
        _a: *mut c_void,
        _b: c_int,
        _c: c_int,
        _d: *mut mdxaBone_t,
        _e: *const vec3_t,
        _f: *const vec3_t,
        _g: c_int,
        _h: *mut qhandle_t,
        _i: *const vec3_t,
    ) -> qboolean {
        unreachable!()
    }
    fn g2api_set_bone_angles(
        &self,
        _a: *mut c_void,
        _b: c_int,
        _c: &str,
        _d: *const vec3_t,
        _e: c_int,
        _f: c_int,
        _g: c_int,
        _h: c_int,
        _i: *mut qhandle_t,
        _j: c_int,
        _k: c_int,
    ) -> qboolean {
        unreachable!()
    }
    fn g2api_set_bone_anim(
        &self,
        _a: *mut c_void,
        _b: c_int,
        _c: &str,
        _d: c_int,
        _e: c_int,
        _f: c_int,
        _g: f32,
        _h: c_int,
        _i: f32,
        _j: c_int,
    ) -> qboolean {
        unreachable!()
    }
    fn g2api_get_bone_anim(
        &self,
        _a: *mut c_void,
        _b: &str,
        _c: c_int,
        _d: *mut f32,
        _e: *mut c_int,
        _f: *mut c_int,
        _g: *mut c_int,
        _h: *mut f32,
        _i: *mut c_int,
        _j: c_int,
    ) -> qboolean {
        unreachable!()
    }
    fn g2api_set_rag_doll(&self, _a: *mut c_void, _b: *mut sharedRagDollParams_t) {
        unreachable!()
    }
    fn g2api_animate_g2_models(
        &self,
        _a: *mut c_void,
        _b: c_int,
        _c: *mut sharedRagDollUpdateParams_t,
    ) {
        unreachable!()
    }
    fn g2api_set_bone_ik_state(
        &self,
        _a: *mut c_void,
        _b: c_int,
        _c: Option<&str>,
        _d: c_int,
        _e: *mut sharedSetBoneIKStateParams_t,
    ) -> qboolean {
        unreachable!()
    }
    fn g2api_ik_move(&self, _a: *mut c_void, _b: c_int, _c: *mut sharedIKMoveParams_t) -> qboolean {
        unreachable!()
    }
    fn g2api_get_surface_render_status(
        &self,
        _a: *mut c_void,
        _b: c_int,
        _c: &str,
    ) -> c_int {
        unreachable!()
    }
    fn fx_play_effect_id(
        &self,
        _a: c_int,
        _b: *const vec3_t,
        _c: *const vec3_t,
        _d: c_int,
        _e: c_int,
    ) {
        unreachable!()
    }
    fn cvar_register(&self, _a: *mut vmCvar_t, _b: &str, _c: &str, _d: c_int) {
        unreachable!()
    }
}

// =============================== TestCallbacks ================================
// Only the QAGAME anim restart-check is reachable; it reads the prior-frame anim
// mirror (what BG_PlayerStateToEntityState writes live at end of each frame).

struct TestCallbacks {
    legs_mirror: c_int,
    torso_mirror: c_int,
}

impl GameCallbacks for TestCallbacks {
    fn entity_legs_anim(&self, _entNum: c_int) -> c_int {
        self.legs_mirror
    }
    fn entity_torso_anim(&self, _entNum: c_int) -> c_int {
        self.torso_mirror
    }

    fn damage(
        &mut self,
        _t: c_int,
        _i: c_int,
        _a: c_int,
        _d: *const vec3_t,
        _p: *const vec3_t,
        _dm: c_int,
        _df: c_int,
        _m: c_int,
    ) {
        unreachable!("damage off the basic pmove saber path")
    }
    fn damage_from_killer(
        &mut self,
        _t: c_int,
        _i: c_int,
        _a: c_int,
        _k: c_int,
        _d: *const vec3_t,
        _p: *const vec3_t,
        _dm: c_int,
        _df: c_int,
        _m: c_int,
    ) {
        unreachable!()
    }
    fn add_event(&mut self, _e: c_int, _ev: c_int, _p: c_int) {
        unreachable!("add_event: events are written to ps.events[] directly, not via a trap")
    }
    fn alloc(&mut self, _size: c_int) -> *mut c_void {
        unreachable!()
    }
    fn new_string(&mut self, _s: &str) -> *mut c_char {
        unreachable!()
    }
    fn play_effect(&mut self, _f: c_int, _o: *const vec3_t, _a: *const vec3_t) {
        unreachable!()
    }
    fn play_effect_id(&mut self, _f: c_int, _o: *const vec3_t, _a: *const vec3_t) -> c_int {
        unreachable!()
    }
    fn sound_index(&mut self, _n: &str) -> c_int {
        unreachable!()
    }
    fn model_index(&mut self, _n: &str) -> c_int {
        unreachable!()
    }
    fn effect_index(&mut self, _n: &str) -> c_int {
        unreachable!()
    }
    fn cheap_weapon_fire(&mut self, _e: c_int, _w: c_int) {
        unreachable!()
    }
    fn client_check_impact_bbrush(&mut self, _e: c_int, _i: c_int) {
        unreachable!()
    }
    fn flyveh_surface_destruction(&mut self, _e: c_int, _t: *mut trace_t, _m: c_int, _f: qboolean) {
        unreachable!()
    }
    fn set_anim(
        &mut self,
        _e: c_int,
        _u: *mut usercmd_t,
        _p: c_int,
        _a: c_int,
        _f: c_int,
        _b: c_int,
    ) {
        unreachable!()
    }
    fn npc_set_anim(&mut self, _e: c_int, _t: c_int, _a: c_int, _p: c_int) {
        unreachable!()
    }
    fn wp_get_vehicle_cam_pos(&mut self, _v: c_int, _p: c_int, _c: *mut vec3_t) {
        unreachable!()
    }
    fn can_be_enemy(&mut self, _e: c_int, _o: c_int) -> qboolean {
        unreachable!()
    }
    fn get_time(&self) -> c_int {
        unreachable!("get_time only fires on a non-world slide impact")
    }
    fn try_grapple(&mut self, _e: c_int) -> qboolean {
        unreachable!()
    }
    fn q3_set_parm(&mut self, _e: c_int, _p: c_int, _v: &str) {
        unreachable!()
    }
    fn board_vehicle(&mut self, _v: c_int, _e: c_int) -> qboolean {
        unreachable!()
    }
    fn update_vehicle(&mut self, _v: c_int, _u: *const usercmd_t) {
        unreachable!()
    }
    fn pm_animate_vehicle(&mut self, _v: c_int) {
        unreachable!()
    }
    fn update_rider(&mut self, _v: c_int, _r: c_int, _u: *mut usercmd_t) {
        unreachable!()
    }
    fn attach_riders(&mut self, _v: c_int) {
        unreachable!()
    }
    fn my_saber(&mut self, _c: c_int, _s: c_int) -> *mut saberInfo_t {
        // The `g_entities`/bgEntity arena is zeroed (module doc), so BG_MySaber
        // returns NULL on both sides — no per-saber saberInfo is read.
        core::ptr::null_mut()
    }
    fn suspended_vehicle_boardable(&self, _v: c_int) -> qboolean {
        unreachable!()
    }
    fn landed_vehicle_boardable(&self, _t: c_int, _s: c_int, _g: c_int) -> qboolean {
        unreachable!()
    }
    fn set_solid_hack(&mut self, _e: c_int) {
        unreachable!()
    }
    fn humanoid_inuse_client(&self, _e: c_int) -> qboolean {
        unreachable!()
    }
    fn fighter_not_suspended(&self, _e: c_int) -> qboolean {
        unreachable!()
    }
    fn set_other_killer(&mut self, _e: c_int, _m: c_int, _v: c_int, _w: c_int) {
        unreachable!()
    }
    fn entity_inuse(&self, _e: c_int) -> qboolean {
        unreachable!()
    }
    fn entity_spawnflags(&self, _e: c_int) -> c_int {
        unreachable!()
    }
    fn entity_takedamage(&self, _e: c_int) -> qboolean {
        unreachable!()
    }
    fn fighter_is_landed(&self, _e: c_int) -> qboolean {
        unreachable!()
    }
}

// ================================ ps baseline ================================
// The melee baseline plus the saber pin-set (mirror of `main_pmove_saber.c`
// `ps_baseline`): single-saber MEDIUM style, saber lit and in-hand.

fn ps_baseline() -> playerState_t {
    let mut ps: playerState_t = unsafe { core::mem::zeroed() };
    ps.pm_type = PM_NORMAL as c_int;
    ps.weapon = WP_SABER;
    ps.weaponstate = WEAPON_READY as c_int;
    ps.stats[STAT_HEALTH as usize] = 100;
    ps.gravity = 800;
    ps.speed = 250.0;
    ps.basespeed = 250;
    ps.standheight = DEFAULT_MAXS_2; // 40
    ps.crouchheight = CROUCH_MAXS_2; // 16
    ps.viewheight = DEFAULT_VIEWHEIGHT; // 36
    ps.groundEntityNum = ENTITYNUM_NONE;
    ps.clientNum = 0;
    ps.m_iVehicleNum = 0;
    ps.commandTime = 0;
    // saber pins.
    ps.fd.saberAnimLevel = saber_styles_t::SS_MEDIUM as c_int;
    ps.fd.saberAnimLevelBase = saber_styles_t::SS_MEDIUM as c_int;
    ps.saberEntityNum = 1; // nonzero: PM_GetSaberStance gives a real stance
    ps.saberHolstered = 0; // sabers ON -> BG_SabersOff() false
    ps.saberMove = 0; // LS_NONE; settles to LS_READY on step 1
    ps
}

fn apply_ps_override(ps: &mut playerState_t, tok: &[&str]) {
    let name = tok[1];
    match name {
        "origin" => {
            ps.origin = [
                parse_float(tok[2]),
                parse_float(tok[3]),
                parse_float(tok[4]),
            ];
        }
        "velocity" => {
            ps.velocity = [
                parse_float(tok[2]),
                parse_float(tok[3]),
                parse_float(tok[4]),
            ];
        }
        "viewangles" => {
            ps.viewangles = [
                parse_float(tok[2]),
                parse_float(tok[3]),
                parse_float(tok[4]),
            ];
        }
        "delta_angles" => {
            ps.delta_angles = [parse_int(tok[2]), parse_int(tok[3]), parse_int(tok[4])];
        }
        "groundEntityNum" => ps.groundEntityNum = parse_int(tok[2]),
        "pm_flags" => ps.pm_flags = parse_int(tok[2]),
        "pm_type" => ps.pm_type = parse_int(tok[2]),
        "legsAnim" => ps.legsAnim = parse_int(tok[2]),
        "torsoAnim" => ps.torsoAnim = parse_int(tok[2]),
        "weapon" => ps.weapon = parse_int(tok[2]),
        "gravity" => ps.gravity = parse_int(tok[2]),
        "speed" => ps.speed = parse_float(tok[2]),
        "basespeed" => ps.basespeed = parse_int(tok[2]),
        "fallingToDeath" => ps.fallingToDeath = parse_int(tok[2]),
        "clientNum" => ps.clientNum = parse_int(tok[2]),
        // --- saber-slice additions (mirror the C psfield table) ---
        "saberEntityNum" => ps.saberEntityNum = parse_int(tok[2]),
        "saberMove" => ps.saberMove = parse_int(tok[2]),
        "saberHolstered" => ps.saberHolstered = parse_int(tok[2]),
        "saberBlocked" => ps.saberBlocked = parse_int(tok[2]),
        "saberInFlight" => ps.saberInFlight = parse_int(tok[2]),
        "saberAnimLevel" => ps.fd.saberAnimLevel = parse_int(tok[2]),
        "saberAnimLevelBase" => ps.fd.saberAnimLevelBase = parse_int(tok[2]),
        other => panic!("unknown ps field '{other}'"),
    }
}

// =================================== dump ====================================

fn f2b(v: f32) -> u32 {
    v.to_bits()
}

fn dump_step(o: &mut String, step: i32, pm: &pmove_t, ps: &playerState_t, ntr: i64, rng: u32) {
    let _ = writeln!(
        o,
        "s={} t={} org={:08x},{:08x},{:08x} vel={:08x},{:08x},{:08x} \
         va={:08x},{:08x},{:08x} da={},{},{} gnd={} pmf={:x} pmt={} \
         la={}:{} ta={}:{} fl={}{} bob={} vh={} ef={:x} seq={} \
         ev={}:{},{}:{} wt={} ws={} spd={:08x} wl={} wtp={} \
         nt={} mn={:08x} mx={:08x} xy={:08x} air={} f2d={} fjz={:08x} \
         ntr={} rng={:08x} \
         sm={} sb={} shl={} sen={} sal={} sac={}",
        step,
        ps.commandTime,
        f2b(ps.origin[0]),
        f2b(ps.origin[1]),
        f2b(ps.origin[2]),
        f2b(ps.velocity[0]),
        f2b(ps.velocity[1]),
        f2b(ps.velocity[2]),
        f2b(ps.viewangles[0]),
        f2b(ps.viewangles[1]),
        f2b(ps.viewangles[2]),
        ps.delta_angles[0],
        ps.delta_angles[1],
        ps.delta_angles[2],
        ps.groundEntityNum,
        ps.pm_flags as u32,
        ps.pm_time,
        ps.legsAnim,
        ps.legsTimer,
        ps.torsoAnim,
        ps.torsoTimer,
        if ps.legsFlip != 0 { 1 } else { 0 },
        if ps.torsoFlip != 0 { 1 } else { 0 },
        ps.bobCycle,
        ps.viewheight,
        ps.eFlags as u32,
        ps.eventSequence,
        ps.events[0],
        ps.eventParms[0],
        ps.events[1],
        ps.eventParms[1],
        ps.weaponTime,
        ps.weaponstate,
        f2b(ps.speed),
        pm.waterlevel,
        pm.watertype,
        pm.numtouch,
        f2b(pm.mins[2]),
        f2b(pm.maxs[2]),
        f2b(pm.xyspeed),
        if ps.inAirAnim != 0 { 1 } else { 0 },
        ps.fallingToDeath,
        f2b(ps.fd.forceJumpZStart),
        ntr,
        rng,
        ps.saberMove,
        ps.saberBlocked,
        ps.saberHolstered,
        ps.saberEntityNum,
        ps.fd.saberAnimLevel,
        ps.saberAttackChainCount,
    );
}

// ============================ scenario driver ================================

struct Cmd {
    dt: c_int,
    fwd: c_int,
    right: c_int,
    up: c_int,
    buttons: c_int,
    yaw: c_int,
    pitch: c_int,
    roll: c_int,
    reps: i32,
    yawinc: c_int,
}

enum Row {
    Brush(Brush),
    Ps(Vec<String>),
    Start,
    Cmd(Cmd),
}

fn parse_scenario(path: &PathBuf) -> Vec<Row> {
    let text = std::fs::read_to_string(path).unwrap();
    let mut rows = Vec::new();
    for line in text.lines() {
        let line = strip_comment(line);
        let tok: Vec<&str> = line.split_whitespace().collect();
        if tok.is_empty() {
            continue;
        }
        match tok[0] {
            "brush" if tok.len() >= 7 => {
                let surf = if tok.len() >= 8 {
                    parse_surf(tok[7])
                } else {
                    0
                };
                rows.push(Row::Brush(Brush {
                    mins: [
                        parse_float(tok[1]),
                        parse_float(tok[2]),
                        parse_float(tok[3]),
                    ],
                    maxs: [
                        parse_float(tok[4]),
                        parse_float(tok[5]),
                        parse_float(tok[6]),
                    ],
                    surface_flags: surf,
                }));
            }
            "ps" if tok.len() >= 3 => {
                rows.push(Row::Ps(tok.iter().map(|s| s.to_string()).collect()));
            }
            "start" => rows.push(Row::Start),
            "cmd" if tok.len() >= 9 => {
                let reps = if tok.len() >= 10 && tok[9].starts_with('x') {
                    tok[9][1..].parse::<i32>().unwrap()
                } else {
                    1
                };
                let yawinc = if tok.len() >= 11 {
                    parse_int(tok[10])
                } else {
                    0
                };
                rows.push(Row::Cmd(Cmd {
                    dt: parse_int(tok[1]),
                    fwd: parse_int(tok[2]),
                    right: parse_int(tok[3]),
                    up: parse_int(tok[4]),
                    buttons: parse_int(tok[5]),
                    yaw: parse_int(tok[6]),
                    pitch: parse_int(tok[7]),
                    roll: parse_int(tok[8]),
                    reps,
                    yawinc,
                }));
            }
            other => panic!("bad fixture line: {other}"),
        }
    }
    rows
}

fn run_scenario(name: &str) -> String {
    let rows = parse_scenario(&fixture_dir().join(format!("{name}.txt")));

    let mut bg = BgState::new();
    // Size the humanoid anim table so BG_ParseAnimationFile (and Pmove's anim
    // reads) write/read into valid backing — the parser fills animset[token].
    bg.bgHumanoidAnimations
        .resize(MAX_TOTALANIMATIONS as usize, unsafe { core::mem::zeroed() });

    let mut traps = TestTraps::new(fixture_dir());
    let mut cb = TestCallbacks {
        legs_mirror: 0,
        torso_mirror: 0,
    };

    // Load the synthetic humanoid animation set (both sides parse the same file).
    let animset = bg.bgHumanoidAnimations.as_mut_ptr();
    let rc = BG_ParseAnimationFile(
        &mut bg,
        &traps,
        &mut cb,
        c"models/players/_humanoid/animation.cfg".as_ptr(),
        animset,
        qtrue,
    );
    assert_ne!(rc, -1, "failed to load synthetic animation.cfg");

    // Brushes are fixed for the scenario; collect them before the run.
    for row in &rows {
        if let Row::Brush(b) = row {
            traps.brushes.push(*b);
        }
    }

    let mut ps = ps_baseline();

    // pmove_t skeleton; cmd fields patched per step.
    let mut pm: pmove_t = unsafe { core::mem::zeroed() };
    pm.tracemask = MASK_PLAYERSOLID;
    pm.animations = bg.bgHumanoidAnimations.as_mut_ptr();
    pm.gametype = 0;

    let mut arena: Vec<bgEntity_t> = (0..8).map(|_| unsafe { core::mem::zeroed() }).collect();
    pm.entSize = core::mem::size_of::<bgEntity_t>() as c_int;

    let mut o = String::new();
    let _ = writeln!(o, "-- scenario {name} --");
    o.push_str("== pmove ==\n");

    let mut step = 0i32;
    let mut prev_server_time = 0i32;

    for row in &rows {
        match row {
            Row::Brush(_) => {}
            Row::Ps(tok) => {
                let refs: Vec<&str> = tok.iter().map(|s| s.as_str()).collect();
                apply_ps_override(&mut ps, &refs);
            }
            Row::Start => {
                // Freeze the anim mirror to the initial ps and emit the pre-move
                // baseline step (ntr=0).
                cb.legs_mirror = ps.legsAnim;
                cb.torso_mirror = ps.torsoAnim;
                traps.trace_count.set(0);
                // pm.ps / baseEnt must point at the live ps / arena for the dump.
                pm.ps = &mut ps as *mut playerState_t;
                pm.baseEnt = arena.as_mut_ptr() as *mut _;
                let rng = bg.rng.holdrand() as u32; // 32-bit tripwire (fixtures draw nothing)
                dump_step(&mut o, step, &pm, &ps, traps.trace_count.get(), rng);
                step += 1;
            }
            Row::Cmd(c) => {
                for r in 0..c.reps {
                    pm.ps = &mut ps as *mut playerState_t;
                    pm.baseEnt = arena.as_mut_ptr() as *mut _;
                    pm.cmd.forwardmove = c.fwd as i8 as c_schar;
                    pm.cmd.rightmove = c.right as i8 as c_schar;
                    pm.cmd.upmove = c.up as i8 as c_schar;
                    pm.cmd.buttons = c.buttons;
                    pm.cmd.weapon = WP_SABER as byte;
                    pm.cmd.angles[0] = (c.pitch as i16) as c_int;
                    pm.cmd.angles[1] = ((c.yaw + r * c.yawinc) as i16) as c_int;
                    pm.cmd.angles[2] = (c.roll as i16) as c_int;
                    prev_server_time += c.dt;
                    pm.cmd.serverTime = prev_server_time;

                    traps.trace_count.set(0);
                    Pmove(&mut pm as *mut pmove_t, &mut bg, &traps, &mut cb);

                    // Mirror ps anims into the stub entity for the next step's
                    // restart-check (BG_PlayerStateToEntityState equivalent).
                    cb.legs_mirror = ps.legsAnim;
                    cb.torso_mirror = ps.torsoAnim;

                    let rng = bg.rng.holdrand() as u32; // 32-bit tripwire (fixtures draw nothing)
                    dump_step(&mut o, step, &pm, &ps, traps.trace_count.get(), rng);
                    step += 1;
                }
            }
        }
    }

    o.push_str("== end ==\n");
    o
}

// =================================== test ====================================

#[test]
fn pmove_saber_parity() {
    let scenarios = [
        "saber-idle",
        "saber-walk",
        "saber-attack-stand",
        "saber-attack-run",
        "saber-attack-strafe",
        "saber-jump",
    ];
    let mut o = String::new();
    for s in scenarios {
        o.push_str(&run_scenario(s));
    }
    compare(env!("CARGO_MANIFEST_DIR"), "pmove_saber", &o);
}
