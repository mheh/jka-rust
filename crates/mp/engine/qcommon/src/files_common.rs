#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    unused_variables,
    unused_mut,
    unused_unsafe,
    clippy::too_many_arguments
)]

//! `files_common.cpp` — platform-independent filesystem core: init/shutdown
//! state, search-path build helpers, filename normalization/comparison, and
//! the write-file/printf-to-file surface.
//!
//! Source: `oracle/codemp/qcommon/files_common.cpp`

use core::ffi::{c_char, c_int, c_long, c_uint, c_void, CStr};
use std::ffi::CString;

use mp_qshared::common::mp::qcommon::tags::memtag_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::{qboolean, qfalse, qtrue};
use native_types::fileHandle_t;

use crate::common::engine_host_view::EngineHostView;
use crate::common::Common;
use crate::files::files_consts::BASEGAME;

// Sweep: extern forward-declares eliminated. Real in-crate callees imported
// (`com_error`, `com_printf`, `Com_StartupVariable`); q_shared helpers
// (`Com_sprintf`, `Q_strncpyz`) and this file's own not-yet-ported `FS_*`
// (files_common.cpp subject) referenced at their canonical homes — the `FS_*`
// left bare at their home; reported.
use crate::common::{com_error, com_printf};
use crate::common_fns::Com_StartupVariable;
use mp_qshared::shared::cvar::{CVAR_INIT, CVAR_SYSTEMINFO};
use mp_qshared::shared::limits::MAX_OSPATH;
use mp_qshared::shared::swap::LittleLong;
use native_string::atoi::atoi;
use native_string::filter::Com_FilterPath;
use native_string::q_string::{Q_stricmp, Q_stricmpn, Q_strlwr};
use native_string::q_strncpyz::Q_strncpyz;

use crate::cmd_common::{Cbuf_AddText, Cmd_Argc, Cmd_Argv, Cmd_TokenizeString};
use crate::cmd_pc::{Cmd_AddCommand, Cmd_RemoveCommand};
use crate::common_fns::{Com_DPrintf, Com_Memcpy, Com_Memset, Com_SafeMode};
use crate::cvar_fns::{Cvar_Get, Cvar_Set};
use crate::files::directory_t::directory_t;
use crate::files::file_handle_data_t::fileHandleData_t;
use crate::files::file_in_pack_s::fileInPack_t;
use crate::files::files_consts::{
    DEMO_PAK_CHECKSUM, MAX_FILEHASH_SIZE, MAX_FOUND_FILES, MAX_PAKFILES, MAX_SEARCH_PATHS,
    MAX_ZPATH,
};
use crate::files::pack_t::pack_t;
use crate::files::searchpath_s::searchpath_t;
use crate::files::unz_types::{unz_file_info, unz_global_info, unz_s};
use crate::files_pc::{
    FS_ClearPakReferences, FS_ConvertPath, FS_Flush, FS_HashFileName, FS_PakIsPure, FS_PathCmp,
    FS_ReorderPurePaks, FS_ReturnPath, FS_ShiftedStrStr,
};
use crate::md4_fns::{Com_BlockChecksum, Com_BlockChecksumKey};
use crate::qcommon::filesystem_limits::{
    FS_CGAME_REF, FS_GENERAL_REF, FS_QAGAME_REF, FS_UI_REF, MAX_FILE_HANDLES,
};
use crate::qcommon::protocol::PROTOCOL_VERSION;
use crate::z_memman_pc::{Hunk_AllocateTempMemory, Z_Free, Z_Malloc};
// Genuinely-unported callees referenced at their canonical homes (honest
// E0425/E0432 escalations): the `unz*` zip seam (open user decision) at
// `crate::files::unz_file`; `Sys_*` platform I/O at `native_platform`.
use crate::files::unz_file::{
    unzClose, unzCloseCurrentFile, unzGetCurrentFileInfo, unzGetCurrentFileInfoPosition,
    unzGetGlobalInfo, unzGoToFirstFile, unzGoToNextFile, unzOpen, unzOpenCurrentFile, unzReOpen,
    unzReadCurrentFile, unzSetCurrentFileInfoPosition, UNZ_OK,
};
use native_platform::{
    sys_fopen, Sys_DefaultCDPath, Sys_DefaultHomePath, Sys_DefaultInstallPath, Sys_EndStreamedFile,
    Sys_ListFiles, Sys_Mkdir,
};

/// Raven `FS_Initialized`.
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:243-245`
pub fn FS_Initialized(common: &mut Common) -> qboolean {
    if !common.fs_searchpaths.is_null() {
        qtrue
    } else {
        qfalse
    }
}

/// Raven `FS_LoadStack`.
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:253-256`
pub fn FS_LoadStack(common: &mut Common) -> c_int {
    common.fs_loadStack
}

/// Raven `FS_ReplaceSeparators`.
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:277-285`
pub fn FS_ReplaceSeparators(path: *mut c_char) {
    // Raven's platform `PATH_SEP` macro normalizes to `/` on this unix target.
    unsafe {
        let mut s = path;
        while *s != 0 {
            if *s == b'/' as c_char || *s == b'\\' as c_char {
                *s = b'/' as c_char;
            }
            s = s.add(1);
        }
    }
}

/// Raven `FS_FilenameCompare` over bytes — the canonical body (§C7 bool:
/// `true` = equal, Raven's `0`). Case-insensitive with `\` and `:` both
/// folding to `/`.
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:345-372`
pub fn FS_FilenameCompareBytes(s1: &[u8], s2: &[u8]) -> bool {
    let fold = |c: u8| -> u8 {
        let c = c.to_ascii_uppercase();
        if c == b'\\' || c == b':' {
            b'/'
        } else {
            c
        }
    };
    s1.len() == s2.len() && s1.iter().zip(s2.iter()).all(|(&a, &b)| fold(a) == fold(b))
}

/// Raven `FS_FilenameCompare` (§C7 bool: `true` = equal, Raven's `0`).
/// Callers still holding C data convert at the site and call
/// [`FS_FilenameCompareBytes`] directly.
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:345-372`
pub fn FS_FilenameCompare(s1: &str, s2: &str) -> bool {
    FS_FilenameCompareBytes(s1.as_bytes(), s2.as_bytes())
}

/// Raven `FS_BuildOSPath` (single-`qpath` overload). Raven's flip-flop
/// return statics collapse into the owned `String` return.
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:294-315`
pub fn FS_BuildOSPath(common: &Common, qpath: &str) -> String {
    // Fix for filenames that are given to FS with a leading "/" (/botfiles/Foo)
    let qpath = qpath.strip_prefix(['\\', '/']).unwrap_or(qpath);

    // FIXME VVFIXME Holy crap this is wrong.
    //	Com_sprintf( temp, sizeof(temp), "/%s/%s", fs_gamedirvar->string, qpath );
    let temp = format!("/{}/{}", "base", qpath).replace('\\', "/"); // FS_ReplaceSeparators

    format!("{}{}", common.cvar(common.fs_basepath).string, temp)
}

/// Raven `FS_BuildOSPath` (`base`/`game`/`qpath` overload).
///
/// Raven overloads `FS_BuildOSPath` by arity; Rust has no fn overloading, so
/// this 4-arg overload is named `FS_BuildOSPath4`. Raven's four rotating
/// return statics collapse into the owned `String` return; Raven's
/// null-or-empty `game` default is the `""` arm.
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:317-336`
pub fn FS_BuildOSPath4(common: &Common, base: &str, game: &str, qpath: &str) -> String {
    let game = if game.is_empty() {
        &common.fs_gamedir
    } else {
        game
    };

    let temp = format!("/{game}/{qpath}").replace('\\', "/"); // FS_ReplaceSeparators

    format!("{base}{temp}")
}

/// Raven `FS_CheckInit`.
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:229-235`
pub fn FS_CheckInit(common: &mut Common) {
    if common.initialized == qfalse {
        unsafe {
            com_error(
                errorParm_t::ERR_FATAL,
                "Filesystem call made without initialization\n".to_string(),
            );
        }
    }
}

