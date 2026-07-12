use core::ffi::{c_int, c_void};

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_MEMSET`.
///
/// Raven's cgame switch reads `dest` through `VMA(1)` and passes the remaining
/// words directly to `Com_Memset(VMA(1), args[2], args[3])`.
///
/// Args source: `oracle/codemp/client/cl_ui.cpp:650`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:624`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:649`
/// Shared trap token source: `oracle/codemp/qcommon/qcommon.h:282`
#[derive(Debug)]
pub struct UiMemsetArgs {
    /// Destination buffer pointer, decoded by Raven as `VMA(1)`.
    dest: *mut c_void,
    /// Fill byte value, read by Raven as `args[2]`.
    val: c_int,
    /// Number of bytes to fill, read by Raven as `args[3]`.
    count: c_int,
}

impl UiMemsetArgs {
    pub const fn new(dest: *mut c_void, val: c_int, count: c_int) -> Self {
        Self { dest, val, count }
    }

    pub const fn dest(&self) -> *mut c_void {
        self.dest
    }

    pub const fn val(&self) -> c_int {
        self.val
    }

    pub const fn count(&self) -> c_int {
        self.count
    }
}

/// `UI_MEMSET` MP cgame imports syscall ABI token.
///
/// Raven: `Com_Memset(VMA(1), args[2], args[3])`
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:130`
/// Args source: `oracle/codemp/client/cl_ui.cpp:650`
/// Output source: `oracle/codemp/client/cl_ui.cpp:651`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:649`
/// Shared trap token source: `oracle/codemp/qcommon/qcommon.h:282`
pub struct UiMemset;

impl OutboundSysCall for UiMemset {
    type Import = MpUiImport;
    type Args = UiMemsetArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_MEMSET;
}

impl EncodeSysCall for UiMemset {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.dest()),
            args.val() as isize,
            args.count() as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiMemset {
    // Raven calls `Com_Memset`, then returns 0; the helper has no semantic output.
    fn decode_return(_word: isize) -> Self::Output {}
}
