//! `files_pc.cpp` — filesystem PC-platform logic (pak purity, path
//! normalization, referenced/loaded pak-list info strings, seek/rename/read).

#![allow(non_snake_case, non_upper_case_globals, unused_variables)]

use core::ffi::{c_char, c_int, c_long, c_uint, c_ulong, c_void};

use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::fs_origin::fsOrigin_t;
use mp_qshared::shared::{fsMode_t, FS_APPEND, FS_APPEND_SYNC, FS_READ, FS_WRITE};
use mp_qshared::shared::{qboolean, qfalse, qtrue};
use native_types::fileHandle_t;

use crate::common::com_error;
use crate::common::engine_host_view::EngineHostView;
use crate::common::Common;
use crate::files::files_consts::{BASEGAME, MAX_SEARCH_PATHS};
use crate::files::pack_t::pack_t;
use crate::files::searchpath_s::searchpath_t;
use crate::qcommon::filesystem_limits::{
    FS_CGAME_REF, FS_GENERAL_REF, FS_QAGAME_REF, FS_UI_REF, NUM_ID_PAKS,
};
use crate::unzip::unztell;

// Raven `S_ClearSoundBuffer` (Raven: `return;` in the null/no-sound build) is
// canonically ported at `mp_engine_client::null::null_snddma::S_ClearSoundBuffer`,
// but `qcommon` cannot depend on `client` (client already depends on qcommon;
// that would be a cycle). Duplicated here as the same no-op per the
// deliberately-callable-no-op allowance.
// Source: `oracle/codemp/null/null_snddma.cpp`
mod null {
    pub fn S_ClearSoundBuffer() {}
}

// Sweep: extern forward-declares eliminated. This crate's own not-yet-ported
// filesystem callees (files.cpp/files_pc.cpp subject) referenced at their
// canonical `files_common` home; `Sys_*` platform I/O at `native_platform`;
// `unzOpenCurrentFile` at the unzip reader; `Z_Malloc`/`Z_Free` at
// `z_memman_pc`. All genuinely unported; reported.
use crate::cmd_common::{Cmd_Argc, Cmd_Argv, Cmd_TokenizeString};
use crate::files::unz_file::unzOpenCurrentFile;
use crate::files_common::{
    FS_AddGameDirectory, FS_BuildOSPath4, FS_CopyFile, FS_CreatePath, FS_FCloseFile,
    FS_FOpenFileRead, FS_FOpenFileWrite, FS_FileForHandle, FS_FilenameCompare, FS_HandleForFile,
    FS_ListFiles, FS_Read, FS_Restart, FS_SV_FOpenFileRead,
};
use crate::sys_engine::{Sys_StreamSeek, Sys_StreamedRead};
use native_platform::{sys_fopen, sys_remove, sys_rename, Sys_BeginStreamedFile, Sys_ListFiles};
use native_string::atoi::atoi;
use native_string::q_strncpyz::Q_strncpyz;

/// Raven `FS_PakIsPure`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:39-56`
pub fn FS_PakIsPure(common: &mut Common, pack: *mut pack_t) -> qboolean {
    if common.fs_numServerPaks != 0 {
        for i in 0..common.fs_numServerPaks {
            // FIXME: also use hashed file names
            if unsafe { (*pack).checksum } == common.fs_serverPaks[i as usize] {
                return mp_qshared::shared::qtrue; // on the aproved list
            }
        }
        return mp_qshared::shared::qfalse; // not on the pure server pak list
    }
    mp_qshared::shared::qtrue
}

/// Raven `FS_HashFileName` (`letter` stays Raven's signed `char` in the
/// accumulate, so high bytes contribute negatively).
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:64-82`
pub fn FS_HashFileName(fname: &str, hashSize: c_int) -> c_long {
    let mut hash: c_long = 0;
    for (i, b) in fname.bytes().enumerate() {
        let mut letter = b.to_ascii_lowercase() as i8;
        if letter == b'.' as i8 {
            break; // don't include extension
        }
        if letter == b'\\' as i8 {
            letter = b'/' as i8; // damn path names
        }
        if letter == b'/' as i8 {
            letter = b'/' as i8; // damn path names
        }
        hash += (letter as c_long) * (i as c_long + 119);
    }
    hash = hash ^ (hash >> 10) ^ (hash >> 20);
    hash &= (hashSize as c_long) - 1;
    hash
}

/// Raven `FS_Remove`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:213-215`
pub fn FS_Remove(osPath: &str) {
    sys_remove(osPath);
}

/// Raven `Sys_GetFileTime`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:568-598`
///
/// Raven's body is Win32 `CreateFile`/`GetFileTime`; this unix target has no
/// faithful substitute wired through this `c_int`-only signature yet.
pub fn Sys_GetFileTime(psFileName: c_int, ft: &mut c_int) -> bool {
    let _ = (psFileName, ft);
    false
}

/// Raven `Sys_FileOutOfDate`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:600-630`
pub fn Sys_FileOutOfDate(
    common: &mut Common,
    psFinalFileName: c_int,
    psDataFileName: c_int,
) -> bool {
    let mut ftFinalFile: c_int = 0;
    let mut ftDataFile: c_int = 0;
    if Sys_GetFileTime(psFinalFileName, &mut ftFinalFile)
        && Sys_GetFileTime(psDataFileName, &mut ftDataFile)
    {
        // timer res only accurate to within 2 seconds on FAT, so can't do exact compare...
        if (ftFinalFile - ftDataFile).abs() <= 20000000 {
            return false; // file not out of date, ie use it.
        }
        return true; // flag return code to copy over a replacement version of this file
    }

    // extra error check, report as suspicious if you find a file locally but not out on the net.,.
    if common.cvar(common.com_developer).integer != 0 {
        if !Sys_GetFileTime(psDataFileName, &mut ftDataFile) {
            crate::common::com_printf(
                common,
                &format!(
                    "Sys_FileOutOfDate: reading {} but it's not on the net!\n",
                    psFinalFileName
                ),
            );
        }
    }

    false
}