/// Raven `FS_Printf`.
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:375-384`
// Raven's C `...` variadic collapses to a pre-formatted `msg: &str` — Rust has
// no safe C-variadic fn definitions; the caller formats `msg` before the call.
pub fn FS_Printf(common: &mut Common, h: fileHandle_t, msg: &str) {
    let bytes = msg.as_bytes();
    unsafe {
        FS_Write(common, bytes.as_ptr() as *const (), bytes.len() as c_int, h);
    }
}

/// Raven `FS_WriteFile`.
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:401-421`
pub fn FS_WriteFile(common: &mut Common, qpath: &str, buffer: *const (), size: c_int) {
    if common.fs_searchpaths.is_null() {
        com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
    }

    if buffer.is_null() {
        com_error(
            errorParm_t::ERR_FATAL,
            "FS_WriteFile: NULL parameter".to_string(),
        );
    }

    let f = FS_FOpenFileWrite(common, qpath);
    if f == 0 {
        com_printf(common, &format!("Failed to open {qpath}\n"));
        return;
    }

    unsafe {
        FS_Write(common, buffer, size, f);
    }

    FS_FCloseFile(common, f);
}

/// Raven `FS_InitFilesystem`.
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:480-511`
pub fn FS_InitFilesystem(view: &mut EngineHostView) {
    unsafe {
        // allow command line parms to override our defaults
        // we have to specially handle this, because normal command
        // line variable sets don't happen until after the filesystem
        // has already been initialized
        Com_StartupVariable(view, Some("fs_cdpath"));
        Com_StartupVariable(view, Some("fs_basepath"));
        Com_StartupVariable(view, Some("fs_homepath"));
        Com_StartupVariable(view, Some("fs_game"));
        Com_StartupVariable(view, Some("fs_copyfiles"));
        Com_StartupVariable(view, Some("fs_restrict"));

        // try to start up normally
        FS_Startup(view, BASEGAME);
        view.common.initialized = qtrue;

        // see if we are going to allow add-ons
        FS_SetRestrictions(view);

        // if we can't find default.cfg, assume that the paths are
        // busted and error out now, rather than getting an unreadable
        // graphics screen when the font fails to load
        let mut buffer: *mut () = core::ptr::null_mut();
        if FS_ReadFile(view, "mpdefault.cfg", &mut buffer as *mut *mut ()) <= 0 {
            // bk001208 - SafeMode see below, FIXME?
            com_error(
                errorParm_t::ERR_FATAL,
                "Couldn't load mpdefault.cfg".to_string(),
            );
        }

        view.common.lastValidBase =
            cap_ospath(&view.common.cvar(view.common.fs_basepath).string).to_string();
        view.common.lastValidGame =
            cap_ospath(&view.common.cvar(view.common.fs_gamedirvar).string).to_string();

        // bk001208 - SafeMode see below, FIXME?
    }
}

// ===========================================================================
// files_pc.cpp filesystem I/O surface.
//
// Ported into the `files_common` module home: the crate's FS API layout has
// every public `FS_*` reachable at `crate::files_common::*` (cm_load/cm_shader/
// cmd_common/common_fns and files_pc.rs all import from here).
// ===========================================================================

// Raven's platform `PATH_SEP` normalizes to '/' on this unix target (ruling 8,
// mirroring `FS_ReplaceSeparators` above).
const PATH_SEP: c_char = b'/' as c_char;

/// Small owned-copy helper for `%s`-style debug prints of C strings.
/// Raven's `Q_stricmp(filename + len - N, ".ext")` tail checks (§19: names
/// shorter than the extension read out of bounds in C; the length guard is
/// the defined pick).
fn tail_matches(name: &str, ext: &str) -> bool {
    name.len() >= ext.len() && name[name.len() - ext.len()..].eq_ignore_ascii_case(ext)
}

/// `Q_strncpyz`'s `MAX_OSPATH` cap over owned strings — the silent 1023-byte
/// cut every former `char[MAX_OSPATH]` write enforced (backing off to a char
/// boundary; Raven cut mid-byte, C didn't care).
fn cap_ospath(s: &str) -> &str {
    if s.len() < MAX_OSPATH {
        return s;
    }
    let mut end = MAX_OSPATH - 1;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

fn cstr(p: *const c_char) -> String {
    if p.is_null() {
        return String::new();
    }
    unsafe { core::ffi::CStr::from_ptr(p).to_string_lossy().into_owned() }
}

/// Raven `fs_scrambledProductId` — obfuscated retail product-id table.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:24-38`
static FS_SCRAMBLED_PRODUCT_ID: [u8; 165] = [
    42, 143, 149, 190, 10, 197, 225, 133, 243, 63, 189, 182, 226, 56, 143, 17, 215, 37, 197, 218,
    50, 103, 24, 235, 246, 191, 183, 149, 160, 170, 230, 52, 176, 231, 15, 194, 236, 247, 159, 168,
    132, 154, 24, 133, 67, 85, 36, 97, 99, 86, 117, 189, 212, 156, 236, 153, 68, 10, 196, 241, 39,
    219, 156, 88, 93, 198, 200, 232, 142, 67, 45, 209, 53, 186, 228, 241, 162, 127, 213, 83, 7,
    121, 11, 93, 123, 243, 148, 240, 229, 42, 42, 6, 215, 239, 112, 120, 240, 244, 104, 12, 38, 47,
    201, 253, 223, 208, 154, 69, 141, 157, 32, 117, 166, 146, 236, 59, 15, 223, 52, 89, 133, 64,
    201, 56, 119, 25, 211, 152, 159, 11, 92, 59, 207, 81, 123, 0, 121, 241, 116, 42, 36, 251, 51,
    149, 79, 165, 12, 106, 187, 225, 203, 99, 102, 69, 97, 81, 27, 107, 81, 178, 63, 35, 185, 64,
    115,
];

/// Raven `FS_HandleForFile`.
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:258-268`
pub fn FS_HandleForFile(common: &mut Common) -> fileHandle_t {
    for i in 1..MAX_FILE_HANDLES as c_int {
        if unsafe { common.fsh[i as usize].handleFiles.file.o }.is_null() {
            return i;
        }
    }
    com_error(
        errorParm_t::ERR_DROP,
        "FS_HandleForFile: none free".to_string(),
    )
}

/// Raven `FS_FileForHandle`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:85-97`
pub fn FS_FileForHandle(common: &mut Common, f: fileHandle_t) -> *mut libc::FILE {
    if f < 0 || f > MAX_FILE_HANDLES as c_int {
        com_error(
            errorParm_t::ERR_DROP,
            "FS_FileForHandle: out of reange".to_string(),
        );
    }
    if common.fsh[f as usize].zipFile == qtrue {
        com_error(
            errorParm_t::ERR_DROP,
            "FS_FileForHandle: can't get FILE on zip file".to_string(),
        );
    }
    unsafe {
        if common.fsh[f as usize].handleFiles.file.o.is_null() {
            com_error(errorParm_t::ERR_DROP, "FS_FileForHandle: NULL".to_string());
        }
        common.fsh[f as usize].handleFiles.file.o as *mut libc::FILE
    }
}

/// Raven `FS_ForceFlush` — disable stdio buffering on the handle's `FILE` so
/// crash-time log data stays valid.
///
/// Source: `oracle/codemp/qcommon/files.cpp:407-412`
pub fn FS_ForceFlush(common: &mut Common, f: fileHandle_t) {
    let file = FS_FileForHandle(common, f);
    unsafe {
        libc::setvbuf(file, core::ptr::null_mut(), libc::_IONBF, 0);
    }
}

/// Raven `FS_filelength`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:117-129`
pub fn FS_filelength(common: &mut Common, f: fileHandle_t) -> c_int {
    unsafe {
        let h = FS_FileForHandle(common, f);
        let pos = libc::ftell(h);
        libc::fseek(h, 0, libc::SEEK_END);
        let end = libc::ftell(h);
        libc::fseek(h, pos, libc::SEEK_SET);
        end as c_int
    }
}

