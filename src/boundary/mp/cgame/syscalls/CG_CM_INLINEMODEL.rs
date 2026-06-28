use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_CM_INLINEMODEL` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:85`
pub struct CgCmInlinemodel;

impl OutboundSysCall for CgCmInlinemodel {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_CM_INLINEMODEL;
}
