use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_GETUSERCMD` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:186`
pub struct CgGetusercmd;

impl OutboundSysCall for CgGetusercmd {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_GETUSERCMD;
}
