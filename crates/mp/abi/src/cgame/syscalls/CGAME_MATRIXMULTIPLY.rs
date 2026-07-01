use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::vec3_t;

/// Arguments for `CGAME_MATRIXMULTIPLY`.
///
/// Raven's engine switch casts all three ABI words with `VMA` as `vec3_t *`.
/// `in1` and `in2` are input 3x3 axes (`vec3_t[3]`), while `out` is the
/// caller-provided output axis buffer. The engine returns `0`; the matrix result
/// is written through `out`, so it is modeled as an argument rather than
/// `Output`.
///
/// Args source: `oracle/oracle/codemp/client/cl_cgame.cpp:666`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:667`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:665`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:289`
#[derive(Debug)]
pub struct CgameMatrixmultiplyArgs {
    in1: *const vec3_t,
    in2: *const vec3_t,
    out: *mut vec3_t,
}

impl CgameMatrixmultiplyArgs {
    pub const fn new(in1: *const vec3_t, in2: *const vec3_t, out: *mut vec3_t) -> Self {
        Self { in1, in2, out }
    }

    pub const fn in1(&self) -> *const vec3_t {
        self.in1
    }

    pub const fn in2(&self) -> *const vec3_t {
        self.in2
    }

    pub const fn out(&self) -> *mut vec3_t {
        self.out
    }
}

/// `CGAME_MATRIXMULTIPLY` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:137`
/// Args source: `oracle/oracle/codemp/client/cl_cgame.cpp:666`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:667`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:665`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:289`
pub struct CgameMatrixmultiply;

impl OutboundSysCall for CgameMatrixmultiply {
    type Import = MpCgameImport;
    type Args = CgameMatrixmultiplyArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CGAME_MATRIXMULTIPLY;
}

impl EncodeSysCall for CgameMatrixmultiply {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.in1()),
            ptr_to_word(args.in2()),
            ptr_to_word(args.out()),
        ])
    }
}

impl DecodeSysCallReturn for CgameMatrixmultiply {
    // `MatrixMultiply` writes through `out`; Raven returns 0 from the syscall arm.
    fn decode_return(_word: isize) -> Self::Output {}
}
