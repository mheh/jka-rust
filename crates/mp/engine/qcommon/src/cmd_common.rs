#![allow(non_snake_case, non_camel_case_types, clippy::too_many_arguments)]
//! `cmd_common.cpp` — the deferred-command buffer (`Cbuf_*`) and the tokenized
//! console-command argument table (`Cmd_Argc`/`Cmd_Argv`/`Cmd_Args*`), plus the
//! handful of built-in commands (`echo`, `wait`, `exec`, `vstr`) registered by
//! `Cmd_Init`.
//!
//! Source: `oracle/codemp/qcommon/cmd_common.cpp`

use core::ffi::{c_char, c_int};

use mp_host_interface::engine_host::EngineHost;

use mp_qshared::shared::limits::MAX_STRING_TOKENS;

use mp_qshared::shared::error_parm::errorParm_t;

use crate::cmd::cmd_consts::{MAX_CMD_BUFFER, MAX_CMD_LINE};
use crate::cmd_pc::{Cmd_ExecuteString, Cmd_List_f};
use crate::collision_world::CollisionWorld;
use crate::common::Common;
use crate::common_fns::Com_Memcpy;

extern "C" {
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn atoi(s: *const c_char) -> c_int;
    fn memmove(
        dest: *mut core::ffi::c_void,
        src: *const core::ffi::c_void,
        n: usize,
    ) -> *mut core::ffi::c_void;
}

// PORT-NOTE(q_math-reach): `Q_strncpyz`/`COM_DefaultExtension`/`va` (q_shared
// primitives) and `Cvar_VariableString` are ported only in `mp_game`, a tier
// above this crate's dependency graph (cm_load.rs/files_common.rs precedent)
// — not reachable here. `Com_Printf`/`Com_Error`/`FS_ReadFile`/`FS_FreeFile`
// are not yet landed in this crate under any importable path. Referenced by
// their exact Raven names, narrowed to this file's call-site shapes (no
// variadic `...` in safe Rust); each escalated as a missing symbol.
extern "Rust" {
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
    fn Com_Printf(common: &mut Common, msg: *const c_char);
    fn Com_Error(
        common: &mut Common,
        cm: &mut CollisionWorld,
        sv: &mut Server,
        rm: &mut RenderModels,
        host: &mut dyn EngineHost,
        code: errorParm_t,
        msg: *const c_char,
    );
    fn COM_DefaultExtension(path: *mut c_char, maxSize: c_int, extension: *const c_char);
    fn FS_ReadFile(
        common: &mut Common,
        cm: &mut CollisionWorld,
        rm: &mut RenderModels,
        host: &mut dyn EngineHost,
        qpath: *const c_char,
        buffer: *mut *mut (),
    ) -> c_int;
    fn FS_FreeFile(common: &mut Common, f: *mut ());
    fn Cvar_VariableString(common: &mut Common, name: *mut c_char) -> *const c_char;
    fn va(fmt: *const c_char, arg: *const c_char) -> *mut c_char;
}

// PORT-NOTE(rm-types): `RenderModels`/`Server` are state-receiver types pinned
// by the engine-fork-discovery preamble's receiver order; neither has landed
// in this crate yet (`Server` lives in `mp_engine_server`, which already
// depends on this crate — importing it here would cycle). Referenced by their
// exact resolved-signature names per the no-stub rule; reported as missing
// symbols (common_fns.rs precedent).
#[allow(dead_code)]
struct RenderModels;
#[allow(dead_code)]
struct Server;

/// `Cbuf_Init`.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:54-59`
pub fn Cbuf_Init(common: &mut Common) {
    // PORT-NOTE(cmd_t): `cmd_t` (`data`/`maxsize`/`cursize`) has no rosetta
    // row; referenced verbatim as the resolved `common.cmd_text` field shape
    // (missing-symbol escalation).
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
    // PORT-NOTE(Q_strncpyz): `Q_strncpyz` is currently only ported in the
    // `mp_game` tier (`crates/mp/game/src/q_shared.rs`); the engine crate
    // cannot depend on it (layering: game sits above engine). Referenced by
    // its exact Raven name per the no-stub rule — missing-symbol escalation.
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
            Com_Printf(
                common,
                b"Cbuf_AddText: overflow\n\0".as_ptr() as *const c_char,
            );
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
            // PORT-NOTE(Com_Printf): `Com_Printf` is not yet ported in this
            // crate (missing-symbol escalation) — referenced verbatim.
            Com_Printf(
                common,
                b"Cbuf_InsertText overflowed\n\0".as_ptr() as *const c_char,
            );
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
        Com_Printf(
            common,
            format!("{} ", unsafe {
                core::ffi::CStr::from_ptr(Cmd_Argv(common, i)).to_string_lossy()
            })
            .as_ptr() as *const c_char,
        );
    }
    Com_Printf(common, b"\n\0".as_ptr() as *const c_char);
}

