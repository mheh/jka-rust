use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_S_UPDATEAMBIENTSET` MP cgame imports syscall boundary token.
///
/// Raven: rww - AS trap implem
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:110`
pub struct CgSUpdateambientset;

impl OutboundSysCall for CgSUpdateambientset {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_S_UPDATEAMBIENTSET;
}
