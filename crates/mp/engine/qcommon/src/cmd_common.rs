#![allow(non_snake_case, non_camel_case_types, clippy::too_many_arguments)]
//! `cmd_common.cpp` — the deferred-command buffer (`Cbuf_*`) and the tokenized
//! console-command argument table (`Cmd_Argc`/`Cmd_Argv`/`Cmd_Args*`), plus the
//! handful of built-in commands (`echo`, `wait`, `exec`, `vstr`) registered by
//! `Cmd_Init`.
//!
//! Source: `oracle/codemp/qcommon/cmd_common.cpp`

use core::ffi::{c_char, c_int, CStr};
use core::slice::{from_raw_parts, from_raw_parts_mut};

use mp_qshared::shared::limits::MAX_STRING_TOKENS;

use mp_qshared::shared::cbuf_exec::cbufExec_t;
use mp_qshared::shared::error_parm::errorParm_t;
use native_string::q_strncpyz::Q_strncpyz;

use crate::cmd::cmd_consts::{MAX_CMD_BUFFER, MAX_CMD_LINE};
use crate::cmd_pc::{Cmd_ExecuteString, Cmd_List_f};
use crate::common::engine_host_view::EngineHostView;
use crate::common::Common;

use libc::memmove;
use native_string::atoi::atoi;
use native_string::latin1_to_string;

use crate::cmd::Cmd_AddCommand;
use crate::common::{com_error, com_printf};
use crate::cvar_fns::Cvar_VariableString;
use crate::files_common::{FS_FreeFile, FS_ReadFile};

/// `Cbuf_Init`.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:54-59`
pub fn Cbuf_Init(common: &mut Common) {
    common.cmd_text.data = common.cmd_text_buf.as_mut_ptr();
    common.cmd_text.maxsize = MAX_CMD_BUFFER as c_int;
    common.cmd_text.cursize = 0;
}

/// `Cmd_Argc`.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:300-302`
pub fn Cmd_Argc(common: &Common) -> c_int {
    common.cmd_argv.len() as c_int
}

/// `Cmd_Argv` (out of range reads Raven's `""` literal).
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:309-314`
pub fn Cmd_Argv(common: &Common, arg: c_int) -> &str {
    common.cmd_argv.get(arg as usize).map_or("", |s| s.as_str())
}

/// `Cmd_ArgvBuffer` — copy one argument into a caller buffer, truncated to
/// `bufferLength`. The VM trap seam owns the buffer, so it arrives as a raw
/// pointer the same way `Cvar_VariableStringBuffer`'s does.
///
/// # Safety
/// `buffer` must point to at least `bufferLength` writable bytes.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:324-326`
pub unsafe fn Cmd_ArgvBuffer(
    common: &Common,
    arg: c_int,
    buffer: *mut c_char,
    bufferLength: c_int,
) {
    let dest = core::slice::from_raw_parts_mut(buffer, bufferLength as usize);
    Q_strncpyz(dest, Cmd_Argv(common, arg), bufferLength as usize);
}

/// `Cmd_Args` — args 1.. space-joined. Raven's `static char
/// cmd_args[MAX_STRING_CHARS]` return scratch becomes the owned return (its
/// size was an unchecked `strcat` target, no defined cap to keep).
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:336-349`
pub fn Cmd_Args(common: &Common) -> String {
    common.cmd_argv.get(1..).unwrap_or(&[]).join(" ")
}

/// `Cmd_ArgsFrom` — args `arg`.. space-joined (negative clamps to 0).
/// Raven's distinct `static char cmd_args[BIG_INFO_STRING]` scratch becomes
/// the owned return.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:358-373`
pub fn Cmd_ArgsFrom(common: &Common, arg: c_int) -> String {
    let arg = arg.max(0) as usize;
    common.cmd_argv.get(arg..).unwrap_or(&[]).join(" ")
}

/// `Cmd_TokenizeString`. Raven's signed-`char` whitespace skip (high bytes
/// count as whitespace) and its unsigned token loop are both kept.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:398-491`
pub fn Cmd_TokenizeString(common: &mut Common, text_in: &str) {
    // clear previous args
    common.cmd_argv.clear();

    let text = text_in.as_bytes();
    let mut i = 0usize;

    loop {
        if common.cmd_argv.len() == MAX_STRING_TOKENS {
            return; // this is usually something malicious
        }

        loop {
            // skip whitespace
            while i < text.len() && text[i] != 0 && (text[i] as i8) <= b' ' as i8 {
                i += 1;
            }
            if i >= text.len() || text[i] == 0 {
                return; // all tokens parsed
            }

            // skip // comments
            if text[i] == b'/' && text.get(i + 1) == Some(&b'/') {
                return; // all tokens parsed
            }

            // skip /* */ comments
            if text[i] == b'/' && text.get(i + 1) == Some(&b'*') {
                while i < text.len()
                    && text[i] != 0
                    && !(text[i] == b'*' && text.get(i + 1) == Some(&b'/'))
                {
                    i += 1;
                }
                if i >= text.len() || text[i] == 0 {
                    return; // all tokens parsed
                }
                i += 2;
            } else {
                break; // we are ready to parse a token
            }
        }

        // handle quoted strings
        if text[i] == b'"' {
            i += 1;
            let start = i;
            while i < text.len() && text[i] != 0 && text[i] != b'"' {
                i += 1;
            }
            common.cmd_argv.push(latin1_to_string(&text[start..i]));
            if i >= text.len() || text[i] == 0 {
                return; // all tokens parsed
            }
            i += 1;
            continue;
        }

        // regular token: skip until whitespace, quote, or command
        let start = i;
        while i < text.len() && text[i] > b' ' {
            if text[i] == b'"' {
                break;
            }

            if text[i] == b'/' && text.get(i + 1) == Some(&b'/') {
                break;
            }

            // skip /* */ comments
            if text[i] == b'/' && text.get(i + 1) == Some(&b'*') {
                break;
            }

            i += 1;
        }
        common.cmd_argv.push(latin1_to_string(&text[start..i]));

        if i >= text.len() || text[i] == 0 {
            return; // all tokens parsed
        }
    }
}

