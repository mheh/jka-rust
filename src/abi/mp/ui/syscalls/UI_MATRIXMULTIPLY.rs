use super::super::MpUiImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::vec3_t;

/// Arguments for `UI_MATRIXMULTIPLY`.
///
/// Raven's engine switch casts all three ABI words with `VMA` as `vec3_t *`.
/// `in1` and `in2` are input 3x3 axes (`vec3_t[3]`), while `out` is the
/// caller-provided output axis buffer. The engine returns `0`; the matrix result
/// is written through `out`, so it is modeled as an argument rather than
/// `Output`.
///
/// Args source: `oracle/oracle/codemp/client/cl_ui.cpp:666`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:667`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:665`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:289`
#[derive(Debug)]
pub struct UiMatrixmultiplyArgs {
    in1: *const vec3_t,
    in2: *const vec3_t,
    out: *mut vec3_t,
}

impl UiMatrixmultiplyArgs {
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

/// `UI_MATRIXMULTIPLY` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:137`
/// Args source: `oracle/oracle/codemp/client/cl_ui.cpp:666`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:667`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:665`
/// Shared trap token source: `oracle/oracle/codemp/qcommon/qcommon.h:289`
pub struct UiMatrixmultiply;

impl OutboundSysCall for UiMatrixmultiply {
    type Import = MpUiImport;
    type Args = UiMatrixmultiplyArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_MATRIXMULTIPLY;
}

impl EncodeSysCall for UiMatrixmultiply {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.in1()),
            ptr_to_word(args.in2()),
            ptr_to_word(args.out()),
        ])
    }
}

impl DecodeSysCallReturn for UiMatrixmultiply {
    // `MatrixMultiply` writes through `out`; Raven returns 0 from the syscall arm.
    fn decode_return(_word: isize) -> Self::Output {}
}