/// Raven `FS_FileCacheable`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:634-642`
pub fn FS_FileCacheable(common: &mut Common, filename: *const c_char) -> bool {
    if common.com_buildScript.is_some() && common.cvar(common.com_buildScript).integer != 0 {
        return true;
    }
    unsafe { !libc::strchr(filename, b'/' as c_int).is_null() }
}

/// Raven `FS_ShiftedStrStr` (§C7 bool: `true` = found) — `strstr` for the
/// obfuscated pak-reference needles, un-shifting the needle first.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:649-658`
pub fn FS_ShiftedStrStr(string: &str, substring: &str, shift: i8) -> bool {
    let needle: Vec<u8> = substring
        .bytes()
        .map(|b| b.wrapping_add(shift as u8))
        .collect();
    !needle.is_empty()
        && string
            .as_bytes()
            .windows(needle.len())
            .any(|w| w == needle.as_slice())
}

/// Raven `FS_ReturnPath` — the (last-separator offset, separator depth) of a
/// qpath (Raven's `zpath` truncated-copy out-param is unread by its one
/// caller and dropped).
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1534-1555`
pub fn FS_ReturnPath(zname: &str) -> (c_int, c_int) {
    let mut len: c_int = 0;
    let mut newdep: c_int = 0;

    for (at, c) in zname.bytes().enumerate() {
        if c == b'/' || c == b'\\' {
            len = at as c_int;
            newdep += 1;
        }
    }

    (len, newdep)
}

/// Raven `Sys_CountFileList`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1803-1816`
pub fn Sys_CountFileList(list: *mut *mut c_char) -> c_uint {
    let mut i: c_uint = 0;
    if !list.is_null() {
        unsafe {
            let mut p = list;
            while !(*p).is_null() {
                p = p.add(1);
                i += 1;
            }
        }
    }
    i
}

/// Raven `FS_ConvertPath`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2025-2032`
pub fn FS_ConvertPath(s: &mut String) {
    *s = s.replace(['\\', ':'], "/");
}

/// Raven `FS_PathCmp` — case-folded with `\`/`:` normalized to `/`, compared
/// as Raven's signed-`char` ints (high bytes order negative).
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2041-2071`
pub fn FS_PathCmp(s1: &str, s2: &str) -> c_int {
    let fold = |c: u8| -> c_int {
        let mut c = c as i8 as c_int;
        if (b'a' as c_int..=b'z' as c_int).contains(&c) {
            c -= b'a' as c_int - b'A' as c_int;
        }
        if c == b'\\' as c_int || c == b':' as c_int {
            c = b'/' as c_int;
        }
        c
    };
    let mut it1 = s1.bytes();
    let mut it2 = s2.bytes();
    loop {
        let c1 = it1.next().map_or(0, fold);
        let c2 = it2.next().map_or(0, fold);

        if c1 < c2 {
            return -1; // strings not equal
        }
        if c1 > c2 {
            return 1;
        }
        if c1 == 0 {
            break;
        }
    }

    0 // strings are equal
}

/// Raven `FS_ReorderPurePaks`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2445-2476`
pub fn FS_ReorderPurePaks(common: &mut Common) {
    // only relevant when connected to pure server
    if common.fs_numServerPaks == 0 {
        return;
    }

    common.fs_reordered = mp_qshared::shared::qfalse;

    unsafe {
        let mut p_insert_index: *mut *mut searchpath_t = &mut common.fs_searchpaths;
        for i in 0..common.fs_numServerPaks {
            let mut p_previous: *mut *mut searchpath_t = p_insert_index;
            let mut s: *mut searchpath_t = *p_insert_index;
            while !s.is_null() {
                // the part of the list before p_insert_index has been sorted already
                if !(*s).pack.is_null() && common.fs_serverPaks[i as usize] == (*(*s).pack).checksum
                {
                    common.fs_reordered = mp_qshared::shared::qtrue;
                    // move this element to the insert list
                    *p_previous = (*s).next;
                    (*s).next = *p_insert_index;
                    *p_insert_index = s;
                    // increment insert list
                    p_insert_index = &mut (*s).next;
                    break; // iterate to next server pack
                }
                p_previous = &mut (*s).next;
                s = *p_previous;
            }
        }
    }
}

/// Raven `FS_GamePureChecksum`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2646-2662`
pub fn FS_GamePureChecksum(common: &mut Common) -> *const c_char {
    // §19: static char info[MAX_STRING_TOKENS] is a rotating scratch/return
    // buffer per the fork-3 three-kind rule -> owned return value on Common.
    common.fs_game_pure_checksum_info[0] = 0;

    unsafe {
        let mut search = common.fs_searchpaths;
        while !search.is_null() {
            // is the element a pak file?
            if !(*search).pack.is_null() {
                if (*(*search).pack).referenced & FS_QAGAME_REF != 0 {
                    let s = format!("{}", (*(*search).pack).checksum);
                    let bytes = s.as_bytes();
                    for (idx, b) in bytes.iter().enumerate() {
                        common.fs_game_pure_checksum_info[idx] = *b as c_char;
                    }
                    common.fs_game_pure_checksum_info[bytes.len()] = 0;
                }
            }
            search = (*search).next;
        }
    }

    common.fs_game_pure_checksum_info.as_ptr()
}