/// Raven `FS_CreatePath` (§C7 bool: `true` = refused, Raven's `qtrue`).
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:138-157`
pub fn FS_CreatePath(common: &mut Common, OSPath: &str) -> bool {
    // make absolutely sure that it can't back up the path
    if OSPath.contains("..") || OSPath.contains("::") {
        com_printf(
            common,
            &format!("WARNING: refusing to create relative path \"{OSPath}\"\n"),
        );
        return true;
    }

    // Raven NUL-punches each separator in place to mkdir the prefix; the
    // owned walk mkdirs each `[..i]` prefix (skipping a leading separator).
    for (i, b) in OSPath.bytes().enumerate().skip(1) {
        if b == PATH_SEP as u8 {
            // create the directory
            Sys_Mkdir(&OSPath[..i]);
        }
    }
    false
}

/// Raven `FS_CopyFile`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:166-205`
pub fn FS_CopyFile(common: &mut Common, fromOSPath: &str, toOSPath: &str) {
    unsafe {
        com_printf(common, &format!("copy {fromOSPath} to {toOSPath}\n"));

        if fromOSPath.contains("journal.dat") || fromOSPath.contains("journaldata.dat") {
            com_printf(common, "Ignoring journal files\n");
            return;
        }

        let mut f = sys_fopen(fromOSPath, c"rb");
        if f.is_null() {
            return;
        }
        libc::fseek(f, 0, libc::SEEK_END);
        let len = libc::ftell(f) as c_int;
        libc::fseek(f, 0, libc::SEEK_SET);

        // direct malloc (developer-only path) per Raven
        let buf = libc::malloc(len as usize) as *mut u8;
        if libc::fread(buf as *mut c_void, 1, len as usize, f) != len as usize {
            com_error(
                errorParm_t::ERR_FATAL,
                "Short read in FS_Copyfiles()\n".to_string(),
            );
        }
        libc::fclose(f);

        if FS_CreatePath(common, toOSPath) {
            return;
        }

        f = sys_fopen(toOSPath, c"wb");
        if f.is_null() {
            return;
        }
        if libc::fwrite(buf as *const c_void, 1, len as usize, f) != len as usize {
            com_error(
                errorParm_t::ERR_FATAL,
                "Short write in FS_Copyfiles()\n".to_string(),
            );
        }
        libc::fclose(f);
        libc::free(buf as *mut c_void);
    }
}

/// Raven `FS_FCloseFile`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:461-483`
pub fn FS_FCloseFile(common: &mut Common, f: fileHandle_t) {
    if common.fs_searchpaths.is_null() {
        com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
    }

    unsafe {
        if common.fsh[f as usize].streamed != qfalse {
            Sys_EndStreamedFile(f);
        }
        if common.fsh[f as usize].zipFile == qtrue {
            unzCloseCurrentFile(common.fsh[f as usize].handleFiles.file.z);
            if common.fsh[f as usize].handleFiles.unique != qfalse {
                unzClose(common.fsh[f as usize].handleFiles.file.z);
            }
            Com_Memset(
                &mut common.fsh[f as usize] as *mut fileHandleData_t as *mut (),
                0,
                core::mem::size_of::<fileHandleData_t>(),
            );
            return;
        }

        // we didn't find it as a pak, so close it as a unique file
        if !common.fsh[f as usize].handleFiles.file.o.is_null() {
            libc::fclose(common.fsh[f as usize].handleFiles.file.o as *mut libc::FILE);
        }
        Com_Memset(
            &mut common.fsh[f as usize] as *mut fileHandleData_t as *mut (),
            0,
            core::mem::size_of::<fileHandleData_t>(),
        );
    }
}

/// Raven `FS_FOpenFileWrite`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:491-524`
pub fn FS_FOpenFileWrite(common: &mut Common, filename: &str) -> fileHandle_t {
    if common.fs_searchpaths.is_null() {
        com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
    }

    let f = FS_HandleForFile(common);
    common.fsh[f as usize].zipFile = qfalse;

    let homepath = common.cvar(common.fs_homepath).string.clone();
    unsafe {
        let ospath = FS_BuildOSPath4(common, &homepath, &common.fs_gamedir.clone(), filename);

        if common.cvar(common.fs_debug).integer != 0 {
            com_printf(common, &format!("FS_FOpenFileWrite: {ospath}\n"));
        }

        if FS_CreatePath(common, &ospath) {
            return 0;
        }

        common.fsh[f as usize].handleFiles.file.o = sys_fopen(&ospath, c"wb") as *mut c_void;

        let name_len = common.fsh[f as usize].name.len();
        Q_strncpyz(&mut common.fsh[f as usize].name, filename, name_len);

        common.fsh[f as usize].handleSync = qfalse;
        if common.fsh[f as usize].handleFiles.file.o.is_null() {
            return 0;
        }
        f
    }
}

/// Raven `FS_SV_FOpenFileRead`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:313-387`
pub fn FS_SV_FOpenFileRead(common: &mut Common, filename: &str, fp: *mut fileHandle_t) -> c_int {
    if common.fs_searchpaths.is_null() {
        com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
    }

    let mut f = FS_HandleForFile(common);
    common.fsh[f as usize].zipFile = qfalse;

    unsafe {
        let name_len = common.fsh[f as usize].name.len();
        Q_strncpyz(&mut common.fsh[f as usize].name, filename, name_len);

        // don't let sound stutter (null build: no-op)

        // search homepath
        let homepath = common.cvar(common.fs_homepath).string.clone();
        let mut ospath = FS_BuildOSPath4(common, &homepath, filename, "");
        ospath.pop(); // strip the trailing slash

        if common.cvar(common.fs_debug).integer != 0 {
            com_printf(
                common,
                &format!("FS_SV_FOpenFileRead (fs_homepath): {ospath}\n"),
            );
        }

        common.fsh[f as usize].handleFiles.file.o = sys_fopen(&ospath, c"rb") as *mut c_void;
        common.fsh[f as usize].handleSync = qfalse;
        if common.fsh[f as usize].handleFiles.file.o.is_null() {
            // NOTE: on non-*nix fs_homepath == fs_basepath
            if !common
                .cvar(common.fs_homepath)
                .string
                .as_bytes()
                .eq_ignore_ascii_case(common.cvar(common.fs_basepath).string.as_bytes())
            {
                // search basepath
                let basepath = common.cvar(common.fs_basepath).string.clone();
                ospath = FS_BuildOSPath4(common, &basepath, filename, "");
                ospath.pop(); // strip the trailing slash

                if common.cvar(common.fs_debug).integer != 0 {
                    com_printf(
                        common,
                        &format!("FS_SV_FOpenFileRead (fs_basepath): {ospath}\n"),
                    );
                }

                common.fsh[f as usize].handleFiles.file.o =
                    sys_fopen(&ospath, c"rb") as *mut c_void;
                common.fsh[f as usize].handleSync = qfalse;

                if common.fsh[f as usize].handleFiles.file.o.is_null() {
                    f = 0;
                }
            }
        }

        if common.fsh[f as usize].handleFiles.file.o.is_null() {
            // search cd path
            let cdpath = common.cvar(common.fs_cdpath).string.clone();
            ospath = FS_BuildOSPath4(common, &cdpath, filename, "");
            ospath.pop(); // strip the trailing slash

            if common.cvar(common.fs_debug).integer != 0 {
                com_printf(
                    common,
                    &format!("FS_SV_FOpenFileRead (fs_cdpath) : {ospath}\n"),
                );
            }

            common.fsh[f as usize].handleFiles.file.o = sys_fopen(&ospath, c"rb") as *mut c_void;
            common.fsh[f as usize].handleSync = qfalse;

            if common.fsh[f as usize].handleFiles.file.o.is_null() {
                f = 0;
            }
        }

        *fp = f;
        if f != 0 {
            return FS_filelength(common, f);
        }
    }
    0
}

