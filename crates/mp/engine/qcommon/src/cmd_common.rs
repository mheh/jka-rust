#![allow(non_snake_case, non_camel_case_types, clippy::too_many_arguments)]
//! `cmd_common.cpp` — the deferred-command buffer (`Cbuf_*`) and the tokenized
//! console-command argument table (`Cmd_Argc`/`Cmd_Argv`/`Cmd_Args*`), plus the
//! handful of built-in commands (`echo`, `wait`, `exec`, `vstr`) registered by
//! `Cmd_Init`.
//!
//! Source: `oracle/codemp/qcommon/cmd_common.cpp`

use core::ffi::{c_char, c_int};

use mp_qshared::shared::limits::MAX_STRING_TOKENS;

use mp_qshared::shared::cbuf_exec::cbufExec_t;
use mp_qshared::shared::error_parm::errorParm_t;

use crate::cmd::cmd_consts::{MAX_CMD_BUFFER, MAX_CMD_LINE};
use crate::cmd_pc::{Cmd_ExecuteString, Cmd_List_f};
use crate::common::engine_host_view::EngineHostView;
use crate::common::Common;
use crate::common_fns::Com_Memcpy;

// Sweep: extern forward-declares eliminated. libc byte helpers (rule 3),
// real in-crate `Com_Printf`/`Com_Error`, and genuinely-unported callees
// referenced at their canonical future homes (q_string / cvar_fns / files /
// cmd). `Com_Printf`'s C-format `%s` sites were already lossy (the decl was
// non-variadic), so &str parity is preserved.
use libc::{atoi, memmove, strcat, strlen};

use crate::cmd::Cmd_AddCommand;
use crate::common::{com_error, com_printf};
use crate::cvar_fns::Cvar_VariableString;
use crate::files_common::{FS_FreeFile, FS_ReadFile};
use mp_qshared::shared::q_format::FmtArg;
use mp_qshared::shared::q_string::{va, COM_DefaultExtension, Q_strncpyz};

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
pub fn Cmd_Argc(common: &mut Common) -> c_int {
    common.cmd_argc
}

/// `Cmd_Argv`.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:309-314`
pub fn Cmd_Argv(common: &mut Common, arg: c_int) -> *mut c_char {
    if (arg as u32) >= common.cmd_argc as u32 {
        // Raven returns a `""` literal here; mirror with a static empty
        // C string (never written through by callers).
        static EMPTY: [c_char; 1] = [0];
        return EMPTY.as_ptr() as *mut c_char;
    }
    common.cmd_argv[arg as usize]
}

/// `Cmd_Args`.
///
/// Raven's `static char cmd_args[MAX_STRING_CHARS]` is genuine cross-call
/// scratch reused every invocation (the resolved signature returns a raw
/// pointer into it) — three-kind rule case 2/3: threaded as a `Common` field
/// (`cmd_args_buf`), never a hidden Rust static.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:336-349`
pub fn Cmd_Args(common: &mut Common) -> *mut c_char {
    common.cmd_args_buf[0] = 0;
    for i in 1..common.cmd_argc {
        unsafe {
            strcat(
                common.cmd_args_buf.as_mut_ptr() as *mut c_char,
                common.cmd_argv[i as usize],
            );
        }
        if i != common.cmd_argc - 1 {
            unsafe {
                strcat(
                    common.cmd_args_buf.as_mut_ptr() as *mut c_char,
                    b" \0".as_ptr() as *const c_char,
                );
            }
        }
    }
    common.cmd_args_buf.as_mut_ptr() as *mut c_char
}

/// `Cmd_ArgsFrom`.
///
/// Raven's `static char cmd_args[BIG_INFO_STRING]` — a distinct local static
/// from `Cmd_Args`'s same-named one (different scope/size); threaded as its
/// own `Common` field (`cmd_args_from_buf`) per the three-kind rule.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:358-373`
pub fn Cmd_ArgsFrom(common: &mut Common, arg: c_int) -> *mut c_char {
    let mut arg = arg;
    common.cmd_args_from_buf[0] = 0;
    if arg < 0 {
        arg = 0;
    }
    for i in arg..common.cmd_argc {
        unsafe {
            strcat(
                common.cmd_args_from_buf.as_mut_ptr() as *mut c_char,
                common.cmd_argv[i as usize],
            );
        }
        if i != common.cmd_argc - 1 {
            unsafe {
                strcat(
                    common.cmd_args_from_buf.as_mut_ptr() as *mut c_char,
                    b" \0".as_ptr() as *const c_char,
                );
            }
        }
    }
    common.cmd_args_from_buf.as_mut_ptr() as *mut c_char
}

