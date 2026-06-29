use core::ffi::c_void;

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::qhandle_t;

/// Arguments for `CG_G2_SETMODELS`.
///
/// Raven wrapper: `syscall( CG_G2_SETMODELS, ghoul2, modelList, skinList);`
/// Raven transport: `G2API_SetGhoul2ModelIndexes( *((CGhoul2Info_v *)args[1]),(qhandle_t *)VMA(2),(qhandle_t *)VMA(3));`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:781-783`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2517`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1307-1309`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2SetmodelsArgs {
    ghoul2: *mut c_void,
    model_list: *mut qhandle_t,
    skin_list: *mut qhandle_t,
}

impl CgG2SetmodelsArgs {
    pub const fn new(
        ghoul2: *mut c_void,
        model_list: *mut qhandle_t,
        skin_list: *mut qhandle_t,
    ) -> Self {
        Self {
            ghoul2,
            model_list,
            skin_list,
        }
    }
}

/// `CG_G2_SETMODELS` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:258`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:781-783`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1307-1309`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1307-1309`
pub struct CgG2Setmodels;

impl OutboundSysCall for CgG2Setmodels {
    type Import = MpCgameImport;
    type Args = CgG2SetmodelsArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_SETMODELS;
}

impl EncodeSysCall for CgG2Setmodels {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ghoul2),
            ptr_to_word(args.model_list),
            ptr_to_word(args.skin_list),
        ])
    }
}

impl DecodeSysCallReturn for CgG2Setmodels {
    fn decode_return(_word: isize) -> Self::Output {}
}
