use core::ffi::c_char;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_CVAR_SET`.
///
/// Raven wrapper: `void trap_Cvar_Set( const char *var_name, const char *value )`.
/// The MP client switch forwards both strings through `VMA` into `Cvar_Set`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:58-59`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:720-722`
#[derive(Debug)]
pub struct CgCvarSetArgs {
    var_name: *const c_char,
    value: *const c_char,
}

impl CgCvarSetArgs {
    /// Construct raw `trap_Cvar_Set` syscall args.
    ///
    /// # Safety
    /// `var_name` and `value` must point to valid NUL-terminated C strings for
    /// the duration of the syscall.
    pub const unsafe fn new(var_name: *const c_char, value: *const c_char) -> Self {
        Self { var_name, value }
    }

    pub const fn var_name(&self) -> *const c_char {
        self.var_name
    }

    pub const fn value(&self) -> *const c_char {
        self.value
    }
}

/// `CG_CVAR_SET` MP cgame imports syscall ABI token.
///
/// Raven wrapper: `syscall( CG_CVAR_SET, var_name, value );`
/// Raven transport: `Cvar_Set( (const char *)VMA(1), (const char *)VMA(2) );`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:67`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:58-59`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:722`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:720-722`
pub struct CgCvarSet;

impl OutboundSysCall for CgCvarSet {
    type Import = MpCgameImport;
    type Args = CgCvarSetArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_CVAR_SET;
}

impl EncodeSysCall for CgCvarSet {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.var_name()), ptr_to_word(args.value())])
    }
}

impl DecodeSysCallReturn for CgCvarSet {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
