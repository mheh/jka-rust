use super::super::SpCgameImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::fileHandle_t;

/// Arguments for `CG_FS_FCLOSEFILE`.
///
/// Raven wrapper: `syscall( CG_FS_FCLOSEFILE, f );`
/// Raven transport: `FS_FCloseFile( args[1] );`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:94-96`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:470-472`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgFsFclosefileArgs {
    file: fileHandle_t,
}

impl CgFsFclosefileArgs {
    pub const fn new(file: fileHandle_t) -> Self {
        Self { file }
    }
}

/// `CG_FS_FCLOSEFILE` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:73`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:94-96`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:470-472`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:470-472`
pub struct CgFsFclosefile;

impl OutboundSysCall for CgFsFclosefile {
    type Import = SpCgameImport;
    type Args = CgFsFclosefileArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_FS_FCLOSEFILE;
}

impl EncodeSysCall for CgFsFclosefile {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.file as isize])
    }
}

impl DecodeSysCallReturn for CgFsFclosefile {
    fn decode_return(_word: isize) -> Self::Output {}
}
