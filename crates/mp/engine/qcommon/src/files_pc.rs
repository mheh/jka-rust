//! `files_pc.cpp` — filesystem PC-platform logic (pak purity, path
//! normalization, referenced/loaded pak-list info strings, seek/rename/read).

#![allow(non_snake_case, non_upper_case_globals, unused_variables)]

use core::ffi::{c_char, c_int, c_long, c_uint, c_ulong, c_void};

use mp_host_interface::engine_host::EngineHost;
use mp_qshared::common::mp::qcommon::tags::memtag_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::fs_origin::fsOrigin_t;
use mp_qshared::shared::limits::{BIG_INFO_STRING, MAX_STRING_TOKENS};
use mp_qshared::shared::qboolean;
use mp_qshared::shared::{fsMode_t, FS_APPEND, FS_APPEND_SYNC, FS_READ, FS_WRITE};
use native_types::fileHandle_t;

use crate::collision_world::CollisionWorld;
use crate::common::Common;
use crate::files::file_in_pack_s::fileInPack_t;
use crate::files::files_consts::{BASEGAME, MAX_SEARCH_PATHS, MAX_ZPATH};
use crate::files::pack_t::pack_t;
use crate::files::searchpath_s::searchpath_t;
use crate::qcommon::filesystem_limits::{
    FS_CGAME_REF, FS_GENERAL_REF, FS_QAGAME_REF, FS_UI_REF, MAX_FILE_HANDLES, NUM_ID_PAKS,
};
use crate::cm_load::RenderModels;

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
use crate::files::unz_file::unzOpenCurrentFile;
use crate::files_common::{
    FS_AddGameDirectory, FS_CopyFile, FS_CreatePath, FS_FCloseFile, FS_FileForHandle,
    FS_FOpenFileRead, FS_FOpenFileWrite, FS_FreeFileList, FS_HandleForFile, FS_ListFiles, FS_Read,
    FS_Restart, FS_SV_FOpenFileRead,
};
use crate::z_memman_pc::{CopyString, Z_Free, Z_Malloc};
use native_platform::{
    Sys_BeginStreamedFile, Sys_FreeFileList, Sys_ListFiles, Sys_StreamedRead, Sys_StreamSeek,
};

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

/// Raven `FS_HashFileName`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:64-82`
pub fn FS_HashFileName(fname: *const c_char, hashSize: c_int) -> c_long {
    let mut hash: c_long = 0;
    let mut i: isize = 0;
    unsafe {
        loop {
            let c = *fname.offset(i);
            if c == 0 {
                break;
            }
            let mut letter = (c as u8 as char).to_ascii_lowercase() as c_char;
            if letter == b'.' as c_char {
                break; // don't include extension
            }
            if letter == b'\\' as c_char {
                letter = b'/' as c_char; // damn path names
            }
            if letter == b'/' as c_char {
                letter = b'/' as c_char; // damn path names
            }
            hash += (letter as c_long) * (i as c_long + 119);
            i += 1;
        }
    }
    hash = hash ^ (hash >> 10) ^ (hash >> 20);
    hash &= (hashSize as c_long) - 1;
    hash
}

