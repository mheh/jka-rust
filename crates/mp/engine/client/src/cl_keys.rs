//! `cl_keys.cpp` — key binding, console input, and text-field editing.
//!
//! Source: `oracle/codemp/client/cl_keys.cpp`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int, c_uint};
use std::ffi::CString;

use libc::{memmove, strcasecmp, strcat, strcpy, strlen, strstr};

use mp_abi::cgame::exports::MpCgameExport;
use mp_abi::cgame::public::tcgincoming_console_command::TCGIncomingConsoleCommand;
use mp_abi::ui::exports::MpUiExport;
use mp_abi::ui::public::ui_menu_command_t::{UIMENU_INGAME, UIMENU_MAIN};
use mp_engine_qcommon::cmd_common::{Cbuf_AddText, Cmd_Argc, Cmd_Argv, Cmd_TokenizeString};
use mp_engine_qcommon::cmd_pc::{Cmd_AddCommand, Cmd_CommandCompletion};
use mp_engine_qcommon::common::common::com_printf;
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::error::com_error;
use mp_engine_qcommon::common::Common;
use mp_engine_qcommon::common_fns::Com_Memcpy;
use mp_engine_qcommon::cvar_fns::{Cvar_CommandCompletion, Cvar_Set, Cvar_VariableValue};
use mp_engine_qcommon::files_common::FS_Printf;
use mp_engine_qcommon::vm_fns::VM_Call;
use mp_engine_qcommon::z_memman_pc::{CopyString, Z_Free};
use mp_game::q_shared::Q_PrintStrlen;
use mp_game::q_shared_cvar_flags::CVAR_ARCHIVE;
use mp_qshared::shared::char_sizes::{BIGCHAR_WIDTH, SMALLCHAR_WIDTH};
use mp_qshared::shared::connstate_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::keycatch::{
    KEYCATCH_CGAME, KEYCATCH_CONSOLE, KEYCATCH_MESSAGE, KEYCATCH_UI,
};
use mp_qshared::shared::limits::{MAX_STRING_CHARS, MAX_TOKEN_CHARS};
use mp_qshared::shared::{qboolean, qfalse, qtrue};
use mp_ui::keycodes::fake_ascii_t::fakeAscii_t;
use mp_ui::keycodes::K_CHAR_FLAG;
use native_platform::sys_main::Sys_GetClipboardData;
use native_string::ctype::tolower;
use native_string::q_string::Q_strcat;
use native_string::q_string::{Q_stricmp, Q_stricmpn};
use native_string::q_strncpyz::Q_strncpyz;
use native_types::field_t;
use native_types::fileHandle_t;

use crate::cl_console::{Con_Bottom, Con_PageDown, Con_PageUp, Con_ToggleConsole_f, Con_Top};
use crate::cl_main::{CL_AddReliableCommand, CL_Disconnect_f};
use crate::cl_scrn::{
    SCR_DrawBigString, SCR_DrawSmallChar, SCR_DrawSmallStringExt, SCR_UpdateScreen,
};
use crate::client_host::{cl_from_view, Client};
use crate::keys::key_globals_s::{COMMAND_HISTORY, MAX_KEYS};
use crate::client_host::snd_from_view;
use crate::snd_dma::S_StopAllSounds;

/// Raven's anonymous `enum { CGAME_EVENT_NONE, ... }` first member.
/// The client crate does not depend on `mp_cgame` (cgame is a loaded VM, not a
/// build dependency), so this mirrors the value `mp_cgame::cg_new_draw` also
/// carries rather than crossing that boundary with a crate dependency.
///
/// Source: `oracle/codemp/cgame/cg_local.h`
const CGAME_EVENT_NONE: c_int = 0;

/// Raven `Field_Clear`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:644-648`
pub fn Field_Clear(edit: *mut field_t) {
    unsafe {
        (*edit).buffer[0] = 0;
        (*edit).cursor = 0;
        (*edit).scroll = 0;
    }
}

/// Raven `FindMatches` command-completion callback.
/// The completion state (`matchCount`/`shortestMatch`/`completionString`) rides
/// on the `Client` island across the completion pass.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:668-691`
pub fn FindMatches(cl: &mut Client, s: *const c_char) {
    unsafe {
        let s_str = core::ffi::CStr::from_ptr(s).to_string_lossy();
        let completion_len = strlen(cl.completionString);
        if Q_stricmpn(
            &s_str,
            &core::ffi::CStr::from_ptr(cl.completionString).to_string_lossy(),
            completion_len as usize,
        ) != 0
        {
            return;
        }
        cl.matchCount += 1;
        if cl.matchCount == 1 {
            Q_strncpyz(&mut cl.shortestMatch, &s_str, MAX_TOKEN_CHARS);
            return;
        }

        // Cut `shortestMatch` down to the amount common with `s`.
        let mut i: usize = 0;
        while *s.add(i) != 0 {
            if tolower(cl.shortestMatch[i] as u8 as char) != tolower(*s.add(i) as u8 as char) {
                cl.shortestMatch[i] = 0;
                break;
            }
            i += 1;
        }
        if *s.add(i) == 0 {
            cl.shortestMatch[i] = 0;
        }
    }
}

/// Raven `PrintMatches` command-completion callback.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:699-703`
pub fn PrintMatches(common: &mut Common, cl: &mut Client, s: *const c_char) {
    unsafe {
        let s_str = core::ffi::CStr::from_ptr(s).to_string_lossy();
        let shortest_str = core::ffi::CStr::from_ptr(cl.shortestMatch.as_ptr()).to_string_lossy();
        let shortest_len = strlen(cl.shortestMatch.as_ptr());
        if Q_stricmpn(&s_str, &shortest_str, shortest_len as usize) == 0 {
            com_printf(common, &format!("    {}\n", s_str));
        }
    }
}

/// Raven `keyConcatArgs`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:705-724`
pub fn keyConcatArgs(common: &mut Common, cl: &mut Client) {
    unsafe {
        let argc = Cmd_Argc(common);
        // The buffer length is read once, so each `Q_strcat` holds the only borrow.
        let size = cl.kg.g_consoleField.buffer.len();
        for i in 1..argc {
            Q_strcat(&mut cl.kg.g_consoleField.buffer, size, " ");
            let arg = Cmd_Argv(common, i);
            if arg.contains(' ') {
                Q_strcat(&mut cl.kg.g_consoleField.buffer, size, "\"");
            }
            Q_strcat(&mut cl.kg.g_consoleField.buffer, size, arg);
            if arg.contains(' ') {
                Q_strcat(&mut cl.kg.g_consoleField.buffer, size, "\"");
            }
        }
    }
}

/// Raven `Key_GetOverstrikeMode`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:990-992`
pub fn Key_GetOverstrikeMode(cl: &mut Client) -> qboolean {
    cl.kg.key_overstrikeMode
}

/// Raven `Key_SetOverstrikeMode`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:995-997`
pub fn Key_SetOverstrikeMode(cl: &mut Client, state: qboolean) {
    cl.kg.key_overstrikeMode = state;
}