/// Raven `FS_FOpenFileRead`.
///
/// Raven's `#ifndef __linux__` fs_copyfiles copy-on-open blocks and the
/// `#ifndef DEDICATED/FINAL_BUILD` unprecached-file client checks are excluded
/// on this unix target (ruling 8; the client `cls` state is not reachable here).
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:672-997`
pub fn FS_FOpenFileRead(
    view: &mut EngineHostView,
    filename: &str,
    file: *mut fileHandle_t,
    uniqueFILE: bool,
) -> c_int {
    // Host-seam: this reader touches only `common` (no host services, no
    // view-forwarding callee), so reborrow it once and keep the body verbatim.
    let common = &mut *view.common;
    let mut hash: c_long = 0;
    let mut filename = filename;

    if common.fs_searchpaths.is_null() {
        com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
    }

    unsafe {
        if file.is_null() {
            com_error(
                errorParm_t::ERR_FATAL,
                "FS_FOpenFileRead: NULL 'file' parameter passed\n".to_string(),
            );
        }
        // (Raven's NULL-'filename' guard is unrepresentable with `&str`.)

        let demoExt = format!(".dm_{}", PROTOCOL_VERSION);

        // qpaths are not supposed to have a leading slash
        if let Some(stripped) = filename.strip_prefix(['/', '\\']) {
            filename = stripped;
        }

        // make absolutely sure that it can't back up the path.
        if filename.contains("..") || filename.contains("::") {
            *file = 0;
            return -1;
        }

        // the q3key file is only readable by the exe at initialization
        if common.com_fullyInitialized && filename.contains("q3key") {
            *file = 0;
            return -1;
        }

        *file = FS_HandleForFile(common);
        common.fsh[*file as usize].handleFiles.unique = if uniqueFILE { qtrue } else { qfalse };

        let mut bFasterToReOpenUsingNewLocalFile;
        loop {
            bFasterToReOpenUsingNewLocalFile = qfalse;

            let mut search = common.fs_searchpaths;
            while !search.is_null() {
                if !(*search).pack.is_null() {
                    hash = FS_HashFileName(filename, (*(*search).pack).hashSize);
                }
                // is the element a pak file?
                if !(*search).pack.is_null()
                    && (&(*(*search).pack).hashTable)[hash as usize].is_some()
                {
                    // disregard if it doesn't match one of the allowed pure pak files
                    if FS_PakIsPure(common, (*search).pack) == qfalse {
                        search = (*search).next;
                        continue;
                    }

                    let pak = (*search).pack;
                    let mut pakFile = (&(*pak).hashTable)[hash as usize];
                    while let Some(fi) = pakFile {
                        // case and separator insensitive comparisons
                        if FS_FilenameCompareBytes(
                            (&(*pak).buildBuffer)[fi as usize].name.as_bytes(),
                            filename.as_bytes(),
                        ) {
                            if (*pak).referenced & FS_GENERAL_REF == 0 {
                                if !tail_matches(filename, ".shader")
                                    && !tail_matches(filename, ".txt")
                                    && !tail_matches(filename, ".str")
                                    && !tail_matches(filename, ".cfg")
                                    && !tail_matches(filename, ".fcf")
                                    && !tail_matches(filename, ".config")
                                    && !filename.contains("levelshots")
                                    && !tail_matches(filename, ".bot")
                                    && !tail_matches(filename, ".arena")
                                    && !tail_matches(filename, ".menu")
                                {
                                    (*pak).referenced |= FS_GENERAL_REF;
                                }
                            }

                            if (*pak).referenced & FS_QAGAME_REF == 0
                                && (FS_ShiftedStrStr(filename, "]T`cZT`X!di`", 13)
                                    || FS_ShiftedStrStr(filename, "]T`cZT`Xk+)!W__", 13))
                            {
                                (*pak).referenced |= FS_QAGAME_REF;
                            }
                            if (*pak).referenced & FS_CGAME_REF == 0
                                && (FS_ShiftedStrStr(filename, "\\`Zf^'jof", 7)
                                    || FS_ShiftedStrStr(filename, "\\`Zf^q1/']ee", 7))
                            {
                                (*pak).referenced |= FS_CGAME_REF;
                            }
                            if (*pak).referenced & FS_UI_REF == 0
                                && (FS_ShiftedStrStr(filename, "pd)lqh", 5)
                                    || FS_ShiftedStrStr(filename, "pds31)_gg", 5))
                            {
                                (*pak).referenced |= FS_UI_REF;
                            }

                            if uniqueFILE {
                                // open a new file on the pakfile (unzip C seam)
                                let pak_filename_c =
                                    std::ffi::CString::new((*pak).pakFilename.as_str()).unwrap();
                                common.fsh[*file as usize].handleFiles.file.z =
                                    unzReOpen(pak_filename_c.as_ptr(), (*pak).handle);
                                if common.fsh[*file as usize].handleFiles.file.z.is_null() {
                                    com_error(
                                        errorParm_t::ERR_FATAL,
                                        format!("Couldn't reopen {}", (*pak).pakFilename),
                                    );
                                }
                            } else {
                                common.fsh[*file as usize].handleFiles.file.z = (*pak).handle;
                            }
                            let name_len = common.fsh[*file as usize].name.len();
                            Q_strncpyz(&mut common.fsh[*file as usize].name, filename, name_len);
                            common.fsh[*file as usize].zipFile = qtrue;
                            let zfi = common.fsh[*file as usize].handleFiles.file.z as *mut unz_s;
                            // in case the file was new
                            let temp = (*zfi).file;
                            // set the file position in the zip file
                            unzSetCurrentFileInfoPosition(
                                (*pak).handle,
                                (&(*pak).buildBuffer)[fi as usize].pos,
                            );
                            // copy the file info into the unzip structure
                            Com_Memcpy(
                                zfi as *mut (),
                                (*pak).handle as *const (),
                                core::mem::size_of::<unz_s>(),
                            );
                            // copy this back into the structure
                            (*zfi).file = temp;
                            // open the file in the zip
                            unzOpenCurrentFile(common.fsh[*file as usize].handleFiles.file.z);
                            common.fsh[*file as usize].zipFilePos =
                                (&(*pak).buildBuffer)[fi as usize].pos as i32;

                            if common.cvar(common.fs_debug).integer != 0 {
                                com_printf(
                                    common,
                                    &format!(
                                        "FS_FOpenFileRead: {} (found in '{}')\n",
                                        filename,
                                        (*pak).pakFilename
                                    ),
                                );
                            }
                            return (*zfi).cur_file_info.uncompressed_size as c_int;
                        }
                        pakFile = (&(*pak).buildBuffer)[fi as usize].next;
                    }
                } else if !(*search).dir.is_null() {
                    // check a file in the directory tree
                    if common.cvar(common.fs_restrict).integer != 0 || common.fs_numServerPaks != 0
                    {
                        if !tail_matches(filename, ".cfg")
                            && !tail_matches(filename, ".fcf")
                            && !tail_matches(filename, ".menu")
                            && !tail_matches(filename, ".game")
                            && !tail_matches(filename, &demoExt)
                            && !tail_matches(filename, ".dat")
                        {
                            search = (*search).next;
                            continue;
                        }
                    }

                    let dir = (*search).dir;

                    let dir_path = (*dir).path.clone();
                    let dir_gamedir = (*dir).gamedir.clone();
                    let netpath = FS_BuildOSPath4(common, &dir_path, &dir_gamedir, filename);
                    common.fsh[*file as usize].handleFiles.file.o =
                        sys_fopen(&netpath, c"rb") as *mut c_void;
                    if common.fsh[*file as usize].handleFiles.file.o.is_null() {
                        search = (*search).next;
                        continue;
                    }

                    if !tail_matches(filename, ".cfg")
                        && !tail_matches(filename, ".fcf")
                        && !tail_matches(filename, ".menu")
                        && !tail_matches(filename, ".game")
                        && !tail_matches(filename, &demoExt)
                        && !tail_matches(filename, ".dat")
                    {
                        // Raven `random()` is unbound in the `libc` crate here; `rand()` is
                        // the available libc PRNG and `fs_fakeChkSum` is a decoy value.
                        common.fs_fakeChkSum = libc::rand();
                    }

                    let name_len = common.fsh[*file as usize].name.len();
                    Q_strncpyz(&mut common.fsh[*file as usize].name, filename, name_len);
                    common.fsh[*file as usize].zipFile = qfalse;
                    if common.cvar(common.fs_debug).integer != 0 {
                        com_printf(
                            common,
                            &format!(
                                "FS_FOpenFileRead: {filename} (found in '{dir_path}/{dir_gamedir}')\n"
                            ),
                        );
                    }
                    // §unix: Raven's fs_copyfiles copy-on-open (Win32) block omitted.
                    return FS_filelength(common, *file);
                }
                search = (*search).next;
            }
            if bFasterToReOpenUsingNewLocalFile == qfalse {
                break;
            }
        }

        Com_DPrintf(common, &format!("Can't find {filename}\n"));
        *file = 0;
        -1
    }
}

/// Raven `FS_Read`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1026-1070`
pub fn FS_Read(common: &mut Common, buffer: *mut (), len: c_int, f: fileHandle_t) -> c_int {
    if common.fs_searchpaths.is_null() {
        com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
    }

    if f == 0 {
        return 0;
    }

    let mut buf = buffer as *mut u8;
    common.fs_readCount += len;

    unsafe {
        if common.fsh[f as usize].zipFile == qfalse {
            let mut remaining = len;
            let mut tries = 0;
            while remaining != 0 {
                let block = remaining;
                let read = libc::fread(
                    buf as *mut c_void,
                    1,
                    block as usize,
                    common.fsh[f as usize].handleFiles.file.o as *mut libc::FILE,
                ) as c_int;
                if read == 0 {
                    // 0 read on windows CD; retry once
                    if tries == 0 {
                        tries = 1;
                    } else {
                        return len - remaining;
                    }
                }
                if read == -1 {
                    com_error(errorParm_t::ERR_FATAL, "FS_Read: -1 bytes read".to_string());
                }
                remaining -= read;
                buf = buf.add(read as usize);
            }
            len
        } else {
            unzReadCurrentFile(
                common.fsh[f as usize].handleFiles.file.z,
                buffer,
                len as c_uint,
            )
        }
    }
}

