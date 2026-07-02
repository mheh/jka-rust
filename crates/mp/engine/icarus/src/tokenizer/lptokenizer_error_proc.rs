#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

/// Raven `LPTokenizerErrorProc` — callback for tokenizer errors.
///
/// Raven: `typedef void (*LPTokenizerErrorProc)(LPCTSTR errString);`
/// Type definition source: `oracle/oracle/codemp/game/../icarus/tokenizer.h:384-384`
pub type LPTokenizerErrorProc = extern "C" fn(*const c_char);