/// `Cmd_Wait_f`.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:32-38`
pub fn Cmd_Wait_f(common: &mut Common) {
    if Cmd_Argc(common) == 2 {
        common.cmd_wait = atoi(Cmd_Argv(common, 1));
    } else {
        common.cmd_wait = 1;
    }
}

// Raven `Cmd_ArgvBuffer` (cmd_common.cpp:324-326) is inlined at its one
// caller, sv_game's `G_ARGV` trap arm (the module out-buffer seam).
// Raven `Cmd_ArgsBuffer` (cmd_common.cpp:383-385) had no caller in the
// dedicated island (its consumers are the client/UI trap arms) and was
// dropped there; the client-side caller has since arrived (the renderer's
// `R_WorldEffect_f`), so it is ported below.

/// `Cmd_ArgsBuffer`. Raven: "The interpreted versions use this because they
/// can't have pointers returned to them." The caller's `char *buffer` +
/// `bufferLength` pair becomes the owned return of [`Cmd_Args`], truncated by
/// Raven's `Q_strncpyz` to `buffer_length - 1` bytes (backed off to the
/// nearest `char` boundary so the return stays a `String`).
///
/// PORT-NOTE: the truncation length is measured in UTF-8 bytes, which
/// overstates the Latin-1 wire length for any byte >= 0x80; the sole call site
/// passes 2048, far inside the cap, so no live path can hit the difference.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:383-385`
pub fn Cmd_ArgsBuffer(common: &Common, buffer_length: usize) -> String {
    // Raven's `Q_strncpyz` fatals on a zero-length destination.
    // Source: oracle/codemp/game/q_shared.c:834-836
    if buffer_length < 1 {
        com_error(
            errorParm_t::ERR_FATAL,
            "Q_strncpyz: destsize < 1".to_string(),
        );
    }

    let mut args = Cmd_Args(common);
    let mut max = buffer_length.saturating_sub(1);
    if args.len() > max {
        while !args.is_char_boundary(max) {
            max -= 1;
        }
        args.truncate(max);
    }
    args
}

/// `Cbuf_AddText`.
///
/// Adds command text at the end of the buffer, does NOT add a final \n.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:68-78`
pub fn Cbuf_AddText(common: &mut Common, text: &str) {
    let bytes = text.as_bytes();
    let l = bytes.len() as c_int;

    if common.cmd_text.cursize + l >= common.cmd_text.maxsize {
        com_printf(common, "Cbuf_AddText: overflow\n");
        return;
    }
    unsafe {
        from_raw_parts_mut(
            common.cmd_text.data.add(common.cmd_text.cursize as usize),
            bytes.len(),
        )
        .copy_from_slice(bytes);
    }
    common.cmd_text.cursize += l;
}

/// `Cbuf_InsertText`.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:91-113`
pub fn Cbuf_InsertText(common: &mut Common, text: &str) {
    let bytes = text.as_bytes();
    let len = bytes.len() as c_int + 1;
    if len + common.cmd_text.cursize > common.cmd_text.maxsize {
        com_printf(common, "Cbuf_InsertText overflowed\n");
        return;
    }

    unsafe {
        // move the existing command text
        let data = common.cmd_text.data;
        let mut i = common.cmd_text.cursize - 1;
        while i >= 0 {
            *data.add((i + len) as usize) = *data.add(i as usize);
            i -= 1;
        }

        // copy the new text in
        from_raw_parts_mut(data, bytes.len()).copy_from_slice(bytes);

        // add a \n
        *data.add((len - 1) as usize) = b'\n';
    }

    common.cmd_text.cursize += len;
}

/// `Cmd_Echo_f`.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:271-278`
pub fn Cmd_Echo_f(common: &mut Common) {
    for i in 1..Cmd_Argc(common) {
        let msg = format!("{} ", Cmd_Argv(common, i));
        com_printf(common, &msg);
    }
    com_printf(common, "\n");
}