/// Raven `FS_Write`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1079-1123`
pub fn FS_Write(common: &mut Common, buffer: *const (), len: c_int, h: fileHandle_t) -> c_int {
    if common.fs_searchpaths.is_null() {
        com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
    }

    if h == 0 {
        return 0;
    }

    let f = FS_FileForHandle(common, h);
    let mut buf = buffer as *const u8;

    let mut remaining = len;
    let mut tries = 0;
    unsafe {
        while remaining != 0 {
            let block = remaining;
            let written = libc::fwrite(buf as *const c_void, 1, block as usize, f) as c_int;
            if written == 0 {
                if tries == 0 {
                    tries = 1;
                } else {
                    com_printf(common, "FS_Write: 0 bytes written\n");
                    return 0;
                }
            }
            if written == -1 {
                com_printf(common, "FS_Write: -1 bytes written\n");
                return 0;
            }
            remaining -= written;
            buf = buf.add(written as usize);
        }
        if common.fsh[h as usize].handleSync != qfalse {
            libc::fflush(f);
        }
    }
    len
}

/// Raven `FS_ReadFile`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1259-1372`
pub fn FS_ReadFile(view: &mut EngineHostView, qpath: &str, buffer: *mut *mut ()) -> c_int {
    if view.common.fs_searchpaths.is_null() {
        com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
    }

    unsafe {
        if qpath.is_empty() {
            com_error(
                errorParm_t::ERR_FATAL,
                "FS_ReadFile with empty name\n".to_string(),
            );
        }

        let mut buf: *mut u8;

        // if this is a .cfg file and we are playing back a journal, read
        // it from the journal file
        let isConfig: qboolean;
        if qpath.contains(".cfg") {
            isConfig = qtrue;
            if view.common.com_journal.is_some()
                && view.common.cvar(view.common.com_journal).integer == 2
            {
                Com_DPrintf(
                    view.common,
                    &format!("Loading {qpath} from journal file.\n"),
                );
                let mut len: c_int = 0;
                let r = FS_Read(
                    view.common,
                    &mut len as *mut c_int as *mut (),
                    core::mem::size_of::<c_int>() as c_int,
                    view.common.com_journalDataFile,
                );
                if r != core::mem::size_of::<c_int>() as c_int {
                    if !buffer.is_null() {
                        *buffer = core::ptr::null_mut();
                    }
                    return -1;
                }
                // if the file didn't exist when the journal was created
                if len == 0 {
                    if buffer.is_null() {
                        return 1; // hack for old journal files
                    }
                    *buffer = core::ptr::null_mut();
                    return -1;
                }
                if buffer.is_null() {
                    return len;
                }

                buf = Hunk_AllocateTempMemory(view, len + 1) as *mut u8;
                *buffer = buf as *mut ();

                let r = FS_Read(
                    view.common,
                    buf as *mut (),
                    len,
                    view.common.com_journalDataFile,
                );
                if r != len {
                    com_error(
                        errorParm_t::ERR_FATAL,
                        "Read from journalDataFile failed".to_string(),
                    );
                }

                view.common.fs_loadCount += 1;
                view.common.fs_loadStack += 1;

                *buf.add(len as usize) = 0;

                return len;
            }
        } else {
            isConfig = qfalse;
        }

        // look for it in the filesystem or pack files
        let mut h: fileHandle_t = 0;
        let mut len = FS_FOpenFileRead(view, qpath, &mut h, false);
        if h == 0 {
            if !buffer.is_null() {
                *buffer = core::ptr::null_mut();
            }
            if isConfig != qfalse
                && view.common.com_journal.is_some()
                && view.common.cvar(view.common.com_journal).integer == 1
            {
                Com_DPrintf(
                    view.common,
                    &format!("Writing zero for {qpath} to journal file.\n"),
                );
                len = 0;
                FS_Write(
                    view.common,
                    &len as *const c_int as *const (),
                    core::mem::size_of::<c_int>() as c_int,
                    view.common.com_journalDataFile,
                );
                FS_Flush(view.common, view.common.com_journalDataFile);
            }
            return -1;
        }

        if buffer.is_null() {
            if isConfig != qfalse
                && view.common.com_journal.is_some()
                && view.common.cvar(view.common.com_journal).integer == 1
            {
                Com_DPrintf(
                    view.common,
                    &format!("Writing len for {qpath} to journal file.\n"),
                );
                FS_Write(
                    view.common,
                    &len as *const c_int as *const (),
                    core::mem::size_of::<c_int>() as c_int,
                    view.common.com_journalDataFile,
                );
                FS_Flush(view.common, view.common.com_journalDataFile);
            }
            FS_FCloseFile(view.common, h);
            return len;
        }

        view.common.fs_loadCount += 1;

        buf = Z_Malloc(view, len + 1, memtag_t::TAG_FILESYS, qfalse, 4) as *mut u8;
        *buf.add(len as usize) = 0; // not calling Z_Malloc with the trailing bZeroIt
        *buffer = buf as *mut ();

        FS_Read(view.common, buf as *mut (), len, h);

        // guarantee a trailing 0 for string operations
        *buf.add(len as usize) = 0;
        FS_FCloseFile(view.common, h);

        // if journalling a config file, write it to the journal file
        if isConfig != qfalse
            && view.common.com_journal.is_some()
            && view.common.cvar(view.common.com_journal).integer == 1
        {
            Com_DPrintf(view.common, &format!("Writing {qpath} to journal file.\n"));
            FS_Write(
                view.common,
                &len as *const c_int as *const (),
                core::mem::size_of::<c_int>() as c_int,
                view.common.com_journalDataFile,
            );
            FS_Write(
                view.common,
                buf as *const (),
                len,
                view.common.com_journalDataFile,
            );
            FS_Flush(view.common, view.common.com_journalDataFile);
        }
        len
    }
}

/// Raven `FS_FreeFile`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1379-1405`
pub fn FS_FreeFile(common: &mut Common, buffer: *mut ()) {
    if common.fs_searchpaths.is_null() {
        com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
    }
    if buffer.is_null() {
        com_error(errorParm_t::ERR_FATAL, "FS_FreeFile( NULL )".to_string());
    }

    Z_Free(common, buffer);
}

/// Raven `FS_LoadZipFile` — build a `pack_t` from a `.pk3`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1423-1522`
fn FS_LoadZipFile(view: &mut EngineHostView, zipfile: &str, basename: &str) -> *mut pack_t {
    unsafe {
        // unzip C seam.
        let zipfile_c = CString::new(zipfile).unwrap();
        let uf = unzOpen(zipfile_c.as_ptr());
        let mut gi: unz_global_info = core::mem::zeroed();
        let err = unzGetGlobalInfo(uf, &mut gi);

        if err != UNZ_OK {
            return core::ptr::null_mut();
        }

        view.common.fs_packFiles += gi.number_entry as c_int;

        // (Raven's extra first pass over the entries only sized the packed
        // name block; the owned entries need no pre-sizing.)
        let mut filename_inzip: [c_char; MAX_ZPATH] = [0; MAX_ZPATH];
        let mut file_info: unz_file_info = core::mem::zeroed();
        let mut fs_headerLongs: Vec<c_int> = Vec::with_capacity(gi.number_entry as usize);

        // hash table size from the number of files in the zip
        let mut i: c_int = 1;
        while i <= MAX_FILEHASH_SIZE as c_int {
            if i > gi.number_entry as c_int {
                break;
            }
            i <<= 1;
        }

        // strip .pk3 if needed
        let mut pak_basename = cap_ospath(basename).to_string();
        if pak_basename.len() > 4
            && pak_basename[pak_basename.len() - 4..].eq_ignore_ascii_case(".pk3")
        {
            pak_basename.truncate(pak_basename.len() - 4);
        }

        let mut pack = Box::new(pack_t {
            pakFilename: cap_ospath(zipfile).to_string(),
            pakBasename: pak_basename,
            pakGamename: String::new(),
            handle: uf,
            checksum: 0,
            pure_checksum: 0,
            numfiles: gi.number_entry as c_int,
            referenced: 0,
            hashSize: i,
            hashTable: vec![None; i as usize],
            buildBuffer: Vec::with_capacity(gi.number_entry as usize),
        });

        unzGoToFirstFile(uf);

        let mut idx: c_int = 0;
        while idx < gi.number_entry as c_int {
            let err = unzGetCurrentFileInfo(
                uf,
                &mut file_info,
                filename_inzip.as_mut_ptr(),
                filename_inzip.len() as core::ffi::c_ulong,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
                0,
            );
            if err != UNZ_OK {
                break;
            }
            if file_info.uncompressed_size > 0 {
                fs_headerLongs.push(LittleLong(file_info.crc as c_int));
            }
            Q_strlwr(&mut filename_inzip);
            let name = CStr::from_ptr(filename_inzip.as_ptr())
                .to_string_lossy()
                .into_owned();
            let hash = FS_HashFileName(&name, pack.hashSize);
            // store the file position in the zip
            let mut pos: core::ffi::c_ulong = 0;
            unzGetCurrentFileInfoPosition(uf, &mut pos);
            pack.buildBuffer.push(fileInPack_t {
                name,
                pos,
                // link into the hash chain (Raven's head insert)
                next: pack.hashTable[hash as usize],
            });
            pack.hashTable[hash as usize] = Some(idx as u32);
            unzGoToNextFile(uf);
            idx += 1;
        }

        pack.checksum = Com_BlockChecksum(
            view.common,
            fs_headerLongs.as_ptr() as *const (),
            4 * fs_headerLongs.len() as c_int,
        ) as c_int;
        pack.pure_checksum = Com_BlockChecksumKey(
            view.common,
            fs_headerLongs.as_ptr() as *mut (),
            4 * fs_headerLongs.len() as c_int,
            LittleLong(view.common.fs_checksumFeed),
        ) as c_int;
        pack.checksum = LittleLong(pack.checksum);
        pack.pure_checksum = LittleLong(pack.pure_checksum);

        // Raven's Z_Malloc'd pack + trailing hash block; the searchpath chain
        // still holds `*mut pack_t`, so the Box is leaked into it and freed by
        // `FS_Shutdown`'s `Box::from_raw`.
        Box::into_raw(pack)
    }
}