/// Raven `Key_IsDown`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:1005-1011`
pub fn Key_IsDown(cl: &mut Client, keynum: c_int) -> qboolean {
    if keynum == -1 {
        return qfalse;
    }
    let upper = cl.keynames[keynum as usize].upper as usize;
    cl.kg.keys[upper].down
}

/// Raven `Key_StringToKeynum`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:1026-1084`
pub fn Key_StringToKeynum(cl: &mut Client, str: *mut c_char) -> c_int {
    unsafe {
        if str.is_null() || *str == 0 {
            return -1;
        }
        // A single-char bind is presumed to be an ascii char bind.
        if *str.add(1) == 0 {
            return cl.keynames[*str as u8 as usize].upper as c_int;
        }

        for i in 0..MAX_KEYS {
            if !cl.keynames[i].name.is_null() && strcasecmp(str, cl.keynames[i].name) == 0 {
                return cl.keynames[i].keynum;
            }
        }

        // Check for a hex code.
        if *str == b'0' as c_char && *str.add(1) == b'x' as c_char && strlen(str) == 4 {
            let mut n1 = *str.add(2) as c_int;
            n1 = if (b'0' as c_int..=b'9' as c_int).contains(&n1) {
                n1 - b'0' as c_int
            } else if (b'A' as c_int..=b'F' as c_int).contains(&n1) {
                n1 - b'A' as c_int + 10
            } else {
                0
            };

            let mut n2 = *str.add(3) as c_int;
            n2 = if (b'0' as c_int..=b'9' as c_int).contains(&n2) {
                n2 - b'0' as c_int
            } else if (b'A' as c_int..=b'F' as c_int).contains(&n2) {
                n2 - b'A' as c_int + 10
            } else {
                0
            };
            return n1 * 16 + n2;
        }

        -1
    }
}

/// Raven `Key_KeynumValid`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:1088-1099`
pub fn Key_KeynumValid(keynum: c_int) -> *const c_char {
    if keynum == -1 {
        return c"<KEY NOT FOUND>".as_ptr();
    }
    if keynum < 0 || keynum as usize >= MAX_KEYS {
        return c"<OUT OF RANGE>".as_ptr();
    }
    core::ptr::null()
}

/// Raven `Key_KeyToName`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:1101-1104`
pub fn Key_KeyToName(cl: &mut Client, keynum: c_int) -> *const c_char {
    cl.keynames[keynum as usize].name
}

/// Raven `Key_KeyToAscii`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:1107-1131`
pub fn Key_KeyToAscii(cl: &mut Client, keynum: c_int) -> *const c_char {
    if cl.keynames[keynum as usize].lower == 0 {
        return core::ptr::null();
    }
    if keynum == fakeAscii_t::A_SPACE as c_int {
        cl.tinyString[0] = fakeAscii_t::A_SHIFT_SPACE as c_char;
    } else if keynum == fakeAscii_t::A_ENTER as c_int {
        cl.tinyString[0] = fakeAscii_t::A_SHIFT_ENTER as c_char;
    } else if keynum == fakeAscii_t::A_KP_ENTER as c_int {
        cl.tinyString[0] = fakeAscii_t::A_SHIFT_KP_ENTER as c_char;
    } else {
        cl.tinyString[0] = cl.keynames[keynum as usize].upper as c_char;
    }
    cl.tinyString[1] = 0;
    cl.tinyString.as_ptr()
}

/// Raven `Key_KeyToHex`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:1133-1147`
pub fn Key_KeyToHex(cl: &mut Client, keynum: c_int) -> *const c_char {
    let i = keynum >> 4;
    let j = keynum & 15;

    cl.tinyString[0] = b'0' as c_char;
    cl.tinyString[1] = b'x' as c_char;
    cl.tinyString[2] = if i > 9 {
        i - 10 + b'A' as c_int
    } else {
        i + b'0' as c_int
    } as c_char;
    cl.tinyString[3] = if j > 9 {
        j - 10 + b'A' as c_int
    } else {
        j + b'0' as c_int
    } as c_char;
    cl.tinyString[4] = 0;

    cl.tinyString.as_ptr()
}

/// Raven `Key_SetBinding`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:1215-1235`
pub fn Key_SetBinding(
    view: &mut EngineHostView,
    cl: &mut Client,
    keynum: c_int,
    binding: *const c_char,
) {
    if keynum == -1 {
        return;
    }

    let upper = cl.keynames[keynum as usize].upper as usize;

    // Free any old binding.
    if !cl.kg.keys[upper].binding.is_null() {
        Z_Free(view.common, cl.kg.keys[upper].binding as *mut ());
        cl.kg.keys[upper].binding = core::ptr::null_mut();
    }

    // Allocate memory for the new binding.
    if !binding.is_null() {
        cl.kg.keys[upper].binding = CopyString(view, binding);
    }

    // A binding change is treated like modifying an archived cvar, so the file
    // write triggers at the next opportunity.
    view.common.cvar_modifiedFlags |= CVAR_ARCHIVE;
}

/// Raven `Key_GetBinding`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:1243-1249`
pub fn Key_GetBinding(cl: &mut Client, keynum: c_int) -> *mut c_char {
    if keynum == -1 {
        return c"".as_ptr() as *mut c_char;
    }
    cl.kg.keys[keynum as usize].binding
}

/// Raven `Key_GetKey`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:1257-1268`
pub fn Key_GetKey(cl: &mut Client, binding: *const c_char) -> c_int {
    unsafe {
        if !binding.is_null() {
            let binding_str = core::ffi::CStr::from_ptr(binding).to_string_lossy();
            for i in 0..256 {
                if !cl.kg.keys[i].binding.is_null() {
                    let bound_str =
                        core::ffi::CStr::from_ptr(cl.kg.keys[i].binding).to_string_lossy();
                    if Q_stricmp(&binding_str, &bound_str) == 0 {
                        return i as c_int;
                    }
                }
            }
        }
        -1
    }
}

/// The substitute for Raven's key-up time UB (DEC-60.2).
/// Any large positive value keeps the retail behavior class, so `IN_KeyUp` credits a released key for the full frame.
const KEYUP_TIME_UB_SUBSTITUTE: c_int = 0x0040_0000;