/// `Cmd_Exec_f`.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:219-241`
pub fn Cmd_Exec_f(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    let mut filename: [c_char; native_types::MAX_QPATH as usize] =
        [0; native_types::MAX_QPATH as usize];

    if Cmd_Argc(common) != 2 {
        Com_Printf(
            common,
            b"exec <filename> : execute a script file\n\0".as_ptr() as *const c_char,
        );
        return;
    }

    unsafe {
        Q_strncpyz(
            filename.as_mut_ptr(),
            Cmd_Argv(common, 1),
            core::mem::size_of_val(&filename) as c_int,
        );
        COM_DefaultExtension(
            filename.as_mut_ptr(),
            core::mem::size_of_val(&filename) as c_int,
            b".cfg\0".as_ptr() as *const c_char,
        );
    }

    let mut f: *mut c_char = core::ptr::null_mut();
    let _len = FS_ReadFile(
        common,
        cm,
        rm,
        host,
        filename.as_ptr(),
        &mut f as *mut _ as *mut *mut (),
    );
    if f.is_null() {
        Com_Printf(common, b"couldn't exec %s\n\0".as_ptr() as *const c_char);
        return;
    }
    Com_Printf(common, b"execing %s\n\0".as_ptr() as *const c_char);

    Cbuf_InsertText(common, f as *const c_char);

    FS_FreeFile(common, f as *mut ());
}

/// `Cmd_Vstr_f`.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:251-261`
pub fn Cmd_Vstr_f(common: &mut Common) {
    if Cmd_Argc(common) != 2 {
        Com_Printf(
            common,
            b"vstr <variablename> : execute a variable command\n\0".as_ptr() as *const c_char,
        );
        return;
    }

    // PORT-NOTE(Cvar_VariableString/va): neither is ported in this crate yet
    // (missing-symbol escalation) — referenced by exact Raven name.
    let v = Cvar_VariableString(common, Cmd_Argv(common, 1));
    Cbuf_InsertText(common, va(b"%s\n\0".as_ptr() as *const c_char, v));
}

/// `Cbuf_Execute`.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:148-202`
pub fn Cbuf_Execute(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    let mut line: [c_char; MAX_CMD_LINE as usize] = [0; MAX_CMD_LINE as usize];

    while common.cmd_text.cursize != 0 {
        if common.cmd_wait != 0 {
            // skip out while text still remains in buffer, leaving it
            // for next frame
            common.cmd_wait -= 1;
            break;
        }

        // find a \n or ; line break
        let text = common.cmd_text.data;

        let mut quotes = 0;
        let mut i: c_int = 0;
        unsafe {
            while i < common.cmd_text.cursize {
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

            if i == common.cmd_text.cursize {
                common.cmd_text.cursize = 0;
            } else {
                i += 1;
                common.cmd_text.cursize -= i;
                memmove(
                    text as *mut core::ffi::c_void,
                    text.add(i as usize) as *const core::ffi::c_void,
                    common.cmd_text.cursize as usize,
                );
            }
        }

        // execute the command line
        Cmd_ExecuteString(common, cm, sv, rm, host, line.as_ptr() as *const c_char);
    }
}

/// `Cmd_Init`.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:501-507`
pub fn Cmd_Init(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    Cmd_AddCommand(
        common,
        cm,
        rm,
        host,
        b"cmdlist\0".as_ptr() as *const c_char,
        Cmd_List_f,
    );
    Cmd_AddCommand(
        common,
        cm,
        rm,
        host,
        b"exec\0".as_ptr() as *const c_char,
        Cmd_Exec_f,
    );
    Cmd_AddCommand(
        common,
        cm,
        rm,
        host,
        b"vstr\0".as_ptr() as *const c_char,
        Cmd_Vstr_f,
    );
    Cmd_AddCommand(
        common,
        cm,
        rm,
        host,
        b"echo\0".as_ptr() as *const c_char,
        Cmd_Echo_f,
    );
    Cmd_AddCommand(
        common,
        cm,
        rm,
        host,
        b"wait\0".as_ptr() as *const c_char,
        Cmd_Wait_f,
    );
}

/// `Cbuf_ExecuteText`.
///
/// Source: `oracle/codemp/qcommon/cmd_common.cpp:121-141`
pub fn Cbuf_ExecuteText(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    exec_when: c_int,
    text: *const c_char,
) {
    use mp_qshared::shared::cbuf_exec::cbufExec_t;
    use mp_qshared::shared::error_parm::errorParm_t;

    match exec_when {
        x if x == cbufExec_t::EXEC_NOW as c_int => {
            if !text.is_null() && unsafe { strlen(text) } > 0 {
                Cmd_ExecuteString(common, cm, sv, rm, host, text);
            } else {
                Cbuf_Execute(common, cm, sv, rm, host);
            }
        }
        x if x == cbufExec_t::EXEC_INSERT as c_int => {
            Cbuf_InsertText(common, text);
        }
        x if x == cbufExec_t::EXEC_APPEND as c_int => {
            Cbuf_AddText(common, text);
        }
        _ => {
            Com_Error(
                common,
                cm,
                sv,
                rm,
                host,
                errorParm_t::ERR_FATAL,
                b"Cbuf_ExecuteText: bad exec_when\0".as_ptr() as *const c_char,
            );
        }
    }
}
