use core::ffi::c_int;

use super::super::SpUiImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_MEMCPY`.
///
/// Raven transport: `Com_Memcpy(VMA(1), VMA(2), args[3]); return 0;`
///
/// Enum source: `oracle/oracle/code/ui/ui_public.h:240`
/// Args source (fallback): `oracle/oracle/code/client/cl_ui.cpp` does not implement `UI_MEMCPY`;
/// fallback transport: `oracle/oracle/codemp/client/cl_ui.cpp:822-823`
/// Shared token source: `oracle/oracle/codemp/qcommon/qcommon.h:282-283`
/// Output source (fallback): `oracle/oracle/codemp/client/cl_ui.cpp:822-823`
#[derive(Debug)]
pub struct UiMemcpyArgs {
    /// Destination buffer pointer, read from `VMA(1)`.
    dest: *mut u8,
    /// Source buffer pointer, read from `VMA(2)`.
    src: *const u8,
    /// Number of bytes copied, read from `args[3]`.
    count: c_int,
}

impl UiMemcpyArgs {
    pub const fn new(dest: *mut u8, src: *const u8, count: c_int) -> Self {
        Self { dest, src, count }
    }

    pub const fn dest(&self) -> *mut u8 {
        self.dest
    }

    pub const fn src(&self) -> *const u8 {
        self.src
    }

    pub const fn count(&self) -> c_int {
        self.count
    }
}

/// `UI_MEMCPY` SP UI imports syscall boundary token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:240`
pub struct UiMemcpy;

impl OutboundSysCall for UiMemcpy {
    type Import = SpUiImport;
    type Args = UiMemcpyArgs;
    type Output = c_int;

    const IMPORT: SpUiImport = SpUiImport::UI_MEMCPY;
}

impl EncodeSysCall for UiMemcpy {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.dest() as *const u8),
            ptr_to_word(args.src()),
            args.count() as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiMemcpy {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
