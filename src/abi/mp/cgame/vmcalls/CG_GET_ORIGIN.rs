use core::ffi::c_int;

use super::super::MpCgameExport;
use crate::abi::generic::{
    word_to_c_int, word_to_mut_ptr, DecodeVmMain, EncodeVmMainReturn, InboundVmCall,
    VmMainTransport,
};
use crate::shared::vec3_t;

/// Arguments for `CG_GET_ORIGIN`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:285-287`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:285-287`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CgGetOriginArgs {
    ent_num: c_int,
    origin: *mut vec3_t,
}

impl CgGetOriginArgs {
    pub const fn new(ent_num: c_int, origin: *mut vec3_t) -> Self {
        Self { ent_num, origin }
    }

    pub const fn ent_num(self) -> c_int {
        self.ent_num
    }

    pub const fn origin(self) -> *mut vec3_t {
        self.origin
    }
}

/// `CG_GET_ORIGIN` MP cgame exports vmMain ABI token.
///
/// Raven: int entnum, vec3_t origin
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:418`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:285-287`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:285-287`
/// Transport/call-site source: no engine call-site found in initial search; module vmMain switch proves arg slots.
pub struct CgGetOrigin;

impl InboundVmCall for CgGetOrigin {
    type Command = MpCgameExport;
    type Args = CgGetOriginArgs;
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_GET_ORIGIN;
}

impl DecodeVmMain for CgGetOrigin {
    fn decode_vm_main(transport: VmMainTransport) -> Self::Args {
        CgGetOriginArgs::new(
            word_to_c_int(transport.arg(0)),
            word_to_mut_ptr(transport.arg(1)),
        )
    }
}

impl EncodeVmMainReturn for CgGetOrigin {
    fn encode_return(_output: Self::Output) -> isize {
        0
    }
}