/// Raven `FS_LoadedPakChecksums`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2672-2688`
pub fn FS_LoadedPakChecksums(common: &mut Common) -> *const c_char {
    // §19: static char info[BIG_INFO_STRING] rotating scratch -> owned return
    // buffer on Common (fork-3).
    let mut info = String::new();
    unsafe {
        let mut search = common.fs_searchpaths;
        while !search.is_null() {
            // is the element a pak file?
            if !(*search).pack.is_null() {
                info.push_str(&format!("{} ", (*(*search).pack).checksum));
            }
            search = (*search).next;
        }
    }
    write_info_scratch(&mut common.fs_loaded_pak_checksums_info, &info)
}

/// Raven `FS_LoadedPakNames`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2698-2717`
pub fn FS_LoadedPakNames(common: &mut Common) -> *const c_char {
    let mut info = String::new();
    unsafe {
        let mut search = common.fs_searchpaths;
        while !search.is_null() {
            if !(*search).pack.is_null() {
                if !info.is_empty() {
                    info.push(' ');
                }
                info.push_str(&(*(*search).pack).pakBasename);
            }
            search = (*search).next;
        }
    }
    write_info_scratch(&mut common.fs_loaded_pak_names_info, &info)
}

/// Raven `FS_LoadedPakPureChecksums`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2728-2744`
pub fn FS_LoadedPakPureChecksums(common: &mut Common) -> *const c_char {
    let mut info = String::new();
    unsafe {
        let mut search = common.fs_searchpaths;
        while !search.is_null() {
            if !(*search).pack.is_null() {
                info.push_str(&format!("{} ", (*(*search).pack).pure_checksum));
            }
            search = (*search).next;
        }
    }
    write_info_scratch(&mut common.fs_loaded_pak_pure_checksums_info, &info)
}

/// Raven `FS_ReferencedPakChecksums`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2754-2771`
pub fn FS_ReferencedPakChecksums(common: &mut Common) -> *const c_char {
    let mut info = String::new();
    unsafe {
        let mut search = common.fs_searchpaths;
        while !search.is_null() {
            if !(*search).pack.is_null() {
                let pak = (*search).pack;
                if (*pak).referenced != 0 || !(*pak).pakGamename.eq_ignore_ascii_case(BASEGAME) {
                    info.push_str(&format!("{} ", (*pak).checksum));
                }
            }
            search = (*search).next;
        }
    }
    write_info_scratch(&mut common.fs_referenced_pak_checksums_info, &info)
}

/// Raven `FS_ReferencedPakPureChecksums`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2783-2822`
pub fn FS_ReferencedPakPureChecksums(common: &mut Common) -> *const c_char {
    let mut info = String::new();
    let mut checksum = common.fs_checksumFeed;
    let mut numPaks: c_int = 0;

    let mut nFlags = FS_CGAME_REF;
    while nFlags != 0 {
        if nFlags & FS_GENERAL_REF != 0 {
            // add a delimter between must haves and general refs
            info.push_str("@ ");
        }
        unsafe {
            let mut search = common.fs_searchpaths;
            while !search.is_null() {
                if !(*search).pack.is_null() && ((*(*search).pack).referenced & nFlags) != 0 {
                    info.push_str(&format!("{} ", (*(*search).pack).pure_checksum));
                    if nFlags & (FS_CGAME_REF | FS_UI_REF) != 0 {
                        break;
                    }
                    checksum ^= (*(*search).pack).pure_checksum;
                    numPaks += 1;
                }
                search = (*search).next;
            }
        }
        if common.fs_fakeChkSum != 0 {
            // only added if a non-pure file is referenced
            info.push_str(&format!("{} ", common.fs_fakeChkSum));
        }
        nFlags >>= 1;
    }
    // last checksum is the encoded number of referenced pk3s
    checksum ^= numPaks;
    info.push_str(&format!("{} ", checksum));

    write_info_scratch(&mut common.fs_referenced_pak_pure_checksums_info, &info)
}

/// Raven `FS_ReferencedPakNames`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2832-2855`
pub fn FS_ReferencedPakNames(common: &mut Common) -> *const c_char {
    let mut info = String::new();
    unsafe {
        // we want to return ALL pk3's from the fs_game path
        // and referenced one's from base
        let mut search = common.fs_searchpaths;
        while !search.is_null() {
            if !(*search).pack.is_null() {
                if !info.is_empty() {
                    info.push(' ');
                }
                let pak = (*search).pack;
                if (*pak).referenced != 0 || !(*pak).pakGamename.eq_ignore_ascii_case(BASEGAME) {
                    info.push_str(&(*pak).pakGamename);
                    info.push('/');
                    info.push_str(&(*pak).pakBasename);
                }
            }
            search = (*search).next;
        }
    }
    write_info_scratch(&mut common.fs_referenced_pak_names_info, &info)
}

/// Raven `FS_ClearPakReferences`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2862-2874`
pub fn FS_ClearPakReferences(common: &mut Common, mut flags: c_int) {
    if flags == 0 {
        flags = -1;
    }
    unsafe {
        let mut search = common.fs_searchpaths;
        while !search.is_null() {
            // is the element a pak file and has it been referenced?
            if !(*search).pack.is_null() {
                (*(*search).pack).referenced &= !flags;
            }
            search = (*search).next;
        }
    }
}

