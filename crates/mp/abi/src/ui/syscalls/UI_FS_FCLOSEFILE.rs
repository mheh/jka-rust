use core::ffi::c_int;

use super::super::MpUiImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `UI_FS_FCLOSEFILE`.
///
/// C ABI: `void trap_FS_FCloseFile( fileHandle_t f )`
/// syscall: `syscall!(UI_FS_FCLOSEFILE, f)`
///
/// Sources:
/// - Args: `oracle/codemp/ui/ui_syscalls.c:95-96`
/// - Output: `oracle/codemp/client/cl_ui.cpp:747`
/// - Transport/switch: `oracle/codemp/client/cl_ui.cpp:745-747`
#[derive(Debug)]
pub struct CgFsFclosefileArgs {
    /// File handle to close (`fileHandle_t`, which is `int` in C).
    pub f: c_int,
}

impl CgFsFclosefileArgs {
    pub fn new(f: c_int) -> Self {
        Self { f }
    }

    pub fn f(&self) -> c_int {
        self.f
    }
}

/// `UI_FS_FCLOSEFILE` MP cgame imports syscall ABI token.
///
/// Raven: ( fileHandle_t f );
/// Source: `oracle/codemp/ui/ui_public.h:76`
pub struct CgFsFclosefile;

impl OutboundSysCall for CgFsFclosefile {
    type Import = MpUiImport;
    type Args = CgFsFclosefileArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_FS_FCLOSEFILE;
}

impl EncodeSysCall for CgFsFclosefile {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.f as isize])
    }
}

impl DecodeSysCallReturn for CgFsFclosefile {
    fn decode_return(_word: isize) -> Self::Output {}
}
