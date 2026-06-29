use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::abi::sp::cgame::types::CGhoul2Info_v;
use crate::codemp::game::q_shared_h::qhandle_t;

/// Arguments for `CG_G2_SETMODELS`.
///
/// Raven wrapper: `syscall( CG_G2_SETMODELS, &ghoul2, modelList, skinList );`
/// Raven transport: `G2API_SetGhoul2ModelIndexes( *((CGhoul2Info_v *)VMA(1) ),(qhandle_t *)VMA(2),(qhandle_t *)VMA(3));`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:486`
/// Args source: `oracle/oracle/code/cgame/cg_local.h:1186`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:794-796`
/// Type definition source: `oracle/oracle/code/game/ghoul2_shared.h:311`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2SetmodelsArgs {
    /// Opaque Raven `CGhoul2Info_v *`, dereferenced by the engine switch.
    ghoul2: *mut CGhoul2Info_v,
    model_list: *mut qhandle_t,
    skin_list: *mut qhandle_t,
}

impl CgG2SetmodelsArgs {
    pub const fn new(
        ghoul2: *mut CGhoul2Info_v,
        model_list: *mut qhandle_t,
        skin_list: *mut qhandle_t,
    ) -> Self {
        Self {
            ghoul2,
            model_list,
            skin_list,
        }
    }

    pub const fn ghoul2(&self) -> *mut CGhoul2Info_v {
        self.ghoul2
    }

    pub const fn model_list(&self) -> *mut qhandle_t {
        self.model_list
    }

    pub const fn skin_list(&self) -> *mut qhandle_t {
        self.skin_list
    }
}

/// `CG_G2_SETMODELS` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:175`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:486`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:794-796`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:794-796`
/// Type definition source: `oracle/oracle/code/game/ghoul2_shared.h:311`
pub struct CgG2Setmodels;

impl OutboundSysCall for CgG2Setmodels {
    type Import = SpCgameImport;
    type Args = CgG2SetmodelsArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_G2_SETMODELS;
}

impl EncodeSysCall for CgG2Setmodels {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.ghoul2()),
            ptr_to_word(args.model_list()),
            ptr_to_word(args.skin_list()),
        ])
    }
}

impl DecodeSysCallReturn for CgG2Setmodels {
    fn decode_return(_word: isize) -> Self::Output {}
}