/// Raven `CL_AddKeyUpCommands`.
///
/// Raven's `%i` argument at `cl_keys.cpp:1433` is the bare identifier `time`,
/// which is the libc `time` function address, not a variable - shipping UB.
/// DEC-60.2 pins ``KEYUP_TIME_UB_SUBSTITUTE`` as the one defined behavior
/// (porting-rules §19): retail's address was always a large positive int, so
/// retail always filed a key release as held for the full frame.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:1416-1453`
pub fn CL_AddKeyUpCommands(common: &mut Common, key: c_int, kb: *mut c_char) {
    unsafe {
        if kb.is_null() {
            return;
        }
        let mut keyevent = qfalse;
        let mut button = [0 as c_char; 1024];
        let mut button_len: usize = 0;
        let mut i: usize = 0;
        loop {
            if *kb.add(i) == b';' as c_char || *kb.add(i) == 0 {
                button[button_len] = 0;
                if button[0] == b'+' as c_char {
                    // Button commands add the keynum and time as parms, so multiple
                    // sources can be discriminated and subframe corrected.
                    let button_str = core::ffi::CStr::from_ptr(button.as_ptr()).to_string_lossy();
                    // DEC-60.2 replaces Raven's UB address print with the pinned constant.
                    let time = KEYUP_TIME_UB_SUBSTITUTE;
                    let cmd = format!("-{} {} {}\n", &button_str[1..], key, time);
                    Cbuf_AddText(common, &cmd);
                    keyevent = qtrue;
                } else if keyevent == qtrue {
                    // Down-only command.
                    let button_str = core::ffi::CStr::from_ptr(button.as_ptr()).to_string_lossy();
                    Cbuf_AddText(common, &button_str);
                    Cbuf_AddText(common, "\n");
                }
                button_len = 0;
                while (*kb.add(i) as u8 as i32) <= b' ' as i32 && *kb.add(i) != 0
                    || *kb.add(i) == b';' as c_char
                {
                    i += 1;
                }
            }
            button[button_len] = *kb.add(i);
            button_len += 1;
            if *kb.add(i) == 0 {
                break;
            }
            i += 1;
        }
    }
}

/// Raven `Field_Paste`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:471-488`
pub fn Field_Paste(common: &mut Common, cl: &mut Client, edit: *mut field_t) {
    unsafe {
        let cbd = Sys_GetClipboardData();
        if cbd.is_null() {
            return;
        }

        // Send as if typed, so insert / overstrike works properly.
        let paste_len = strlen(cbd);
        for i in 0..paste_len {
            Field_CharEvent(common, cl, edit, *cbd.add(i) as c_int);
        }

        Z_Free(common, cbd as *mut ());
    }
}

/// Raven `Field_CharEvent`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:567-637`
pub fn Field_CharEvent(common: &mut Common, cl: &mut Client, edit: *mut field_t, ch: c_int) {
    unsafe {
        if ch == b'v' as c_int - b'a' as c_int + 1 {
            // ctrl-v is paste
            Field_Paste(common, cl, edit);
            return;
        }

        if ch == b'c' as c_int - b'a' as c_int + 1 {
            // ctrl-c clears the field
            Field_Clear(edit);
            return;
        }

        let len = strlen((*edit).buffer.as_ptr()) as c_int;

        if ch == b'h' as c_int - b'a' as c_int + 1 {
            // ctrl-h is backspace
            if (*edit).cursor > 0 {
                let buf = (*edit).buffer.as_mut_ptr();
                memmove(
                    buf.add((*edit).cursor as usize - 1) as *mut _,
                    buf.add((*edit).cursor as usize) as *const _,
                    (len + 1 - (*edit).cursor) as usize,
                );
                (*edit).cursor -= 1;
                if (*edit).cursor < (*edit).scroll {
                    (*edit).scroll -= 1;
                }
            }
            return;
        }

        if ch == b'a' as c_int - b'a' as c_int + 1 {
            // ctrl-a is home
            (*edit).cursor = 0;
            (*edit).scroll = 0;
            return;
        }

        if ch == b'e' as c_int - b'a' as c_int + 1 {
            // ctrl-e is end
            (*edit).cursor = len;
            (*edit).scroll = (*edit).cursor - (*edit).widthInChars;
            return;
        }

        // Ignore any other non-printable chars.
        if ch < 32 {
            return;
        }

        if cl.kg.key_overstrikeMode == qtrue {
            if (*edit).cursor == (*edit).buffer.len() as c_int - 1 {
                return;
            }
            (*edit).buffer[(*edit).cursor as usize] = ch as c_char;
            (*edit).cursor += 1;
        } else {
            // insert mode
            if len == (*edit).buffer.len() as c_int - 1 {
                return; // all full
            }
            let buf = (*edit).buffer.as_mut_ptr();
            memmove(
                buf.add((*edit).cursor as usize + 1) as *mut _,
                buf.add((*edit).cursor as usize) as *const _,
                (len + 1 - (*edit).cursor) as usize,
            );
            (*edit).buffer[(*edit).cursor as usize] = ch as c_char;
            (*edit).cursor += 1;
        }

        if (*edit).cursor >= (*edit).widthInChars {
            (*edit).scroll += 1;
        }

        if (*edit).cursor == len + 1 {
            (*edit).buffer[(*edit).cursor as usize] = 0;
        }
    }
}

/// Raven `ConcatRemaining`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:726-737`
pub fn ConcatRemaining(
    common: &mut Common,
    cl: &mut Client,
    src: *const c_char,
    start: *const c_char,
) {
    unsafe {
        let found = strstr(src, start);
        if found.is_null() {
            keyConcatArgs(common, cl);
            return;
        }

        let str_ptr = found.add(strlen(start));
        let str_str = core::ffi::CStr::from_ptr(str_ptr).to_string_lossy();
        let buffer_len = cl.kg.g_consoleField.buffer.len();
        Q_strcat(&mut cl.kg.g_consoleField.buffer, buffer_len, &str_str);
    }
}

/// Raven `Key_KeynumToAscii`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:1150-1172`
pub fn Key_KeynumToAscii(cl: &mut Client, keynum: c_int) -> *const c_char {
    let mut name = Key_KeynumValid(keynum);

    // Check for printable ascii.
    if name.is_null() && keynum > 0 && keynum < 256 {
        name = Key_KeyToAscii(cl, keynum);
    }
    // Check for a friendly name (JOYx and AUXx buttons).
    if name.is_null() {
        name = Key_KeyToName(cl, keynum);
    }
    // Fall back to a hex number.
    if name.is_null() {
        name = Key_KeyToHex(cl, keynum);
    }
    name
}

/// Raven `Key_KeynumToString`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:1184-1206`
pub fn Key_KeynumToString(cl: &mut Client, keynum: c_int) -> *const c_char {
    let mut name = Key_KeynumValid(keynum);

    // Check for a friendly name.
    if name.is_null() {
        name = Key_KeyToName(cl, keynum);
    }
    // Check for printable ascii.
    if name.is_null() && keynum > 0 && keynum < 256 {
        name = Key_KeyToAscii(cl, keynum);
    }
    // Fall back to a hex number.
    if name.is_null() {
        name = Key_KeyToHex(cl, keynum);
    }
    name
}

/// Raven `Key_Unbind_f`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:1275-1293`
pub fn Key_Unbind_f(view: &mut EngineHostView, cl: &mut Client) {
    if Cmd_Argc(view.common) != 2 {
        com_printf(view.common, "unbind <key> : remove commands from a key\n");
        return;
    }

    let b = Key_StringToKeynum(cl, Cmd_Argv(view.common, 1).as_ptr() as *mut c_char);
    if b == -1 {
        com_printf(
            view.common,
            &format!("\"{}\" isn't a valid key\n", Cmd_Argv(view.common, 1)),
        );
        return;
    }

    Key_SetBinding(view, cl, b, c"".as_ptr());
}

