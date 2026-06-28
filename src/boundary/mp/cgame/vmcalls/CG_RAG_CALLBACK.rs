use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_RAG_CALLBACK` MP cgame exports vmMain boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:412`
pub struct CgRagCallback;

impl InboundVmCall for CgRagCallback {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_RAG_CALLBACK;
}
