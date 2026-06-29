use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::GameImport;

/// `G_MATRIXMULTIPLY` outbound game-to-engine syscall.
///
/// Maps to `MatrixMultiply( (vec3_t *)VMA(1), (vec3_t *)VMA(2), (vec3_t *)VMA(3) )` in `sv_game.cpp`.
/// All three arguments are raw pointers mirroring the C ABI's VMA(1)–VMA(3).
/// `out` is an out-param: the engine writes the result through it.
#[derive(Debug)]
pub struct GMatrixmultiplyArgs {
    pub in1: *const vec3_t,
    pub in2: *const vec3_t,
    pub out: *mut vec3_t,
}

impl GMatrixmultiplyArgs {
    pub fn new(in1: *const vec3_t, in2: *const vec3_t, out: *mut vec3_t) -> Self {
        Self { in1, in2, out }
    }

    pub fn in1(&self) -> *const vec3_t {
        self.in1
    }
    pub fn in2(&self) -> *const vec3_t {
        self.in2
    }
    pub fn out(&self) -> *mut vec3_t {
        self.out
    }
}

/// `G_MATRIXMULTIPLY` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:283`
pub struct GMatrixmultiply;

impl OutboundSysCall for GMatrixmultiply {
    type Import = GameImport;
    type Args = GMatrixmultiplyArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_MATRIXMULTIPLY;
}

impl EncodeSysCall for GMatrixmultiply {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.in1 as *const _),
            ptr_to_word(a.in2 as *const _),
            ptr_to_word(a.out as *const _),
        ])
    }
}

impl DecodeSysCallReturn for GMatrixmultiply {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
