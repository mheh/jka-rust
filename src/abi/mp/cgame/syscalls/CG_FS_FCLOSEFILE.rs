use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `CG_FS_FCLOSEFILE`.
///
/// C ABI: `void trap_FS_FCloseFile( fileHandle_t f )`
/// syscall: `syscall!(CG_FS_FCLOSEFILE, f)`
///
/// Sources:
/// - Args: `oracle/oracle/codemp/cgame/cg_syscalls.c:95-96`
/// - Output: `oracle/oracle/codemp/client/cl_cgame.cpp:747`
/// - Transport/switch: `oracle/oracle/codemp/client/cl_cgame.cpp:745-747`
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

/// `CG_FS_FCLOSEFILE` MP cgame imports syscall ABI token.
///
/// Raven: ( fileHandle_t f );
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:76`
pub struct CgFsFclosefile;

impl OutboundSysCall for CgFsFclosefile {
    type Import = MpCgameImport;
    type Args = CgFsFclosefileArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_FS_FCLOSEFILE;
}

impl EncodeSysCall for CgFsFclosefile {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.f as isize])
    }
}

impl DecodeSysCallReturn for CgFsFclosefile {
    fn decode_return(_word: isize) -> Self::Output {}
}
