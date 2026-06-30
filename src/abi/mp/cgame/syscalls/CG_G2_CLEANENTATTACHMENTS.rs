use super::super::MpCgameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `CG_G2_CLEANENTATTACHMENTS`.
///
/// Raven wrapper: `syscall(CG_G2_CLEANENTATTACHMENTS);`
/// Raven transport: `G2API_CleanEntAttachments(); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1070-1072`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2589`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1628-1630`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgG2CleanentattachmentsArgs;

impl CgG2CleanentattachmentsArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `CG_G2_CLEANENTATTACHMENTS` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:323`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1070-1072`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1628-1630`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1628-1630`
pub struct CgG2Cleanentattachments;

impl OutboundSysCall for CgG2Cleanentattachments {
    type Import = MpCgameImport;
    type Args = CgG2CleanentattachmentsArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_CLEANENTATTACHMENTS;
}

impl EncodeSysCall for CgG2Cleanentattachments {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgG2Cleanentattachments {
    fn decode_return(_word: isize) -> Self::Output {}
}