/// `Cmd_TokenizeString`.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:398-491`
pub fn Cmd_TokenizeString(common: &mut Common, text_in: *const c_char) {
    // clear previous args
    common.cmd_argc = 0;

    if text_in.is_null() {
        return;
    }

    unsafe {
        let mut text = text_in;
        let mut text_out = common.cmd_tokenized.as_mut_ptr() as *mut c_char;

        loop {
            if common.cmd_argc == MAX_STRING_TOKENS as c_int {
                return; // this is usually something malicious
            }

            loop {
                // skip whitespace
                while *text != 0 && *text <= b' ' as c_char {
                    text = text.add(1);
                }
                if *text == 0 {
                    return; // all tokens parsed
                }

                // skip // comments
                if *text == b'/' as c_char && *text.add(1) == b'/' as c_char {
                    return; // all tokens parsed
                }

                // skip /* */ comments
                if *text == b'/' as c_char && *text.add(1) == b'*' as c_char {
                    while *text != 0 && (*text != b'*' as c_char || *text.add(1) != b'/' as c_char)
                    {
                        text = text.add(1);
                    }
                    if *text == 0 {
                        return; // all tokens parsed
                    }
                    text = text.add(2);
                } else {
                    break; // we are ready to parse a token
                }
            }

            // handle quoted strings
            if *text == b'"' as c_char {
                common.cmd_argv[common.cmd_argc as usize] = text_out;
                common.cmd_argc += 1;
                text = text.add(1);
                while *text != 0 && *text != b'"' as c_char {
                    *text_out = *text;
                    text_out = text_out.add(1);
                    text = text.add(1);
                }
                *text_out = 0;
                text_out = text_out.add(1);
                if *text == 0 {
                    return; // all tokens parsed
                }
                text = text.add(1);
                continue;
            }

            // regular token
            common.cmd_argv[common.cmd_argc as usize] = text_out;
            common.cmd_argc += 1;

            // skip until whitespace, quote, or command
            while *(text as *const u8) > b' ' {
                if *text == b'"' as c_char {
                    break;
                }

                if *text == b'/' as c_char && *text.add(1) == b'/' as c_char {
                    break;
                }

                // skip /* */ comments
                if *text == b'/' as c_char && *text.add(1) == b'*' as c_char {
                    break;
                }

                *text_out = *text;
                text_out = text_out.add(1);
                text = text.add(1);
            }

            *text_out = 0;
            text_out = text_out.add(1);

            if *text == 0 {
                return; // all tokens parsed
            }
        }
    }
}

/// `Cmd_Wait_f`.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:32-38`
pub fn Cmd_Wait_f(common: &mut Common) {
    if Cmd_Argc(common) == 2 {
        common.cmd_wait = unsafe { atoi(Cmd_Argv(common, 1)) };
    } else {
        common.cmd_wait = 1;
    }
}

/// `Cmd_ArgvBuffer`.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:324-326`
pub fn Cmd_ArgvBuffer(common: &mut Common, arg: c_int, buffer: *mut c_char, bufferLength: c_int) {
    Q_strncpyz(buffer, Cmd_Argv(common, arg), bufferLength);
}

/// `Cmd_ArgsBuffer`.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:383-385`
pub fn Cmd_ArgsBuffer(common: &mut Common, buffer: *mut c_char, bufferLength: c_int) {
    Q_strncpyz(buffer, Cmd_Args(common), bufferLength);
}

/// `Cbuf_AddText`.
///
/// Adds command text at the end of the buffer, does NOT add a final \n.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:68-78`
pub fn Cbuf_AddText(common: &mut Common, text: *const c_char) {
    unsafe {
        let l = strlen(text) as c_int;

        if common.cmd_text.cursize + l >= common.cmd_text.maxsize {
            com_printf(common, "Cbuf_AddText: overflow\n");
            return;
        }
        Com_Memcpy(
            common.cmd_text.data.add(common.cmd_text.cursize as usize) as *mut (),
            text as *const (),
            l as usize,
        );
        common.cmd_text.cursize += l;
    }
}