/// Raven `FS_AddFileToList` — the dedup push onto the owned list (Raven's
/// `MAX_FOUND_FILES-1` cap and `Q_stricmp` duplicate check kept).
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1562-1577`
fn FS_AddFileToList(name: &str, list: &mut Vec<String>) {
    if list.len() == MAX_FOUND_FILES - 1 {
        return;
    }
    for entry in list.iter() {
        if Q_stricmp(entry, name) == 0 {
            return; // already in list
        }
    }
    list.push(name.to_string());
}

/// Raven `FS_ListFilteredFiles` — Raven's Z_Malloc'd `char**`/`numfiles`
/// return becomes an owned `Vec<String>` (Raven's NULL-`path` arm is
/// unrepresentable here).
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1587-1717`
fn FS_ListFilteredFiles(
    view: &mut EngineHostView,
    path: &str,
    extension: &str,
    filter: Option<&str>,
) -> Vec<String> {
    if view.common.fs_searchpaths.is_null() {
        com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
    }

    unsafe {
        let mut list: Vec<String> = Vec::new();

        let mut pathLength = path.len() as c_int;
        if path.ends_with(['\\', '/']) {
            pathLength -= 1;
        }
        let extensionLength = extension.len() as c_int;
        let (_, pathDepth) = FS_ReturnPath(path);

        // search through the path, one element at a time, adding to list
        let mut search = view.common.fs_searchpaths;
        while !search.is_null() {
            if !(*search).pack.is_null() {
                if FS_PakIsPure(view.common, (*search).pack) == qfalse {
                    search = (*search).next;
                    continue;
                }
                let pak = &*(*search).pack;
                for i in 0..pak.numfiles as usize {
                    let name = &pak.buildBuffer[i].name;
                    if let Some(filter) = filter {
                        // case insensitive
                        if !Com_FilterPath(filter, name, false) {
                            continue;
                        }
                        FS_AddFileToList(name, &mut list);
                    } else {
                        let (zpathLen, depth) = FS_ReturnPath(name);

                        if (depth - pathDepth) > 2
                            || pathLength > zpathLen
                            || Q_stricmpn(name, path, pathLength as usize) != 0
                        {
                            continue;
                        }

                        // check for extension match
                        let length = name.len() as c_int;
                        if length < extensionLength {
                            continue;
                        }
                        if Q_stricmp(&name[(length - extensionLength) as usize..], extension) != 0 {
                            continue;
                        }

                        let mut temp = pathLength;
                        if pathLength != 0 {
                            temp += 1; // include the '/'
                        }
                        let stripped = name.get(temp as usize..).unwrap_or("").to_string();
                        FS_AddFileToList(&stripped, &mut list);
                    }
                }
            } else if !(*search).dir.is_null() {
                // don't scan directories for files if we are pure or restricted
                if (view.common.cvar(view.common.fs_restrict).integer != 0
                    || view.common.fs_numServerPaks != 0)
                    && (Q_stricmp(extension, "fcf") != 0
                        || view.common.cvar(view.common.fs_restrict).integer != 0)
                {
                    // rww - allow scanning for fcf files outside of pak even if pure
                    search = (*search).next;
                    continue;
                } else {
                    let dir_path = (*(*search).dir).path.clone();
                    let dir_gamedir = (*(*search).dir).gamedir.clone();
                    let netpath = FS_BuildOSPath4(view.common, &dir_path, &dir_gamedir, path);
                    let sysFiles = Sys_ListFiles(&netpath, Some(extension), filter, false);
                    for name in &sysFiles {
                        FS_AddFileToList(name, &mut list);
                    }
                }
            }
            search = (*search).next;
        }

        list
    }
}

/// Raven `FS_ListFiles`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1724-1726`
pub fn FS_ListFiles(view: &mut EngineHostView, path: &str, extension: &str) -> Vec<String> {
    FS_ListFilteredFiles(view, path, extension, None)
}

// Raven `FS_FreeFileList` (files_pc.cpp:1733-1750) is dropped: the owned
// `Vec<String>` lists free themselves.

/// Raven `FS_AddGameDirectory`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2212-2294`
pub fn FS_AddGameDirectory(view: &mut EngineHostView, path: &str, dir: &str) {
    unsafe {
        // this fixes the case where fs_basepath == fs_cdpath (full installs)
        let mut sp = view.common.fs_searchpaths;
        while !sp.is_null() {
            if !(*sp).dir.is_null()
                && (*(*sp).dir)
                    .path
                    .as_bytes()
                    .eq_ignore_ascii_case(path.as_bytes())
                && (*(*sp).dir)
                    .gamedir
                    .as_bytes()
                    .eq_ignore_ascii_case(dir.as_bytes())
            {
                return; // we've already got this one
            }
            sp = (*sp).next;
        }

        view.common.fs_gamedir = cap_ospath(dir).to_string();

        // add the directory to the search path
        let mut search = Z_Malloc(
            view,
            core::mem::size_of::<searchpath_t>() as c_int,
            memtag_t::TAG_FILESYS,
            qtrue,
            4,
        ) as *mut searchpath_t;
        // Raven's Z_Malloc'd directory_t; the Box is leaked into the raw
        // searchpath chain and freed by `FS_Shutdown`'s `Box::from_raw`.
        (*search).dir = Box::into_raw(Box::new(directory_t {
            path: cap_ospath(path).to_string(),
            gamedir: cap_ospath(dir).to_string(),
        }));
        (*search).next = view.common.fs_searchpaths;
        view.common.fs_searchpaths = search;

        let thedir = search;

        // find all pak files in this directory
        let mut pakfile = FS_BuildOSPath4(view.common, path, dir, "");
        pakfile.pop(); // strip trailing slash

        let mut pakfiles = Sys_ListFiles(&pakfile, Some(".pk3"), None, false);

        // sort so later alphabetic matches override earlier ones (pak1 > pak0)
        pakfiles.truncate(MAX_PAKFILES);

        // Raven `qsort(sorted, numfiles, 4, paksort)`; equal keys are impossible
        // (unique pak filenames), so a stable slice sort matches faithfully.
        pakfiles.sort_by(|a, b| FS_PathCmp(a, b).cmp(&0));

        for sorted_name in &pakfiles {
            let pakfile = FS_BuildOSPath4(view.common, path, dir, sorted_name);
            let pak = FS_LoadZipFile(view, &pakfile, sorted_name);
            if pak.is_null() {
                continue;
            }
            // store the game name for downloading
            (*pak).pakGamename = cap_ospath(dir).to_string();

            search = Z_Malloc(
                view,
                core::mem::size_of::<searchpath_t>() as c_int,
                memtag_t::TAG_FILESYS,
                qtrue,
                4,
            ) as *mut searchpath_t;
            (*search).pack = pak;

            if view.common.fs_dirbeforepak.is_some()
                && view.common.cvar(view.common.fs_dirbeforepak).integer != 0
                && !thedir.is_null()
            {
                let mut oldnext = (*thedir).next;
                (*thedir).next = search;

                while !oldnext.is_null() {
                    (*search).next = oldnext;
                    search = (*search).next;
                    oldnext = (*oldnext).next;
                }
            } else {
                (*search).next = view.common.fs_searchpaths;
                view.common.fs_searchpaths = search;
            }
        }
    }
}

