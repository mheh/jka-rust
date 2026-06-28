use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_SNAPVECTOR` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:209`
pub struct CgSnapvector;

impl OutboundSysCall for CgSnapvector {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_SNAPVECTOR;
}
