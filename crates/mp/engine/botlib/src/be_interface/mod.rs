//! MP botlib `be_interface.h` types.

pub mod botlib_globals_s;

use core::ffi::c_int;

use crate::BotLib;

/// Convenience wrapper around `bot.botimport.Print` for call sites that pass
/// a single already-formatted message (no C varargs) — mirrors the direct
/// `bot.botimport.Print.unwrap()(...)` call shape used in
/// `be_interface_fns.rs`.
///
/// Source: `oracle/codemp/botlib/be_interface.h` (`botlib_import_t::Print`
/// used throughout `botlib/*.cpp` as `botimport.Print(type, msg)`).
pub unsafe fn botimport_print(bot: &mut BotLib, r#type: c_int, msg: &str) {
    let c = std::ffi::CString::new(msg).unwrap();
    bot.botimport.Print.unwrap()(r#type, c.as_ptr() as *mut core::ffi::c_char);
}
