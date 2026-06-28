use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_WE_ADDWEATHERZONE` MP cgame imports syscall boundary token.
///
/// Raven: Adding trap to get weather working
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:336`
pub struct CgWeAddweatherzone;

impl OutboundSysCall for CgWeAddweatherzone {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_WE_ADDWEATHERZONE;
}
