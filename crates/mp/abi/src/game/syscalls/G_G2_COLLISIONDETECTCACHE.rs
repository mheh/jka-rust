use core::ffi::{c_int, c_void};

use super::super::MpGameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use abi_transport::pass_float;
use mp_qshared::shared::CollisionRecord_t;

/// `G_G2_COLLISIONDETECTCACHE` outbound game-to-engine syscall.
///
/// Mirrors the C ABI:
/// ```c
/// G2API_CollisionDetectCache(
///     (CollisionRecord_t*)VMA(1),   // args[1]  out: collision results
///     *((CGhoul2Info_v*)args[2]),   // args[2]  ghoul2 handle (opaque C++ class)
///     (const float*)VMA(3),         // args[3]  angles (vec3_t)
///     (const float*)VMA(4),         // args[4]  origin (vec3_t)
///     args[5],                       // args[5]  time (int)
///     args[6],                       // args[6]  entNum (int)
///     (float*)VMA(7),               // args[7]  scale (vec3_t out)
///     (float*)VMA(8),               // args[8]  out vec3_t
///     (float*)VMA(9),               // args[9]  out vec3_t
///     G2VertSpaceServer,            // injected by engine, not a VM arg
///     args[10],                      // args[10] int
///     args[11],                      // args[11] int
///     VMF(12)                        // args[12] float radius/LOD
/// );
/// return 0; // void
/// ```
#[derive(Debug)]
pub struct GG2CollisiondetectcacheArgs {
    /// args[1]: out-param collision record array
    pub collision_records: *mut CollisionRecord_t,
    /// args[2]: CGhoul2Info_v* (opaque C++ class pointer)
    pub ghoul2: *mut c_void,
    /// args[3]: angles (const vec3_t)
    pub angles: *const f32,
    /// args[4]: origin (const vec3_t)
    pub origin: *const f32,
    /// args[5]: time
    pub time: c_int,
    /// args[6]: entity number
    pub ent_num: c_int,
    /// args[7]: scale (vec3_t out)
    pub scale: *mut f32,
    /// args[8]: out vec3_t
    pub out_vec8: *mut f32,
    /// args[9]: out vec3_t
    pub out_vec9: *mut f32,
    /// args[10]: int param
    pub param10: c_int,
    /// args[11]: int param
    pub param11: c_int,
    /// args[12]: float param (radius / LOD bias)
    pub float_param: f32,
}

impl GG2CollisiondetectcacheArgs {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        collision_records: *mut CollisionRecord_t,
        ghoul2: *mut c_void,
        angles: *const f32,
        origin: *const f32,
        time: c_int,
        ent_num: c_int,
        scale: *mut f32,
        out_vec8: *mut f32,
        out_vec9: *mut f32,
        param10: c_int,
        param11: c_int,
        float_param: f32,
    ) -> Self {
        Self {
            collision_records,
            ghoul2,
            angles,
            origin,
            time,
            ent_num,
            scale,
            out_vec8,
            out_vec9,
            param10,
            param11,
            float_param,
        }
    }

    pub fn collision_records(&self) -> *mut CollisionRecord_t {
        self.collision_records
    }
    pub fn ghoul2(&self) -> *mut c_void {
        self.ghoul2
    }
    pub fn angles(&self) -> *const f32 {
        self.angles
    }
    pub fn origin(&self) -> *const f32 {
        self.origin
    }
    pub fn time(&self) -> c_int {
        self.time
    }
    pub fn ent_num(&self) -> c_int {
        self.ent_num
    }
    pub fn scale(&self) -> *mut f32 {
        self.scale
    }
    pub fn out_vec8(&self) -> *mut f32 {
        self.out_vec8
    }
    pub fn out_vec9(&self) -> *mut f32 {
        self.out_vec9
    }
    pub fn param10(&self) -> c_int {
        self.param10
    }
    pub fn param11(&self) -> c_int {
        self.param11
    }
    pub fn float_param(&self) -> f32 {
        self.float_param
    }
}

/// `G_G2_COLLISIONDETECTCACHE` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:531`
pub struct GG2Collisiondetectcache;

impl OutboundSysCall for GG2Collisiondetectcache {
    type Import = MpGameImport;
    type Args = GG2CollisiondetectcacheArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_G2_COLLISIONDETECTCACHE;
}

impl EncodeSysCall for GG2Collisiondetectcache {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.collision_records), // args[1]
            ptr_to_word(a.ghoul2),            // args[2]
            ptr_to_word(a.angles),            // args[3]
            ptr_to_word(a.origin),            // args[4]
            a.time as isize,                  // args[5]
            a.ent_num as isize,               // args[6]
            ptr_to_word(a.scale),             // args[7]
            ptr_to_word(a.out_vec8),          // args[8]
            ptr_to_word(a.out_vec9),          // args[9]
            a.param10 as isize,               // args[10]
            a.param11 as isize,               // args[11]
            pass_float(a.float_param),        // args[12]
        ])
    }
}

impl DecodeSysCallReturn for GG2Collisiondetectcache {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
