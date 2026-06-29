use core::ffi::{c_char, c_int};

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_CVAR_GETHIDDENVALUE`.
///
/// Raven wrapper: `int trap_Cvar_GetHiddenVarValue(const char *name)`.
/// Raven transport: `CL_GetValueForHidden((const char *)VMA(1))`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:66-68`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:726-727`
#[derive(Debug)]
pub struct CgCvarGethiddenvalueArgs {
    name: *const c_char,
}

impl CgCvarGethiddenvalueArgs {
    /// Construct raw `trap_Cvar_GetHiddenVarValue` syscall args.
    ///
    /// # Safety
    /// `name` must point to a valid NUL-terminated C string for the duration of
    /// the syscall.
    pub const unsafe fn new(name: *const c_char) -> Self {
        Self { name }
    }

    pub const fn name(&self) -> *const c_char {
        self.name
    }
}

/// `CG_CVAR_GETHIDDENVALUE` MP cgame imports syscall boundary token.
///
/// Raven wrapper: `return syscall(CG_CVAR_GETHIDDENVALUE, name);`
/// Raven transport: `return CL_GetValueForHidden((const char *)VMA(1));`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:69`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:66-68`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:726-727`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:726-727`
pub struct CgCvarGethiddenvalue;

impl OutboundSysCall for CgCvarGethiddenvalue {
    type Import = MpCgameImport;
    type Args = CgCvarGethiddenvalueArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_CVAR_GETHIDDENVALUE;
}

impl EncodeSysCall for CgCvarGethiddenvalue {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.name())])
    }
}

impl DecodeSysCallReturn for CgCvarGethiddenvalue {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
