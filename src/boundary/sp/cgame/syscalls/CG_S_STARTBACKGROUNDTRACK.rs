use core::ffi::c_char;

use super::super::SpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::qboolean;

/// Arguments for `CG_S_STARTBACKGROUNDTRACK`.
///
/// Raven: empty name stops music.
/// Raven wrapper: `syscall( CG_S_STARTBACKGROUNDTRACK, intro, loop, bForceStart );`
/// Raven transport: `S_StartBackgroundTrack((const char *) VMA(1), (const char *) VMA(2), args[3]);`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:233-234`
/// Args source: `oracle/oracle/code/cgame/cg_local.h:983`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:607-609`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSStartbackgroundtrackArgs {
    intro: *const c_char,
    loop_: *const c_char,
    b_force_start: qboolean,
}

impl CgSStartbackgroundtrackArgs {
    pub const fn new(intro: *const c_char, loop_: *const c_char, b_force_start: qboolean) -> Self {
        Self {
            intro,
            loop_,
            b_force_start,
        }
    }

    pub const fn intro(&self) -> *const c_char {
        self.intro
    }

    pub const fn loop_(&self) -> *const c_char {
        self.loop_
    }

    pub const fn b_force_start(&self) -> qboolean {
        self.b_force_start
    }
}

/// `CG_S_STARTBACKGROUNDTRACK` SP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:99`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:233-234`
/// Args source: `oracle/oracle/code/cgame/cg_local.h:983`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:607-609`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:607-609`
pub struct CgSStartbackgroundtrack;

impl OutboundSysCall for CgSStartbackgroundtrack {
    type Import = SpCgameImport;
    type Args = CgSStartbackgroundtrackArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_S_STARTBACKGROUNDTRACK;
}

impl EncodeSysCall for CgSStartbackgroundtrack {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.intro()),
            ptr_to_word(args.loop_()),
            args.b_force_start() as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgSStartbackgroundtrack {
    fn decode_return(_word: isize) -> Self::Output {}
}
