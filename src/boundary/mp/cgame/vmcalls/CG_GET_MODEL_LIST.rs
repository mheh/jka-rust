use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_GET_MODEL_LIST` MP cgame exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:400`
pub struct CgGetModelList;

impl InboundVmCall for CgGetModelList {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_GET_MODEL_LIST;
}
