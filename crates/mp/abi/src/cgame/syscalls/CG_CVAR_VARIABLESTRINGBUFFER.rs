use super::super::MpCgameImport;
use core::ffi::{c_char, c_int};

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_CVAR_VARIABLESTRINGBUFFER`.
///
/// Raven cgame calls `syscall( CG_CVAR_VARIABLESTRINGBUFFER, var_name, buffer, bufsize )`.
/// The client switch decodes `var_name` and `buffer` with `VMA`, reads `bufsize`
/// directly from `args[3]`, and writes the cvar string into the provided buffer.
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:62-63`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:723-724`
#[derive(Debug)]
pub struct CgCvarVariablestringbufferArgs {
    var_name: *const c_char,
    buffer: *mut c_char,
    bufsize: c_int,
}

impl CgCvarVariablestringbufferArgs {
    /// Construct raw `trap_Cvar_VariableStringBuffer` syscall args.
    ///
    /// # Safety
    /// `var_name` must point to a valid NUL-terminated C string, and `buffer`
    /// must be valid for writes of up to `bufsize` bytes for the duration of
    /// the syscall.
    pub const unsafe fn new(var_name: *const c_char, buffer: *mut c_char, bufsize: c_int) -> Self {
        Self {
            var_name,
            buffer,
            bufsize,
        }
    }

    pub const fn var_name(&self) -> *const c_char {
        self.var_name
    }

    pub const fn buffer(&self) -> *mut c_char {
        self.buffer
    }

    pub const fn bufsize(&self) -> c_int {
        self.bufsize
    }
}

/// `CG_CVAR_VARIABLESTRINGBUFFER` MP cgame imports syscall ABI token.
///
/// Raven: `( const char *var_name, char *buffer, int bufsize )`.
/// Enum value source: `oracle/codemp/cgame/cg_public.h:68`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:62-63`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:725`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:723-724`
pub struct CgCvarVariablestringbuffer;

impl OutboundSysCall for CgCvarVariablestringbuffer {
    type Import = MpCgameImport;
    type Args = CgCvarVariablestringbufferArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_CVAR_VARIABLESTRINGBUFFER;
}

impl EncodeSysCall for CgCvarVariablestringbuffer {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.var_name()),
            ptr_to_word(args.buffer()),
            args.bufsize() as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgCvarVariablestringbuffer {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
