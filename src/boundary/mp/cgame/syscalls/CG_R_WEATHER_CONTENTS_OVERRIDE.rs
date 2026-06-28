use super::super::MpCgameImport;
use crate::boundary::generic::OutboundSysCall;

/// `CG_R_WEATHER_CONTENTS_OVERRIDE` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:333`
pub struct CgRWeatherContentsOverride;

impl OutboundSysCall for CgRWeatherContentsOverride {
    type Import = MpCgameImport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_WEATHER_CONTENTS_OVERRIDE;
}