/// Raven `Key_Unbindall_f`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:1300-1311`
pub fn Key_Unbindall_f(view: &mut EngineHostView, cl: &mut Client) {
    for i in 0..MAX_KEYS {
        if !cl.kg.keys[i].binding.is_null() {
            Key_SetBinding(view, cl, i as c_int, c"".as_ptr());
        }
    }
}

/// Raven `Key_Bind_f`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:1320-1358`
pub fn Key_Bind_f(view: &mut EngineHostView, cl: &mut Client) {
    unsafe {
        let c = Cmd_Argc(view.common);

        if c < 2 {
            com_printf(
                view.common,
                "bind <key> [command] : attach a command to a key\n",
            );
            return;
        }
        let b = Key_StringToKeynum(cl, Cmd_Argv(view.common, 1).as_ptr() as *mut c_char);
        if b == -1 {
            com_printf(
                view.common,
                &format!("\"{}\" isn't a valid key\n", Cmd_Argv(view.common, 1)),
            );
            return;
        }

        if c == 2 {
            if !cl.kg.keys[b as usize].binding.is_null() {
                let bound =
                    core::ffi::CStr::from_ptr(cl.kg.keys[b as usize].binding).to_string_lossy();
                com_printf(
                    view.common,
                    &format!("\"{}\" = \"{}\"\n", Cmd_Argv(view.common, 1), bound),
                );
            } else {
                com_printf(
                    view.common,
                    &format!("\"{}\" is not bound\n", Cmd_Argv(view.common, 1)),
                );
            }
            return;
        }

        // Copy the rest of the command line.
        let mut cmd = [0u8; 1024];
        cmd[0] = 0;
        for i in 2..c {
            let arg = Cmd_Argv(view.common, i);
            strcat(
                cmd.as_mut_ptr() as *mut c_char,
                arg.as_ptr() as *const c_char,
            );
            if i != c - 1 {
                strcat(cmd.as_mut_ptr() as *mut c_char, c" ".as_ptr());
            }
        }

        let cmd_str = core::ffi::CStr::from_ptr(cmd.as_ptr() as *const c_char).to_string_lossy();
        Key_SetBinding(
            view,
            cl,
            b,
            std::ffi::CString::new(&*cmd_str).unwrap().as_ptr(),
        );
    }
}

/// Raven `Field_KeyDownEvent`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:500-560`
pub fn Field_KeyDownEvent(common: &mut Common, cl: &mut Client, edit: *mut field_t, key: c_int) {
    unsafe {
        // shift-insert is paste
        if (key == fakeAscii_t::A_INSERT as c_int || key == fakeAscii_t::A_KP_0 as c_int)
            && cl.kg.keys[fakeAscii_t::A_SHIFT as usize].down == qtrue
        {
            Field_Paste(common, cl, edit);
            return;
        }

        let len = strlen((*edit).buffer.as_ptr()) as c_int;

        if key == fakeAscii_t::A_DELETE as c_int {
            if (*edit).cursor < len {
                let buf = (*edit).buffer.as_mut_ptr();
                memmove(
                    buf.add((*edit).cursor as usize) as *mut _,
                    buf.add((*edit).cursor as usize + 1) as *const _,
                    (len - (*edit).cursor) as usize,
                );
            }
            return;
        }

        if key == fakeAscii_t::A_CURSOR_RIGHT as c_int {
            if (*edit).cursor < len {
                (*edit).cursor += 1;
            }
            if (*edit).cursor >= (*edit).scroll + (*edit).widthInChars && (*edit).cursor <= len {
                (*edit).scroll += 1;
            }
            return;
        }

        if key == fakeAscii_t::A_CURSOR_LEFT as c_int {
            if (*edit).cursor > 0 {
                (*edit).cursor -= 1;
            }
            if (*edit).cursor < (*edit).scroll {
                (*edit).scroll -= 1;
            }
            return;
        }

        if key == fakeAscii_t::A_HOME as c_int
            || (cl.keynames[key as usize].lower == b'a' as u16
                && cl.kg.keys[fakeAscii_t::A_CTRL as usize].down == qtrue)
        {
            (*edit).cursor = 0;
            return;
        }

        if key == fakeAscii_t::A_END as c_int
            || (cl.keynames[key as usize].lower == b'e' as u16
                && cl.kg.keys[fakeAscii_t::A_CTRL as usize].down == qtrue)
        {
            (*edit).cursor = len;
            return;
        }

        if key == fakeAscii_t::A_INSERT as c_int {
            cl.kg.key_overstrikeMode = if cl.kg.key_overstrikeMode == qtrue {
                qfalse
            } else {
                qtrue
            };
            return;
        }
    }
}

/// Raven `CompleteCommand`.
///
/// Raven passes `FindMatches`/`PrintMatches` as C callbacks. The port takes the
/// two name lists back instead and runs the same two visits here, so both
/// helpers keep their `cl` and `common` receivers.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:747-800`
pub fn CompleteCommand(common: &mut Common, cl: &mut Client) {
    unsafe {
        let edit: *mut field_t = &mut cl.kg.g_consoleField;
        let mut temp: field_t = core::mem::zeroed();

        // Only the first token matters for completion purposes.
        Cmd_TokenizeString(
            common,
            core::ffi::CStr::from_ptr((*edit).buffer.as_ptr())
                .to_string_lossy()
                .as_ref(),
        );

        cl.completionString = Cmd_Argv(common, 0).as_ptr() as *mut c_char;
        if *cl.completionString == b'\\' as c_char || *cl.completionString == b'/' as c_char {
            cl.completionString = cl.completionString.add(1);
        }
        cl.matchCount = 0;
        cl.shortestMatch[0] = 0;

        if strlen(cl.completionString) == 0 {
            return;
        }

        let mut names = Cmd_CommandCompletion(common);
        names.extend(Cvar_CommandCompletion(common));
        for name in &names {
            let name_c = CString::new(name.as_str()).unwrap_or_default();
            FindMatches(cl, name_c.as_ptr());
        }

        if cl.matchCount == 0 {
            return; // no matches
        }

        Com_Memcpy(
            &mut temp as *mut field_t as *mut (),
            edit as *const (),
            core::mem::size_of::<field_t>(),
        );

        if cl.matchCount == 1 {
            let shortest = core::ffi::CStr::from_ptr(cl.shortestMatch.as_ptr()).to_string_lossy();
            let out = format!("\\{}", shortest);
            let out_c = std::ffi::CString::new(out).unwrap();
            core::ffi::CStr::from_ptr(out_c.as_ptr())
                .to_bytes_with_nul()
                .iter()
                .enumerate()
                .for_each(|(i, b)| {
                    if i < (*edit).buffer.len() {
                        (*edit).buffer[i] = *b as c_char;
                    }
                });
            if Cmd_Argc(common) == 1 {
                // The buffer length is read first, so `Q_strcat` holds the only borrow.
                let size = cl.kg.g_consoleField.buffer.len();
                Q_strcat(&mut cl.kg.g_consoleField.buffer, size, " ");
            } else {
                let completion_str = core::ffi::CStr::from_ptr(cl.completionString)
                    .to_string_lossy()
                    .into_owned();
                ConcatRemaining(
                    common,
                    cl,
                    temp.buffer.as_ptr(),
                    completion_str.as_ptr() as *const c_char,
                );
            }
            (*edit).cursor = strlen((*edit).buffer.as_ptr()) as c_int;
            return;
        }

        // Multiple matches, complete to the shortest.
        let shortest = core::ffi::CStr::from_ptr(cl.shortestMatch.as_ptr()).to_string_lossy();
        let out = format!("\\{}", shortest);
        let out_c = std::ffi::CString::new(out).unwrap();
        out_c
            .as_bytes_with_nul()
            .iter()
            .enumerate()
            .for_each(|(i, b)| {
                if i < (*edit).buffer.len() {
                    (*edit).buffer[i] = *b as c_char;
                }
            });
        (*edit).cursor = strlen((*edit).buffer.as_ptr()) as c_int;
        let completion_str = core::ffi::CStr::from_ptr(cl.completionString)
            .to_string_lossy()
            .into_owned();
        ConcatRemaining(
            common,
            cl,
            temp.buffer.as_ptr(),
            completion_str.as_ptr() as *const c_char,
        );

        com_printf(
            common,
            &format!(
                "]{}\n",
                core::ffi::CStr::from_ptr((*edit).buffer.as_ptr()).to_string_lossy()
            ),
        );

        // Run through again, printing matches.
        let mut names = Cmd_CommandCompletion(common);
        names.extend(Cvar_CommandCompletion(common));
        for name in &names {
            let name_c = CString::new(name.as_str()).unwrap_or_default();
            PrintMatches(common, cl, name_c.as_ptr());
        }
    }
}