/// Raven `FS_Startup`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2483-2576`
pub fn FS_Startup(view: &mut EngineHostView, gameName: &str) {
    com_printf(view.common, "----- FS_Startup -----\n");

    view.common.fs_debug = Some(Cvar_Get(view, "fs_debug", "0", 0));
    view.common.fs_copyfiles = Some(Cvar_Get(view, "fs_copyfiles", "0", CVAR_INIT));
    view.common.fs_cdpath = Some(Cvar_Get(view, "fs_cdpath", Sys_DefaultCDPath(), CVAR_INIT));
    view.common.fs_basepath = Some(Cvar_Get(
        view,
        "fs_basepath",
        Sys_DefaultInstallPath(),
        CVAR_INIT,
    ));
    view.common.fs_basegame = Some(Cvar_Get(view, "fs_basegame", "", CVAR_INIT));

    let default_home = Sys_DefaultHomePath();
    let home_path = if default_home.is_empty() {
        view.common.cvar(view.common.fs_basepath).string.clone()
    } else {
        default_home.to_string()
    };
    view.common.fs_homepath = Some(Cvar_Get(view, "fs_homepath", &home_path, CVAR_INIT));
    view.common.fs_gamedirvar = Some(Cvar_Get(view, "fs_game", "", CVAR_INIT | CVAR_SYSTEMINFO));
    view.common.fs_restrict = Some(Cvar_Get(view, "fs_restrict", "", CVAR_INIT));
    view.common.fs_dirbeforepak = Some(Cvar_Get(view, "fs_dirbeforepak", "0", CVAR_INIT));

    // BASEGAME is Raven's hardcoded "base".
    let basegame = "base";

    // FS_AddGameDirectory never touches the fs_* cvars, so the values are
    // snapshotted once.
    let cdpath = view.common.cvar(view.common.fs_cdpath).string.clone();
    let basepath = view.common.cvar(view.common.fs_basepath).string.clone();
    let homepath = view.common.cvar(view.common.fs_homepath).string.clone();
    let fs_basegame = view.common.cvar(view.common.fs_basegame).string.clone();
    let gamedirvar = view.common.cvar(view.common.fs_gamedirvar).string.clone();

    // add search path elements in reverse priority order
    if !cdpath.is_empty() {
        FS_AddGameDirectory(view, &cdpath, gameName);
    }
    if !basepath.is_empty() {
        FS_AddGameDirectory(view, &basepath, gameName);
    }
    if !basepath.is_empty()
        && !homepath
            .as_bytes()
            .eq_ignore_ascii_case(basepath.as_bytes())
    {
        FS_AddGameDirectory(view, &homepath, gameName);
    }

    // additional base game so mods can be based upon other mods
    if !fs_basegame.is_empty()
        && gameName
            .as_bytes()
            .eq_ignore_ascii_case(basegame.as_bytes())
        && !fs_basegame
            .as_bytes()
            .eq_ignore_ascii_case(gameName.as_bytes())
    {
        if !cdpath.is_empty() {
            FS_AddGameDirectory(view, &cdpath, &fs_basegame);
        }
        if !basepath.is_empty() {
            FS_AddGameDirectory(view, &basepath, &fs_basegame);
        }
        if !homepath.is_empty()
            && !homepath
                .as_bytes()
                .eq_ignore_ascii_case(basepath.as_bytes())
        {
            FS_AddGameDirectory(view, &homepath, &fs_basegame);
        }
    }

    // additional game folder for mods
    if !gamedirvar.is_empty()
        && gameName
            .as_bytes()
            .eq_ignore_ascii_case(basegame.as_bytes())
        && !gamedirvar
            .as_bytes()
            .eq_ignore_ascii_case(gameName.as_bytes())
    {
        if !cdpath.is_empty() {
            FS_AddGameDirectory(view, &cdpath, &gamedirvar);
        }
        if !basepath.is_empty() {
            FS_AddGameDirectory(view, &basepath, &gamedirvar);
        }
        if !homepath.is_empty()
            && !homepath
                .as_bytes()
                .eq_ignore_ascii_case(basepath.as_bytes())
        {
            FS_AddGameDirectory(view, &homepath, &gamedirvar);
        }
    }

    // add our commands
    Cmd_AddCommand(
        view,
        "path",
        Some(|view: &mut EngineHostView| FS_Path_f(view.common)),
    );
    Cmd_AddCommand(
        view,
        "dir",
        Some(|view: &mut EngineHostView| FS_Dir_f(view)),
    );
    Cmd_AddCommand(
        view,
        "fdir",
        Some(|view: &mut EngineHostView| FS_NewDir_f(view)),
    );
    Cmd_AddCommand(
        view,
        "touchFile",
        Some(|view: &mut EngineHostView| FS_TouchFile_f(view)),
    );

    // reorder the pure pk3 files according to server order
    FS_ReorderPurePaks(view.common);

    // print the current search paths
    FS_Path_f(view.common);

    view.common.cvar_mut(view.common.fs_gamedirvar).modified = false; // just loaded, not modified

    com_printf(view.common, "----------------------\n");

    com_printf(
        view.common,
        &format!("{} files in pk3 files\n", view.common.fs_packFiles),
    );
}

/// Raven `FS_SetRestrictions`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2587-2637`
pub fn FS_SetRestrictions(view: &mut EngineHostView) {
    unsafe {
        // if fs_restrict is set, don't even look for the id file
        if view.common.cvar(view.common.fs_restrict).integer == 0 {
            // look for the full game id
            let mut productId: *mut c_char = core::ptr::null_mut();
            FS_ReadFile(
                view,
                "productid.txt",
                &mut productId as *mut *mut c_char as *mut *mut (),
            );
            if !productId.is_null() {
                // check against the hardcoded string
                let mut seed: c_int = 102270;
                let mut i: usize = 0;
                while i < FS_SCRAMBLED_PRODUCT_ID.len() {
                    if (FS_SCRAMBLED_PRODUCT_ID[i] as c_int ^ (seed & 255))
                        != *productId.add(i) as c_int
                    {
                        break;
                    }
                    // C `69069*seed+1` wraps on overflow.
                    seed = seed.wrapping_mul(69069).wrapping_add(1);
                    i += 1;
                }

                FS_FreeFile(view.common, productId as *mut ());

                if i == FS_SCRAMBLED_PRODUCT_ID.len() {
                    return; // no restrictions
                }
                com_error(
                    errorParm_t::ERR_FATAL,
                    "Invalid product identification".to_string(),
                );
            }
        }
    }

    Cvar_Set(view, "fs_restrict", "1");

    com_printf(view.common, "\nRunning in restricted demo mode.\n\n");

    // restart the filesystem with just the demo directory
    FS_Shutdown(view.common, qfalse);
    FS_Startup(view, "demo");

    // make sure the pak file has the header checksum we expect
    unsafe {
        let mut path = view.common.fs_searchpaths;
        while !path.is_null() {
            if !(*path).pack.is_null() {
                // a tiny attempt to keep the checksum from being scannable
                if ((*(*path).pack).checksum ^ 0x0226_1994u32 as c_int)
                    != (DEMO_PAK_CHECKSUM as c_int ^ 0x0226_1994u32 as c_int)
                {
                    com_error(
                        errorParm_t::ERR_FATAL,
                        format!("Corrupted pak0.pk3: {}", (*(*path).pack).checksum as c_uint),
                    );
                }
            }
            path = (*path).next;
        }
    }
}

