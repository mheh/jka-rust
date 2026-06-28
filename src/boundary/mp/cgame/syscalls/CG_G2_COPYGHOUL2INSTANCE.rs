use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_G2_COPYGHOUL2INSTANCE` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:273`
pub struct CgG2Copyghoul2instance;

impl OutboundSysCall for CgG2Copyghoul2instance {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_COPYGHOUL2INSTANCE;
}