/// Raven `Key_WriteBindings`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:1367-1376`
pub fn Key_WriteBindings(common: &mut Common, cl: &mut Client, f: fileHandle_t) {
    unsafe {
        FS_Printf(common, f, "unbindall\n");
        for i in 0..MAX_KEYS {
            if !cl.kg.keys[i].binding.is_null() && *cl.kg.keys[i].binding != 0 {
                let name =
                    core::ffi::CStr::from_ptr(Key_KeynumToString(cl, i as c_int)).to_string_lossy();
                let binding = core::ffi::CStr::from_ptr(cl.kg.keys[i].binding).to_string_lossy();
                FS_Printf(common, f, &format!("bind {} \"{}\"\n", name, binding));
            }
        }
    }
}

/// Raven `Key_Bindlist_f`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:1387-1395`
pub fn Key_Bindlist_f(common: &mut Common, cl: &mut Client) {
    unsafe {
        for i in 0..MAX_KEYS {
            if !cl.kg.keys[i].binding.is_null() && *cl.kg.keys[i].binding != 0 {
                let ascii =
                    core::ffi::CStr::from_ptr(Key_KeynumToAscii(cl, i as c_int)).to_string_lossy();
                let name =
                    core::ffi::CStr::from_ptr(Key_KeynumToString(cl, i as c_int)).to_string_lossy();
                let binding = core::ffi::CStr::from_ptr(cl.kg.keys[i].binding).to_string_lossy();
                com_printf(
                    common,
                    &format!("Key : {} ({}) \"{}\"\n", ascii, name, binding),
                );
            }
        }
    }
}

/// Raven `CL_CharEvent`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:1658-1681`
pub fn CL_CharEvent(common: &mut Common, cl: &mut Client, key: c_int) {
    // The console key never doubles as a char.
    if key == b'`' as c_int || key == b'~' as c_int {
        return;
    }

    // Distribute the key-down event to the appropriate handler.
    if cl.cls.keyCatchers & KEYCATCH_CONSOLE != 0 {
        let edit: *mut field_t = &mut cl.kg.g_consoleField;
        Field_CharEvent(common, cl, edit, key);
    } else if cl.cls.keyCatchers & KEYCATCH_UI != 0 {
        VM_Call(
            common,
            cl.uivm,
            MpUiExport::UI_KEY_EVENT as c_int,
            &[(key | K_CHAR_FLAG) as isize, qtrue as isize],
        );
    } else if cl.cls.keyCatchers & KEYCATCH_MESSAGE != 0 {
        let edit: *mut field_t = &mut cl.chatField;
        Field_CharEvent(common, cl, edit, key);
    } else if cl.cls.state == connstate_t::CA_DISCONNECTED {
        let edit: *mut field_t = &mut cl.kg.g_consoleField;
        Field_CharEvent(common, cl, edit, key);
    }
}

/// Raven `Field_VariableSizeDraw`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:374-454`
pub fn Field_VariableSizeDraw(
    view: &mut EngineHostView,
    cl: &mut Client,
    edit: *mut field_t,
    x: c_int,
    y: c_int,
    width: c_int,
    size: c_int,
    showCursor: qboolean,
) {
    unsafe {
        let mut draw_len = (*edit).widthInChars;
        let len = strlen((*edit).buffer.as_ptr()) as c_int + 1;

        // Guarantee that the cursor stays visible.
        let prestep;
        if len <= draw_len {
            prestep = 0;
        } else {
            if (*edit).scroll + draw_len > len {
                (*edit).scroll = len - draw_len;
                if (*edit).scroll < 0 {
                    (*edit).scroll = 0;
                }
            }
            prestep = (*edit).scroll;
        }

        if prestep + draw_len > len {
            draw_len = len - prestep;
        }

        // Extract `draw_len` characters from the field at `prestep`.
        if draw_len as usize >= MAX_STRING_CHARS {
            com_error(
                errorParm_t::ERR_DROP,
                "drawLen >= MAX_STRING_CHARS".to_string(),
            );
        }

        let mut str_buf = [0 as c_char; MAX_STRING_CHARS];
        Com_Memcpy(
            str_buf.as_mut_ptr() as *mut (),
            (*edit).buffer.as_ptr().add(prestep as usize) as *const (),
            draw_len as usize,
        );
        str_buf[draw_len as usize] = 0;

        // Draw the field text.
        if size == SMALLCHAR_WIDTH {
            let mut color = [1.0f32, 1.0, 1.0, 1.0];
            SCR_DrawSmallStringExt(view, cl, x, y, str_buf.as_ptr(), color.as_mut_ptr(), false);
        } else {
            // Draw the big string with a drop shadow.
            SCR_DrawBigString(view, cl, x, y, str_buf.as_ptr(), 1.0);
        }

        // Draw the cursor.
        if showCursor != qtrue {
            return;
        }

        if ((cl.cls.realtime >> 8) & 1) != 0 {
            return; // off blink
        }

        let cursor_char = if cl.kg.key_overstrikeMode == qtrue {
            11
        } else {
            10
        };

        let i = draw_len - (Q_PrintStrlen(str_buf.as_ptr()) + 1);

        if size == SMALLCHAR_WIDTH {
            SCR_DrawSmallChar(
                view,
                cl,
                x + ((*edit).cursor - prestep - i) * size,
                y,
                cursor_char,
            );
        } else {
            str_buf[0] = cursor_char as c_char;
            str_buf[1] = 0;
            SCR_DrawBigString(
                view,
                cl,
                x + ((*edit).cursor - prestep - i) * size,
                y,
                str_buf.as_ptr(),
                1.0,
            );
        }
    }
}

