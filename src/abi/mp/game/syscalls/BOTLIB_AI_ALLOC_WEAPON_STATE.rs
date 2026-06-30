use core::ffi::c_int;

use super::super::MpGameImport;

use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_ALLOC_WEAPON_STATE` outbound game-to-engine syscall.
///
/// C: `int trap_BotAllocWeaponState(void);`
/// No inputs; returns an opaque integer weapon-state handle.
#[derive(Debug)]
pub struct BotlibAiAllocWeaponStateArgs;

impl BotlibAiAllocWeaponStateArgs {
    pub fn new() -> Self {
        Self
    }
}

/// `BOTLIB_AI_ALLOC_WEAPON_STATE` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:478`
pub struct BotlibAiAllocWeaponState;

impl OutboundSysCall for BotlibAiAllocWeaponState {
    type Import = MpGameImport;
    type Args = BotlibAiAllocWeaponStateArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_ALLOC_WEAPON_STATE;
}

impl EncodeSysCall for BotlibAiAllocWeaponState {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for BotlibAiAllocWeaponState {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
