use core::ffi::c_int;

use super::super::MpCgameExport;
use abi_transport::generic::{
    word_to_c_int, DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport,
};

/// Arguments for `CG_CALC_LERP_POSITIONS`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CgCalcLerpPositionsArgs {
    entity_num: c_int,
}

impl CgCalcLerpPositionsArgs {
    pub const fn new(entity_num: c_int) -> Self {
        Self { entity_num }
    }

    pub const fn entity_num(self) -> c_int {
        self.entity_num
    }
}

/// `CG_CALC_LERP_POSITIONS` MP cgame exports vmMain ABI token.
///
/// Raven: void CG_CalcEntityLerpPositions(int num);
/// Enum value source: `oracle/codemp/cgame/cg_public.h:402-403`
/// Args source: `oracle/codemp/cgame/cg_main.c:239-241`
/// Output source: `oracle/codemp/cgame/cg_main.c:239-241`
/// Transport/switch source: no engine call-site found in initial search; module vmMain switch at `oracle/codemp/cgame/cg_main.c:239-241` proves arg0 entity index.
pub struct CgCalcLerpPositions;

impl InboundVmCall for CgCalcLerpPositions {
    type Command = MpCgameExport;
    type Args = CgCalcLerpPositionsArgs;
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_CALC_LERP_POSITIONS;
}

impl DecodeVmMain for CgCalcLerpPositions {
    fn decode_vm_main(transport: VmMainTransport) -> Self::Args {
        CgCalcLerpPositionsArgs::new(word_to_c_int(transport.arg(0)))
    }
}

impl EncodeVmMainReturn for CgCalcLerpPositions {
    fn encode_return(_output: Self::Output) -> isize {
        0
    }
}