/// Raven `Field_Draw`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:456-459`
pub fn Field_Draw(
    view: &mut EngineHostView,
    cl: &mut Client,
    edit: *mut field_t,
    x: c_int,
    y: c_int,
    width: c_int,
    showCursor: qboolean,
) {
    Field_VariableSizeDraw(view, cl, edit, x, y, width, SMALLCHAR_WIDTH, showCursor);
}

/// Raven `Field_BigDraw`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:461-464`
pub fn Field_BigDraw(
    view: &mut EngineHostView,
    cl: &mut Client,
    edit: *mut field_t,
    x: c_int,
    y: c_int,
    width: c_int,
    showCursor: qboolean,
) {
    Field_VariableSizeDraw(view, cl, edit, x, y, width, BIGCHAR_WIDTH, showCursor);
}

/// Raven `Message_Key`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:951-985`
pub fn Message_Key(common: &mut Common, cl: &mut Client, key: c_int) {
    unsafe {
        let mut buffer = [0u8; MAX_STRING_CHARS];

        if key == fakeAscii_t::A_ESCAPE as c_int {
            cl.cls.keyCatchers &= !KEYCATCH_MESSAGE;
            Field_Clear(&mut cl.chatField);
            return;
        }

        if key == fakeAscii_t::A_ENTER as c_int || key == fakeAscii_t::A_KP_ENTER as c_int {
            if cl.chatField.buffer[0] != 0 && cl.cls.state == connstate_t::CA_ACTIVE {
                let text =
                    core::ffi::CStr::from_ptr(cl.chatField.buffer.as_ptr()).to_string_lossy();
                let line = if cl.chat_playerNum != -1 {
                    format!("tell {} \"{}\"\n", cl.chat_playerNum, text)
                } else if cl.chat_team == qtrue {
                    format!("say_team \"{}\"\n", text)
                } else {
                    format!("say \"{}\"\n", text)
                };
                let line_c = std::ffi::CString::new(line).unwrap();
                for (i, b) in line_c.as_bytes_with_nul().iter().enumerate() {
                    if i < buffer.len() {
                        buffer[i] = *b;
                    }
                }

                CL_AddReliableCommand(cl, buffer.as_ptr() as *const c_char);
            }
            cl.cls.keyCatchers &= !KEYCATCH_MESSAGE;
            Field_Clear(&mut cl.chatField);
            return;
        }

        let edit: *mut field_t = &mut cl.chatField;
        Field_KeyDownEvent(common, cl, edit, key);
    }
}

/// `Cmd_AddCommand` registers `Key_Bind_f`, whose real shape now takes
/// `(view, cl)`. This adapter casts the client out of `view` first.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:1403-1409`
fn Key_Bind_f_cmd(view: &mut EngineHostView) {
    let cl = unsafe { cl_from_view(view) };
    Key_Bind_f(view, cl);
}

/// `Cmd_AddCommand` registers `Key_Unbind_f`, whose real shape now takes
/// `(view, cl)`. This adapter casts the client out of `view` first.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:1403-1409`
fn Key_Unbind_f_cmd(view: &mut EngineHostView) {
    let cl = unsafe { cl_from_view(view) };
    Key_Unbind_f(view, cl);
}

/// `Cmd_AddCommand` registers `Key_Unbindall_f`, whose real shape now takes
/// `(view, cl)`. This adapter casts the client out of `view` first.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:1403-1409`
fn Key_Unbindall_f_cmd(view: &mut EngineHostView) {
    let cl = unsafe { cl_from_view(view) };
    Key_Unbindall_f(view, cl);
}

/// `Cmd_AddCommand` registers `Key_Bindlist_f`, whose real shape now takes
/// `(common, cl)`. This adapter casts the client out of `view` first.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:1403-1409`
fn Key_Bindlist_f_cmd(view: &mut EngineHostView) {
    let cl = unsafe { cl_from_view(view) };
    Key_Bindlist_f(view.common, cl);
}

/// Raven `CL_InitKeyCommands`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:1403-1409`
pub fn CL_InitKeyCommands(view: &mut EngineHostView) {
    Cmd_AddCommand(view, "bind", Some(Key_Bind_f_cmd));
    Cmd_AddCommand(view, "unbind", Some(Key_Unbind_f_cmd));
    Cmd_AddCommand(view, "unbindall", Some(Key_Unbindall_f_cmd));
    Cmd_AddCommand(view, "bindlist", Some(Key_Bindlist_f_cmd));
}

