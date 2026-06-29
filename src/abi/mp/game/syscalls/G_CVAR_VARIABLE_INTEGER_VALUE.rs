use core::ffi::c_int;
use std::ffi::CString;

use crate::ffi::GameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `G_CVAR_VARIABLE_INTEGER_VALUE`.
#[derive(Debug)]
pub struct GCvarVariableIntegerValueArgs {
    var_name: CString,
}

impl GCvarVariableIntegerValueArgs {
    pub fn new(var_name: impl Into<CString>) -> Self {
        Self {
            var_name: var_name.into(),
        }
    }

    pub fn var_name(&self) -> &CString {
        &self.var_name
    }
}

/// `G_CVAR_VARIABLE_INTEGER_VALUE` MP game imports syscall ABI token.
///
/// Raven: ( const char *var_name );
/// Source: `oracle/oracle/codemp/game/g_public.h:124`
pub struct GCvarVariableIntegerValue;

impl OutboundSysCall for GCvarVariableIntegerValue {
    type Import = GameImport;
    type Args = GCvarVariableIntegerValueArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_CVAR_VARIABLE_INTEGER_VALUE;
}

impl EncodeSysCall for GCvarVariableIntegerValue {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.var_name().as_ptr())])
    }
}

impl DecodeSysCallReturn for GCvarVariableIntegerValue {
    // `trap_Cvar_VariableIntegerValue` returns `int`.
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
