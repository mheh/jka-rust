use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_ADDCOMMAND` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:79`
pub struct CgAddcommand;

impl OutboundSysCall for CgAddcommand {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_ADDCOMMAND;
}