/// Raven `Console_Key`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:810-939`
pub fn Console_Key(view: &mut EngineHostView, cl: &mut Client, key: c_int) {
    unsafe {
        // ctrl-L clears the screen
        if cl.keynames[key as usize].lower == b'l' as u16
            && cl.kg.keys[fakeAscii_t::A_CTRL as usize].down == qtrue
        {
            Cbuf_AddText(view.common, "clear\n");
            return;
        }

        // Enter finishes the line.
        if key == fakeAscii_t::A_ENTER as c_int || key == fakeAscii_t::A_KP_ENTER as c_int {
            // If not in the game, explicitly prepend a slash if needed.
            if cl.cls.state != connstate_t::CA_ACTIVE
                && cl.kg.g_consoleField.buffer[0] != b'\\' as c_char
                && cl.kg.g_consoleField.buffer[0] != b'/' as c_char
            {
                let temp = core::ffi::CStr::from_ptr(cl.kg.g_consoleField.buffer.as_ptr())
                    .to_string_lossy()
                    .into_owned();
                let mut temp_buf = [0 as c_char; MAX_STRING_CHARS];
                Q_strncpyz(&mut temp_buf, &temp, MAX_STRING_CHARS);
                let out = format!("\\{}", temp);
                let out_c = std::ffi::CString::new(out).unwrap();
                for (i, b) in out_c.as_bytes_with_nul().iter().enumerate() {
                    if i < cl.kg.g_consoleField.buffer.len() {
                        cl.kg.g_consoleField.buffer[i] = *b as c_char;
                    }
                }
                cl.kg.g_consoleField.cursor += 1;
            } else {
                // Explicit commands do not need a leading slash.
                CompleteCommand(view.common, cl);
            }

            com_printf(
                view.common,
                &format!(
                    "]{}\n",
                    core::ffi::CStr::from_ptr(cl.kg.g_consoleField.buffer.as_ptr())
                        .to_string_lossy()
                ),
            );

            // A leading slash is an explicit command.
            if cl.kg.g_consoleField.buffer[0] == b'\\' as c_char
                || cl.kg.g_consoleField.buffer[0] == b'/' as c_char
            {
                if !cl.cgvm.is_null() && !cl.cl.mSharedMemory.is_null() {
                    // Do not run this unless cgame is inited and shared memory is valid.
                    let buf =
                        core::ffi::CStr::from_ptr(cl.kg.g_consoleField.buffer.as_ptr().add(1))
                            .to_string_lossy()
                            .into_owned();
                    let icc = cl.cl.mSharedMemory as *mut TCGIncomingConsoleCommand;
                    strcpy(
                        (*icc).conCommand.as_mut_ptr() as *mut c_char,
                        std::ffi::CString::new(buf).unwrap().as_ptr(),
                    );

                    if VM_Call(
                        view.common,
                        cl.cgvm,
                        MpCgameExport::CG_INCOMING_CONSOLE_COMMAND as c_int,
                        &[],
                    ) != 0
                    {
                        // Let mod authors filter client console messages so they can cut them off if they want.
                        let text =
                            core::ffi::CStr::from_ptr(cl.kg.g_consoleField.buffer.as_ptr().add(1))
                                .to_string_lossy();
                        Cbuf_AddText(view.common, &text);
                        Cbuf_AddText(view.common, "\n");
                    } else if (*icc).conCommand[0] != 0 {
                        // The VM call says to execute this command in place.
                        let text =
                            core::ffi::CStr::from_ptr((*icc).conCommand.as_ptr() as *const c_char)
                                .to_string_lossy();
                        Cbuf_AddText(view.common, &text);
                        Cbuf_AddText(view.common, "\n");
                    }
                } else {
                    // Just execute it.
                    let text =
                        core::ffi::CStr::from_ptr(cl.kg.g_consoleField.buffer.as_ptr().add(1))
                            .to_string_lossy();
                    Cbuf_AddText(view.common, &text);
                    Cbuf_AddText(view.common, "\n");
                }
            } else {
                // Other text is a chat message.
                if cl.kg.g_consoleField.buffer[0] == 0 {
                    return; // empty lines just scroll the console without adding to history
                }
                Cbuf_AddText(view.common, "cmd say ");
                let text = core::ffi::CStr::from_ptr(cl.kg.g_consoleField.buffer.as_ptr())
                    .to_string_lossy();
                Cbuf_AddText(view.common, &text);
                Cbuf_AddText(view.common, "\n");
            }

            // Copy the line to the history buffer.
            let hist_index = (cl.kg.nextHistoryLine % COMMAND_HISTORY as c_int) as usize;
            cl.kg.historyEditLines[hist_index] = core::ptr::read(&cl.kg.g_consoleField);
            cl.kg.nextHistoryLine += 1;
            cl.kg.historyLine = cl.kg.nextHistoryLine;

            Field_Clear(&mut cl.kg.g_consoleField);

            cl.kg.g_consoleField.widthInChars = cl.g_console_field_width;

            if cl.cls.state == connstate_t::CA_DISCONNECTED {
                SCR_UpdateScreen(view, cl); // force an update, because the command may take some time
            }
            return;
        }

        // Command completion.
        if key == fakeAscii_t::A_TAB as c_int {
            CompleteCommand(view.common, cl);
            return;
        }

        // Command history (ctrl-p / ctrl-n for unix style).
        if key == fakeAscii_t::A_CURSOR_UP as c_int
            || (cl.keynames[key as usize].lower == b'p' as u16
                && cl.kg.keys[fakeAscii_t::A_CTRL as usize].down == qtrue)
        {
            if cl.kg.nextHistoryLine - cl.kg.historyLine < COMMAND_HISTORY as c_int
                && cl.kg.historyLine > 0
            {
                cl.kg.historyLine -= 1;
            }
            let idx = (cl.kg.historyLine % COMMAND_HISTORY as c_int) as usize;
            cl.kg.g_consoleField = core::ptr::read(&cl.kg.historyEditLines[idx]);
            return;
        }

        if key == fakeAscii_t::A_CURSOR_DOWN as c_int
            || (cl.keynames[key as usize].lower == b'n' as u16
                && cl.kg.keys[fakeAscii_t::A_CTRL as usize].down == qtrue)
        {
            if cl.kg.historyLine == cl.kg.nextHistoryLine {
                return;
            }
            cl.kg.historyLine += 1;
            let idx = (cl.kg.historyLine % COMMAND_HISTORY as c_int) as usize;
            cl.kg.g_consoleField = core::ptr::read(&cl.kg.historyEditLines[idx]);
            return;
        }

        // Console scrolling.
        if key == fakeAscii_t::A_PAGE_UP as c_int {
            Con_PageUp(cl);
            return;
        }

        if key == fakeAscii_t::A_PAGE_DOWN as c_int {
            Con_PageDown(cl);
            return;
        }

        // ctrl-home is the top of the console.
        if key == fakeAscii_t::A_HOME as c_int
            && cl.kg.keys[fakeAscii_t::A_CTRL as usize].down == qtrue
        {
            Con_Top(cl);
            return;
        }

        // ctrl-end is the bottom of the console.
        if key == fakeAscii_t::A_END as c_int
            && cl.kg.keys[fakeAscii_t::A_CTRL as usize].down == qtrue
        {
            Con_Bottom(cl);
            return;
        }

        // Pass to the normal editline routine.
        let edit: *mut field_t = &mut cl.kg.g_consoleField;
        Field_KeyDownEvent(view.common, cl, edit, key);
    }
}

