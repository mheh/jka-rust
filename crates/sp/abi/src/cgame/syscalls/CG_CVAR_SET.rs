use core::ffi::c_char;

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_CVAR_SET`.
///
/// Raven wrapper: `syscall( CG_CVAR_SET, var_name, value );`
/// Raven transport: `Cvar_Set( (const char *) VMA(1), (const char *) VMA(2) );`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:66-68`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:451-453`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgCvarSetArgs {
    var_name: *const c_char,
    value: *const c_char,
}

impl CgCvarSetArgs {
    /// # Safety
    /// `var_name` and `value` must point to valid NUL-terminated C strings.
    pub const unsafe fn new(var_name: *const c_char, value: *const c_char) -> Self {
        Self { var_name, value }
    }
}

/// `CG_CVAR_SET` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:66`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:66-68`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:451-453`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:451-453`
pub struct CgCvarSet;

impl OutboundSysCall for CgCvarSet {
    type Import = SpCgameImport;
    type Args = CgCvarSetArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_CVAR_SET;
}

impl EncodeSysCall for CgCvarSet {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.var_name), ptr_to_word(args.value)])
    }
}

impl DecodeSysCallReturn for CgCvarSet {
    fn decode_return(_word: isize) -> Self::Output {}
}
