use core::ffi::{c_char, c_int};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_KEY_GETKEY`.
///
/// Raven wrapper: `int trap_Key_GetKey( const char *binding )`.
/// Raven transport: `Key_GetKey( (const char *)VMA(1) )`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:537-538`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:996-997`
#[derive(Debug)]
pub struct CgKeyGetkeyArgs {
    binding: *const c_char,
}

impl CgKeyGetkeyArgs {
    /// Construct raw `trap_Key_GetKey` syscall args.
    ///
    /// # Safety
    /// `binding` must point to a valid NUL-terminated C string for the duration
    /// of the syscall.
    pub const unsafe fn new(binding: *const c_char) -> Self {
        Self { binding }
    }

    pub const fn binding(&self) -> *const c_char {
        self.binding
    }
}

/// `CG_KEY_GETKEY` MP cgame imports syscall ABI token.
///
/// Raven wrapper: `return syscall( CG_KEY_GETKEY, binding );`
/// Raven transport: `return Key_GetKey( (const char *)VMA(1) );`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:197`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:537-538`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:996-997`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:996-997`
pub struct CgKeyGetkey;

impl OutboundSysCall for CgKeyGetkey {
    type Import = MpCgameImport;
    type Args = CgKeyGetkeyArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_KEY_GETKEY;
}

impl EncodeSysCall for CgKeyGetkey {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.binding())])
    }
}

impl DecodeSysCallReturn for CgKeyGetkey {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