/// Raven `CL_KeyEvent`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:1462-1648`
pub fn CL_KeyEvent(
    view: &mut EngineHostView,
    cl: &mut Client,
    key: c_int,
    down: qboolean,
    time: c_uint,
) {
    unsafe {
        // Update the auto-repeat status and BUTTON_ANY status.
        let upper = cl.keynames[key as usize].upper as usize;
        cl.kg.keys[upper].down = down;
        if down == qtrue {
            cl.kg.keys[upper].repeats += 1;
            if cl.kg.keys[upper].repeats == 1 {
                cl.kg.anykeydown = qtrue;
                cl.kg.keyDownCount += 1;
            }
        } else {
            cl.kg.keys[upper].repeats = 0;
            cl.kg.keyDownCount -= 1;
            if cl.kg.keyDownCount <= 0 {
                cl.kg.anykeydown = qfalse;
                cl.kg.keyDownCount = 0;
            }
        }

        // The console key is hardcoded, so the user can never unbind it.
        if key == fakeAscii_t::A_CONSOLE as c_int {
            if down != qtrue {
                return;
            }
            Con_ToggleConsole_f(view.common, cl);
            return;
        }

        // Keys can still be used for bound actions.
        let mut key = key;
        if down == qtrue && cl.cls.state == connstate_t::CA_CINEMATIC && cl.cls.keyCatchers == 0 {
            if Cvar_VariableValue(view.common, "com_cameraMode") == 0.0 {
                Cvar_Set(view, "nextdemo", "");
                key = fakeAscii_t::A_ESCAPE as c_int;
            }
        }

        // Escape is always handled specially.
        if key == fakeAscii_t::A_ESCAPE as c_int && down == qtrue {
            if cl.cls.keyCatchers & KEYCATCH_MESSAGE != 0 {
                // Clear message mode.
                Message_Key(view.common, cl, key);
                return;
            }

            // Escape always gets out of CGAME stuff.
            if cl.cls.keyCatchers & KEYCATCH_CGAME != 0 {
                cl.cls.keyCatchers &= !KEYCATCH_CGAME;
                VM_Call(
                    view.common,
                    cl.cgvm,
                    MpCgameExport::CG_EVENT_HANDLING as c_int,
                    &[CGAME_EVENT_NONE as isize],
                );
                return;
            }

            if cl.cls.keyCatchers & KEYCATCH_UI == 0 {
                if cl.cls.state == connstate_t::CA_ACTIVE && cl.clc.demoplaying != qtrue {
                    VM_Call(
                        view.common,
                        cl.uivm,
                        MpUiExport::UI_SET_ACTIVE_MENU as c_int,
                        &[UIMENU_INGAME as isize],
                    );
                } else {
                    CL_Disconnect_f(view, cl);
                    // SAFETY: view-constructor slot, single-threaded, no other live cast.
                    let snd = snd_from_view(view);
                    S_StopAllSounds(view.common, snd);
                    VM_Call(
                        view.common,
                        cl.uivm,
                        MpUiExport::UI_SET_ACTIVE_MENU as c_int,
                        &[UIMENU_MAIN as isize],
                    );
                }
                return;
            }

            VM_Call(
                view.common,
                cl.uivm,
                MpUiExport::UI_KEY_EVENT as c_int,
                &[key as isize, down as isize],
            );
            return;
        }

        // Key-up events only run actions if the bound command is a button
        // command (a leading `+`). These still run in console and menu mode,
        // to keep the character from continuing an action started before a
        // mode switch.
        if down != qtrue {
            let kb = cl.kg.keys[upper].binding;

            CL_AddKeyUpCommands(view.common, key, kb);

            if cl.cls.keyCatchers & KEYCATCH_UI != 0 && !cl.uivm.is_null() {
                VM_Call(
                    view.common,
                    cl.uivm,
                    MpUiExport::UI_KEY_EVENT as c_int,
                    &[key as isize, down as isize],
                );
            } else if cl.cls.keyCatchers & KEYCATCH_CGAME != 0 && !cl.cgvm.is_null() {
                VM_Call(
                    view.common,
                    cl.cgvm,
                    MpCgameExport::CG_KEY_EVENT as c_int,
                    &[key as isize, down as isize],
                );
            }

            return;
        }

        // Distribute the key-down event to the appropriate handler.
        if cl.cls.keyCatchers & KEYCATCH_CONSOLE != 0 {
            Console_Key(view, cl, key);
        } else if cl.cls.keyCatchers & KEYCATCH_UI != 0 {
            if !cl.uivm.is_null() {
                VM_Call(
                    view.common,
                    cl.uivm,
                    MpUiExport::UI_KEY_EVENT as c_int,
                    &[key as isize, down as isize],
                );
            }
        } else if cl.cls.keyCatchers & KEYCATCH_CGAME != 0 {
            if !cl.cgvm.is_null() {
                VM_Call(
                    view.common,
                    cl.cgvm,
                    MpCgameExport::CG_KEY_EVENT as c_int,
                    &[key as isize, down as isize],
                );
            }
        } else if cl.cls.keyCatchers & KEYCATCH_MESSAGE != 0 {
            Message_Key(view.common, cl, key);
        } else if cl.cls.state == connstate_t::CA_DISCONNECTED {
            Console_Key(view, cl, key);
        } else {
            // Send the bound action.
            let kb = cl.kg.keys[upper].binding;
            if !kb.is_null() {
                if *kb == b'+' as c_char {
                    let mut button = [0 as c_char; 1024];
                    let mut button_len = 0usize;
                    let mut i = 0usize;
                    loop {
                        if *kb.add(i) == b';' as c_char || *kb.add(i) == 0 {
                            button[button_len] = 0;
                            if button[0] == b'+' as c_char {
                                let button_str =
                                    core::ffi::CStr::from_ptr(button.as_ptr()).to_string_lossy();
                                let cmd = format!("{} {} {}\n", button_str, key, time);
                                Cbuf_AddText(view.common, &cmd);
                            } else {
                                let button_str =
                                    core::ffi::CStr::from_ptr(button.as_ptr()).to_string_lossy();
                                Cbuf_AddText(view.common, &button_str);
                                Cbuf_AddText(view.common, "\n");
                            }
                            button_len = 0;
                            while (*kb.add(i) as u8 as i32) <= b' ' as i32 && *kb.add(i) != 0
                                || *kb.add(i) == b';' as c_char
                            {
                                i += 1;
                            }
                        }
                        button[button_len] = *kb.add(i);
                        button_len += 1;
                        if *kb.add(i) == 0 {
                            break;
                        }
                        i += 1;
                    }
                } else {
                    // Down-only command.
                    if !cl.cgvm.is_null() && !cl.cl.mSharedMemory.is_null() {
                        // Do not run this unless cgame is inited and shared memory is valid.
                        let icc = cl.cl.mSharedMemory as *mut TCGIncomingConsoleCommand;
                        strcpy((*icc).conCommand.as_mut_ptr() as *mut c_char, kb);

                        if VM_Call(
                            view.common,
                            cl.cgvm,
                            MpCgameExport::CG_INCOMING_CONSOLE_COMMAND as c_int,
                            &[],
                        ) != 0
                        {
                            // Let mod authors filter client console messages so they can cut them off if they want.
                            let kb_str = core::ffi::CStr::from_ptr(kb).to_string_lossy();
                            Cbuf_AddText(view.common, &kb_str);
                            Cbuf_AddText(view.common, "\n");
                        } else if (*icc).conCommand[0] != 0 {
                            // The VM call says to execute this command in place.
                            let text = core::ffi::CStr::from_ptr(
                                (*icc).conCommand.as_ptr() as *const c_char
                            )
                            .to_string_lossy();
                            Cbuf_AddText(view.common, &text);
                            Cbuf_AddText(view.common, "\n");
                        }
                    } else {
                        // Otherwise, just run it.
                        let kb_str = core::ffi::CStr::from_ptr(kb).to_string_lossy();
                        Cbuf_AddText(view.common, &kb_str);
                        Cbuf_AddText(view.common, "\n");
                    }
                }
            }
        }
    }
}

/// Raven `Key_ClearStates`.
///
/// Source: `oracle/codemp/client/cl_keys.cpp:1689-1703`
pub fn Key_ClearStates(view: &mut EngineHostView, cl: &mut Client) {
    cl.kg.anykeydown = qfalse;

    for i in 0..MAX_KEYS {
        if cl.kg.keys[i].down == qtrue {
            CL_KeyEvent(view, cl, i as c_int, qfalse, 0);
        }
        cl.kg.keys[i].down = qfalse;
        cl.kg.keys[i].repeats = 0;
    }
}