/// Raven `FS_Shutdown` — frees all resources and closes all files.
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:430-470`
pub fn FS_Shutdown(common: &mut Common, closemfp: qboolean) {
    let _ = closemfp; // FS_MISSING off in this build

    for i in 0..MAX_FILE_HANDLES as c_int {
        if common.fsh[i as usize].fileSize != 0 {
            FS_FCloseFile(common, i);
        }
    }

    // free everything
    unsafe {
        let mut p = common.fs_searchpaths;
        while !p.is_null() {
            let next = (*p).next;

            if !(*p).pack.is_null() {
                unzClose((*(*p).pack).handle);
                // the Boxed pack owns its hashTable/buildBuffer Vecs (Raven's
                // two Z_Malloc blocks)
                drop(Box::from_raw((*p).pack));
            }
            if !(*p).dir.is_null() {
                // the Boxed directory_t owns its path/gamedir Strings
                drop(Box::from_raw((*p).dir));
            }
            Z_Free(common, p as *mut ());
            p = next;
        }
    }

    // any FS_ calls will now be an error until reinitialized
    common.fs_searchpaths = core::ptr::null_mut();

    Cmd_RemoveCommand(common, "path");
    Cmd_RemoveCommand(common, "dir");
    Cmd_RemoveCommand(common, "fdir");
    Cmd_RemoveCommand(common, "touchFile");
}

/// Raven `FS_PureServerSetLoadedPaks`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2887-2936`
pub fn FS_PureServerSetLoadedPaks(view: &mut EngineHostView, pakSums: &str, pakNames: &str) {
    Cmd_TokenizeString(view.common, pakSums);

    let mut c = Cmd_Argc(view.common);
    if c > MAX_SEARCH_PATHS as c_int {
        c = MAX_SEARCH_PATHS as c_int;
    }

    view.common.fs_numServerPaks = c;

    for i in 0..c as usize {
        view.common.fs_serverPaks[i] = atoi(Cmd_Argv(view.common, i as c_int));
    }

    if view.common.fs_numServerPaks != 0 {
        Com_DPrintf(view.common, "Connected to a pure server.\n");
    } else if view.common.fs_reordered != qfalse {
        // force a restart to make sure the search order will be correct
        Com_DPrintf(view.common, "FS search reorder is required\n");
        FS_Restart(view, view.common.fs_checksumFeed);
        return;
    }

    view.common.fs_serverPakNames.clear();
    if !pakNames.is_empty() {
        Cmd_TokenizeString(view.common, pakNames);

        let mut d = Cmd_Argc(view.common);
        if d > MAX_SEARCH_PATHS as c_int {
            d = MAX_SEARCH_PATHS as c_int;
        }

        for i in 0..d as usize {
            let name = Cmd_Argv(view.common, i as c_int).to_owned();
            view.common.fs_serverPakNames.push(name);
        }
    }
}

/// Raven `FS_Restart`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2988-3040`
pub fn FS_Restart(view: &mut EngineHostView, checksumFeed: c_int) {
    // free anything we currently have loaded
    FS_Shutdown(view.common, qfalse);

    // set the checksum feed
    view.common.fs_checksumFeed = checksumFeed;

    // clear pak references
    FS_ClearPakReferences(view.common, 0);

    // try to start up normally
    FS_Startup(view, "base");

    // see if we are going to allow add-ons
    FS_SetRestrictions(view);

    // if we can't find default.cfg, the paths are busted
    if FS_ReadFile(view, "mpdefault.cfg", core::ptr::null_mut()) <= 0 {
        // might happen when connecting to a pure server not using BASEGAME/pak0.pk3
        if !view.common.lastValidBase.is_empty() {
            FS_PureServerSetLoadedPaks(view, "", "");
            let last_base = view.common.lastValidBase.clone();
            Cvar_Set(view, "fs_basepath", &last_base);
            let last_game = view.common.lastValidGame.clone();
            Cvar_Set(view, "fs_gamedirvar", &last_game);
            view.common.lastValidBase.clear();
            view.common.lastValidGame.clear();
            Cvar_Set(view, "fs_restrict", "0");
            FS_Restart(view, checksumFeed);
            com_error(errorParm_t::ERR_DROP, "Invalid game folder\n".to_string());
        }
        com_error(
            errorParm_t::ERR_FATAL,
            "Couldn't load mpdefault.cfg".to_string(),
        );
    }

    // new check before safeMode
    if Q_stricmp(
        &view.common.cvar(view.common.fs_gamedirvar).string,
        &view.common.lastValidGame,
    ) != 0
    {
        // skip the jampconfig.cfg if "safe" is on the command line
        if Com_SafeMode(view.common) == qfalse {
            // MP dedicated build (`#ifdef DEDICATED`) execs jampserver.cfg.
            Cbuf_AddText(view.common, "exec jampserver.cfg\n");
        }
    }

    view.common.lastValidBase =
        cap_ospath(&view.common.cvar(view.common.fs_basepath).string).to_string();
    view.common.lastValidGame =
        cap_ospath(&view.common.cvar(view.common.fs_gamedirvar).string).to_string();
}

/// Raven `FS_SortFileList` — a stable insertion sort by `FS_PathCmp` over a
/// `Z_Malloc`ed scratch pointer array; the owned list sorts in place (stable,
/// same ordering).
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2078-2099`
fn FS_SortFileList(filelist: &mut [String]) {
    filelist.sort_by(|a, b| FS_PathCmp(a, b).cmp(&0));
}

/// Raven `FS_Dir_f`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1989-2018`
pub fn FS_Dir_f(view: &mut EngineHostView) {
    if Cmd_Argc(view.common) < 2 || Cmd_Argc(view.common) > 3 {
        com_printf(view.common, "usage: dir <directory> [extension]\n");
        return;
    }

    let path = Cmd_Argv(view.common, 1).to_owned();
    let extension = if Cmd_Argc(view.common) == 2 {
        String::new()
    } else {
        Cmd_Argv(view.common, 2).to_owned()
    };

    com_printf(view.common, &format!("Directory of {path} {extension}\n"));
    com_printf(view.common, "---------------\n");

    let dirnames = FS_ListFiles(view, &path, &extension);

    for name in &dirnames {
        com_printf(view.common, &format!("{name}\n"));
    }
}

/// Raven `FS_NewDir_f`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2106-2132`
pub fn FS_NewDir_f(view: &mut EngineHostView) {
    if Cmd_Argc(view.common) < 2 {
        com_printf(view.common, "usage: fdir <filter>\n");
        com_printf(view.common, "example: fdir *q3dm*.bsp\n");
        return;
    }

    let filter = Cmd_Argv(view.common, 1).to_owned();

    com_printf(view.common, "---------------\n");

    let mut dirnames = FS_ListFilteredFiles(view, "", "", Some(&filter));

    FS_SortFileList(&mut dirnames);

    for name in &mut dirnames {
        FS_ConvertPath(name);
        com_printf(view.common, &format!("{name}\n"));
    }
    com_printf(view.common, &format!("{} files listed\n", dirnames.len()));
}

/// Raven `FS_Path_f`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2140-2167`
pub fn FS_Path_f(common: &mut Common) {
    com_printf(common, "Current search path:\n");
    unsafe {
        let mut s = common.fs_searchpaths;
        while !s.is_null() {
            if !(*s).pack.is_null() {
                com_printf(
                    common,
                    &format!(
                        "{} ({} files)\n",
                        (*(*s).pack).pakFilename,
                        (*(*s).pack).numfiles
                    ),
                );
                if common.fs_numServerPaks != 0 {
                    if FS_PakIsPure(common, (*s).pack) == qfalse {
                        com_printf(common, "    not on the pure list\n");
                    } else {
                        com_printf(common, "    on the pure list\n");
                    }
                }
            } else {
                com_printf(
                    common,
                    &format!("{}/{}\n", (*(*s).dir).path, (*(*s).dir).gamedir),
                );
            }
            s = (*s).next;
        }

        com_printf(common, "\n");
        for i in 1..MAX_FILE_HANDLES as c_int {
            if !common.fsh[i as usize].handleFiles.file.o.is_null() {
                com_printf(
                    common,
                    &format!(
                        "handle {}: {}\n",
                        i,
                        cstr(common.fsh[i as usize].name.as_ptr())
                    ),
                );
            }
        }
    }
}

/// Raven `FS_TouchFile_f`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2177-2189`
pub fn FS_TouchFile_f(view: &mut EngineHostView) {
    if Cmd_Argc(view.common) != 2 {
        com_printf(view.common, "Usage: touchFile <file>\n");
        return;
    }

    let arg = Cmd_Argv(view.common, 1).to_owned();
    let mut f: fileHandle_t = 0;
    FS_FOpenFileRead(view, &arg, &mut f, false);
    if f != 0 {
        FS_FCloseFile(view.common, f);
    }
}
