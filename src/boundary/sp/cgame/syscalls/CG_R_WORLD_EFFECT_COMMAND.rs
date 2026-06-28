use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_WORLD_EFFECT_COMMAND` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:183`
pub struct CgRWorldEffectCommand;

impl OutboundSysCall for CgRWorldEffectCommand {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_WORLD_EFFECT_COMMAND;
}
