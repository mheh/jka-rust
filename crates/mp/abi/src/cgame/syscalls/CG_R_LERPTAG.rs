use core::ffi::{c_char, c_int, c_void};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use abi_transport::pass_float;

/// Arguments for `CG_R_LERPTAG`.
///
/// Raven wrapper: `return syscall( CG_R_LERPTAG, tag, mod, startFrame, endFrame, PASSFLOAT(frac), tagName );`
/// Raven transport: `return re.LerpTag( (orientation_t *)VMA(1), args[2], args[3], args[4], VMF(5), (const char *)VMA(6) );`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:377-379`
/// Args source: `oracle/codemp/cgame/cg_local.h:2281-2282`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:934-935`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgRLerptagArgs {
    tag: *mut c_void,
    mod_: c_int,
    start_frame: c_int,
    end_frame: c_int,
    frac: f32,
    tag_name: *const c_char,
}

impl CgRLerptagArgs {
    pub const fn new(
        tag: *mut c_void,
        mod_: c_int,
        start_frame: c_int,
        end_frame: c_int,
        frac: f32,
        tag_name: *const c_char,
    ) -> Self {
        Self {
            tag,
            mod_,
            start_frame,
            end_frame,
            frac,
            tag_name,
        }
    }

    pub const fn frac(&self) -> f32 {
        self.frac
    }
}

/// `CG_R_LERPTAG` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:162`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:377-379`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:934-935`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:934-935`
pub struct CgRLerptag;

impl OutboundSysCall for CgRLerptag {
    type Import = MpCgameImport;
    type Args = CgRLerptagArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_LERPTAG;
}

impl EncodeSysCall for CgRLerptag {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.tag),
            args.mod_ as isize,
            args.start_frame as isize,
            args.end_frame as isize,
            pass_float(args.frac()),
            ptr_to_word(args.tag_name),
        ])
    }
}

impl DecodeSysCallReturn for CgRLerptag {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