/// `Cmd_Exec_f`.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:219-241`
pub fn Cmd_Exec_f(view: &mut EngineHostView) {
    if Cmd_Argc(view.common) != 2 {
        com_printf(view.common, "exec <filename> : execute a script file\n");
        return;
    }

    // Raven's Q_strncpyz into a MAX_QPATH buffer + COM_DefaultExtension: the
    // ".cfg" default appends when the name has no extension.
    let mut filename = Cmd_Argv(view.common, 1).to_string();
    if !filename
        .rsplit('/')
        .next()
        .unwrap_or(&filename)
        .contains('.')
    {
        filename.push_str(".cfg");
    }

    let mut f: *mut c_char = core::ptr::null_mut();
    let _len = FS_ReadFile(view, &filename, &mut f as *mut _ as *mut *mut ());
    if f.is_null() {
        com_printf(view.common, &format!("couldn't exec {filename}\n"));
        return;
    }
    com_printf(view.common, &format!("execing {filename}\n"));

    let text = latin1_to_string(unsafe { CStr::from_ptr(f) }.to_bytes());
    Cbuf_InsertText(view.common, &text);

    FS_FreeFile(view.common, f as *mut ());
}

/// `Cmd_Vstr_f`.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:251-261`
pub fn Cmd_Vstr_f(common: &mut Common) {
    if Cmd_Argc(common) != 2 {
        com_printf(common, "vstr <variablename> : execute a variable command\n");
        return;
    }

    let cmd = format!("{}\n", Cvar_VariableString(common, Cmd_Argv(common, 1)));
    Cbuf_InsertText(common, &cmd);
}

/// `Cbuf_Execute`. Raven's `char line[MAX_CMD_LINE]` copy scratch becomes an
/// owned `String` built from the ring before the deletion `memmove`; the
/// `MAX_CMD_LINE - 1` truncation clamp is kept (including its quirk: the
/// unexecuted tail of an over-long line stays in the buffer as the next line).
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:148-202`
pub fn Cbuf_Execute(view: &mut EngineHostView) {
    while view.common.cmd_text.cursize != 0 {
        if view.common.cmd_wait != 0 {
            // skip out while text still remains in buffer, leaving it
            // for next frame
            view.common.cmd_wait -= 1;
            break;
        }

        // find a \n or ; line break
        let text = view.common.cmd_text.data;

        let mut quotes = 0;
        let mut i: c_int = 0;
        let line;
        unsafe {
            while i < view.common.cmd_text.cursize {
                let c = *text.add(i as usize);
                if c == b'"' {
                    quotes += 1;
                }
                if (quotes & 1) == 0 && c == b';' {
                    break; // don't break if inside a quoted string
                }
                if c == b'\n' || c == b'\r' {
                    break;
                }
                i += 1;
            }

            if i >= MAX_CMD_LINE as c_int - 1 {
                i = MAX_CMD_LINE as c_int - 1;
            }

            line = latin1_to_string(from_raw_parts(text, i as usize));

            // delete the text from the command buffer and move remaining commands down
            // this is necessary because commands (exec) can insert data at the
            // beginning of the text buffer

            if i == view.common.cmd_text.cursize {
                view.common.cmd_text.cursize = 0;
            } else {
                i += 1;
                view.common.cmd_text.cursize -= i;
                memmove(
                    text as *mut core::ffi::c_void,
                    text.add(i as usize) as *const core::ffi::c_void,
                    view.common.cmd_text.cursize as usize,
                );
            }
        }

        // execute the command line
        Cmd_ExecuteString(view, &line);
    }
}

/// `Cmd_Init`.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:501-507`
pub fn Cmd_Init(view: &mut EngineHostView) {
    Cmd_AddCommand(view, "cmdlist", Some(|view| Cmd_List_f(view.common)));
    Cmd_AddCommand(view, "exec", Some(|view| Cmd_Exec_f(view)));
    Cmd_AddCommand(view, "vstr", Some(|view| Cmd_Vstr_f(view.common)));
    Cmd_AddCommand(view, "echo", Some(|view| Cmd_Echo_f(view.common)));
    Cmd_AddCommand(view, "wait", Some(|view| Cmd_Wait_f(view.common)));
}

/// `Cbuf_ExecuteText`.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:121-141`
pub fn Cbuf_ExecuteText(view: &mut EngineHostView, exec_when: c_int, text: &str) {
    match exec_when {
        x if x == cbufExec_t::EXEC_NOW as c_int => {
            if !text.is_empty() {
                Cmd_ExecuteString(view, text);
            } else {
                Cbuf_Execute(view);
            }
        }
        x if x == cbufExec_t::EXEC_INSERT as c_int => {
            Cbuf_InsertText(view.common, text);
        }
        x if x == cbufExec_t::EXEC_APPEND as c_int => {
            Cbuf_AddText(view.common, text);
        }
        _ => {
            com_error(
                errorParm_t::ERR_FATAL,
                "Cbuf_ExecuteText: bad exec_when".to_string(),
            );
        }
    }
}