/// Raven `FS_Flush`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:3128-3130`
pub fn FS_Flush(common: &mut Common, f: fileHandle_t) {
    unsafe {
        libc::fflush(common.fsh[f as usize].handleFiles.file.o as *mut libc::FILE);
    }
}

// Raven `paksort` (files_pc.cpp:2194-2201), the C `qsort` comparator shim, is
// dropped: the owned pak lists sort directly through `FS_PathCmp`.

/// Raven `FS_idPak` (§C7 bool: `true` = one of the id paks).
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2301-2313`
pub fn FS_idPak(pak: &str, base: &str) -> bool {
    for i in 0..NUM_ID_PAKS {
        if FS_FilenameCompare(pak, &format!("{base}/assets{i}")) {
            return true;
        }
    }
    false
}

/// Raven `FS_FTell`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:3118-3126`
pub fn FS_FTell(common: &mut Common, f: fileHandle_t) -> c_int {
    unsafe {
        if common.fsh[f as usize].zipFile == mp_qshared::shared::qtrue {
            crate::unzip::unztell(common.fsh[f as usize].handleFiles.file.z) as c_int
        } else {
            libc::ftell(common.fsh[f as usize].handleFiles.file.o as *mut libc::FILE) as c_int
        }
    }
}

/// Raven `FS_FileExists` (§C7 bool: `true` = present).
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:227-240`
pub fn FS_FileExists(common: &mut Common, file: &str) -> bool {
    let homepath = common.cvar(common.fs_homepath).string.clone();
    let testpath = FS_BuildOSPath4(common, &homepath, &common.fs_gamedir.clone(), file);
    let f = sys_fopen(&testpath, c"rb");
    if !f.is_null() {
        unsafe { libc::fclose(f) };
        return true;
    }
    false
}

/// Raven `FS_SV_FileExists` (§C7 bool: `true` = present).
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:249-263`
pub fn FS_SV_FileExists(common: &mut Common, file: &str) -> bool {
    let homepath = common.cvar(common.fs_homepath).string.clone();
    let mut testpath = FS_BuildOSPath4(common, &homepath, file, "");
    testpath.pop(); // strip the trailing slash

    let f = sys_fopen(&testpath, c"rb");
    if !f.is_null() {
        unsafe { libc::fclose(f) };
        return true;
    }
    false
}

/// Raven `FS_ComparePaks`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2339-2409`
pub fn FS_ComparePaks(
    common: &mut Common,
    neededpaks: *mut c_char,
    len: c_int,
    dlstring: qboolean,
) -> qboolean {
    if common.fs_numServerReferencedPaks == 0 {
        return mp_qshared::shared::qfalse; // Server didn't send any pack information along
    }

    unsafe {
        *neededpaks = 0;

        for i in 0..common.fs_numServerReferencedPaks as usize {
            let mut havepak = false;

            // never autodownload any of the id paks
            let name = common
                .fs_serverReferencedPakNames
                .get(i)
                .cloned()
                .unwrap_or_default();
            if FS_idPak(&name, "base") || FS_idPak(&name, "missionpack") {
                continue;
            }

            let mut sp = common.fs_searchpaths;
            while !sp.is_null() {
                if !(*sp).pack.is_null()
                    && (*(*sp).pack).checksum == common.fs_serverReferencedPaks[i]
                {
                    havepak = true; // This is it!
                    break;
                }
                sp = (*sp).next;
            }

            if !havepak && !name.is_empty() {
                // Don't got it
                if dlstring != mp_qshared::shared::qfalse {
                    // Remote name
                    append_cstr(neededpaks, len, "@");
                    append_cstr(neededpaks, len, &name);
                    append_cstr(neededpaks, len, ".pk3");

                    // Local name
                    append_cstr(neededpaks, len, "@");
                    // Do we have one with the same name?
                    if FS_SV_FileExists(common, &format!("{}.pk3", name)) {
                        // We already have one called this, we need to download it to another name
                        // Make something up with the checksum in it
                        let st = format!("{}.{:08x}.pk3", name, common.fs_serverReferencedPaks[i]);
                        append_cstr(neededpaks, len, &st);
                    } else {
                        append_cstr(neededpaks, len, &name);
                        append_cstr(neededpaks, len, ".pk3");
                    }
                } else {
                    append_cstr(neededpaks, len, &name);
                    append_cstr(neededpaks, len, ".pk3");
                    // Do we have one with the same name?
                    if FS_SV_FileExists(common, &format!("{}.pk3", name)) {
                        append_cstr(neededpaks, len, " (local file exists with wrong checksum)");
                    }
                    append_cstr(neededpaks, len, "\n");
                }
            }
        }
        if *neededpaks != 0 {
            return mp_qshared::shared::qtrue;
        }
    }

    mp_qshared::shared::qfalse // We have them all
}

/// Raven `FS_SV_FOpenFileWrite`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:272-304`
pub fn FS_SV_FOpenFileWrite(common: &mut Common, filename: &str) -> fileHandle_t {
    if common.fs_searchpaths.is_null() {
        crate::common::com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
    }

    let homepath = common.cvar(common.fs_homepath).string.clone();
    let mut ospath = FS_BuildOSPath4(common, &homepath, filename, "");
    ospath.pop(); // strip the trailing slash

    let f = FS_HandleForFile(common);
    common.fsh[f as usize].zipFile = mp_qshared::shared::qfalse;

    if common.cvar(common.fs_debug).integer != 0 {
        crate::common::com_printf(common, &format!("FS_SV_FOpenFileWrite: {ospath}\n"));
    }

    if FS_CreatePath(common, &ospath) {
        return 0;
    }

    unsafe {
        // Com_DPrintf( "writing to: %s\n", ospath );
        common.fsh[f as usize].handleFiles.file.o = sys_fopen(&ospath, c"wb") as *mut c_void;

        let name_len = common.fsh[f as usize].name.len();
        Q_strncpyz(&mut common.fsh[f as usize].name, filename, name_len);

        common.fsh[f as usize].handleSync = mp_qshared::shared::qfalse;
        if common.fsh[f as usize].handleFiles.file.o.is_null() {
            return 0;
        }
    }
    f
}

