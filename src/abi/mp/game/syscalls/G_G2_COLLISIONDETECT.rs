use core::ffi::{c_int, c_void};

use super::super::MpGameImport;
use crate::abi::pass_float;
use crate::shared::CollisionRecord_t;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_G2_COLLISIONDETECT` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GG2CollisiondetectArgs {
    /// Output collision record map (caller-allocated array).
    coll_rec_map: *mut CollisionRecord_t,
    /// Ghoul2 model instance handle (opaque void*).
    ghoul2: *mut c_void,
    /// Model angles (const vec3_t).
    angles: *const f32,
    /// Model position (const vec3_t).
    position: *const f32,
    /// Frame number / time.
    frame_number: c_int,
    /// Entity number.
    ent_num: c_int,
    /// Ray start point (vec3_t).
    ray_start: *mut f32,
    /// Ray end point (vec3_t).
    ray_end: *mut f32,
    /// Model scale (vec3_t).
    scale: *mut f32,
    /// Trace flags.
    trace_flags: c_int,
    /// LOD level.
    use_lod: c_int,
    /// Collision radius.
    f_radius: f32,
}

impl GG2CollisiondetectArgs {
    pub fn new(
        coll_rec_map: *mut CollisionRecord_t,
        ghoul2: *mut c_void,
        angles: *const f32,
        position: *const f32,
        frame_number: c_int,
        ent_num: c_int,
        ray_start: *mut f32,
        ray_end: *mut f32,
        scale: *mut f32,
        trace_flags: c_int,
        use_lod: c_int,
        f_radius: f32,
    ) -> Self {
        Self {
            coll_rec_map,
            ghoul2,
            angles,
            position,
            frame_number,
            ent_num,
            ray_start,
            ray_end,
            scale,
            trace_flags,
            use_lod,
            f_radius,
        }
    }

    pub fn coll_rec_map(&self) -> *mut CollisionRecord_t {
        self.coll_rec_map
    }

    pub fn ghoul2(&self) -> *mut c_void {
        self.ghoul2
    }

    pub fn angles(&self) -> *const f32 {
        self.angles
    }

    pub fn position(&self) -> *const f32 {
        self.position
    }

    pub fn frame_number(&self) -> c_int {
        self.frame_number
    }

    pub fn ent_num(&self) -> c_int {
        self.ent_num
    }

    pub fn ray_start(&self) -> *mut f32 {
        self.ray_start
    }

    pub fn ray_end(&self) -> *mut f32 {
        self.ray_end
    }

    pub fn scale(&self) -> *mut f32 {
        self.scale
    }

    pub fn trace_flags(&self) -> c_int {
        self.trace_flags
    }

    pub fn use_lod(&self) -> c_int {
        self.use_lod
    }

    pub fn f_radius(&self) -> f32 {
        self.f_radius
    }
}

/// `G_G2_COLLISIONDETECT` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:530`
pub struct GG2Collisiondetect;

impl OutboundSysCall for GG2Collisiondetect {
    type Import = MpGameImport;
    type Args = GG2CollisiondetectArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_G2_COLLISIONDETECT;
}

impl EncodeSysCall for GG2Collisiondetect {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.coll_rec_map),
            ptr_to_word(a.ghoul2),
            ptr_to_word(a.angles),
            ptr_to_word(a.position),
            a.frame_number as isize,
            a.ent_num as isize,
            ptr_to_word(a.ray_start),
            ptr_to_word(a.ray_end),
            ptr_to_word(a.scale),
            a.trace_flags as isize,
            a.use_lod as isize,
            pass_float(a.f_radius),
        ])
    }
}

impl DecodeSysCallReturn for GG2Collisiondetect {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
