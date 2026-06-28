use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_S_REGISTERSOUND` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:106`
pub struct CgSRegistersound;

impl OutboundSysCall for CgSRegistersound {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_S_REGISTERSOUND;
}