/// Raven `FS_SV_Rename`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:395-419`
pub fn FS_SV_Rename(common: &mut Common, from: &str, to: &str) {
    if common.fs_searchpaths.is_null() {
        crate::common::com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
    }

    // don't let sound stutter
    null::S_ClearSoundBuffer();

    let homepath = common.cvar(common.fs_homepath).string.clone();
    let mut from_ospath = FS_BuildOSPath4(common, &homepath, from, "");
    let mut to_ospath = FS_BuildOSPath4(common, &homepath, to, "");
    from_ospath.pop(); // strip the trailing slash
    to_ospath.pop();

    if common.cvar(common.fs_debug).integer != 0 {
        crate::common::com_printf(
            common,
            &format!("FS_SV_Rename: {from_ospath} --> {to_ospath}\n"),
        );
    }

    if sys_rename(&from_ospath, &to_ospath) != 0 {
        // Failed, try copying it and deleting the original
        FS_CopyFile(common, &from_ospath, &to_ospath);
        FS_Remove(&from_ospath);
    }
}

/// Raven `FS_Rename`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:427-449`
pub fn FS_Rename(common: &mut Common, from: &str, to: &str) {
    if common.fs_searchpaths.is_null() {
        crate::common::com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
    }

    // don't let sound stutter
    null::S_ClearSoundBuffer();

    let homepath = common.cvar(common.fs_homepath).string.clone();
    let gamedir = common.fs_gamedir.clone();
    let from_ospath = FS_BuildOSPath4(common, &homepath, &gamedir, from);
    let to_ospath = FS_BuildOSPath4(common, &homepath, &gamedir, to);

    if common.cvar(common.fs_debug).integer != 0 {
        crate::common::com_printf(
            common,
            &format!("FS_Rename: {from_ospath} --> {to_ospath}\n"),
        );
    }

    if sys_rename(&from_ospath, &to_ospath) != 0 {
        // Failed, try copying it and deleting the original
        FS_CopyFile(common, &from_ospath, &to_ospath);
        FS_Remove(&from_ospath);
    }
}

/// Raven `FS_FOpenFileAppend`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:532-564`
pub fn FS_FOpenFileAppend(common: &mut Common, filename: &str) -> fileHandle_t {
    if common.fs_searchpaths.is_null() {
        crate::common::com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
    }

    let f = FS_HandleForFile(common);
    common.fsh[f as usize].zipFile = mp_qshared::shared::qfalse;

    let name_len = common.fsh[f as usize].name.len();
    Q_strncpyz(&mut common.fsh[f as usize].name, filename, name_len);

    // don't let sound stutter
    null::S_ClearSoundBuffer();

    let homepath = common.cvar(common.fs_homepath).string.clone();
    let ospath = FS_BuildOSPath4(common, &homepath, &common.fs_gamedir.clone(), filename);

    if common.cvar(common.fs_debug).integer != 0 {
        crate::common::com_printf(common, &format!("FS_FOpenFileAppend: {ospath}\n"));
    }

    if FS_CreatePath(common, &ospath) {
        return 0;
    }

    unsafe {
        common.fsh[f as usize].handleFiles.file.o = sys_fopen(&ospath, c"ab") as *mut c_void;
        common.fsh[f as usize].handleSync = mp_qshared::shared::qfalse;
        if common.fsh[f as usize].handleFiles.file.o.is_null() {
            return 0;
        }
    }
    f
}

/// Raven `FS_Read2`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1007-1024`
pub fn FS_Read2(common: &mut Common, buffer: *mut (), len: c_int, f: fileHandle_t) -> c_int {
    if common.fs_searchpaths.is_null() {
        crate::common::com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
    }

    if f == 0 {
        return 0;
    }
    if common.fsh[f as usize].streamed != mp_qshared::shared::qfalse {
        common.fsh[f as usize].streamed = mp_qshared::shared::qfalse;
        let r = Sys_StreamedRead(common, buffer, len, 1, f);
        common.fsh[f as usize].streamed = mp_qshared::shared::qtrue;
        r
    } else {
        FS_Read(common, buffer, len, f)
    }
}

