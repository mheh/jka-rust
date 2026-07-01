use core::ffi::{c_char, c_int, c_void};

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use abi_transport::pass_float;
use sp_qshared::shared::qhandle_t;

/// Arguments for `CG_R_LERPTAG`.
///
/// Raven wrapper:
/// `cgi_R_LerpTag( orientation_t *tag, qhandle_t mod, int startFrame, int endFrame, float frac, const char *tagName )`
/// Raven transport: `re.LerpTag( (orientation_t *) VMA(1), args[2], args[3], args[4], VMF(5), (const char *) VMA(6) );`
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:144`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:409-411`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:723-725`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:723-725`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgRLerptagArgs {
    tag: *mut c_void,
    mod_: qhandle_t,
    start_frame: c_int,
    end_frame: c_int,
    frac: f32,
    tag_name: *const c_char,
}

impl CgRLerptagArgs {
    pub const fn new(
        tag: *mut c_void,
        mod_: qhandle_t,
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

    pub const fn tag(&self) -> *mut c_void {
        self.tag
    }

    pub const fn mod_(&self) -> qhandle_t {
        self.mod_
    }

    pub const fn start_frame(&self) -> c_int {
        self.start_frame
    }

    pub const fn end_frame(&self) -> c_int {
        self.end_frame
    }

    pub const fn frac(&self) -> f32 {
        self.frac
    }

    pub const fn tag_name(&self) -> *const c_char {
        self.tag_name
    }
}

/// `CG_R_LERPTAG` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:144`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:409-411`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:723-725`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:723-725`
pub struct CgRLerptag;

impl OutboundSysCall for CgRLerptag {
    type Import = SpCgameImport;
    type Args = CgRLerptagArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_LERPTAG;
}

impl EncodeSysCall for CgRLerptag {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.tag()),
            args.mod_() as isize,
            args.start_frame() as isize,
            args.end_frame() as isize,
            pass_float(args.frac()),
            ptr_to_word(args.tag_name()),
        ])
    }
}

impl DecodeSysCallReturn for CgRLerptag {
    fn decode_return(_word: isize) -> Self::Output {}
}
