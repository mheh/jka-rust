use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_OPENUIMENU` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:190`
pub struct CgOpenuimenu;

impl OutboundSysCall for CgOpenuimenu {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_OPENUIMENU;
}