/// Raven `FS_Seek`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1131-1181`
pub fn FS_Seek(view: &mut EngineHostView, f: fileHandle_t, offset: c_long, origin: c_int) -> c_int {
    // §19: `foo[65536]` is read-before-write only through FS_Read (which
    // fills it) — zero-init to be safe.
    let mut foo = [0u8; 65536];

    if view.common.fs_searchpaths.is_null() {
        crate::common::com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
    }

    if view.common.fsh[f as usize].streamed != mp_qshared::shared::qfalse {
        view.common.fsh[f as usize].streamed = mp_qshared::shared::qfalse;
        Sys_StreamSeek(view, f, offset, origin);
        view.common.fsh[f as usize].streamed = mp_qshared::shared::qtrue;
    }

    if view.common.fsh[f as usize].zipFile == mp_qshared::shared::qtrue {
        if offset == 0 && origin == fsOrigin_t::FS_SEEK_SET as c_int {
            // set the file position in the zip file (also sets the current file info)
            unsafe {
                crate::unzip::unzSetCurrentFileInfoPosition(
                    view.common.fsh[f as usize].handleFiles.file.z,
                    view.common.fsh[f as usize].zipFilePos as c_ulong,
                );
            }
            unsafe { unzOpenCurrentFile(view.common.fsh[f as usize].handleFiles.file.z) }
        } else if offset < 65536 {
            // set the file position in the zip file (also sets the current file info)
            unsafe {
                crate::unzip::unzSetCurrentFileInfoPosition(
                    view.common.fsh[f as usize].handleFiles.file.z,
                    view.common.fsh[f as usize].zipFilePos as c_ulong,
                );
            }
            unsafe {
                unzOpenCurrentFile(view.common.fsh[f as usize].handleFiles.file.z);
            }
            FS_Read(view.common, foo.as_mut_ptr() as *mut (), offset as c_int, f)
        } else {
            crate::common::com_error(
                errorParm_t::ERR_FATAL,
                "ZIP FILE FSEEK NOT YET IMPLEMENTED\n".to_string(),
            )
        }
    } else {
        let file = FS_FileForHandle(view.common, f);
        let _origin = match origin {
            x if x == fsOrigin_t::FS_SEEK_CUR as c_int => libc::SEEK_CUR,
            x if x == fsOrigin_t::FS_SEEK_END as c_int => libc::SEEK_END,
            x if x == fsOrigin_t::FS_SEEK_SET as c_int => libc::SEEK_SET,
            _ => crate::common::com_error(
                errorParm_t::ERR_FATAL,
                "Bad origin in FS_Seek\n".to_string(),
            ),
        };

        unsafe { libc::fseek(file, offset as libc::c_long, _origin) }
    }
}

/// Raven `FS_FileIsInPAK` — Raven's `1`/`-1` return plus optional
/// `int *pChecksum` out-param collapse per §C7: `Some(pure_checksum)` when the
/// file is in an allowed pak (the ruling-59a shape at the host seam).
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1191-1249`
pub fn FS_FileIsInPAK(common: &mut Common, filename: &str) -> Option<c_int> {
    if common.fs_searchpaths.is_null() {
        com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
    }

    // qpaths are not supposed to have a leading slash
    let filename = filename.strip_prefix(['/', '\\']).unwrap_or(filename);

    // make absolutely sure that it can't back up the path.
    if filename.contains("..") || filename.contains("::") {
        return None;
    }

    // search through the path, one element at a time
    unsafe {
        let mut search = common.fs_searchpaths;
        while !search.is_null() {
            let mut hash: c_long = 0;
            if !(*search).pack.is_null() {
                hash = FS_HashFileName(filename, (*(*search).pack).hashSize);
            }
            // is the element a pak file?
            if !(*search).pack.is_null() && (&(*(*search).pack).hashTable)[hash as usize].is_some()
            {
                // disregard if it doesn't match one of the allowed pure pak files
                if FS_PakIsPure(common, (*search).pack) == mp_qshared::shared::qfalse {
                    search = (*search).next;
                    continue;
                }

                // look through all the pak file elements
                let pak = &*(*search).pack;
                let mut pakFile = pak.hashTable[hash as usize];
                while let Some(fi) = pakFile {
                    // case and separator insensitive comparisons
                    if FS_FilenameCompare(&pak.buildBuffer[fi as usize].name, filename) {
                        return Some(pak.pure_checksum);
                    }
                    pakFile = pak.buildBuffer[fi as usize].next;
                }
            }
            search = (*search).next;
        }
    }
    None
}

/// Raven `Sys_ConcatenateFileLists`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1818-1854`
pub fn Sys_ConcatenateFileLists(
    view: &mut EngineHostView,
    list0: *mut *mut c_char,
    list1: *mut *mut c_char,
    list2: *mut *mut c_char,
) -> *mut *mut c_char {
    let mut total_length: usize = 0;

    total_length += Sys_CountFileList(list0) as usize;
    total_length += Sys_CountFileList(list1) as usize;
    total_length += Sys_CountFileList(list2) as usize;

    // Create new list.
    // Raven's chain is Z_Malloc/Z_Free end-to-end; the lists come from the
    // libc-malloc'd native Sys_ListFiles here, so the whole chain uses libc.
    let cat = unsafe { libc::calloc(total_length + 1, core::mem::size_of::<*mut c_char>()) }
        as *mut *mut c_char;
    let mut dst = cat;

    unsafe {
        // Copy over lists.
        if !list0.is_null() {
            let mut src = list0;
            while !(*src).is_null() {
                *dst = *src;
                src = src.add(1);
                dst = dst.add(1);
            }
        }
        if !list1.is_null() {
            let mut src = list1;
            while !(*src).is_null() {
                *dst = *src;
                src = src.add(1);
                dst = dst.add(1);
            }
        }
        if !list2.is_null() {
            let mut src = list2;
            while !(*src).is_null() {
                *dst = *src;
                src = src.add(1);
                dst = dst.add(1);
            }
        }

        // Terminate the list
        *dst = core::ptr::null_mut();

        // Free our old lists.
        // NOTE: not freeing their content, it's been merged in dst and still being used
        if !list0.is_null() {
            libc::free(list0 as *mut libc::c_void);
        }
        if !list1.is_null() {
            libc::free(list1 as *mut libc::c_void);
        }
        if !list2.is_null() {
            libc::free(list2 as *mut libc::c_void);
        }
    }

    cat
}

