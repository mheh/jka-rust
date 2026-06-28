use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_ARGS` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:72`
pub struct CgArgs;

impl OutboundSysCall for CgArgs {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_ARGS;
}
