use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_KEY_GETKEY` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:197`
pub struct CgKeyGetkey;

impl OutboundSysCall for CgKeyGetkey {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_KEY_GETKEY;
}
