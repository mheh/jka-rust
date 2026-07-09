use core::ffi::{c_char, c_int};

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use sp_qshared::shared::vmCvar_t;

/// Arguments for `CG_CVAR_REGISTER`.
///
/// Raven wrapper: `syscall( CG_CVAR_REGISTER, vmCvar, varName, defaultValue, flags );`
/// Raven transport: `Cvar_Register( (vmCvar_t *) VMA(1), (const char *) VMA(2), (const char *) VMA(3), args[4] );`
///
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:58-60`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:445-447`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgCvarRegisterArgs {
    vm_cvar: *mut vmCvar_t,
    var_name: *const c_char,
    default_value: *const c_char,
    flags: c_int,
}

impl CgCvarRegisterArgs {
    /// # Safety
    /// `vm_cvar` must be valid for writes, and `var_name` and `default_value`
    /// must point to valid NUL-terminated C strings.
    pub const unsafe fn new(
        vm_cvar: *mut vmCvar_t,
        var_name: *const c_char,
        default_value: *const c_char,
        flags: c_int,
    ) -> Self {
        Self {
            vm_cvar,
            var_name,
            default_value,
            flags,
        }
    }
}

/// `CG_CVAR_REGISTER` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/code/cgame/cg_public.h:64`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:58-60`
/// Output source: `oracle/code/client/cl_cgame.cpp:445-447`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:445-447`
pub struct CgCvarRegister;

impl OutboundSysCall for CgCvarRegister {
    type Import = SpCgameImport;
    type Args = CgCvarRegisterArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_CVAR_REGISTER;
}

impl EncodeSysCall for CgCvarRegister {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.vm_cvar),
            ptr_to_word(args.var_name),
            ptr_to_word(args.default_value),
            args.flags as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgCvarRegister {
    fn decode_return(_word: isize) -> Self::Output {}
}
