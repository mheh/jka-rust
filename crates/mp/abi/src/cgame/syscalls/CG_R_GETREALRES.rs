use core::ffi::c_int;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_R_GETREALRES`.
///
/// Raven: get screen resolution -rww.
/// Raven wrapper: `syscall( CG_R_GETREALRES, w, h );`
/// Raven transport: writes `glConfig.vidWidth` to `(int *)VMA(1)` and
/// `glConfig.vidHeight` to `(int *)VMA(2)`, then returns 0.
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:431-434`
/// Args source: `oracle/codemp/cgame/cg_local.h:2303`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1066-1073`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRGetrealresArgs {
    w: *mut c_int,
    h: *mut c_int,
}

impl CgRGetrealresArgs {
    pub const fn new(w: *mut c_int, h: *mut c_int) -> Self {
        Self { w, h }
    }
}

/// `CG_R_GETREALRES` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:173`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:431-434`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1066-1073`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1066-1073`
pub struct CgRGetrealres;

impl OutboundSysCall for CgRGetrealres {
    type Import = MpCgameImport;
    type Args = CgRGetrealresArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_GETREALRES;
}

impl EncodeSysCall for CgRGetrealres {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.w), ptr_to_word(args.h)])
    }
}

impl DecodeSysCallReturn for CgRGetrealres {
    fn decode_return(_word: isize) -> Self::Output {}
}
