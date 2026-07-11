//! `cvar.cpp` free functions (zero-dependency closure slice).
//!
//! Destination `_fns` escape: the `cvar/` directory already holds the cvar
//! types/consts, so `cvar.cpp`'s functions land here (per packet DESTINATION).
//!
//! Source: `oracle/codemp/qcommon/cvar.cpp`

use core::ffi::{c_char, c_int, c_long};

use mp_qshared::shared::cvar::{cvar_t, CVAR_INTERNAL};
use mp_qshared::shared::q_string::Info_SetValueForKey_Big;
use mp_qshared::shared::{qboolean, qfalse, qtrue};

use crate::common::Common;
use crate::cvar::cvar_consts::FILE_HASH_SIZE;

/// `strchr(s, c) != NULL` — scans the NUL-terminated string for `c`.
///
/// Local libc mirror; house rule: libc symbols use the Rust equivalent, no
/// resolved signature needed.
unsafe fn strchr_present(s: *const c_char, c: c_char) -> bool {
    let mut p = s;
    loop {
        if *p == c {
            return true;
        }
        if *p == 0 {
            return false;
        }
        p = p.offset(1);
    }
}

/// Raven `generateHashValue` — cvar-name hash (file-local static).
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:41-55`
pub fn generateHashValue(fname: *const c_char) -> c_long {
    let mut hash: c_long = 0;
    let mut i: c_int = 0;
    unsafe {
        while *fname.offset(i as isize) != 0 {
            // tolower((unsigned char)fname[i]) stored into `char letter`.
            let letter = (*fname.offset(i as isize) as u8).to_ascii_lowercase() as c_char;
            hash += (letter as c_long) * ((i + 119) as c_long);
            i += 1;
        }
    }
    hash &= FILE_HASH_SIZE as c_long - 1;
    hash
}

/// Raven `Cvar_ValidateString`.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:62-76`
pub fn Cvar_ValidateString(s: *const c_char) -> qboolean {
    unsafe {
        if s.is_null() {
            return qfalse;
        }
        if strchr_present(s, b'\\' as c_char) {
            return qfalse;
        }
        if strchr_present(s, b'"' as c_char) {
            return qfalse;
        }
        if strchr_present(s, b';' as c_char) {
            return qfalse;
        }
        qtrue
    }
}

/// Raven `Cvar_CommandCompletion`.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:166-177`
pub fn Cvar_CommandCompletion(common: &mut Common, callback: fn(*const c_char)) {
    let mut cvar: *mut cvar_t = common.cvar_vars;
    unsafe {
        while !cvar.is_null() {
            // Dont show internal cvars
            if (*cvar).flags & CVAR_INTERNAL != 0 {
                cvar = (*cvar).next;
                continue;
            }
            callback((*cvar).name);
            cvar = (*cvar).next;
        }
    }
}

/// Raven `Cvar_InfoString_Big`.
///
/// The `static char info[BIG_INFO_STRING]` return buffer is the owning
/// `Common.cvar_info_string_big` field (fork-3 return-buffer static); the
/// returned pointer aliases it exactly as Raven's static.
/// Source: `oracle/codemp/qcommon/cvar.cpp:854-869`
pub fn Cvar_InfoString_Big(common: &mut Common, bit: c_int) -> *mut c_char {
    common.cvar_info_string_big[0] = 0;
    let info = common.cvar_info_string_big.as_mut_ptr();

    let mut var: *mut cvar_t = common.cvar_vars;
    unsafe {
        while !var.is_null() {
            if ((*var).flags & CVAR_INTERNAL) == 0 && ((*var).flags & bit) != 0 {
                Info_SetValueForKey_Big(info, (*var).name, (*var).string);
            }
            var = (*var).next;
        }
    }
    info
}
