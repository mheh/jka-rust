use core::ffi::c_char;

use super::super::SpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_UI_PARSEEXT`.
///
/// Raven wrapper: `cgi_UI_ParseExt(token);`
/// Raven transport: `char **holdPtr; holdPtr = (char **) VMA(1); *holdPtr = PC_ParseExt(); return 0;`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:603-605`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:876-881`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgUiParseextArgs {
    token: *mut *mut c_char,
}

impl CgUiParseextArgs {
    /// # Safety
    /// `token` must point to a writable pointer slot that can be overwritten.
    pub const unsafe fn new(token: *mut *mut c_char) -> Self {
        Self { token }
    }

    pub const fn token(&self) -> *mut *mut c_char {
        self.token
    }
}

/// `CG_UI_PARSEEXT` SP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:201`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:603-605`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:876-881`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:876-881`
pub struct CgUiParseext;

impl OutboundSysCall for CgUiParseext {
    type Import = SpCgameImport;
    type Args = CgUiParseextArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_UI_PARSEEXT;
}

impl EncodeSysCall for CgUiParseext {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.token())])
    }
}

impl DecodeSysCallReturn for CgUiParseext {
    fn decode_return(_word: isize) -> Self::Output {}
}
