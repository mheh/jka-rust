use core::ffi::c_int;

use super::super::MpCgameExport;
use crate::abi::generic::{
    word_to_c_int, word_to_mut_ptr, DecodeVmMain, EncodeVmMainReturn, InboundVmCall,
    VmMainTransport,
};
use crate::shared::vec3_t;

/// Arguments for `CG_GET_ANGLES`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:289-291`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:289-291`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CgGetAnglesArgs {
    ent_num: c_int,
    angles: *mut vec3_t,
}

impl CgGetAnglesArgs {
    pub const fn new(ent_num: c_int, angles: *mut vec3_t) -> Self {
        Self { ent_num, angles }
    }

    pub const fn ent_num(self) -> c_int {
        self.ent_num
    }

    pub const fn angles(self) -> *mut vec3_t {
        self.angles
    }
}

/// `CG_GET_ANGLES` MP cgame exports vmMain ABI token.
///
/// Raven: int entnum, vec3_t angle
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:419`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:289-291`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:289-291`
/// Transport/call-site source: no engine call-site found in initial search; module vmMain switch proves arg slots.
pub struct CgGetAngles;

impl InboundVmCall for CgGetAngles {
    type Command = MpCgameExport;
    type Args = CgGetAnglesArgs;
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_GET_ANGLES;
}

impl DecodeVmMain for CgGetAngles {
    fn decode_vm_main(transport: VmMainTransport) -> Self::Args {
        CgGetAnglesArgs::new(
            word_to_c_int(transport.arg(0)),
            word_to_mut_ptr(transport.arg(1)),
        )
    }
}

impl EncodeVmMainReturn for CgGetAngles {
    fn encode_return(_output: Self::Output) -> isize {
        0
    }
}
