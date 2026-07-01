use super::super::MpGameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use core::ffi::c_int;

/// `BOTLIB_AI_RESET_WEAPON_STATE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiResetWeaponStateArgs {
    weaponstate: c_int,
}

impl BotlibAiResetWeaponStateArgs {
    pub fn new(weaponstate: c_int) -> Self {
        Self { weaponstate }
    }

    pub fn weaponstate(&self) -> c_int {
        self.weaponstate
    }
}

/// `BOTLIB_AI_RESET_WEAPON_STATE` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:480`
pub struct BotlibAiResetWeaponState;

impl OutboundSysCall for BotlibAiResetWeaponState {
    type Import = MpGameImport;
    type Args = BotlibAiResetWeaponStateArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_RESET_WEAPON_STATE;
}

impl EncodeSysCall for BotlibAiResetWeaponState {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.weaponstate as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiResetWeaponState {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
