#![allow(non_camel_case_types, non_snake_case)]

use crate::be_ai_weight::weightconfig_s::WeightConfigHandle;

/// Raven `bot_weaponstate_t` — the weapon state of a single bot.
///
/// `weaponweightconfig` became a `WeightConfigHandle` into the `BotLib`
/// weight-config arena (porting-rules §F17); the struct is botlib-internal
/// (never crosses the ABI seam), so `#[repr(C)]` and its layout asserts are
/// dropped. It stays zero-valid (`None`/null) for `GetClearedMemory`.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_weap.cpp:105-109`
pub struct bot_weaponstate_t {
    /// weapon weight configuration
    pub weaponweightconfig: Option<WeightConfigHandle>,
    /// weapon weight index
    pub weaponweightindex: *mut i32,
}

pub type bot_weaponstate_s = bot_weaponstate_t;
