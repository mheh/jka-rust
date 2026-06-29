use super::super::MpCgameExport;
use crate::boundary::generic::{EncodeVmMainReturn, InboundVmCall};

/// `CG_MISC_ENT` MP cgame exports vmMain boundary token.
///
/// Raven: rwwRMG - added
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:435`
/// Args source: `oracle/oracle/codemp/cgame/cg_public.h:521-526`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:342-344`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:342-344`
/// Transport/call-site source: no engine call-site found in initial search; module vmMain switch proves return.
pub struct CgMiscEnt;

impl InboundVmCall for CgMiscEnt {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args; payload is TCGMiscEnt in cg.sharedBuffer/cl.mSharedMemory.
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_MISC_ENT;
}

impl EncodeVmMainReturn for CgMiscEnt {
    fn encode_return(_output: Self::Output) -> isize {
        0
    }
}
