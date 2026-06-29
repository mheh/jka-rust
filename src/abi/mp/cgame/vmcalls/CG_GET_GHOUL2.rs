use core::ffi::{c_int, c_void};

use super::super::MpCgameExport;
use crate::abi::generic::{
    word_to_c_int, DecodeVmMain, EncodeVmMainReturn, InboundVmCall, VmMainTransport,
};

/// Arguments for `CG_GET_GHOUL2`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:231-234`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CgGetGhoul2Args {
    ent_num: c_int,
}

impl CgGetGhoul2Args {
    pub const fn new(ent_num: c_int) -> Self {
        Self { ent_num }
    }

    pub const fn ent_num(self) -> c_int {
        self.ent_num
    }
}

/// `CG_GET_GHOUL2` MP cgame exports vmMain ABI token.
///
/// Raven: used by effect bolting, which is actually not used at all.
/// Raven: use at your own risk.
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:399`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:231-234`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:231-234`
/// Transport/call-site source: no engine call-site found in initial search; module vmMain switch proves arg slots.
pub struct CgGetGhoul2;

impl InboundVmCall for CgGetGhoul2 {
    type Command = MpCgameExport;
    type Args = CgGetGhoul2Args;
    type Output = *mut c_void;

    const COMMAND: MpCgameExport = MpCgameExport::CG_GET_GHOUL2;
}

impl DecodeVmMain for CgGetGhoul2 {
    fn decode_vm_main(transport: VmMainTransport) -> Self::Args {
        CgGetGhoul2Args::new(word_to_c_int(transport.arg(0)))
    }
}

impl EncodeVmMainReturn for CgGetGhoul2 {
    fn encode_return(output: Self::Output) -> isize {
        output as isize
    }
}
