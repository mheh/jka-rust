use super::super::MpUiImport;
use core::ffi::{c_char, c_int};

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::vmCvar_t;

/// Arguments for `UI_CVAR_REGISTER`.
///
/// Raven cgame calls `syscall( UI_CVAR_REGISTER, vmCvar, varName, defaultValue, flags )`.
/// The client switch decodes `vmCvar`, `var_name`, and `default_value` with
/// `VMA`, then reads `flags` directly from the fourth argument word.
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:50-51`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:714-715`
#[derive(Debug)]
pub struct CgCvarRegisterArgs {
    /// Optional module-side cvar mirror populated by the engine.
    cvar: *mut vmCvar_t,
    var_name: *const c_char,
    default_value: *const c_char,
    flags: c_int,
}

impl CgCvarRegisterArgs {
    /// Construct raw `trap_Cvar_Register` syscall args.
    ///
    /// # Safety
    /// `var_name` and `default_value` must point to valid NUL-terminated C
    /// strings for the duration of the syscall. `cvar` must be valid for any
    /// in-place mirror update performed by the engine.
    pub const unsafe fn new(
        cvar: *mut vmCvar_t,
        var_name: *const c_char,
        default_value: *const c_char,
        flags: c_int,
    ) -> Self {
        Self {
            cvar,
            var_name,
            default_value,
            flags,
        }
    }

    pub const fn cvar(&self) -> *mut vmCvar_t {
        self.cvar
    }

    pub const fn var_name(&self) -> *const c_char {
        self.var_name
    }

    pub const fn default_value(&self) -> *const c_char {
        self.default_value
    }

    pub const fn flags(&self) -> c_int {
        self.flags
    }
}

/// `UI_CVAR_REGISTER` MP cgame imports syscall ABI token.
///
/// Raven: `( vmCvar_t *vmCvar, const char *varName, const char *defaultValue, int flags )`.
/// Enum value source: `oracle/codemp/ui/ui_public.h:65`
/// Args source: `oracle/codemp/ui/ui_syscalls.c:50-51`
/// Output source: `oracle/codemp/client/cl_ui.cpp:716`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:714-715`
pub struct CgCvarRegister;

impl OutboundSysCall for CgCvarRegister {
    type Import = MpUiImport;
    type Args = CgCvarRegisterArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_CVAR_REGISTER;
}

impl EncodeSysCall for CgCvarRegister {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.cvar()),
            ptr_to_word(args.var_name()),
            ptr_to_word(args.default_value()),
            args.flags() as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgCvarRegister {
    // `trap_Cvar_Register` is `void`; Raven's switch returns 0 after registration.
    fn decode_return(_word: isize) -> Self::Output {}
}