/// `Cbuf_InsertText`.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:91-113`
pub fn Cbuf_InsertText(common: &mut Common, text: *const c_char) {
    unsafe {
        let len = strlen(text) as c_int + 1;
        if len + common.cmd_text.cursize > common.cmd_text.maxsize {
            com_printf(common, "Cbuf_InsertText overflowed\n");
            return;
        }

        // move the existing command text
        let data = common.cmd_text.data;
        let mut i = common.cmd_text.cursize - 1;
        while i >= 0 {
            *data.add((i + len) as usize) = *data.add(i as usize);
            i -= 1;
        }

        // copy the new text in
        Com_Memcpy(data as *mut (), text as *const (), (len - 1) as usize);

        // add a \n
        *data.add((len - 1) as usize) = b'\n';

        common.cmd_text.cursize += len;
    }
}

/// `Cmd_Echo_f`.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:271-278`
pub fn Cmd_Echo_f(common: &mut Common) {
    for i in 1..Cmd_Argc(common) {
        unsafe {
            let arg = core::ffi::CStr::from_ptr(Cmd_Argv(common, i)).to_string_lossy();
            let msg = format!("{} ", arg);
            com_printf(common, &msg);
        }
    }
    com_printf(common, "\n");
}

/// `Cmd_Exec_f`.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:219-241`
pub fn Cmd_Exec_f(view: &mut EngineHostView) {
    let mut filename: [c_char; native_types::MAX_QPATH as usize] =
        [0; native_types::MAX_QPATH as usize];

    if Cmd_Argc(view.common) != 2 {
        com_printf(view.common, "exec <filename> : execute a script file\n");
        return;
    }

    Q_strncpyz(
        filename.as_mut_ptr(),
        Cmd_Argv(view.common, 1),
        core::mem::size_of_val(&filename) as c_int,
    );
    COM_DefaultExtension(
        filename.as_mut_ptr(),
        core::mem::size_of_val(&filename) as c_int,
        b".cfg\0".as_ptr() as *const c_char,
    );

    let mut f: *mut c_char = core::ptr::null_mut();
    let _len = FS_ReadFile(view, filename.as_ptr(), &mut f as *mut _ as *mut *mut ());
    let name = unsafe { core::ffi::CStr::from_ptr(filename.as_ptr()) }
        .to_string_lossy()
        .into_owned();
    if f.is_null() {
        com_printf(view.common, &format!("couldn't exec {name}\n"));
        return;
    }
    com_printf(view.common, &format!("execing {name}\n"));

    Cbuf_InsertText(view.common, f as *const c_char);

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

    unsafe {
        let arg1 = Cmd_Argv(common, 1);
        let v = Cvar_VariableString(common, arg1);
        Cbuf_InsertText(
            common,
            va(b"%s\n\0".as_ptr() as *const c_char, &[FmtArg::cstr(v)]),
        );
    }
}

/// `Cbuf_Execute`.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:148-202`
pub fn Cbuf_Execute(view: &mut EngineHostView) {
    let mut line: [c_char; MAX_CMD_LINE as usize] = [0; MAX_CMD_LINE as usize];

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

            Com_Memcpy(line.as_mut_ptr() as *mut (), text as *const (), i as usize);
            line[i as usize] = 0;

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
        Cmd_ExecuteString(view, line.as_ptr() as *const c_char);
    }
}

/// `Cmd_Init`.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:501-507`
pub fn Cmd_Init(view: &mut EngineHostView) {
    Cmd_AddCommand(
        view,
        b"cmdlist\0".as_ptr() as *const c_char,
        Some(|view| Cmd_List_f(view.common)),
    );
    Cmd_AddCommand(
        view,
        b"exec\0".as_ptr() as *const c_char,
        Some(|view| Cmd_Exec_f(view)),
    );
    Cmd_AddCommand(
        view,
        b"vstr\0".as_ptr() as *const c_char,
        Some(|view| Cmd_Vstr_f(view.common)),
    );
    Cmd_AddCommand(
        view,
        b"echo\0".as_ptr() as *const c_char,
        Some(|view| Cmd_Echo_f(view.common)),
    );
    Cmd_AddCommand(
        view,
        b"wait\0".as_ptr() as *const c_char,
        Some(|view| Cmd_Wait_f(view.common)),
    );
}

/// `Cbuf_ExecuteText`.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:121-141`
pub fn Cbuf_ExecuteText(view: &mut EngineHostView, exec_when: c_int, text: *const c_char) {
    match exec_when {
        x if x == cbufExec_t::EXEC_NOW as c_int => {
            if !text.is_null() && unsafe { strlen(text) } > 0 {
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
