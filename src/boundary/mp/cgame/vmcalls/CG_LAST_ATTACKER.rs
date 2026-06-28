use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_LAST_ATTACKER` MP cgame exports vmMain boundary token.
///
/// Raven: int (*CG_CrosshairPlayer)( void );
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:381`
pub struct CgLastAttacker;

impl InboundVmCall for CgLastAttacker {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_LAST_ATTACKER;
}
