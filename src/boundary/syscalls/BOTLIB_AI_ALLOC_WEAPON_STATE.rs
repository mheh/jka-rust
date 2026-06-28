use core::ffi::c_int;

use crate::ffi::GameImport;

use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

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

pub struct BotlibAiAllocWeaponState;

impl OutboundSysCall for BotlibAiAllocWeaponState {
    type Args = BotlibAiAllocWeaponStateArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AI_ALLOC_WEAPON_STATE;
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