/// Raven `FS_Remove`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:213-215`
pub fn FS_Remove(osPath: *const c_char) {
    unsafe {
        libc::remove(osPath);
    }
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
    if unsafe { (*common.com_developer).integer } != 0 {
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
    if !common.com_buildScript.is_null() && unsafe { (*common.com_buildScript).integer } != 0 {
        return true;
    }
    unsafe { !libc::strchr(filename, b'/' as c_int).is_null() }
}

/// Raven `FS_ShiftedStrStr`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:649-658`
pub fn FS_ShiftedStrStr(
    string: *const c_char,
    substring: *const c_char,
    shift: c_int,
) -> *mut c_char {
    let mut buf = [0 as c_char; MAX_STRING_TOKENS];
    let mut i: isize = 0;
    unsafe {
        loop {
            let c = *substring.offset(i);
            if c == 0 {
                break;
            }
            buf[i as usize] = c + shift as c_char;
            i += 1;
        }
        buf[i as usize] = 0;
        libc::strstr(string, buf.as_ptr())
    }
}

/// Raven `FS_ReturnPath`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1534-1555`
pub fn FS_ReturnPath(zname: *const c_char, zpath: *mut c_char, depth: *mut c_int) -> c_int {
    let mut len: c_int = 0;
    let mut at: isize = 0;
    let mut newdep: c_int = 0;

    unsafe {
        *zpath = 0;

        while *zname.offset(at) != 0 {
            let c = *zname.offset(at);
            if c == b'/' as c_char || c == b'\\' as c_char {
                len = at as c_int;
                newdep += 1;
            }
            at += 1;
        }
        libc::strcpy(zpath, zname);
        *zpath.offset(len as isize) = 0;
        *depth = newdep;
    }

    len
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
pub fn FS_ConvertPath(s: *mut c_char) {
    unsafe {
        let mut p = s;
        while *p != 0 {
            if *p == b'\\' as c_char || *p == b':' as c_char {
                *p = b'/' as c_char;
            }
            p = p.add(1);
        }
    }
}

/// Raven `FS_PathCmp`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2041-2071`
pub fn FS_PathCmp(s1: *const c_char, s2: *const c_char) -> c_int {
    let mut p1 = s1;
    let mut p2 = s2;
    loop {
        let (mut c1, mut c2);
        unsafe {
            c1 = *p1 as c_int;
            c2 = *p2 as c_int;
            p1 = p1.add(1);
            p2 = p2.add(1);
        }

        if (b'a' as c_int..=b'z' as c_int).contains(&c1) {
            c1 -= b'a' as c_int - b'A' as c_int;
        }
        if (b'a' as c_int..=b'z' as c_int).contains(&c2) {
            c2 -= b'a' as c_int - b'A' as c_int;
        }

        if c1 == b'\\' as c_int || c1 == b':' as c_int {
            c1 = b'/' as c_int;
        }
        if c2 == b'\\' as c_int || c2 == b':' as c_int {
            c2 = b'/' as c_int;
        }

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
                info.push_str(&c_str_to_string((*(*search).pack).pakBasename.as_ptr()));
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
                let gamename = c_str_to_string((*pak).pakGamename.as_ptr());
                if (*pak).referenced != 0 || !gamename.eq_ignore_ascii_case(BASEGAME) {
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
                let gamename = c_str_to_string((*pak).pakGamename.as_ptr());
                if (*pak).referenced != 0 || !gamename.eq_ignore_ascii_case(BASEGAME) {
                    info.push_str(&gamename);
                    info.push('/');
                    info.push_str(&c_str_to_string((*pak).pakBasename.as_ptr()));
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

/// Raven `paksort`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2194-2201`
pub fn paksort(a: *const (), b: *const ()) -> c_int {
    unsafe {
        let aa = *(a as *const *const c_char);
        let bb = *(b as *const *const c_char);
        FS_PathCmp(aa, bb)
    }
}

/// Raven `FS_idPak`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2301-2313`
pub fn FS_idPak(pak: *mut c_char, base: *mut c_char) -> qboolean {
    let base_str = unsafe { c_str_to_string(base) };
    let mut i = 0;
    while i < NUM_ID_PAKS {
        let candidate = format!("{}/assets{}", base_str, i);
        let candidate_c = std::ffi::CString::new(candidate).unwrap();
        if unsafe { crate::files_common::FS_FilenameCompare(pak, candidate_c.as_ptr()) } == 0 {
            break;
        }
        i += 1;
    }
    if i < NUM_ID_PAKS {
        mp_qshared::shared::qtrue
    } else {
        mp_qshared::shared::qfalse
    }
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

/// Raven `FS_FileExists`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:227-240`
pub fn FS_FileExists(common: &mut Common, file: *const c_char) -> qboolean {
    unsafe {
        let testpath = crate::files_common::FS_BuildOSPath4(
            common,
            (*common.fs_homepath).string,
            common.fs_gamedir.as_ptr(),
            file,
        );
        let f = libc::fopen(testpath, c"rb".as_ptr());
        if !f.is_null() {
            libc::fclose(f);
            return mp_qshared::shared::qtrue;
        }
    }
    mp_qshared::shared::qfalse
}

/// Raven `FS_SV_FileExists`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:249-263`
pub fn FS_SV_FileExists(common: &mut Common, file: *const c_char) -> qboolean {
    unsafe {
        let testpath = crate::files_common::FS_BuildOSPath4(
            common,
            (*common.fs_homepath).string,
            file,
            c"".as_ptr(),
        );
        let len = libc::strlen(testpath);
        *testpath.add(len - 1) = 0;

        let f = libc::fopen(testpath, c"rb".as_ptr());
        if !f.is_null() {
            libc::fclose(f);
            return mp_qshared::shared::qtrue;
        }
    }
    mp_qshared::shared::qfalse
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
            let mut havepak = mp_qshared::shared::qfalse;

            // never autodownload any of the id paks
            let base_c = c"base".as_ptr();
            let missionpack_c = c"missionpack".as_ptr();
            if FS_idPak(common.fs_serverReferencedPakNames[i], base_c as *mut c_char)
                != mp_qshared::shared::qfalse
                || FS_idPak(
                    common.fs_serverReferencedPakNames[i],
                    missionpack_c as *mut c_char,
                ) != mp_qshared::shared::qfalse
            {
                continue;
            }

            let mut sp = common.fs_searchpaths;
            while !sp.is_null() {
                if !(*sp).pack.is_null()
                    && (*(*sp).pack).checksum == common.fs_serverReferencedPaks[i]
                {
                    havepak = mp_qshared::shared::qtrue; // This is it!
                    break;
                }
                sp = (*sp).next;
            }

            if havepak == mp_qshared::shared::qfalse
                && !common.fs_serverReferencedPakNames[i].is_null()
                && *common.fs_serverReferencedPakNames[i] != 0
            {
                // Don't got it
                let name = c_str_to_string(common.fs_serverReferencedPakNames[i]);
                if dlstring != mp_qshared::shared::qfalse {
                    // Remote name
                    append_cstr(neededpaks, len, "@");
                    append_cstr(neededpaks, len, &name);
                    append_cstr(neededpaks, len, ".pk3");

                    // Local name
                    append_cstr(neededpaks, len, "@");
                    // Do we have one with the same name?
                    let dl_name = std::ffi::CString::new(format!("{}.pk3", name)).unwrap();
                    if FS_SV_FileExists(common, dl_name.as_ptr()) != mp_qshared::shared::qfalse {
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
                    let dl_name = std::ffi::CString::new(format!("{}.pk3", name)).unwrap();
                    if FS_SV_FileExists(common, dl_name.as_ptr()) != mp_qshared::shared::qfalse {
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
pub fn FS_SV_FOpenFileWrite(common: &mut Common, filename: *const c_char) -> fileHandle_t {
    if common.fs_searchpaths.is_null() {
        crate::common::com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
    }

    unsafe {
        let ospath = crate::files_common::FS_BuildOSPath4(
            common,
            (*common.fs_homepath).string,
            filename,
            c"".as_ptr(),
        );
        let len = libc::strlen(ospath);
        *ospath.add(len - 1) = 0;

        let f = unsafe { FS_HandleForFile(common) };
        common.fsh[f as usize].zipFile = mp_qshared::shared::qfalse;

        if (*common.fs_debug).integer != 0 {
            crate::common::com_printf(
                common,
                &format!("FS_SV_FOpenFileWrite: {}\n", c_str_to_string(ospath)),
            );
        }

        if FS_CreatePath(common, ospath) != 0 {
            return 0;
        }

        // Com_DPrintf( "writing to: %s\n", ospath );
        common.fsh[f as usize].handleFiles.file.o =
            libc::fopen(ospath, c"wb".as_ptr()) as *mut c_void;

        copy_cname(
            common.fsh[f as usize].name.as_mut_ptr(),
            filename,
            common.fsh[f as usize].name.len(),
        );

        common.fsh[f as usize].handleSync = mp_qshared::shared::qfalse;
        if common.fsh[f as usize].handleFiles.file.o.is_null() {
            return 0;
        }
        f
    }
}

/// Raven `FS_SV_Rename`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:395-419`
pub fn FS_SV_Rename(common: &mut Common, from: *const c_char, to: *const c_char) {
    if common.fs_searchpaths.is_null() {
        crate::common::com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
    }

    // don't let sound stutter
    null::S_ClearSoundBuffer();

    unsafe {
        let from_ospath = crate::files_common::FS_BuildOSPath4(
            common,
            (*common.fs_homepath).string,
            from,
            c"".as_ptr(),
        );
        let to_ospath = crate::files_common::FS_BuildOSPath4(
            common,
            (*common.fs_homepath).string,
            to,
            c"".as_ptr(),
        );
        *from_ospath.add(libc::strlen(from_ospath) - 1) = 0;
        *to_ospath.add(libc::strlen(to_ospath) - 1) = 0;

        if (*common.fs_debug).integer != 0 {
            crate::common::com_printf(
                common,
                &format!(
                    "FS_SV_Rename: {} --> {}\n",
                    c_str_to_string(from_ospath),
                    c_str_to_string(to_ospath)
                ),
            );
        }

        if libc::rename(from_ospath, to_ospath) != 0 {
            // Failed, try copying it and deleting the original
            FS_CopyFile(common, from_ospath, to_ospath);
            FS_Remove(from_ospath);
        }
    }
}

/// Raven `FS_Rename`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:427-449`
pub fn FS_Rename(common: &mut Common, from: *const c_char, to: *const c_char) {
    if common.fs_searchpaths.is_null() {
        crate::common::com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
    }

    // don't let sound stutter
    null::S_ClearSoundBuffer();

    unsafe {
        let from_ospath = crate::files_common::FS_BuildOSPath4(
            common,
            (*common.fs_homepath).string,
            common.fs_gamedir.as_ptr(),
            from,
        );
        let to_ospath = crate::files_common::FS_BuildOSPath4(
            common,
            (*common.fs_homepath).string,
            common.fs_gamedir.as_ptr(),
            to,
        );

        if (*common.fs_debug).integer != 0 {
            crate::common::com_printf(
                common,
                &format!(
                    "FS_Rename: {} --> {}\n",
                    c_str_to_string(from_ospath),
                    c_str_to_string(to_ospath)
                ),
            );
        }

        if libc::rename(from_ospath, to_ospath) != 0 {
            // Failed, try copying it and deleting the original
            FS_CopyFile(common, from_ospath, to_ospath);
            FS_Remove(from_ospath);
        }
    }
}

/// Raven `FS_FOpenFileAppend`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:532-564`
pub fn FS_FOpenFileAppend(common: &mut Common, filename: *const c_char) -> fileHandle_t {
    if common.fs_searchpaths.is_null() {
        crate::common::com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
    }

    let f = unsafe { FS_HandleForFile(common) };
    common.fsh[f as usize].zipFile = mp_qshared::shared::qfalse;

    unsafe {
        copy_cname(
            common.fsh[f as usize].name.as_mut_ptr(),
            filename,
            common.fsh[f as usize].name.len(),
        );
    }

    // don't let sound stutter
    null::S_ClearSoundBuffer();

    unsafe {
        let ospath = crate::files_common::FS_BuildOSPath4(
            common,
            (*common.fs_homepath).string,
            common.fs_gamedir.as_ptr(),
            filename,
        );

        if (*common.fs_debug).integer != 0 {
            crate::common::com_printf(
                common,
                &format!("FS_FOpenFileAppend: {}\n", c_str_to_string(ospath)),
            );
        }

        if FS_CreatePath(common, ospath) != 0 {
            return 0;
        }

        common.fsh[f as usize].handleFiles.file.o =
            libc::fopen(ospath, c"ab".as_ptr()) as *mut c_void;
        common.fsh[f as usize].handleSync = mp_qshared::shared::qfalse;
        if common.fsh[f as usize].handleFiles.file.o.is_null() {
            return 0;
        }
        f
    }
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
        let r = unsafe { Sys_StreamedRead(buffer, len, 1, f) };
        common.fsh[f as usize].streamed = mp_qshared::shared::qtrue;
        r
    } else {
        unsafe { FS_Read(common, buffer, len, f) }
    }
}

/// Raven `FS_Seek`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1131-1181`
pub fn FS_Seek(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    f: fileHandle_t,
    offset: c_long,
    origin: c_int,
) -> c_int {
    // §19: `foo[65536]` is read-before-write only through FS_Read (which
    // fills it) — zero-init to be safe.
    let mut foo = [0u8; 65536];

    if common.fs_searchpaths.is_null() {
        crate::common::com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
        return -1;
    }

    if common.fsh[f as usize].streamed != mp_qshared::shared::qfalse {
        common.fsh[f as usize].streamed = mp_qshared::shared::qfalse;
        unsafe { Sys_StreamSeek(f, offset, origin) };
        common.fsh[f as usize].streamed = mp_qshared::shared::qtrue;
    }

    if common.fsh[f as usize].zipFile == mp_qshared::shared::qtrue {
        if offset == 0 && origin == fsOrigin_t::FS_SEEK_SET as c_int {
            // set the file position in the zip file (also sets the current file info)
            unsafe {
                crate::unzip::unzSetCurrentFileInfoPosition(
                    common.fsh[f as usize].handleFiles.file.z,
                    common.fsh[f as usize].zipFilePos as c_ulong,
                );
            }
            unsafe {
                unzOpenCurrentFile(
                    common,
                    cm,
                    rm,
                    host,
                    common.fsh[f as usize].handleFiles.file.z,
                )
            }
        } else if offset < 65536 {
            // set the file position in the zip file (also sets the current file info)
            unsafe {
                crate::unzip::unzSetCurrentFileInfoPosition(
                    common.fsh[f as usize].handleFiles.file.z,
                    common.fsh[f as usize].zipFilePos as c_ulong,
                );
            }
            unsafe {
                unzOpenCurrentFile(
                    common,
                    cm,
                    rm,
                    host,
                    common.fsh[f as usize].handleFiles.file.z,
                );
            }
            unsafe { FS_Read(common, foo.as_mut_ptr() as *mut (), offset as c_int, f) }
        } else {
            crate::common::com_error(
                errorParm_t::ERR_FATAL,
                "ZIP FILE FSEEK NOT YET IMPLEMENTED\n".to_string(),
            );
            -1
        }
    } else {
        let file = unsafe { FS_FileForHandle(common, f) };
        let _origin = match origin {
            x if x == fsOrigin_t::FS_SEEK_CUR as c_int => libc::SEEK_CUR,
            x if x == fsOrigin_t::FS_SEEK_END as c_int => libc::SEEK_END,
            x if x == fsOrigin_t::FS_SEEK_SET as c_int => libc::SEEK_SET,
            _ => {
                crate::common::com_error(
                    errorParm_t::ERR_FATAL,
                    "Bad origin in FS_Seek\n".to_string(),
                );
                libc::SEEK_CUR
            }
        };

        unsafe { libc::fseek(file, offset as libc::c_long, _origin) }
    }
}

/// Raven `FS_FileIsInPAK`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1191-1249`
pub fn FS_FileIsInPAK(
    common: &mut Common,
    mut filename: *const c_char,
    pChecksum: *mut c_int,
) -> c_int {
    if common.fs_searchpaths.is_null() {
        crate::common::com_error(
            errorParm_t::ERR_FATAL,
            "Filesystem call made without initialization\n".to_string(),
        );
    }

    if filename.is_null() {
        crate::common::com_error(
            errorParm_t::ERR_FATAL,
            "FS_FOpenFileRead: NULL 'filename' parameter passed\n".to_string(),
        );
    }

    unsafe {
        // qpaths are not supposed to have a leading slash
        if *filename == b'/' as c_char || *filename == b'\\' as c_char {
            filename = filename.add(1);
        }
    }

    // make absolutely sure that it can't back up the path.
    let name = unsafe { c_str_to_string(filename) };
    if name.contains("..") || name.contains("::") {
        return -1;
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
            if !(*search).pack.is_null()
                && !(*(*search).pack)
                    .hashTable
                    .add(hash as usize)
                    .read()
                    .is_null()
            {
                // disregard if it doesn't match one of the allowed pure pak files
                if FS_PakIsPure(common, (*search).pack) == mp_qshared::shared::qfalse {
                    search = (*search).next;
                    continue;
                }

                // look through all the pak file elements
                let pak = (*search).pack;
                let mut pakFile: *mut fileInPack_t = *(*pak).hashTable.add(hash as usize);
                loop {
                    // case and separator insensitive comparisons
                    if crate::files_common::FS_FilenameCompare((*pakFile).name, filename) == 0 {
                        if !pChecksum.is_null() {
                            *pChecksum = (*pak).pure_checksum;
                        }
                        return 1;
                    }
                    pakFile = (*pakFile).next;
                    if pakFile.is_null() {
                        break;
                    }
                }
            }
            search = (*search).next;
        }
    }
    -1
}

/// Raven `Sys_ConcatenateFileLists`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1818-1854`
pub fn Sys_ConcatenateFileLists(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    list0: *mut *mut c_char,
    list1: *mut *mut c_char,
    list2: *mut *mut c_char,
) -> *mut *mut c_char {
    let mut total_length: usize = 0;

    total_length += Sys_CountFileList(list0) as usize;
    total_length += Sys_CountFileList(list1) as usize;
    total_length += Sys_CountFileList(list2) as usize;

    // Create new list.
    let cat = unsafe {
        Z_Malloc(
            common,
            cm,
            rm,
            host,
            ((total_length + 1) * core::mem::size_of::<*mut c_char>()) as c_int,
            memtag_t::TAG_FILESYS,
            mp_qshared::shared::qtrue,
            4,
        )
    } as *mut *mut c_char;
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
            Z_Free(common, list0 as *mut ());
        }
        if !list1.is_null() {
            Z_Free(common, list1 as *mut ());
        }
        if !list2.is_null() {
            Z_Free(common, list2 as *mut ());
        }
    }

    cat
}

/// Raven `FS_UpdateGamedir`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2419-2436`
pub fn FS_UpdateGamedir(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    unsafe {
        let gamedirvar_str = c_str_to_string((*common.fs_gamedirvar).string);
        if !gamedirvar_str.is_empty() && !gamedirvar_str.eq_ignore_ascii_case(BASEGAME) {
            let cdpath_str = c_str_to_string((*common.fs_cdpath).string);
            if !cdpath_str.is_empty() {
                FS_AddGameDirectory(
                    common,
                    cm,
                    rm,
                    host,
                    (*common.fs_cdpath).string,
                    (*common.fs_gamedirvar).string,
                );
            }
            let basepath_str = c_str_to_string((*common.fs_basepath).string);
            if !basepath_str.is_empty() {
                FS_AddGameDirectory(
                    common,
                    cm,
                    rm,
                    host,
                    (*common.fs_basepath).string,
                    (*common.fs_gamedirvar).string,
                );
            }
            let homepath_str = c_str_to_string((*common.fs_homepath).string);
            if !homepath_str.is_empty() && !homepath_str.eq_ignore_ascii_case(&basepath_str) {
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
    }
}

/// Raven `FS_PureServerSetReferencedPaks`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:2947-2981`
pub fn FS_PureServerSetReferencedPaks(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    pakSums: *const c_char,
    pakNames: *const c_char,
) {
    crate::cmd_common::Cmd_TokenizeString(common, pakSums);

    let mut c = crate::cmd_common::Cmd_Argc(common);
    if c > MAX_SEARCH_PATHS as c_int {
        c = MAX_SEARCH_PATHS as c_int;
    }

    common.fs_numServerReferencedPaks = c;

    for i in 0..c as usize {
        let arg = crate::cmd_common::Cmd_Argv(common, i as c_int);
        common.fs_serverReferencedPaks[i] = unsafe { libc::atoi(arg) };
    }

    for i in 0..c as usize {
        if !common.fs_serverReferencedPakNames[i].is_null() {
            unsafe { Z_Free(common, common.fs_serverReferencedPakNames[i] as *mut ()) };
        }
        common.fs_serverReferencedPakNames[i] = core::ptr::null_mut();
    }
    let names_str = unsafe {
        if pakNames.is_null() {
            String::new()
        } else {
            c_str_to_string(pakNames)
        }
    };
    if !names_str.is_empty() {
        crate::cmd_common::Cmd_TokenizeString(common, pakNames);

        let mut d = crate::cmd_common::Cmd_Argc(common);
        if d > MAX_SEARCH_PATHS as c_int {
            d = MAX_SEARCH_PATHS as c_int;
        }

        for i in 0..d as usize {
            let arg = crate::cmd_common::Cmd_Argv(common, i as c_int);
            common.fs_serverReferencedPakNames[i] =
                unsafe { CopyString(common, cm, rm, host, arg) };
        }
    }
}

/// Raven `FS_ConditionalRestart`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:3048-3054`
pub fn FS_ConditionalRestart(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    checksumFeed: c_int,
) -> qboolean {
    unsafe {
        if (*common.fs_gamedirvar).modified != 0 || checksumFeed != common.fs_checksumFeed {
            FS_Restart(common, cm, rm, host, checksumFeed);
            return mp_qshared::shared::qtrue;
        }
    }
    mp_qshared::shared::qfalse
}

/// Raven `FS_GetModList`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1866-1977`
pub fn FS_GetModList(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    listbuf: *mut c_char,
    bufsize: c_int,
) -> c_int {
    let mut n_mods: c_int = 0;
    let mut n_total: c_int = 0;

    unsafe {
        *listbuf = 0;

        let mut dummy: c_int = 0;
        let p_files0 = Sys_ListFiles(
            common,
            (*common.fs_homepath).string,
            core::ptr::null(),
            core::ptr::null_mut(),
            &mut dummy,
            mp_qshared::shared::qtrue,
        );
        let p_files1 = Sys_ListFiles(
            common,
            (*common.fs_basepath).string,
            core::ptr::null(),
            core::ptr::null_mut(),
            &mut dummy,
            mp_qshared::shared::qtrue,
        );
        let p_files2 = Sys_ListFiles(
            common,
            (*common.fs_cdpath).string,
            core::ptr::null(),
            core::ptr::null_mut(),
            &mut dummy,
            mp_qshared::shared::qtrue,
        );
        // we searched for mods in the three paths
        // it is likely that we have duplicate names now, which we will cleanup below
        let p_files = Sys_ConcatenateFileLists(common, cm, rm, host, p_files0, p_files1, p_files2);
        let n_potential = Sys_CountFileList(p_files);

        for i in 0..n_potential as isize {
            let name = *p_files.offset(i);
            // NOTE: cleaner would involve more changes
            // ignore duplicate mod directories
            let mut b_drop = false;
            if i != 0 {
                for j in 0..i {
                    if libc::strcasecmp(*p_files.offset(j), name) == 0 {
                        // this one can be dropped
                        b_drop = true;
                        break;
                    }
                }
            }
            if b_drop {
                continue;
            }
            let name_str = c_str_to_string(name);
            // we drop "base" "." and ".."
            if !name_str.eq_ignore_ascii_case(BASEGAME) && !name_str.starts_with('.') {
                // now we need to find some .pk3 files to validate the mod
                let mut path = crate::files_common::FS_BuildOSPath4(
                    common,
                    (*common.fs_basepath).string,
                    name,
                    c"".as_ptr(),
                );
                let mut n_paks: c_int = 0;
                let mut p_paks = Sys_ListFiles(
                    common,
                    path,
                    c".pk3".as_ptr(),
                    core::ptr::null_mut(),
                    &mut n_paks,
                    mp_qshared::shared::qfalse,
                );
                Sys_FreeFileList(common, p_paks); // we only use Sys_ListFiles to check wether .pk3 files are present

                // Try on cd path
                if n_paks <= 0 {
                    path = crate::files_common::FS_BuildOSPath4(
                        common,
                        (*common.fs_cdpath).string,
                        name,
                        c"".as_ptr(),
                    );
                    n_paks = 0;
                    p_paks = Sys_ListFiles(
                        common,
                        path,
                        c".pk3".as_ptr(),
                        core::ptr::null_mut(),
                        &mut n_paks,
                        mp_qshared::shared::qfalse,
                    );
                    Sys_FreeFileList(common, p_paks);
                }

                // try on home path
                if n_paks <= 0 {
                    path = crate::files_common::FS_BuildOSPath4(
                        common,
                        (*common.fs_homepath).string,
                        name,
                        c"".as_ptr(),
                    );
                    n_paks = 0;
                    p_paks = Sys_ListFiles(
                        common,
                        path,
                        c".pk3".as_ptr(),
                        core::ptr::null_mut(),
                        &mut n_paks,
                        mp_qshared::shared::qfalse,
                    );
                    Sys_FreeFileList(common, p_paks);
                }

                if n_paks > 0 {
                    let n_len = name_str.len() + 1;
                    // nLen is the length of the mod path
                    // we need to see if there is a description available
                    let desc_path_str = format!("{}/description.txt", name_str);
                    let desc_path_c = std::ffi::CString::new(desc_path_str).unwrap();
                    let mut desc_handle: fileHandle_t = 0;
                    let mut n_desc_len =
                        FS_SV_FOpenFileRead(common, desc_path_c.as_ptr(), &mut desc_handle);
                    let desc_str: String;
                    if n_desc_len > 0 && desc_handle != 0 {
                        let file = FS_FileForHandle(common, desc_handle);
                        let mut buf = [0u8; 49];
                        n_desc_len = libc::fread(buf.as_mut_ptr() as *mut _, 1, 48, file) as c_int;
                        if n_desc_len >= 0 {
                            buf[n_desc_len as usize] = 0;
                        }
                        FS_FCloseFile(common, desc_handle);
                        desc_str = c_str_to_string(buf.as_ptr() as *const c_char);
                    } else {
                        desc_str = name_str.clone();
                    }
                    let n_desc_len = desc_str.len() + 1;

                    if (n_total as usize) + n_len + 1 + n_desc_len + 1 < bufsize as usize {
                        copy_cname(listbuf, name, n_len);
                        let listbuf2 = listbuf.add(n_len);
                        let desc_c = std::ffi::CString::new(desc_str).unwrap();
                        copy_cname(listbuf2, desc_c.as_ptr(), n_desc_len);
                        n_total += (n_len + n_desc_len) as c_int;
                        n_mods += 1;
                    } else {
                        break;
                    }
                }
            }
        }
        Sys_FreeFileList(common, p_files);
    }

    n_mods
}

/// Raven `FS_FOpenFileByMode`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:3064-3116`
pub fn FS_FOpenFileByMode(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    qpath: *const c_char,
    f: *mut fileHandle_t,
    mode: fsMode_t,
) -> c_int {
    let mut sync = mp_qshared::shared::qfalse;

    let r;
    match mode {
        m if m == FS_READ => unsafe {
            r = FS_FOpenFileRead(common, cm, rm, host, qpath, f, mp_qshared::shared::qtrue);
        },
        m if m == FS_WRITE => unsafe {
            *f = FS_FOpenFileWrite(common, qpath);
            r = if *f == 0 { -1 } else { 0 };
        },
        m if m == FS_APPEND_SYNC || m == FS_APPEND => unsafe {
            sync = if m == FS_APPEND_SYNC {
                mp_qshared::shared::qtrue
            } else {
                sync
            };
            *f = FS_FOpenFileAppend(common, qpath);
            r = if *f == 0 { -1 } else { 0 };
        },
        _ => {
            crate::common::com_error(
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
            if common.fsh[*f as usize].zipFile == mp_qshared::shared::qtrue {
                common.fsh[*f as usize].baseOffset =
                    crate::unzip::unztell(common.fsh[*f as usize].handleFiles.file.z) as c_int;
            } else {
                common.fsh[*f as usize].baseOffset =
                    libc::ftell(common.fsh[*f as usize].handleFiles.file.o as *mut libc::FILE)
                        as c_int;
            }
            common.fsh[*f as usize].fileSize = r;
            common.fsh[*f as usize].streamed = mp_qshared::shared::qfalse;

            if mode == FS_READ {
                Sys_BeginStreamedFile(common, *f, 0x4000);
                common.fsh[*f as usize].streamed = mp_qshared::shared::qtrue;
            }
        }
        common.fsh[*f as usize].handleSync = sync;
    }

    r
}

/// Raven `FS_GetFileList`.
///
/// Source: `oracle/codemp/qcommon/files_pc.cpp:1758-1788`
pub fn FS_GetFileList(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    path: *const c_char,
    extension: *const c_char,
    listbuf: *mut c_char,
    bufsize: c_int,
) -> c_int {
    let mut n_total: c_int = 0;

    unsafe {
        *listbuf = 0;

        if c_str_to_string(path) == "$modlist" {
            return FS_GetModList(common, cm, rm, host, listbuf, bufsize);
        }

        let mut n_files: c_int = 0;
        let p_files = FS_ListFiles(common, cm, rm, host, path, extension, &mut n_files);

        let mut i = 0;
        let mut listbuf = listbuf;
        while i < n_files {
            let entry = *p_files.offset(i as isize);
            let n_len = libc::strlen(entry) + 1;
            if (n_total as usize) + n_len + 1 < bufsize as usize {
                copy_cname(listbuf, entry, n_len);
                listbuf = listbuf.add(n_len);
                n_total += n_len as c_int;
            } else {
                n_files = i;
                break;
            }
            i += 1;
        }

        FS_FreeFileList(common, p_files);

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

fn copy_cname(dst: *mut c_char, src: *const c_char, size: usize) {
    unsafe {
        let len = libc::strlen(src).min(size.saturating_sub(1));
        core::ptr::copy_nonoverlapping(src, dst, len);
        *dst.add(len) = 0;
    }
}
