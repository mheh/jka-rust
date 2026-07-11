#![allow(non_camel_case_types, non_snake_case, clippy::missing_safety_doc)]

//! MP botlib `be_ai_weap.cpp` — bot weapon AI: weapon-config validation and the
//! per-client weapon-state handle table.
//!
//! Source: `oracle/codemp/botlib/be_ai_weap.cpp`
//!
//! Destination `_fns` escape: the `be_ai_weap/` directory already holds the
//! types, so `be_ai_weap.cpp`'s functions land here.

use core::ffi::{c_char, c_int};

use mp_qshared::common::mp::botlib::print_type::{PRT_ERROR, PRT_FATAL};
use mp_qshared::shared::limits::MAX_CLIENTS;
use mp_qshared::shared::{qfalse, qtrue};

use crate::be_ai_weap::bot_weaponstate_s::bot_weaponstate_t;
use crate::BotLib;

/// Raven `BotValidWeaponNumber`.
///
/// Source: `oracle/codemp/botlib/be_ai_weap.cpp:120-128`
pub fn BotValidWeaponNumber(bot: &mut BotLib, weaponnum: c_int) -> c_int {
    if weaponnum <= 0 || weaponnum > unsafe { (*bot.weaponconfig).numweapons } {
        unsafe {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"weapon number out of range\n".as_ptr() as *mut c_char,
            );
        }
        return qfalse;
    }
    qtrue
}

/// Raven `BotWeaponStateFromHandle`.
///
/// Source: `oracle/codemp/botlib/be_ai_weap.cpp:135-148`
pub fn BotWeaponStateFromHandle(bot: &mut BotLib, handle: c_int) -> *mut bot_weaponstate_t {
    if handle <= 0 || handle > MAX_CLIENTS as c_int {
        unsafe {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"move state handle %d out of range\n".as_ptr() as *mut c_char,
                handle,
            );
        }
        return core::ptr::null_mut();
    }
    if bot.botweaponstates[handle as usize].is_null() {
        unsafe {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"invalid move state %d\n".as_ptr() as *mut c_char,
                handle,
            );
        }
        return core::ptr::null_mut();
    }
    bot.botweaponstates[handle as usize]
}
