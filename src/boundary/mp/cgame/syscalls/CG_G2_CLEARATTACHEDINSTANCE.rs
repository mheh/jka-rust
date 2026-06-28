use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_G2_CLEARATTACHEDINSTANCE` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:322`
pub struct CgG2Clearattachedinstance;

impl OutboundSysCall for CgG2Clearattachedinstance {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_CLEARATTACHEDINSTANCE;
}
