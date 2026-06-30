use core::ffi::c_char;

use super::super::MpUiImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::qboolean;

/// Arguments for `UI_S_STARTBACKGROUNDTRACK`.
///
/// Raven wrapper: `syscall( UI_S_STARTBACKGROUNDTRACK, intro, loop, bReturnWithoutStarting );`
/// Raven transport: `S_StartBackgroundTrack( (const char *)VMA(1), (const char *)VMA(2), qfalse); return 0;`
///
/// The MP UI wrapper sends `bReturnWithoutStarting`, but the MP client switch
/// ignores `args[3]` and passes `qfalse`.
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:396-397`
/// Args source: `oracle/oracle/codemp/ui/ui_local.h:1001`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1176-1178`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiSStartbackgroundtrackArgs {
    intro: *const c_char,
    loop_: *const c_char,
    b_return_without_starting: qboolean,
}

impl UiSStartbackgroundtrackArgs {
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

/// `UI_S_STARTBACKGROUNDTRACK` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:93`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:396-397`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1176-1178`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1176-1178`
pub struct UiSStartbackgroundtrack;

impl OutboundSysCall for UiSStartbackgroundtrack {
    type Import = MpUiImport;
    type Args = UiSStartbackgroundtrackArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_S_STARTBACKGROUNDTRACK;
}

impl EncodeSysCall for UiSStartbackgroundtrack {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.intro()),
            ptr_to_word(args.loop_()),
            args.b_return_without_starting() as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiSStartbackgroundtrack {
    fn decode_return(_word: isize) -> Self::Output {}
}