/// Raven `FS_UpdateGamedir`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2419-2436`
pub fn FS_UpdateGamedir(view: &mut EngineHostView) {
    let gamedirvar = view.common.cvar(view.common.fs_gamedirvar).string.clone();
    if !gamedirvar.is_empty() && !gamedirvar.eq_ignore_ascii_case(BASEGAME) {
        let cdpath = view.common.cvar(view.common.fs_cdpath).string.clone();
        if !cdpath.is_empty() {
            FS_AddGameDirectory(view, &cdpath, &gamedirvar);
        }
        let basepath = view.common.cvar(view.common.fs_basepath).string.clone();
        if !basepath.is_empty() {
            FS_AddGameDirectory(view, &basepath, &gamedirvar);
        }
        let homepath = view.common.cvar(view.common.fs_homepath).string.clone();
        if !homepath.is_empty() && !homepath.eq_ignore_ascii_case(&basepath) {
            FS_AddGameDirectory(view, &homepath, &gamedirvar);
        }
    }
}

/// Raven `FS_PureServerSetReferencedPaks`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2947-2981`
pub fn FS_PureServerSetReferencedPaks(view: &mut EngineHostView, pakSums: &str, pakNames: &str) {
    Cmd_TokenizeString(view.common, pakSums);

    let mut c = Cmd_Argc(view.common);
    if c > MAX_SEARCH_PATHS as c_int {
        c = MAX_SEARCH_PATHS as c_int;
    }

    view.common.fs_numServerReferencedPaks = c;

    for i in 0..c as usize {
        view.common.fs_serverReferencedPaks[i] = atoi(Cmd_Argv(view.common, i as c_int));
    }

    view.common.fs_serverReferencedPakNames.clear();
    if !pakNames.is_empty() {
        Cmd_TokenizeString(view.common, pakNames);

        let mut d = Cmd_Argc(view.common);
        if d > MAX_SEARCH_PATHS as c_int {
            d = MAX_SEARCH_PATHS as c_int;
        }

        for i in 0..d as usize {
            let name = Cmd_Argv(view.common, i as c_int).to_owned();
            view.common.fs_serverReferencedPakNames.push(name);
        }
    }
}

/// Raven `FS_ConditionalRestart`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:3048-3054`
pub fn FS_ConditionalRestart(view: &mut EngineHostView, checksumFeed: c_int) -> qboolean {
    if view.common.cvar(view.common.fs_gamedirvar).modified
        || checksumFeed != view.common.fs_checksumFeed
    {
        FS_Restart(view, checksumFeed);
        return mp_qshared::shared::qtrue;
    }
    mp_qshared::shared::qfalse
}

/// Raven `FS_GetModList`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1866-1977`
pub fn FS_GetModList(view: &mut EngineHostView, listbuf: *mut c_char, bufsize: c_int) -> c_int {
    let mut n_mods: c_int = 0;
    let mut n_total: c_int = 0;
    let mut listbuf = listbuf;

    let homepath = view.common.cvar(view.common.fs_homepath).string.clone();
    let basepath = view.common.cvar(view.common.fs_basepath).string.clone();
    let cdpath = view.common.cvar(view.common.fs_cdpath).string.clone();
    unsafe {
        *listbuf = 0;

        // we searched for mods in the three paths
        // it is likely that we have duplicate names now, which we will cleanup below
        // (Raven's Sys_ConcatenateFileLists is a Vec extend now.)
        let mut p_files = Sys_ListFiles(&homepath, None, None, true);
        p_files.extend(Sys_ListFiles(&basepath, None, None, true));
        p_files.extend(Sys_ListFiles(&cdpath, None, None, true));

        for i in 0..p_files.len() {
            let name_str = p_files[i].clone();
            // NOTE: cleaner would involve more changes
            // ignore duplicate mod directories
            let mut b_drop = false;
            if i != 0 {
                for j in 0..i {
                    if p_files[j].eq_ignore_ascii_case(&name_str) {
                        // this one can be dropped
                        b_drop = true;
                        break;
                    }
                }
            }
            if b_drop {
                continue;
            }
            // we drop "base" "." and ".."
            if !name_str.eq_ignore_ascii_case(BASEGAME) && !name_str.starts_with('.') {
                // now we need to find some .pk3 files to validate the mod
                // (we only use Sys_ListFiles to check whether .pk3 files are present)
                let mut path = FS_BuildOSPath4(view.common, &basepath, &name_str, "");
                let mut n_paks = Sys_ListFiles(&path, Some(".pk3"), None, false).len();

                // Try on cd path
                if n_paks == 0 {
                    path = FS_BuildOSPath4(view.common, &cdpath, &name_str, "");
                    n_paks = Sys_ListFiles(&path, Some(".pk3"), None, false).len();
                }

                // try on home path
                if n_paks == 0 {
                    path = FS_BuildOSPath4(view.common, &homepath, &name_str, "");
                    n_paks = Sys_ListFiles(&path, Some(".pk3"), None, false).len();
                }

                if n_paks > 0 {
                    let n_len = name_str.len() + 1;
                    // nLen is the length of the mod path
                    // we need to see if there is a description available
                    let mut desc_handle: fileHandle_t = 0;
                    let mut n_desc_len = FS_SV_FOpenFileRead(
                        view.common,
                        &format!("{}/description.txt", name_str),
                        &mut desc_handle,
                    );
                    let desc_str: String;
                    if n_desc_len > 0 && desc_handle != 0 {
                        let file = FS_FileForHandle(view.common, desc_handle);
                        let mut buf = [0u8; 49];
                        n_desc_len = libc::fread(buf.as_mut_ptr() as *mut _, 1, 48, file) as c_int;
                        if n_desc_len >= 0 {
                            buf[n_desc_len as usize] = 0;
                        }
                        FS_FCloseFile(view.common, desc_handle);
                        desc_str = c_str_to_string(buf.as_ptr() as *const c_char);
                    } else {
                        desc_str = name_str.clone();
                    }
                    let n_desc_len = desc_str.len() + 1;

                    if (n_total as usize) + n_len + 1 + n_desc_len + 1 < bufsize as usize {
                        // module listbuf packing (the trap seam), advancing per
                        // Raven's `listbuf += nLen` (the prior port rewrote
                        // every mod at offset 0)
                        Q_strncpyz(
                            core::slice::from_raw_parts_mut(listbuf, n_len),
                            &name_str,
                            n_len,
                        );
                        listbuf = listbuf.add(n_len);
                        Q_strncpyz(
                            core::slice::from_raw_parts_mut(listbuf, n_desc_len),
                            &desc_str,
                            n_desc_len,
                        );
                        listbuf = listbuf.add(n_desc_len);
                        n_total += (n_len + n_desc_len) as c_int;
                        n_mods += 1;
                    } else {
                        break;
                    }
                }
            }
        }
    }

    n_mods
}

