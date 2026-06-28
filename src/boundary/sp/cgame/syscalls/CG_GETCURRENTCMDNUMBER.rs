use super::super::SpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_GETCURRENTCMDNUMBER` SP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/code/cgame/cg_public.h:158`
pub struct CgGetcurrentcmdnumber;

impl OutboundSysCall for CgGetcurrentcmdnumber {
    type Import = SpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: SpCgameImport = SpCgameImport::CG_GETCURRENTCMDNUMBER;
}
