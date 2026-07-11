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

use core::ffi::{c_char, c_int, c_long, c_uint, c_void};

use mp_host_interface::engine_host::EngineHost;
use mp_qshared::common::mp::qcommon::tags::memtag_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::{qboolean, qfalse, qtrue};
use native_types::fileHandle_t;

use crate::collision_world::CollisionWorld;
use crate::common::Common;
use crate::files::files_consts::BASEGAME;
use crate::cm_load::RenderModels;
// Source: `mp_engine_client::client::client_connection_t::MAX_OSPATH` (value 1024)
const MAX_OSPATH: usize = 1024;

// Sweep: extern forward-declares eliminated. Real in-crate callees imported
// (`com_error`, `com_printf`, `Com_StartupVariable`); q_shared helpers
// (`Com_sprintf`, `Q_strncpyz`) and this file's own not-yet-ported `FS_*`
// (files_common.cpp subject) referenced at their canonical homes — the `FS_*`
// left bare at their home; reported.
use crate::common::{com_error, com_printf};
use crate::common_fns::{Com_FilterPath, Com_StartupVariable};
use mp_qshared::shared::cvar::{CVAR_INIT, CVAR_SYSTEMINFO};
use mp_qshared::shared::q_string::{Com_sprintf, Q_stricmp, Q_stricmpn, Q_strlwr, Q_strncpyz};
use mp_qshared::shared::swap::LittleLong;

