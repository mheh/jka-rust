use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_REGISTERSHADER` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:119`
pub struct CgRRegistershader;

impl OutboundSysCall for CgRRegistershader {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_REGISTERSHADER;
}
