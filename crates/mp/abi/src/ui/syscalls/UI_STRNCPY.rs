use super::super::MpUiImport;
use core::ffi::{c_char, c_int};

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_STRNCPY`.
///
/// Raven's MP client switch reads `dest` with `VMA(1)`, `src` with `VMA(2)`,
/// and `count` from `args[3]`, then calls C `strncpy`.
///
/// Args source: `oracle/codemp/client/cl_ui.cpp:656`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:655`
/// Shared trap token source: `oracle/codemp/qcommon/qcommon.h:284`
#[derive(Debug)]
pub struct UiStrncpyArgs {
    dest: *mut c_char,
    src: *const c_char,
    count: c_int,
}

impl UiStrncpyArgs {
    /// Construct the raw `strncpy` syscall args.
    ///
    /// # Safety
    /// `dest` must be valid for writes of up to `count` bytes, `src` must be a
    /// valid C string readable for the same operation, and the buffers must obey
    /// C `strncpy` aliasing requirements.
    pub const unsafe fn new(dest: *mut c_char, src: *const c_char, count: c_int) -> Self {
        Self { dest, src, count }
    }

    pub const fn dest(&self) -> *mut c_char {
        self.dest
    }

    pub const fn src(&self) -> *const c_char {
        self.src
    }

    pub const fn count(&self) -> c_int {
        self.count
    }
}

/// `UI_STRNCPY` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:132`
/// Output source: `oracle/codemp/client/cl_ui.cpp:656`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:655`
/// Shared trap token source: `oracle/codemp/qcommon/qcommon.h:284`
pub struct UiStrncpy;

impl OutboundSysCall for UiStrncpy {
    type Import = MpUiImport;
    type Args = UiStrncpyArgs;
    type Output = *mut c_char;

    const IMPORT: MpUiImport = MpUiImport::UI_STRNCPY;
}

impl EncodeSysCall for UiStrncpy {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.dest()),
            ptr_to_word(args.src()),
            args.count() as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiStrncpy {
    fn decode_return(word: isize) -> Self::Output {
        word as *mut c_char
    }
}