use crate::cmd_common::{Cbuf_AddText, Cmd_Argc, Cmd_Argv, Cmd_TokenizeString};
use crate::cmd_pc::{Cmd_AddCommand, Cmd_RemoveCommand};
use crate::common_fns::{Com_DPrintf, Com_Memcpy, Com_Memset, Com_SafeMode};
use crate::cvar_fns::{Cvar_Get, Cvar_Set};
use crate::files::directory_t::directory_t;
use crate::files::file_handle_data_t::fileHandleData_t;
use crate::files::file_in_pack_s::fileInPack_t;
use crate::files::files_consts::{
    DEMO_PAK_CHECKSUM, MAX_FILEHASH_SIZE, MAX_FOUND_FILES, MAX_PAKFILES, MAX_SEARCH_PATHS, MAX_ZPATH,
};
use crate::files::pack_t::pack_t;
use crate::files::searchpath_s::searchpath_t;
use crate::files::unz_types::{unz_file_info, unz_global_info, unz_s};
use crate::files_pc::{
    paksort, FS_ClearPakReferences, FS_ConvertPath, FS_Flush, FS_HashFileName, FS_PakIsPure,
    FS_PathCmp, FS_ReorderPurePaks, FS_ReturnPath, FS_ShiftedStrStr,
};
use crate::md4_fns::{Com_BlockChecksum, Com_BlockChecksumKey};
use crate::qcommon::filesystem_limits::{
    FS_CGAME_REF, FS_GENERAL_REF, FS_QAGAME_REF, FS_UI_REF, MAX_FILE_HANDLES,
};
use crate::qcommon::protocol::PROTOCOL_VERSION;
use crate::z_memman_pc::{CopyString, Hunk_AllocateTempMemory, Z_Free, Z_Malloc};
// Genuinely-unported callees referenced at their canonical homes (honest
// E0425/E0432 escalations): the `unz*` zip seam (open user decision) at
// `crate::files::unz_file`; `Sys_*` platform I/O at `native_platform`.
use crate::files::unz_file::{
    unzClose, unzCloseCurrentFile, unzGetCurrentFileInfo, unzGetCurrentFileInfoPosition,
    unzGetGlobalInfo, unzGoToFirstFile, unzGoToNextFile, unzOpen, unzOpenCurrentFile, unzReOpen,
    unzReadCurrentFile, unzSetCurrentFileInfoPosition, UNZ_OK,
};
use native_platform::{
    Sys_DefaultCDPath, Sys_DefaultHomePath, Sys_DefaultInstallPath, Sys_EndStreamedFile,
    Sys_FreeFileList, Sys_ListFiles, Sys_Mkdir,
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

/// Raven `FS_FilenameCompare`.
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:345-372`
pub fn FS_FilenameCompare(s1: *const c_char, s2: *const c_char) -> qboolean {
    unsafe {
        let mut p1 = s1;
        let mut p2 = s2;
        loop {
            let mut c1 = *p1 as c_int;
            let mut c2 = *p2 as c_int;
            p1 = p1.add(1);
            p2 = p2.add(1);

            if c1 >= b'a' as c_int && c1 <= b'z' as c_int {
                c1 -= b'a' as c_int - b'A' as c_int;
            }
            if c2 >= b'a' as c_int && c2 <= b'z' as c_int {
                c2 -= b'a' as c_int - b'A' as c_int;
            }

            if c1 == b'\\' as c_int || c1 == b':' as c_int {
                c1 = b'/' as c_int;
            }
            if c2 == b'\\' as c_int || c2 == b':' as c_int {
                c2 = b'/' as c_int;
            }

            if c1 != c2 {
                return -1; // strings not equal
            }
            if c1 == 0 {
                break;
            }
        }
        0 // strings are equal
    }
}

/// Raven `FS_BuildOSPath` (single-`qpath` overload).
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:294-315`
pub fn FS_BuildOSPath(common: &mut Common, mut qpath: *const c_char) -> *mut c_char {
    let mut temp: [c_char; 1024] = [0; 1024];
    common.fs_build_os_path_toggle ^= 1; // flip-flop to allow two returns without clash

    unsafe {
        // Fix for filenames that are given to FS with a leading "/" (/botfiles/Foo)
        if *qpath == b'\\' as c_char || *qpath == b'/' as c_char {
            qpath = qpath.add(1);
        }

        // FIXME VVFIXME Holy crap this is wrong.
        //	Com_sprintf( temp, sizeof(temp), "/%s/%s", fs_gamedirvar->string, qpath );
        let qpath_str = core::ffi::CStr::from_ptr(qpath).to_string_lossy();
        Com_sprintf(
            temp.as_mut_ptr(),
            temp.len() as c_int,
            &format!("/{}/{}", "base", qpath_str),
        );

        FS_ReplaceSeparators(temp.as_mut_ptr());

        let toggle = common.fs_build_os_path_toggle as usize;
        let base_str = core::ffi::CStr::from_ptr((*common.fs_basepath).string).to_string_lossy();
        let temp_str = core::ffi::CStr::from_ptr(temp.as_ptr()).to_string_lossy();
        let ospath_ptr = common.fs_build_os_path_buf[toggle].as_mut_ptr();
        Com_sprintf(
            ospath_ptr,
            common.fs_build_os_path_buf[toggle].len() as c_int,
            &format!("{}{}", base_str, temp_str),
        );

        ospath_ptr
    }
}

/// Raven `FS_BuildOSPath` (`base`/`game`/`qpath` overload).
///
/// Raven overloads `FS_BuildOSPath` by arity; Rust has no fn overloading, so
/// this 4-arg overload is named `FS_BuildOSPath4`.
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:317-336`
pub fn FS_BuildOSPath4(
    common: &mut Common,
    base: *const c_char,
    mut game: *const c_char,
    qpath: *const c_char,
) -> *mut c_char {
    let mut temp: [c_char; 1024] = [0; 1024];
    // Raven gives each overload its own fn-scope statics; this overload's
    // `ospath[4]`/`toggle` are separate from the single-`qpath` overload above.
    common.fs_build_os_path4_toggle = (common.fs_build_os_path4_toggle + 1) & 3; // allows four returns without clash (increased from 2 during fs_copyfiles 2 enhancement)

    unsafe {
        if game.is_null() || *game == 0 {
            game = common.fs_gamedir.as_ptr();
        }

        let game_str = core::ffi::CStr::from_ptr(game).to_string_lossy();
        let qpath_str = core::ffi::CStr::from_ptr(qpath).to_string_lossy();
        Com_sprintf(
            temp.as_mut_ptr(),
            temp.len() as c_int,
            &format!("/{}/{}", game_str, qpath_str),
        );
        FS_ReplaceSeparators(temp.as_mut_ptr());

        let toggle = common.fs_build_os_path4_toggle as usize;
        let base_str = core::ffi::CStr::from_ptr(base).to_string_lossy();
        let temp_str = core::ffi::CStr::from_ptr(temp.as_ptr()).to_string_lossy();
        let ospath_ptr = common.fs_build_os_path4_buf[toggle].as_mut_ptr();
        Com_sprintf(
            ospath_ptr,
            common.fs_build_os_path4_buf[toggle].len() as c_int,
            &format!("{}{}", base_str, temp_str),
        );

        ospath_ptr
    }
}

/// Raven `FS_CheckInit`.
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:229-235`
pub fn FS_CheckInit(common: &mut Common) {
    if common.initialized == qfalse {
        unsafe {
            com_error(errorParm_t::ERR_FATAL, "Filesystem call made without initialization\n".to_string());
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
pub fn FS_WriteFile(common: &mut Common, qpath: *const c_char, buffer: *const (), size: c_int) {
    unsafe {
        if common.fs_searchpaths.is_null() {
            com_error(errorParm_t::ERR_FATAL, "Filesystem call made without initialization\n".to_string());
        }

        if qpath.is_null() || buffer.is_null() {
            com_error(errorParm_t::ERR_FATAL, "FS_WriteFile: NULL parameter".to_string());
        }

        let f = FS_FOpenFileWrite(common, qpath);
        if f == 0 {
            let qpath_str = core::ffi::CStr::from_ptr(qpath).to_string_lossy();
            com_printf(common, &format!("Failed to open {}\n", qpath_str));
            return;
        }

        FS_Write(common, buffer, size, f);

        FS_FCloseFile(common, f);
    }
}

/// Raven `FS_InitFilesystem`.
///
/// Source: `oracle/codemp/qcommon/files_common.cpp:480-511`
pub fn FS_InitFilesystem(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    unsafe {
        // allow command line parms to override our defaults
        // we have to specially handle this, because normal command
        // line variable sets don't happen until after the filesystem
        // has already been initialized
        Com_StartupVariable(common, cm, rm, host, c"fs_cdpath".as_ptr());
        Com_StartupVariable(common, cm, rm, host, c"fs_basepath".as_ptr());
        Com_StartupVariable(common, cm, rm, host, c"fs_homepath".as_ptr());
        Com_StartupVariable(common, cm, rm, host, c"fs_game".as_ptr());
        Com_StartupVariable(common, cm, rm, host, c"fs_copyfiles".as_ptr());
        Com_StartupVariable(common, cm, rm, host, c"fs_restrict".as_ptr());

        // try to start up normally
        let basegame = std::ffi::CString::new(BASEGAME).unwrap();
        FS_Startup(common, cm, rm, host, basegame.as_ptr());
        common.initialized = qtrue;

        // see if we are going to allow add-ons
        FS_SetRestrictions(common, cm, rm, host);

        // if we can't find default.cfg, assume that the paths are
        // busted and error out now, rather than getting an unreadable
        // graphics screen when the font fails to load
        let mut buffer: *mut () = core::ptr::null_mut();
        if FS_ReadFile(
            common,
            cm,
            rm,
            host,
            c"mpdefault.cfg".as_ptr(),
            &mut buffer as *mut *mut (),
        ) <= 0
        {
            // bk001208 - SafeMode see below, FIXME?
            com_error(errorParm_t::ERR_FATAL, "Couldn't load mpdefault.cfg".to_string());
        }

        Q_strncpyz(
            common.lastValidBase.as_mut_ptr(),
            (*common.fs_basepath).string,
            common.lastValidBase.len() as c_int,
        );
        Q_strncpyz(
            common.lastValidGame.as_mut_ptr(),
            (*common.fs_gamedirvar).string,
            common.lastValidGame.len() as c_int,
        );

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
    50, 103, 24, 235, 246, 191, 183, 149, 160, 170, 230, 52, 176, 231, 15, 194, 236, 247, 159,
    168, 132, 154, 24, 133, 67, 85, 36, 97, 99, 86, 117, 189, 212, 156, 236, 153, 68, 10, 196, 241,
    39, 219, 156, 88, 93, 198, 200, 232, 142, 67, 45, 209, 53, 186, 228, 241, 162, 127, 213, 83, 7,
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
    com_error(errorParm_t::ERR_DROP, "FS_HandleForFile: none free".to_string());
    0
}

/// Raven `FS_FileForHandle`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:85-97`
pub fn FS_FileForHandle(common: &mut Common, f: fileHandle_t) -> *mut libc::FILE {
    if f < 0 || f > MAX_FILE_HANDLES as c_int {
        com_error(errorParm_t::ERR_DROP, "FS_FileForHandle: out of reange".to_string());
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

/// Raven `FS_CreatePath`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:138-157`
pub fn FS_CreatePath(common: &mut Common, OSPath: *mut c_char) -> qboolean {
    unsafe {
        // make absolutely sure that it can't back up the path
        if !libc::strstr(OSPath, c"..".as_ptr()).is_null()
            || !libc::strstr(OSPath, c"::".as_ptr()).is_null()
        {
            com_printf(
                common,
                &format!("WARNING: refusing to create relative path \"{}\"\n", cstr(OSPath)),
            );
            return qtrue;
        }

        let mut ofs = OSPath.add(1);
        while *ofs != 0 {
            if *ofs == PATH_SEP {
                // create the directory
                *ofs = 0;
                Sys_Mkdir(OSPath);
                *ofs = PATH_SEP;
            }
            ofs = ofs.add(1);
        }
    }
    qfalse
}

/// Raven `FS_CopyFile`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:166-205`
pub fn FS_CopyFile(common: &mut Common, fromOSPath: *mut c_char, toOSPath: *mut c_char) {
    unsafe {
        com_printf(common, &format!("copy {} to {}\n", cstr(fromOSPath), cstr(toOSPath)));

        if !libc::strstr(fromOSPath, c"journal.dat".as_ptr()).is_null()
            || !libc::strstr(fromOSPath, c"journaldata.dat".as_ptr()).is_null()
        {
            com_printf(common, "Ignoring journal files\n");
            return;
        }

        let mut f = libc::fopen(fromOSPath, c"rb".as_ptr());
        if f.is_null() {
            return;
        }
        libc::fseek(f, 0, libc::SEEK_END);
        let len = libc::ftell(f) as c_int;
        libc::fseek(f, 0, libc::SEEK_SET);

        // direct malloc (developer-only path) per Raven
        let buf = libc::malloc(len as usize) as *mut u8;
        if libc::fread(buf as *mut c_void, 1, len as usize, f) != len as usize {
            com_error(errorParm_t::ERR_FATAL, "Short read in FS_Copyfiles()\n".to_string());
        }
        libc::fclose(f);

        if FS_CreatePath(common, toOSPath) != 0 {
            return;
        }

        f = libc::fopen(toOSPath, c"wb".as_ptr());
        if f.is_null() {
            return;
        }
        if libc::fwrite(buf as *const c_void, 1, len as usize, f) != len as usize {
            com_error(errorParm_t::ERR_FATAL, "Short write in FS_Copyfiles()\n".to_string());
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
pub fn FS_FOpenFileWrite(common: &mut Common, filename: *const c_char) -> fileHandle_t {
    if common.fs_searchpaths.is_null() {
        com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
    }

    let f = FS_HandleForFile(common);
    common.fsh[f as usize].zipFile = qfalse;

    unsafe {
        let ospath = FS_BuildOSPath4(
            common,
            (*common.fs_homepath).string,
            common.fs_gamedir.as_ptr(),
            filename,
        );

        if (*common.fs_debug).integer != 0 {
            com_printf(common, &format!("FS_FOpenFileWrite: {}\n", cstr(ospath)));
        }

        if FS_CreatePath(common, ospath) != 0 {
            return 0;
        }

        common.fsh[f as usize].handleFiles.file.o = libc::fopen(ospath, c"wb".as_ptr()) as *mut c_void;

        Q_strncpyz(
            common.fsh[f as usize].name.as_mut_ptr(),
            filename,
            common.fsh[f as usize].name.len() as c_int,
        );

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
pub fn FS_SV_FOpenFileRead(
    common: &mut Common,
    filename: *const c_char,
    fp: *mut fileHandle_t,
) -> c_int {
    if common.fs_searchpaths.is_null() {
        com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
    }

    let mut f = FS_HandleForFile(common);
    common.fsh[f as usize].zipFile = qfalse;

    unsafe {
        Q_strncpyz(
            common.fsh[f as usize].name.as_mut_ptr(),
            filename,
            common.fsh[f as usize].name.len() as c_int,
        );

        // don't let sound stutter (null build: no-op)

        // search homepath
        let mut ospath =
            FS_BuildOSPath4(common, (*common.fs_homepath).string, filename, c"".as_ptr());
        *ospath.add(libc::strlen(ospath) - 1) = 0;

        if (*common.fs_debug).integer != 0 {
            com_printf(
                common,
                &format!("FS_SV_FOpenFileRead (fs_homepath): {}\n", cstr(ospath)),
            );
        }

        common.fsh[f as usize].handleFiles.file.o = libc::fopen(ospath, c"rb".as_ptr()) as *mut c_void;
        common.fsh[f as usize].handleSync = qfalse;
        if common.fsh[f as usize].handleFiles.file.o.is_null() {
            // NOTE: on non-*nix fs_homepath == fs_basepath
            if Q_stricmp((*common.fs_homepath).string, (*common.fs_basepath).string) != 0 {
                // search basepath
                ospath =
                    FS_BuildOSPath4(common, (*common.fs_basepath).string, filename, c"".as_ptr());
                *ospath.add(libc::strlen(ospath) - 1) = 0;

                if (*common.fs_debug).integer != 0 {
                    com_printf(
                        common,
                        &format!("FS_SV_FOpenFileRead (fs_basepath): {}\n", cstr(ospath)),
                    );
                }

                common.fsh[f as usize].handleFiles.file.o =
                    libc::fopen(ospath, c"rb".as_ptr()) as *mut c_void;
                common.fsh[f as usize].handleSync = qfalse;

                if common.fsh[f as usize].handleFiles.file.o.is_null() {
                    f = 0;
                }
            }
        }

        if common.fsh[f as usize].handleFiles.file.o.is_null() {
            // search cd path
            ospath = FS_BuildOSPath4(common, (*common.fs_cdpath).string, filename, c"".as_ptr());
            *ospath.add(libc::strlen(ospath) - 1) = 0;

            if (*common.fs_debug).integer != 0 {
                com_printf(
                    common,
                    &format!("FS_SV_FOpenFileRead (fs_cdpath) : {}\n", cstr(ospath)),
                );
            }

            common.fsh[f as usize].handleFiles.file.o =
                libc::fopen(ospath, c"rb".as_ptr()) as *mut c_void;
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
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    filename: *const c_char,
    file: *mut fileHandle_t,
    uniqueFILE: qboolean,
) -> c_int {
    let mut hash: c_long = 0;
    let mut filename = filename;
    let mut demoExt: [c_char; 16] = [0; 16];

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
        if filename.is_null() {
            com_error(
                errorParm_t::ERR_FATAL,
                "FS_FOpenFileRead: NULL 'filename' parameter passed\n".to_string(),
            );
        }

        Com_sprintf(
            demoExt.as_mut_ptr(),
            demoExt.len() as c_int,
            &format!(".dm_{}", PROTOCOL_VERSION),
        );

        // qpaths are not supposed to have a leading slash
        if *filename == b'/' as c_char || *filename == b'\\' as c_char {
            filename = filename.add(1);
        }

        // make absolutely sure that it can't back up the path.
        if !libc::strstr(filename, c"..".as_ptr()).is_null()
            || !libc::strstr(filename, c"::".as_ptr()).is_null()
        {
            *file = 0;
            return -1;
        }

        // the q3key file is only readable by the exe at initialization
        if common.com_fullyInitialized && !libc::strstr(filename, c"q3key".as_ptr()).is_null() {
            *file = 0;
            return -1;
        }

        *file = FS_HandleForFile(common);
        common.fsh[*file as usize].handleFiles.unique = uniqueFILE;

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
                    && !(*(*(*search).pack).hashTable.add(hash as usize)).is_null()
                {
                    // disregard if it doesn't match one of the allowed pure pak files
                    if FS_PakIsPure(common, (*search).pack) == qfalse {
                        search = (*search).next;
                        continue;
                    }

                    let pak = (*search).pack;
                    let mut pakFile = *(*pak).hashTable.add(hash as usize);
                    loop {
                        // case and separator insensitive comparisons
                        if FS_FilenameCompare((*pakFile).name, filename) == 0 {
                            let l = libc::strlen(filename) as c_int;
                            if (*pak).referenced & FS_GENERAL_REF == 0 {
                                if Q_stricmp(filename.offset((l - 7) as isize), c".shader".as_ptr()) != 0
                                    && Q_stricmp(filename.offset((l - 4) as isize), c".txt".as_ptr()) != 0
                                    && Q_stricmp(filename.offset((l - 4) as isize), c".str".as_ptr()) != 0
                                    && Q_stricmp(filename.offset((l - 4) as isize), c".cfg".as_ptr()) != 0
                                    && Q_stricmp(filename.offset((l - 4) as isize), c".fcf".as_ptr()) != 0
                                    && Q_stricmp(filename.offset((l - 7) as isize), c".config".as_ptr()) != 0
                                    && libc::strstr(filename, c"levelshots".as_ptr()).is_null()
                                    && Q_stricmp(filename.offset((l - 4) as isize), c".bot".as_ptr()) != 0
                                    && Q_stricmp(filename.offset((l - 6) as isize), c".arena".as_ptr()) != 0
                                    && Q_stricmp(filename.offset((l - 5) as isize), c".menu".as_ptr()) != 0
                                {
                                    (*pak).referenced |= FS_GENERAL_REF;
                                }
                            }

                            if (*pak).referenced & FS_QAGAME_REF == 0
                                && (!FS_ShiftedStrStr(filename, c"]T`cZT`X!di`".as_ptr(), 13).is_null()
                                    || !FS_ShiftedStrStr(filename, c"]T`cZT`Xk+)!W__".as_ptr(), 13).is_null())
                            {
                                (*pak).referenced |= FS_QAGAME_REF;
                            }
                            if (*pak).referenced & FS_CGAME_REF == 0
                                && (!FS_ShiftedStrStr(filename, c"\\`Zf^'jof".as_ptr(), 7).is_null()
                                    || !FS_ShiftedStrStr(filename, c"\\`Zf^q1/']ee".as_ptr(), 7).is_null())
                            {
                                (*pak).referenced |= FS_CGAME_REF;
                            }
                            if (*pak).referenced & FS_UI_REF == 0
                                && (!FS_ShiftedStrStr(filename, c"pd)lqh".as_ptr(), 5).is_null()
                                    || !FS_ShiftedStrStr(filename, c"pds31)_gg".as_ptr(), 5).is_null())
                            {
                                (*pak).referenced |= FS_UI_REF;
                            }

                            if uniqueFILE != qfalse {
                                // open a new file on the pakfile
                                common.fsh[*file as usize].handleFiles.file.z =
                                    unzReOpen((*pak).pakFilename.as_ptr(), (*pak).handle);
                                if common.fsh[*file as usize].handleFiles.file.z.is_null() {
                                    com_error(
                                        errorParm_t::ERR_FATAL,
                                        format!("Couldn't reopen {}", cstr((*pak).pakFilename.as_ptr())),
                                    );
                                }
                            } else {
                                common.fsh[*file as usize].handleFiles.file.z = (*pak).handle;
                            }
                            Q_strncpyz(
                                common.fsh[*file as usize].name.as_mut_ptr(),
                                filename,
                                common.fsh[*file as usize].name.len() as c_int,
                            );
                            common.fsh[*file as usize].zipFile = qtrue;
                            let zfi = common.fsh[*file as usize].handleFiles.file.z as *mut unz_s;
                            // in case the file was new
                            let temp = (*zfi).file;
                            // set the file position in the zip file
                            unzSetCurrentFileInfoPosition((*pak).handle, (*pakFile).pos);
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
                            common.fsh[*file as usize].zipFilePos = (*pakFile).pos as i32;

                            if (*common.fs_debug).integer != 0 {
                                com_printf(
                                    common,
                                    &format!(
                                        "FS_FOpenFileRead: {} (found in '{}')\n",
                                        cstr(filename),
                                        cstr((*pak).pakFilename.as_ptr())
                                    ),
                                );
                            }
                            return (*zfi).cur_file_info.uncompressed_size as c_int;
                        }
                        pakFile = (*pakFile).next;
                        if pakFile.is_null() {
                            break;
                        }
                    }
                } else if !(*search).dir.is_null() {
                    // check a file in the directory tree
                    let l = libc::strlen(filename) as c_int;
                    if (*common.fs_restrict).integer != 0 || common.fs_numServerPaks != 0 {
                        if Q_stricmp(filename.offset((l - 4) as isize), c".cfg".as_ptr()) != 0
                            && Q_stricmp(filename.offset((l - 4) as isize), c".fcf".as_ptr()) != 0
                            && Q_stricmp(filename.offset((l - 5) as isize), c".menu".as_ptr()) != 0
                            && Q_stricmp(filename.offset((l - 5) as isize), c".game".as_ptr()) != 0
                            && Q_stricmp(
                                filename.offset((l - libc::strlen(demoExt.as_ptr()) as c_int) as isize),
                                demoExt.as_ptr(),
                            ) != 0
                            && Q_stricmp(filename.offset((l - 4) as isize), c".dat".as_ptr()) != 0
                        {
                            search = (*search).next;
                            continue;
                        }
                    }

                    let dir = (*search).dir;

                    let netpath = FS_BuildOSPath4(
                        common,
                        (*dir).path.as_ptr(),
                        (*dir).gamedir.as_ptr(),
                        filename,
                    );
                    common.fsh[*file as usize].handleFiles.file.o =
                        libc::fopen(netpath, c"rb".as_ptr()) as *mut c_void;
                    if common.fsh[*file as usize].handleFiles.file.o.is_null() {
                        search = (*search).next;
                        continue;
                    }

                    if Q_stricmp(filename.offset((l - 4) as isize), c".cfg".as_ptr()) != 0
                        && Q_stricmp(filename.offset((l - 4) as isize), c".fcf".as_ptr()) != 0
                        && Q_stricmp(filename.offset((l - 5) as isize), c".menu".as_ptr()) != 0
                        && Q_stricmp(filename.offset((l - 5) as isize), c".game".as_ptr()) != 0
                        && Q_stricmp(
                            filename.offset((l - libc::strlen(demoExt.as_ptr()) as c_int) as isize),
                            demoExt.as_ptr(),
                        ) != 0
                        && Q_stricmp(filename.offset((l - 4) as isize), c".dat".as_ptr()) != 0
                    {
                        // Raven `random()` is unbound in the `libc` crate here; `rand()` is
                        // the available libc PRNG and `fs_fakeChkSum` is a decoy value.
                        common.fs_fakeChkSum = libc::rand();
                    }

                    Q_strncpyz(
                        common.fsh[*file as usize].name.as_mut_ptr(),
                        filename,
                        common.fsh[*file as usize].name.len() as c_int,
                    );
                    common.fsh[*file as usize].zipFile = qfalse;
                    if (*common.fs_debug).integer != 0 {
                        com_printf(
                            common,
                            &format!(
                                "FS_FOpenFileRead: {} (found in '{}/{}')\n",
                                cstr(filename),
                                cstr((*dir).path.as_ptr()),
                                cstr((*dir).gamedir.as_ptr())
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

        Com_DPrintf(common, &format!("Can't find {}\n", cstr(filename)));
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
            unzReadCurrentFile(common.fsh[f as usize].handleFiles.file.z, buffer, len as c_uint)
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
pub fn FS_ReadFile(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    qpath: *const c_char,
    buffer: *mut *mut (),
) -> c_int {
    if common.fs_searchpaths.is_null() {
        com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
    }

    unsafe {
        if qpath.is_null() || *qpath == 0 {
            com_error(errorParm_t::ERR_FATAL, "FS_ReadFile with empty name\n".to_string());
        }

        let mut buf: *mut u8;

        // if this is a .cfg file and we are playing back a journal, read
        // it from the journal file
        let isConfig: qboolean;
        if !libc::strstr(qpath, c".cfg".as_ptr()).is_null() {
            isConfig = qtrue;
            if !common.com_journal.is_null() && (*common.com_journal).integer == 2 {
                Com_DPrintf(common, &format!("Loading {} from journal file.\n", cstr(qpath)));
                let mut len: c_int = 0;
                let r = FS_Read(
                    common,
                    &mut len as *mut c_int as *mut (),
                    core::mem::size_of::<c_int>() as c_int,
                    common.com_journalDataFile,
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

                buf = Hunk_AllocateTempMemory(common, cm, rm, host, len + 1) as *mut u8;
                *buffer = buf as *mut ();

                let r = FS_Read(common, buf as *mut (), len, common.com_journalDataFile);
                if r != len {
                    com_error(
                        errorParm_t::ERR_FATAL,
                        "Read from journalDataFile failed".to_string(),
                    );
                }

                common.fs_loadCount += 1;
                common.fs_loadStack += 1;

                *buf.add(len as usize) = 0;

                return len;
            }
        } else {
            isConfig = qfalse;
        }

        // look for it in the filesystem or pack files
        let mut h: fileHandle_t = 0;
        let mut len = FS_FOpenFileRead(common, cm, rm, host, qpath, &mut h, qfalse);
        if h == 0 {
            if !buffer.is_null() {
                *buffer = core::ptr::null_mut();
            }
            if isConfig != qfalse
                && !common.com_journal.is_null()
                && (*common.com_journal).integer == 1
            {
                Com_DPrintf(common, &format!("Writing zero for {} to journal file.\n", cstr(qpath)));
                len = 0;
                FS_Write(
                    common,
                    &len as *const c_int as *const (),
                    core::mem::size_of::<c_int>() as c_int,
                    common.com_journalDataFile,
                );
                FS_Flush(common, common.com_journalDataFile);
            }
            return -1;
        }

        if buffer.is_null() {
            if isConfig != qfalse
                && !common.com_journal.is_null()
                && (*common.com_journal).integer == 1
            {
                Com_DPrintf(common, &format!("Writing len for {} to journal file.\n", cstr(qpath)));
                FS_Write(
                    common,
                    &len as *const c_int as *const (),
                    core::mem::size_of::<c_int>() as c_int,
                    common.com_journalDataFile,
                );
                FS_Flush(common, common.com_journalDataFile);
            }
            FS_FCloseFile(common, h);
            return len;
        }

        common.fs_loadCount += 1;

        buf = Z_Malloc(common, cm, rm, host, len + 1, memtag_t::TAG_FILESYS, qfalse, 4) as *mut u8;
        *buf.add(len as usize) = 0; // not calling Z_Malloc with the trailing bZeroIt
        *buffer = buf as *mut ();

        FS_Read(common, buf as *mut (), len, h);

        // guarantee a trailing 0 for string operations
        *buf.add(len as usize) = 0;
        FS_FCloseFile(common, h);

        // if journalling a config file, write it to the journal file
        if isConfig != qfalse && !common.com_journal.is_null() && (*common.com_journal).integer == 1
        {
            Com_DPrintf(common, &format!("Writing {} to journal file.\n", cstr(qpath)));
            FS_Write(
                common,
                &len as *const c_int as *const (),
                core::mem::size_of::<c_int>() as c_int,
                common.com_journalDataFile,
            );
            FS_Write(common, buf as *const (), len, common.com_journalDataFile);
            FS_Flush(common, common.com_journalDataFile);
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
fn FS_LoadZipFile(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    zipfile: *mut c_char,
    basename: *const c_char,
) -> *mut pack_t {
    let mut fs_numHeaderLongs: c_int = 0;

    unsafe {
        let uf = unzOpen(zipfile);
        let mut gi: unz_global_info = core::mem::zeroed();
        let err = unzGetGlobalInfo(uf, &mut gi);

        if err != UNZ_OK {
            return core::ptr::null_mut();
        }

        common.fs_packFiles += gi.number_entry as c_int;

        let mut len: c_int = 0;
        let mut filename_inzip: [c_char; MAX_ZPATH] = [0; MAX_ZPATH];
        let mut file_info: unz_file_info = core::mem::zeroed();
        unzGoToFirstFile(uf);
        for _ in 0..gi.number_entry as c_int {
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
            len += libc::strlen(filename_inzip.as_ptr()) as c_int + 1;
            unzGoToNextFile(uf);
        }

        let buildBuffer = Z_Malloc(
            common,
            cm,
            rm,
            host,
            gi.number_entry as c_int * core::mem::size_of::<fileInPack_t>() as c_int + len,
            memtag_t::TAG_FILESYS,
            qtrue,
            4,
        ) as *mut fileInPack_t;
        let mut namePtr = (buildBuffer as *mut c_char)
            .add(gi.number_entry as usize * core::mem::size_of::<fileInPack_t>());
        let fs_headerLongs = Z_Malloc(
            common,
            cm,
            rm,
            host,
            gi.number_entry as c_int * core::mem::size_of::<c_int>() as c_int,
            memtag_t::TAG_FILESYS,
            qtrue,
            4,
        ) as *mut c_int;

        // hash table size from the number of files in the zip
        let mut i: c_int = 1;
        while i <= MAX_FILEHASH_SIZE as c_int {
            if i > gi.number_entry as c_int {
                break;
            }
            i <<= 1;
        }

        let pack = Z_Malloc(
            common,
            cm,
            rm,
            host,
            core::mem::size_of::<pack_t>() as c_int
                + i * core::mem::size_of::<*mut fileInPack_t>() as c_int,
            memtag_t::TAG_FILESYS,
            qtrue,
            4,
        ) as *mut pack_t;
        (*pack).hashSize = i;
        (*pack).hashTable =
            (pack as *mut c_char).add(core::mem::size_of::<pack_t>()) as *mut *mut fileInPack_t;
        for j in 0..(*pack).hashSize {
            *(*pack).hashTable.add(j as usize) = core::ptr::null_mut();
        }

        Q_strncpyz((*pack).pakFilename.as_mut_ptr(), zipfile, (*pack).pakFilename.len() as c_int);
        Q_strncpyz((*pack).pakBasename.as_mut_ptr(), basename, (*pack).pakBasename.len() as c_int);

        // strip .pk3 if needed
        let bl = libc::strlen((*pack).pakBasename.as_ptr()) as c_int;
        if bl > 4
            && Q_stricmp((*pack).pakBasename.as_ptr().offset((bl - 4) as isize), c".pk3".as_ptr())
                == 0
        {
            *(*pack).pakBasename.as_mut_ptr().offset((bl - 4) as isize) = 0;
        }

        (*pack).handle = uf;
        (*pack).numfiles = gi.number_entry as c_int;
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
                *fs_headerLongs.add(fs_numHeaderLongs as usize) = LittleLong(file_info.crc as c_int);
                fs_numHeaderLongs += 1;
            }
            Q_strlwr(filename_inzip.as_mut_ptr());
            let hash = FS_HashFileName(filename_inzip.as_ptr(), (*pack).hashSize);
            (*buildBuffer.add(idx as usize)).name = namePtr;
            libc::strcpy(namePtr, filename_inzip.as_ptr());
            namePtr = namePtr.add(libc::strlen(filename_inzip.as_ptr()) + 1);
            // store the file position in the zip
            unzGetCurrentFileInfoPosition(uf, &mut (*buildBuffer.add(idx as usize)).pos);
            (*buildBuffer.add(idx as usize)).next = *(*pack).hashTable.add(hash as usize);
            *(*pack).hashTable.add(hash as usize) = buildBuffer.add(idx as usize);
            unzGoToNextFile(uf);
            idx += 1;
        }

        (*pack).checksum =
            Com_BlockChecksum(common, fs_headerLongs as *const (), 4 * fs_numHeaderLongs) as c_int;
        (*pack).pure_checksum = Com_BlockChecksumKey(
            common,
            fs_headerLongs as *mut (),
            4 * fs_numHeaderLongs,
            LittleLong(common.fs_checksumFeed),
        ) as c_int;
        (*pack).checksum = LittleLong((*pack).checksum);
        (*pack).pure_checksum = LittleLong((*pack).pure_checksum);

        Z_Free(common, fs_headerLongs as *mut ());

        (*pack).buildBuffer = buildBuffer;
        pack
    }
}

/// Raven `FS_AddFileToList`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1562-1577`
fn FS_AddFileToList(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    name: *mut c_char,
    list: &mut [*mut c_char; MAX_FOUND_FILES],
    nfiles: c_int,
) -> c_int {
    if nfiles == MAX_FOUND_FILES as c_int - 1 {
        return nfiles;
    }
    unsafe {
        for i in 0..nfiles as usize {
            if Q_stricmp(name, list[i]) == 0 {
                return nfiles; // already in list
            }
        }
        list[nfiles as usize] = CopyString(common, cm, rm, host, name);
    }
    nfiles + 1
}

/// Raven `FS_ListFilteredFiles`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1587-1717`
fn FS_ListFilteredFiles(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    path: *const c_char,
    extension: *const c_char,
    filter: *mut c_char,
    numfiles: *mut c_int,
) -> *mut *mut c_char {
    if common.fs_searchpaths.is_null() {
        com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
    }

    unsafe {
        let mut extension = extension;
        if path.is_null() {
            *numfiles = 0;
            return core::ptr::null_mut();
        }
        if extension.is_null() {
            extension = c"".as_ptr();
        }

        let mut list: [*mut c_char; MAX_FOUND_FILES] = [core::ptr::null_mut(); MAX_FOUND_FILES];
        let mut nfiles: c_int = 0;
        let mut zpath: [c_char; MAX_ZPATH] = [0; MAX_ZPATH];

        let mut pathLength = libc::strlen(path) as c_int;
        if *path.offset((pathLength - 1) as isize) == b'\\' as c_char
            || *path.offset((pathLength - 1) as isize) == b'/' as c_char
        {
            pathLength -= 1;
        }
        let extensionLength = libc::strlen(extension) as c_int;
        let mut pathDepth: c_int = 0;
        FS_ReturnPath(path, zpath.as_mut_ptr(), &mut pathDepth);

        // search through the path, one element at a time, adding to list
        let mut search = common.fs_searchpaths;
        while !search.is_null() {
            if !(*search).pack.is_null() {
                if FS_PakIsPure(common, (*search).pack) == qfalse {
                    search = (*search).next;
                    continue;
                }
                let pak = (*search).pack;
                let buildBuffer = (*pak).buildBuffer;
                for i in 0..(*pak).numfiles {
                    let name = (*buildBuffer.add(i as usize)).name;
                    if !filter.is_null() {
                        // case insensitive
                        if Com_FilterPath(filter, name, qfalse) == 0 {
                            continue;
                        }
                        nfiles = FS_AddFileToList(common, cm, rm, host, name, &mut list, nfiles);
                    } else {
                        let mut depth: c_int = 0;
                        let zpathLen = FS_ReturnPath(name, zpath.as_mut_ptr(), &mut depth);

                        if (depth - pathDepth) > 2
                            || pathLength > zpathLen
                            || Q_stricmpn(name, path, pathLength) != 0
                        {
                            continue;
                        }

                        // check for extension match
                        let length = libc::strlen(name) as c_int;
                        if length < extensionLength {
                            continue;
                        }
                        if Q_stricmp(name.offset((length - extensionLength) as isize), extension) != 0
                        {
                            continue;
                        }

                        let mut temp = pathLength;
                        if pathLength != 0 {
                            temp += 1; // include the '/'
                        }
                        nfiles = FS_AddFileToList(
                            common,
                            cm,
                            rm,
                            host,
                            name.offset(temp as isize),
                            &mut list,
                            nfiles,
                        );
                    }
                }
            } else if !(*search).dir.is_null() {
                // don't scan directories for files if we are pure or restricted
                if ((*common.fs_restrict).integer != 0 || common.fs_numServerPaks != 0)
                    && (extension.is_null()
                        || Q_stricmp(extension, c"fcf".as_ptr()) != 0
                        || (*common.fs_restrict).integer != 0)
                {
                    // rww - allow scanning for fcf files outside of pak even if pure
                    search = (*search).next;
                    continue;
                } else {
                    let netpath = FS_BuildOSPath4(
                        common,
                        (*(*search).dir).path.as_ptr(),
                        (*(*search).dir).gamedir.as_ptr(),
                        path,
                    );
                    let mut numSysFiles: c_int = 0;
                    let sysFiles = Sys_ListFiles(netpath, extension, filter, &mut numSysFiles, qfalse);
                    for i in 0..numSysFiles {
                        let name = *sysFiles.add(i as usize);
                        nfiles = FS_AddFileToList(common, cm, rm, host, name, &mut list, nfiles);
                    }
                    Sys_FreeFileList(sysFiles);
                }
            }
            search = (*search).next;
        }

        // return a copy of the list
        *numfiles = nfiles;
        if nfiles == 0 {
            return core::ptr::null_mut();
        }

        let listCopy = Z_Malloc(
            common,
            cm,
            rm,
            host,
            (nfiles + 1) * core::mem::size_of::<*mut c_char>() as c_int,
            memtag_t::TAG_FILESYS,
            qfalse,
            4,
        ) as *mut *mut c_char;
        for i in 0..nfiles {
            *listCopy.add(i as usize) = list[i as usize];
        }
        *listCopy.add(nfiles as usize) = core::ptr::null_mut();

        listCopy
    }
}

/// Raven `FS_ListFiles`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1724-1726`
pub fn FS_ListFiles(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    path: *const c_char,
    extension: *const c_char,
    numfiles: *mut c_int,
) -> *mut *mut c_char {
    FS_ListFilteredFiles(common, cm, rm, host, path, extension, core::ptr::null_mut(), numfiles)
}

/// Raven `FS_FreeFileList`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1733-1750`
pub fn FS_FreeFileList(common: &mut Common, fileList: *mut *mut c_char) {
    if common.fs_searchpaths.is_null() {
        com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
    }

    if fileList.is_null() {
        return;
    }

    unsafe {
        let mut i = 0;
        while !(*fileList.add(i)).is_null() {
            Z_Free(common, *fileList.add(i) as *mut ());
            i += 1;
        }
        Z_Free(common, fileList as *mut ());
    }
}

/// Raven `FS_AddGameDirectory`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2212-2294`
pub fn FS_AddGameDirectory(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    path: *const c_char,
    dir: *const c_char,
) {
    unsafe {
        // this fixes the case where fs_basepath == fs_cdpath (full installs)
        let mut sp = common.fs_searchpaths;
        while !sp.is_null() {
            if !(*sp).dir.is_null()
                && Q_stricmp((*(*sp).dir).path.as_ptr(), path) == 0
                && Q_stricmp((*(*sp).dir).gamedir.as_ptr(), dir) == 0
            {
                return; // we've already got this one
            }
            sp = (*sp).next;
        }

        Q_strncpyz(common.fs_gamedir.as_mut_ptr(), dir, common.fs_gamedir.len() as c_int);

        // add the directory to the search path
        let mut search = Z_Malloc(
            common,
            cm,
            rm,
            host,
            core::mem::size_of::<searchpath_t>() as c_int,
            memtag_t::TAG_FILESYS,
            qtrue,
            4,
        ) as *mut searchpath_t;
        (*search).dir = Z_Malloc(
            common,
            cm,
            rm,
            host,
            core::mem::size_of::<directory_t>() as c_int,
            memtag_t::TAG_FILESYS,
            qtrue,
            4,
        ) as *mut directory_t;

        Q_strncpyz((*(*search).dir).path.as_mut_ptr(), path, (*(*search).dir).path.len() as c_int);
        Q_strncpyz(
            (*(*search).dir).gamedir.as_mut_ptr(),
            dir,
            (*(*search).dir).gamedir.len() as c_int,
        );
        (*search).next = common.fs_searchpaths;
        common.fs_searchpaths = search;

        let thedir = search;

        // find all pak files in this directory
        let mut pakfile = FS_BuildOSPath4(common, path, dir, c"".as_ptr());
        *pakfile.offset((libc::strlen(pakfile) - 1) as isize) = 0; // strip trailing slash

        let mut numfiles: c_int = 0;
        let pakfiles =
            Sys_ListFiles(pakfile, c".pk3".as_ptr(), core::ptr::null_mut(), &mut numfiles, qfalse);

        // sort so later alphabetic matches override earlier ones (pak1 > pak0)
        if numfiles > MAX_PAKFILES as c_int {
            numfiles = MAX_PAKFILES as c_int;
        }
        let mut sorted: [*mut c_char; MAX_PAKFILES] = [core::ptr::null_mut(); MAX_PAKFILES];
        for i in 0..numfiles as usize {
            sorted[i] = *pakfiles.add(i);
        }

        // Raven `qsort(sorted, numfiles, 4, paksort)`; equal keys are impossible
        // (unique pak filenames), so a stable slice sort matches faithfully.
        sorted[..numfiles as usize].sort_by(|a, b| {
            paksort(a as *const *mut c_char as *const (), b as *const *mut c_char as *const ()).cmp(&0)
        });

        for i in 0..numfiles as usize {
            pakfile = FS_BuildOSPath4(common, path, dir, sorted[i]);
            let pak = FS_LoadZipFile(common, cm, rm, host, pakfile, sorted[i]);
            if pak.is_null() {
                continue;
            }
            // store the game name for downloading
            libc::strcpy((*pak).pakGamename.as_mut_ptr(), dir);

            search = Z_Malloc(
                common,
                cm,
                rm,
                host,
                core::mem::size_of::<searchpath_t>() as c_int,
                memtag_t::TAG_FILESYS,
                qtrue,
                4,
            ) as *mut searchpath_t;
            (*search).pack = pak;

            if !common.fs_dirbeforepak.is_null()
                && (*common.fs_dirbeforepak).integer != 0
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
                (*search).next = common.fs_searchpaths;
                common.fs_searchpaths = search;
            }
        }

        // done
        Sys_FreeFileList(pakfiles);
    }
}

/// Raven `FS_Startup`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2483-2576`
pub fn FS_Startup(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    gameName: *const c_char,
) {
    com_printf(common, "----- FS_Startup -----\n");

    common.fs_debug = Cvar_Get(common, cm, rm, host, c"fs_debug".as_ptr(), c"0".as_ptr(), 0);
    common.fs_copyfiles =
        Cvar_Get(common, cm, rm, host, c"fs_copyfiles".as_ptr(), c"0".as_ptr(), CVAR_INIT);
    common.fs_cdpath = Cvar_Get(
        common,
        cm,
        rm,
        host,
        c"fs_cdpath".as_ptr(),
        Sys_DefaultCDPath(),
        CVAR_INIT,
    );
    common.fs_basepath = Cvar_Get(
        common,
        cm,
        rm,
        host,
        c"fs_basepath".as_ptr(),
        Sys_DefaultInstallPath(),
        CVAR_INIT,
    );
    common.fs_basegame =
        Cvar_Get(common, cm, rm, host, c"fs_basegame".as_ptr(), c"".as_ptr(), CVAR_INIT);

    unsafe {
        let mut homePath = Sys_DefaultHomePath();
        if homePath.is_null() || *homePath == 0 {
            homePath = (*common.fs_basepath).string;
        }
        common.fs_homepath =
            Cvar_Get(common, cm, rm, host, c"fs_homepath".as_ptr(), homePath, CVAR_INIT);
        common.fs_gamedirvar = Cvar_Get(
            common,
            cm,
            rm,
            host,
            c"fs_game".as_ptr(),
            c"".as_ptr(),
            CVAR_INIT | CVAR_SYSTEMINFO,
        );
        common.fs_restrict =
            Cvar_Get(common, cm, rm, host, c"fs_restrict".as_ptr(), c"".as_ptr(), CVAR_INIT);
        common.fs_dirbeforepak =
            Cvar_Get(common, cm, rm, host, c"fs_dirbeforepak".as_ptr(), c"0".as_ptr(), CVAR_INIT);

        // BASEGAME is Raven's hardcoded "base".
        let basegame = c"base".as_ptr();

        // add search path elements in reverse priority order
        if *(*common.fs_cdpath).string != 0 {
            FS_AddGameDirectory(common, cm, rm, host, (*common.fs_cdpath).string, gameName);
        }
        if *(*common.fs_basepath).string != 0 {
            FS_AddGameDirectory(common, cm, rm, host, (*common.fs_basepath).string, gameName);
        }
        if *(*common.fs_basepath).string != 0
            && Q_stricmp((*common.fs_homepath).string, (*common.fs_basepath).string) != 0
        {
            FS_AddGameDirectory(common, cm, rm, host, (*common.fs_homepath).string, gameName);
        }

        // additional base game so mods can be based upon other mods
        if *(*common.fs_basegame).string != 0
            && Q_stricmp(gameName, basegame) == 0
            && Q_stricmp((*common.fs_basegame).string, gameName) != 0
        {
            if *(*common.fs_cdpath).string != 0 {
                FS_AddGameDirectory(
                    common,
                    cm,
                    rm,
                    host,
                    (*common.fs_cdpath).string,
                    (*common.fs_basegame).string,
                );
            }
            if *(*common.fs_basepath).string != 0 {
                FS_AddGameDirectory(
                    common,
                    cm,
                    rm,
                    host,
                    (*common.fs_basepath).string,
                    (*common.fs_basegame).string,
                );
            }
            if *(*common.fs_homepath).string != 0
                && Q_stricmp((*common.fs_homepath).string, (*common.fs_basepath).string) != 0
            {
                FS_AddGameDirectory(
                    common,
                    cm,
                    rm,
                    host,
                    (*common.fs_homepath).string,
                    (*common.fs_basegame).string,
                );
            }
        }

        // additional game folder for mods
        if *(*common.fs_gamedirvar).string != 0
            && Q_stricmp(gameName, basegame) == 0
            && Q_stricmp((*common.fs_gamedirvar).string, gameName) != 0
        {
            if *(*common.fs_cdpath).string != 0 {
                FS_AddGameDirectory(
                    common,
                    cm,
                    rm,
                    host,
                    (*common.fs_cdpath).string,
                    (*common.fs_gamedirvar).string,
                );
            }
            if *(*common.fs_basepath).string != 0 {
                FS_AddGameDirectory(
                    common,
                    cm,
                    rm,
                    host,
                    (*common.fs_basepath).string,
                    (*common.fs_gamedirvar).string,
                );
            }
            if *(*common.fs_homepath).string != 0
                && Q_stricmp((*common.fs_homepath).string, (*common.fs_basepath).string) != 0
            {
                FS_AddGameDirectory(
                    common,
                    cm,
                    rm,
                    host,
                    (*common.fs_homepath).string,
                    (*common.fs_gamedirvar).string,
                );
            }
        }

        // add our commands
        Cmd_AddCommand(
            common,
            cm,
            rm,
            host,
            c"path".as_ptr(),
            Some(|common, _cm, _sv, _rm, _rmg, _g2, _host| FS_Path_f(common)),
        );
        Cmd_AddCommand(
            common,
            cm,
            rm,
            host,
            c"dir".as_ptr(),
            Some(|common, cm, _sv, rm, _rmg, _g2, host| FS_Dir_f(common, cm, rm, host)),
        );
        Cmd_AddCommand(
            common,
            cm,
            rm,
            host,
            c"fdir".as_ptr(),
            Some(|common, cm, _sv, rm, _rmg, _g2, host| FS_NewDir_f(common, cm, rm, host)),
        );
        Cmd_AddCommand(
            common,
            cm,
            rm,
            host,
            c"touchFile".as_ptr(),
            Some(|common, cm, _sv, rm, _rmg, _g2, host| FS_TouchFile_f(common, cm, rm, host)),
        );

        // reorder the pure pk3 files according to server order
        FS_ReorderPurePaks(common);

        // print the current search paths
        FS_Path_f(common);

        (*common.fs_gamedirvar).modified = qfalse; // just loaded, not modified

        com_printf(common, "----------------------\n");

        com_printf(common, &format!("{} files in pk3 files\n", common.fs_packFiles));
    }
}

/// Raven `FS_SetRestrictions`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2587-2637`
pub fn FS_SetRestrictions(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    unsafe {
        // if fs_restrict is set, don't even look for the id file
        if (*common.fs_restrict).integer == 0 {
            // look for the full game id
            let mut productId: *mut c_char = core::ptr::null_mut();
            FS_ReadFile(
                common,
                cm,
                rm,
                host,
                c"productid.txt".as_ptr(),
                &mut productId as *mut *mut c_char as *mut *mut (),
            );
            if !productId.is_null() {
                // check against the hardcoded string
                let mut seed: c_int = 102270;
                let mut i: usize = 0;
                while i < FS_SCRAMBLED_PRODUCT_ID.len() {
                    if (FS_SCRAMBLED_PRODUCT_ID[i] as c_int ^ (seed & 255)) != *productId.add(i) as c_int
                    {
                        break;
                    }
                    // C `69069*seed+1` wraps on overflow.
                    seed = seed.wrapping_mul(69069).wrapping_add(1);
                    i += 1;
                }

                FS_FreeFile(common, productId as *mut ());

                if i == FS_SCRAMBLED_PRODUCT_ID.len() {
                    return; // no restrictions
                }
                com_error(errorParm_t::ERR_FATAL, "Invalid product identification".to_string());
            }
        }
    }

    Cvar_Set(common, cm, rm, host, c"fs_restrict".as_ptr(), c"1".as_ptr());

    com_printf(common, "\nRunning in restricted demo mode.\n\n");

    // restart the filesystem with just the demo directory
    FS_Shutdown(common, qfalse);
    FS_Startup(common, cm, rm, host, c"demo".as_ptr());

    // make sure the pak file has the header checksum we expect
    unsafe {
        let mut path = common.fs_searchpaths;
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
                Z_Free(common, (*(*p).pack).buildBuffer as *mut ());
                Z_Free(common, (*p).pack as *mut ());
            }
            if !(*p).dir.is_null() {
                Z_Free(common, (*p).dir as *mut ());
            }
            Z_Free(common, p as *mut ());
            p = next;
        }
    }

    // any FS_ calls will now be an error until reinitialized
    common.fs_searchpaths = core::ptr::null_mut();

    Cmd_RemoveCommand(common, c"path".as_ptr());
    Cmd_RemoveCommand(common, c"dir".as_ptr());
    Cmd_RemoveCommand(common, c"fdir".as_ptr());
    Cmd_RemoveCommand(common, c"touchFile".as_ptr());
}

/// Raven `FS_PureServerSetLoadedPaks`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2887-2936`
pub fn FS_PureServerSetLoadedPaks(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    pakSums: *const c_char,
    pakNames: *const c_char,
) {
    Cmd_TokenizeString(common, pakSums);

    let mut c = Cmd_Argc(common);
    if c > MAX_SEARCH_PATHS as c_int {
        c = MAX_SEARCH_PATHS as c_int;
    }

    common.fs_numServerPaks = c;

    for i in 0..c as usize {
        let arg = Cmd_Argv(common, i as c_int);
        common.fs_serverPaks[i] = unsafe { libc::atoi(arg) };
    }

    if common.fs_numServerPaks != 0 {
        Com_DPrintf(common, "Connected to a pure server.\n");
    } else if common.fs_reordered != qfalse {
        // force a restart to make sure the search order will be correct
        Com_DPrintf(common, "FS search reorder is required\n");
        FS_Restart(common, cm, rm, host, common.fs_checksumFeed);
        return;
    }

    for i in 0..c as usize {
        if !common.fs_serverPakNames[i].is_null() {
            Z_Free(common, common.fs_serverPakNames[i] as *mut ());
        }
        common.fs_serverPakNames[i] = core::ptr::null_mut();
    }
    let names_present = !pakNames.is_null() && unsafe { *pakNames != 0 };
    if names_present {
        Cmd_TokenizeString(common, pakNames);

        let mut d = Cmd_Argc(common);
        if d > MAX_SEARCH_PATHS as c_int {
            d = MAX_SEARCH_PATHS as c_int;
        }

        for i in 0..d as usize {
            let arg = Cmd_Argv(common, i as c_int);
            common.fs_serverPakNames[i] = unsafe { CopyString(common, cm, rm, host, arg) };
        }
    }
}

/// Raven `FS_Restart`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2988-3040`
pub fn FS_Restart(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    checksumFeed: c_int,
) {
    // free anything we currently have loaded
    FS_Shutdown(common, qfalse);

    // set the checksum feed
    common.fs_checksumFeed = checksumFeed;

    // clear pak references
    FS_ClearPakReferences(common, 0);

    // try to start up normally
    FS_Startup(common, cm, rm, host, c"base".as_ptr());

    // see if we are going to allow add-ons
    FS_SetRestrictions(common, cm, rm, host);

    unsafe {
        // if we can't find default.cfg, the paths are busted
        if FS_ReadFile(common, cm, rm, host, c"mpdefault.cfg".as_ptr(), core::ptr::null_mut()) <= 0 {
            // might happen when connecting to a pure server not using BASEGAME/pak0.pk3
            if common.lastValidBase[0] != 0 {
                FS_PureServerSetLoadedPaks(common, cm, rm, host, c"".as_ptr(), c"".as_ptr());
                Cvar_Set(
                    common,
                    cm,
                    rm,
                    host,
                    c"fs_basepath".as_ptr(),
                    common.lastValidBase.as_ptr(),
                );
                Cvar_Set(
                    common,
                    cm,
                    rm,
                    host,
                    c"fs_gamedirvar".as_ptr(),
                    common.lastValidGame.as_ptr(),
                );
                common.lastValidBase[0] = 0;
                common.lastValidGame[0] = 0;
                Cvar_Set(common, cm, rm, host, c"fs_restrict".as_ptr(), c"0".as_ptr());
                FS_Restart(common, cm, rm, host, checksumFeed);
                com_error(errorParm_t::ERR_DROP, "Invalid game folder\n".to_string());
                return;
            }
            com_error(errorParm_t::ERR_FATAL, "Couldn't load mpdefault.cfg".to_string());
        }

        // new check before safeMode
        if Q_stricmp((*common.fs_gamedirvar).string, common.lastValidGame.as_ptr()) != 0 {
            // skip the jampconfig.cfg if "safe" is on the command line
            if Com_SafeMode(common) == qfalse {
                // MP dedicated build (`#ifdef DEDICATED`) execs jampserver.cfg.
                Cbuf_AddText(common, c"exec jampserver.cfg\n".as_ptr());
            }
        }

        Q_strncpyz(
            common.lastValidBase.as_mut_ptr(),
            (*common.fs_basepath).string,
            common.lastValidBase.len() as c_int,
        );
        Q_strncpyz(
            common.lastValidGame.as_mut_ptr(),
            (*common.fs_gamedirvar).string,
            common.lastValidGame.len() as c_int,
        );
    }
}

/// Raven `FS_SortFileList`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2078-2099`
fn FS_SortFileList(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    filelist: *mut *mut c_char,
    numfiles: c_int,
) {
    unsafe {
        let sortedlist = Z_Malloc(
            common,
            cm,
            rm,
            host,
            (numfiles + 1) * core::mem::size_of::<*mut c_char>() as c_int,
            memtag_t::TAG_FILESYS,
            qtrue,
            4,
        ) as *mut *mut c_char;
        *sortedlist = core::ptr::null_mut();
        let mut numsortedfiles: c_int = 0;
        for i in 0..numfiles {
            let mut j: c_int = 0;
            while j < numsortedfiles {
                if FS_PathCmp(*filelist.add(i as usize), *sortedlist.add(j as usize)) < 0 {
                    break;
                }
                j += 1;
            }
            let mut k = numsortedfiles;
            while k > j {
                *sortedlist.add(k as usize) = *sortedlist.add((k - 1) as usize);
                k -= 1;
            }
            *sortedlist.add(j as usize) = *filelist.add(i as usize);
            numsortedfiles += 1;
        }
        Com_Memcpy(
            filelist as *mut (),
            sortedlist as *const (),
            numfiles as usize * core::mem::size_of::<*mut c_char>(),
        );
        Z_Free(common, sortedlist as *mut ());
    }
}

/// Raven `FS_Dir_f`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1989-2018`
pub fn FS_Dir_f(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    if Cmd_Argc(common) < 2 || Cmd_Argc(common) > 3 {
        com_printf(common, "usage: dir <directory> [extension]\n");
        return;
    }

    let path;
    let extension;
    if Cmd_Argc(common) == 2 {
        path = Cmd_Argv(common, 1);
        extension = c"".as_ptr() as *mut c_char;
    } else {
        path = Cmd_Argv(common, 1);
        extension = Cmd_Argv(common, 2);
    }

    unsafe {
        com_printf(common, &format!("Directory of {} {}\n", cstr(path), cstr(extension)));
        com_printf(common, "---------------\n");

        let mut ndirs: c_int = 0;
        let dirnames = FS_ListFiles(common, cm, rm, host, path, extension, &mut ndirs);

        for i in 0..ndirs {
            com_printf(common, &format!("{}\n", cstr(*dirnames.add(i as usize))));
        }
        FS_FreeFileList(common, dirnames);
    }
}

/// Raven `FS_NewDir_f`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2106-2132`
pub fn FS_NewDir_f(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    if Cmd_Argc(common) < 2 {
        com_printf(common, "usage: fdir <filter>\n");
        com_printf(common, "example: fdir *q3dm*.bsp\n");
        return;
    }

    let filter = Cmd_Argv(common, 1);

    com_printf(common, "---------------\n");

    unsafe {
        let mut ndirs: c_int = 0;
        let dirnames = FS_ListFilteredFiles(
            common,
            cm,
            rm,
            host,
            c"".as_ptr(),
            c"".as_ptr(),
            filter,
            &mut ndirs,
        );

        FS_SortFileList(common, cm, rm, host, dirnames, ndirs);

        for i in 0..ndirs {
            FS_ConvertPath(*dirnames.add(i as usize));
            com_printf(common, &format!("{}\n", cstr(*dirnames.add(i as usize))));
        }
        com_printf(common, &format!("{} files listed\n", ndirs));
        FS_FreeFileList(common, dirnames);
    }
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
                        cstr((*(*s).pack).pakFilename.as_ptr()),
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
                    &format!(
                        "{}/{}\n",
                        cstr((*(*s).dir).path.as_ptr()),
                        cstr((*(*s).dir).gamedir.as_ptr())
                    ),
                );
            }
            s = (*s).next;
        }

        com_printf(common, "\n");
        for i in 1..MAX_FILE_HANDLES as c_int {
            if !common.fsh[i as usize].handleFiles.file.o.is_null() {
                com_printf(
                    common,
                    &format!("handle {}: {}\n", i, cstr(common.fsh[i as usize].name.as_ptr())),
                );
            }
        }
    }
}

/// Raven `FS_TouchFile_f`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2177-2189`
pub fn FS_TouchFile_f(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    if Cmd_Argc(common) != 2 {
        com_printf(common, "Usage: touchFile <file>\n");
        return;
    }

    unsafe {
        let arg = Cmd_Argv(common, 1);
        let mut f: fileHandle_t = 0;
        FS_FOpenFileRead(common, cm, rm, host, arg, &mut f, qfalse);
        if f != 0 {
            FS_FCloseFile(common, f);
        }
    }
}
