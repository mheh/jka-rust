use core::ffi::c_int;
use std::ffi::CString;

use crate::ffi::types::vmCvar_t;
use crate::ffi::GameImport;

use super::super::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `G_CVAR_REGISTER`.
///
/// `cvar` may be null when the caller wants the engine to register the cvar
/// without keeping a module-side `vmCvar_t` mirror.
#[derive(Debug)]
pub struct GCvarRegisterArgs {
    cvar: *mut vmCvar_t,
    var_name: CString,
    default_value: CString,
    flags: c_int,
}

impl GCvarRegisterArgs {
    pub fn new(
        cvar: *mut vmCvar_t,
        var_name: impl Into<CString>,
        default_value: impl Into<CString>,
        flags: c_int,
    ) -> Self {
        Self {
            cvar,
            var_name: var_name.into(),
            default_value: default_value.into(),
            flags,
        }
    }

    pub const fn cvar(&self) -> *mut vmCvar_t {
        self.cvar
    }

    pub fn var_name(&self) -> &CString {
        &self.var_name
    }

    pub fn default_value(&self) -> &CString {
        &self.default_value
    }

    pub const fn flags(&self) -> c_int {
        self.flags
    }
}

/// `G_CVAR_REGISTER` outbound game-to-engine syscall.
pub struct GCvarRegister;

impl OutboundSysCall for GCvarRegister {
    type Args = GCvarRegisterArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_CVAR_REGISTER;
}

impl EncodeSysCall for GCvarRegister {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.cvar()),
            ptr_to_word(args.var_name().as_ptr()),
            ptr_to_word(args.default_value().as_ptr()),
            args.flags() as isize,
        ])
    }
}

impl DecodeSysCallReturn for GCvarRegister {
    // `trap_Cvar_Register` is `void`; the engine's return word carries nothing.
    fn decode_return(_word: isize) -> Self::Output {}
}
