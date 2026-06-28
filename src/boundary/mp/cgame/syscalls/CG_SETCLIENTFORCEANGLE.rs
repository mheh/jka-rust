use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_SETCLIENTFORCEANGLE` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:188`
pub struct CgSetclientforceangle;

impl OutboundSysCall for CgSetclientforceangle {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_SETCLIENTFORCEANGLE;
}
