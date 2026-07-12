use core::ffi::c_int;

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_MEMCPY`.
///
/// Raven transport: `Com_Memcpy(VMA(1), VMA(2), args[3])`.
///
/// Args source: `oracle/codemp/client/cl_ui.cpp:653`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:652`
/// Shared trap token source: `oracle/codemp/qcommon/qcommon.h:283`
#[derive(Debug)]
pub struct UiMemcpyArgs {
    /// Destination buffer read through `VMA(1)`.
    dest: *mut u8,
    /// Source buffer read through `VMA(2)`.
    src: *const u8,
    /// Number of bytes copied from `args[3]`.
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

/// `UI_MEMCPY` MP cgame imports syscall ABI token.
///
/// Raven: "DO NOT EVER add a GAME/CGAME/UI generic call without adding a trap
/// to match"; generic traps are shared and ordered from 100.
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:131`
/// Output source: `oracle/codemp/client/cl_ui.cpp:654`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:652`
/// Shared trap token source: `oracle/codemp/qcommon/qcommon.h:283`
pub struct UiMemcpy;

impl OutboundSysCall for UiMemcpy {
    type Import = MpUiImport;
    type Args = UiMemcpyArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_MEMCPY;
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