/// Raven `FS_FOpenFileByMode`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:3064-3116`
pub fn FS_FOpenFileByMode(
    view: &mut EngineHostView,
    qpath: &str,
    f: *mut fileHandle_t,
    mode: fsMode_t,
) -> c_int {
    let mut sync = qfalse;

    let r;
    match mode {
        m if m == FS_READ => {
            r = FS_FOpenFileRead(view, qpath, f, true);
        }
        m if m == FS_WRITE => unsafe {
            *f = FS_FOpenFileWrite(view.common, qpath);
            r = if *f == 0 { -1 } else { 0 };
        },
        m if m == FS_APPEND_SYNC || m == FS_APPEND => unsafe {
            sync = if m == FS_APPEND_SYNC { qtrue } else { sync };
            *f = FS_FOpenFileAppend(view.common, qpath);
            r = if *f == 0 { -1 } else { 0 };
        },
        _ => {
            com_error(
                errorParm_t::ERR_FATAL,
                "FSH_FOpenFile: bad mode".to_string(),
            );
        }
    }

    if f.is_null() {
        return r;
    }

    unsafe {
        if *f != 0 {
            if view.common.fsh[*f as usize].zipFile == qtrue {
                view.common.fsh[*f as usize].baseOffset =
                    unztell(view.common.fsh[*f as usize].handleFiles.file.z) as c_int;
            } else {
                view.common.fsh[*f as usize].baseOffset =
                    libc::ftell(view.common.fsh[*f as usize].handleFiles.file.o as *mut libc::FILE)
                        as c_int;
            }
            view.common.fsh[*f as usize].fileSize = r;
            view.common.fsh[*f as usize].streamed = qfalse;

            if mode == FS_READ {
                Sys_BeginStreamedFile(*f, 0x4000);
                view.common.fsh[*f as usize].streamed = qtrue;
            }
        }
        view.common.fsh[*f as usize].handleSync = sync;
    }

    r
}

/// Raven `FS_GetFileList`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1758-1788`
pub fn FS_GetFileList(
    view: &mut EngineHostView,
    path: &str,
    extension: &str,
    listbuf: *mut c_char,
    bufsize: c_int,
) -> c_int {
    let mut n_total: c_int = 0;

    unsafe {
        *listbuf = 0;

        if path == "$modlist" {
            return FS_GetModList(view, listbuf, bufsize);
        }

        let files = FS_ListFiles(view, path, extension);

        let mut n_files = files.len() as c_int;
        let mut listbuf = listbuf;
        for (i, entry) in files.iter().enumerate() {
            let n_len = entry.len() + 1;
            if (n_total as usize) + n_len + 1 < bufsize as usize {
                Q_strncpyz(
                    core::slice::from_raw_parts_mut(listbuf, n_len),
                    entry,
                    n_len,
                );
                listbuf = listbuf.add(n_len);
                n_total += n_len as c_int;
            } else {
                n_files = i as c_int;
                break;
            }
        }

        n_files
    }
}

/// Raven-faithful helper for the "static char info[N]" rotating-scratch
/// pattern (fork-3 three-kind rule): writes `s` (plus NUL) into the owned
/// scratch buffer on `Common`, truncating to the buffer's fixed size, and
/// returns a pointer to it exactly as the Raven fn returns its static buffer.
fn write_info_scratch(dst: &mut [c_char], s: &str) -> *const c_char {
    let bytes = s.as_bytes();
    let n = bytes.len().min(dst.len() - 1);
    for (i, b) in bytes[..n].iter().enumerate() {
        dst[i] = *b as c_char;
    }
    dst[n] = 0;
    dst.as_ptr()
}

unsafe fn c_str_to_string(s: *const c_char) -> String {
    if s.is_null() {
        return String::new();
    }
    std::ffi::CStr::from_ptr(s).to_string_lossy().into_owned()
}

fn append_cstr(dst: *mut c_char, size: c_int, s: &str) {
    unsafe {
        let cur = libc::strlen(dst);
        let c = std::ffi::CString::new(s).unwrap();
        let remaining = (size as usize).saturating_sub(cur + 1);
        let copy_len = (c.as_bytes().len()).min(remaining);
        core::ptr::copy_nonoverlapping(c.as_ptr(), dst.add(cur), copy_len);
        *dst.add(cur + copy_len) = 0;
    }
}
