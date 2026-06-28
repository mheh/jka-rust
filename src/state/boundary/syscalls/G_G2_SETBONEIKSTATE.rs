use core::ffi::c_int;
use std::ffi::CString;

use crate::codemp::game::q_shared_h::sharedSetBoneIKStateParams_t;
use crate::ffi::types::qboolean;
use crate::ffi::GameImport;

use super::super::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_G2_SETBONEIKSTATE` outbound game-to-engine syscall.
///
/// Mirrors `trap_G2API_SetBoneIKState(void *ghoul2, int time, const char *boneName,
/// int ikState, sharedSetBoneIKStateParams_t *params) -> qboolean`.
#[derive(Debug)]
pub struct GG2SetboneikstateArgs {
    /// Opaque Ghoul2 instance handle (void *).
    pub ghoul2: *mut core::ffi::c_void,
    /// Current server time.
    pub time: c_int,
    /// Bone name (NUL-terminated).
    pub bone_name: CString,
    /// IK state to set (e.g. `IKS_DYNAMIC`).
    pub ik_state: c_int,
    /// IK state params struct (may be null for IKS_NONE).
    pub params: *mut sharedSetBoneIKStateParams_t,
}

impl GG2SetboneikstateArgs {
    pub fn new(
        ghoul2: *mut core::ffi::c_void,
        time: c_int,
        bone_name: CString,
        ik_state: c_int,
        params: *mut sharedSetBoneIKStateParams_t,
    ) -> Self {
        Self { ghoul2, time, bone_name, ik_state, params }
    }

    pub fn ghoul2(&self) -> *mut core::ffi::c_void {
        self.ghoul2
    }
    pub fn time(&self) -> c_int {
        self.time
    }
    pub fn bone_name(&self) -> &CString {
        &self.bone_name
    }
    pub fn ik_state(&self) -> c_int {
        self.ik_state
    }
    pub fn params(&self) -> *mut sharedSetBoneIKStateParams_t {
        self.params
    }
}

pub struct GG2Setboneikstate;

impl OutboundSysCall for GG2Setboneikstate {
    type Args = GG2SetboneikstateArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_G2_SETBONEIKSTATE;
}

impl EncodeSysCall for GG2Setboneikstate {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2 as *const _),
            a.time as isize,
            ptr_to_word(a.bone_name.as_ptr()),
            a.ik_state as isize,
            ptr_to_word(a.params as *const _),
        ])
    }
}

impl DecodeSysCallReturn for GG2Setboneikstate {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
