use core::ffi::c_char;

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::qboolean;

/// Arguments for `CG_S_STARTBACKGROUNDTRACK`.
///
/// Raven: empty name stops music.
/// Raven wrapper: `syscall( CG_S_STARTBACKGROUNDTRACK, intro, loop, bReturnWithoutStarting )`.
/// Raven transport:
/// `S_StartBackgroundTrack( (const char *)VMA(1), (const char *)VMA(2), args[3]?qtrue:qfalse ); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:233-234`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2236`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:842-844`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSStartbackgroundtrackArgs {
    intro: *const c_char,
    loop_: *const c_char,
    b_return_without_starting: qboolean,
}

impl CgSStartbackgroundtrackArgs {
    pub const fn new(
        intro: *const c_char,
        loop_: *const c_char,
        b_return_without_starting: qboolean,
    ) -> Self {
        Self {
            intro,
            loop_,
            b_return_without_starting,
        }
    }

    pub const fn intro(&self) -> *const c_char {
        self.intro
    }

    pub const fn loop_(&self) -> *const c_char {
        self.loop_
    }

    pub const fn b_return_without_starting(&self) -> qboolean {
        self.b_return_without_starting
    }
}

/// `CG_S_STARTBACKGROUNDTRACK` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:107`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:233-234`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:842-844`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:842-844`
pub struct CgSStartbackgroundtrack;

impl OutboundSysCall for CgSStartbackgroundtrack {
    type Import = MpCgameImport;
    type Args = CgSStartbackgroundtrackArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_S_STARTBACKGROUNDTRACK;
}

impl EncodeSysCall for CgSStartbackgroundtrack {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.intro()),
            ptr_to_word(args.loop_()),
            args.b_return_without_starting() as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgSStartbackgroundtrack {
    fn decode_return(_word: isize) -> Self::Output {}
}
