use core::ffi::{c_int, c_void};

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::abi::pass_float;
use crate::codemp::game::q_shared_h::CollisionRecord_t;

/// Arguments for `CG_G2_COLLISIONDETECT`.
///
/// Raven wrapper: `void trap_G2API_CollisionDetect(...)`.
/// Raven transport: `G2API_CollisionDetect((CollisionRecord_t*)VMA(1), *((CGhoul2Info_v *)args[2]), ...,
///     G2VertSpaceClient, args[10], args[11], VMF(12)); return 0;`.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:265`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:820-835`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2508`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1339-1352`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1339-1352`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgG2CollisiondetectArgs {
    /// `CollisionRecord_t *collRecMap`, decoded by Raven from `VMA(1)`.
    coll_rec_map: *mut CollisionRecord_t,
    /// Ghoul2 handle, decoded by Raven as `*((CGhoul2Info_v *)args[2])`.
    ghoul2: *mut c_void,
    /// Model angles, decoded by Raven from `VMA(3)`.
    angles: *const f32,
    /// Model position, decoded by Raven from `VMA(4)`.
    position: *const f32,
    /// Frame number.
    frame_number: c_int,
    /// Entity number.
    ent_num: c_int,
    /// Ray start point, decoded by Raven from `VMA(7)`.
    ray_start: *mut f32,
    /// Ray end point, decoded by Raven from `VMA(8)`.
    ray_end: *mut f32,
    /// Model scale, decoded by Raven from `VMA(9)`.
    scale: *mut f32,
    /// Trace flags.
    trace_flags: c_int,
    /// LOD selector.
    use_lod: c_int,
    /// Collision radius, transported with `VMF(12)` / `PASSFLOAT`.
    f_radius: f32,
}

impl CgG2CollisiondetectArgs {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
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
}

/// `CG_G2_COLLISIONDETECT` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:265`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:820-835`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2508`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1339-1352`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1339-1352`
pub struct CgG2Collisiondetect;

impl OutboundSysCall for CgG2Collisiondetect {
    type Import = MpCgameImport;
    type Args = CgG2CollisiondetectArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_COLLISIONDETECT;
}

impl EncodeSysCall for CgG2Collisiondetect {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.coll_rec_map),
            ptr_to_word(args.ghoul2),
            ptr_to_word(args.angles),
            ptr_to_word(args.position),
            args.frame_number as isize,
            args.ent_num as isize,
            ptr_to_word(args.ray_start),
            ptr_to_word(args.ray_end),
            ptr_to_word(args.scale),
            args.trace_flags as isize,
            args.use_lod as isize,
            pass_float(args.f_radius),
        ])
    }
}

impl DecodeSysCallReturn for CgG2Collisiondetect {
    fn decode_return(_word: isize) -> Self::Output {}
}
