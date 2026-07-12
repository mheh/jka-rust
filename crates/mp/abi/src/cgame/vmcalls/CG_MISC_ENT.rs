use super::super::shared_buffer::{SharedBufferPayload, TCGMiscEnt};
use super::super::MpCgameExport;
use abi_transport::generic::{EncodeVmMainReturn, InboundVmCall};

/// `CG_MISC_ENT` MP cgame exports vmMain ABI token.
///
/// Raven: rwwRMG - added
/// Enum source: `oracle/codemp/cgame/cg_public.h:435`
/// Shared-buffer source: `oracle/codemp/cgame/cg_public.h:521-526`
/// Args source: `oracle/codemp/cgame/cg_main.c:342-344`, `oracle/codemp/cgame/cg_main.c:582-587`
/// Output source: `oracle/codemp/cgame/cg_main.c:342-344`, `oracle/codemp/cgame/cg_main.c:599-621`
/// Transport/switch source: `oracle/codemp/cgame/cg_main.c:342-344`
/// Transport/call-site source: `oracle/codemp/RMG/RM_Terrain.cpp:447-451`
/// Shared-buffer payload type source: `oracle/codemp/cgame/cg_public.h:521-526`
pub struct CgMiscEnt;

impl InboundVmCall for CgMiscEnt {
    type Command = MpCgameExport;
    type Args = SharedBufferPayload<TCGMiscEnt>;
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_MISC_ENT;
}

impl EncodeVmMainReturn for CgMiscEnt {
    fn encode_return(_output: Self::Output) -> isize {
        0
    }
}
