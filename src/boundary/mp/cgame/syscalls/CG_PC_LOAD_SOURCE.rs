use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_PC_LOAD_SOURCE` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:200`
pub struct CgPcLoadSource;

impl OutboundSysCall for CgPcLoadSource {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_PC_LOAD_SOURCE;
}
