//! `Sys_*` OS surface transcribed from Raven's unix shared layer.
//!
//! This is the deliberately-native platform layer (campaign-settled): rather
//! than porting `win32/`, the Rust host reimplements the platform surface with
//! libc, mirroring the observable behavior of Raven's unix tree. Directory
//! scanning, `mkdir`, and the default-path resolvers all match
//! `oracle/codemp/unix/unix_shared.cpp`.
//!
//! The `char **` list `Sys_ListFiles` returns is a libc-`malloc`'d array of
//! libc-`strdup`'d strings (Raven uses `Z_Malloc`/`CopyString`; native_platform
//! cannot take an uphill edge to qcommon's zone allocator, so the pair uses
//! libc `malloc`/`free` self-consistently). `Sys_FreeFileList` frees exactly
//! that shape.
//!
//! Source: `oracle/codemp/unix/unix_shared.cpp:84-329`

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use std::ffi::CStr;
use std::sync::OnceLock;

use native_string::filter::Com_FilterPathBytes;
use native_types::{qboolean, qfalse, qtrue};

/// `MAX_FOUND_FILES` — Raven's stack list cap in `Sys_ListFiles`.
/// Source: `oracle/codemp/unix/unix_shared.cpp:103`
const MAX_FOUND_FILES: usize = 0x1000;

// ===========================================================================
// Directory scanning
// ===========================================================================

/// `Sys_Mkdir` (unix): `mkdir(path, 0777)`, ignoring the result.
///
/// Source: `oracle/codemp/unix/unix_shared.cpp:84-87`
pub fn Sys_Mkdir(path: *const c_char) {
    unsafe {
        libc::mkdir(path, 0o777);
    }
}

/// Read a `readdir` entry's `d_name` as a lossless byte string.
unsafe fn dirent_name(d: *const libc::dirent) -> Vec<u8> {
    CStr::from_ptr((*d).d_name.as_ptr()).to_bytes().to_vec()
}

/// `Sys_ListFilteredFiles` (unix): recursive glob scan collecting relative
/// paths that match `filter` via `Com_FilterPath`.
///
/// Source: `oracle/codemp/unix/unix_shared.cpp:106-155`
unsafe fn Sys_ListFilteredFiles(
    basedir: &CStr,
    subdirs: &[u8],
    filter: *mut c_char,
    list: &mut Vec<*mut c_char>,
) {
    if list.len() >= MAX_FOUND_FILES - 1 {
        return;
    }

    // search = subdirs ? "basedir/subdirs" : "basedir"
    let base_bytes = basedir.to_bytes();
    let mut search: Vec<u8> = Vec::new();
    search.extend_from_slice(base_bytes);
    if !subdirs.is_empty() {
        search.push(b'/');
        search.extend_from_slice(subdirs);
    }
    let search_c = match std::ffi::CString::new(search.clone()) {
        Ok(c) => c,
        Err(_) => return,
    };

    let fdir = libc::opendir(search_c.as_ptr());
    if fdir.is_null() {
        return;
    }

    loop {
        let d = libc::readdir(fdir);
        if d.is_null() {
            break;
        }
        let dname = dirent_name(d);

        // filename = "search/dname" for the stat
        let mut statpath: Vec<u8> = search.clone();
        statpath.push(b'/');
        statpath.extend_from_slice(&dname);
        let statpath_c = match std::ffi::CString::new(statpath) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let mut st: libc::stat = core::mem::zeroed();
        if libc::stat(statpath_c.as_ptr(), &mut st) == -1 {
            continue;
        }

        if st.st_mode & libc::S_IFDIR != 0 {
            if dname != b"." && dname != b".." {
                // newsubdirs = subdirs ? "subdirs/dname" : "dname"
                let mut newsubdirs: Vec<u8> = Vec::new();
                if !subdirs.is_empty() {
                    newsubdirs.extend_from_slice(subdirs);
                    newsubdirs.push(b'/');
                }
                newsubdirs.extend_from_slice(&dname);
                Sys_ListFilteredFiles(basedir, &newsubdirs, filter, list);
            }
        }
        if list.len() >= MAX_FOUND_FILES - 1 {
            break;
        }
        // filename = "subdirs/dname" for the filter match
        let mut relname: Vec<u8> = Vec::new();
        relname.extend_from_slice(subdirs);
        relname.push(b'/');
        relname.extend_from_slice(&dname);
        let relname_c = match std::ffi::CString::new(relname) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if !Com_FilterPathBytes(
            CStr::from_ptr(filter).to_bytes(),
            relname_c.to_bytes(),
            false,
        ) {
            continue;
        }
        list.push(libc::strdup(relname_c.as_ptr()));
    }

    libc::closedir(fdir);
}

/// `Sys_ListFiles` (unix): directory scan with an extension filter (or a glob
/// `filter`, or dirs-only when `wantsubs` / `extension == "/"`). Returns a
/// libc-`malloc`'d `char**` list (freed by `Sys_FreeFileList`) and writes the
/// count through `numfiles`.
///
/// Source: `oracle/codemp/unix/unix_shared.cpp:159-254`
pub fn Sys_ListFiles(
    directory: *const c_char,
    extension: *const c_char,
    filter: *mut c_char,
    numfiles: *mut c_int,
    wantsubs: qboolean,
) -> *mut *mut c_char {
    unsafe {
        let mut dironly = wantsubs;
        let mut list: Vec<*mut c_char> = Vec::new();

        if !filter.is_null() {
            let dir_c = CStr::from_ptr(directory);
            Sys_ListFilteredFiles(dir_c, b"", filter, &mut list);
            *numfiles = list.len() as c_int;
            return finalize_list(list, numfiles);
        }

        // extension = extension ? extension : ""
        let mut ext: Vec<u8> = if extension.is_null() {
            Vec::new()
        } else {
            CStr::from_ptr(extension).to_bytes().to_vec()
        };
        // "/" alone means "directories only, no extension match"
        if ext == b"/" {
            ext.clear();
            dironly = qtrue;
        }
        let ext_len = ext.len();

        let fdir = libc::opendir(directory);
        if fdir.is_null() {
            *numfiles = 0;
            return core::ptr::null_mut();
        }

        let dir_bytes = CStr::from_ptr(directory).to_bytes().to_vec();
        loop {
            let d = libc::readdir(fdir);
            if d.is_null() {
                break;
            }
            let dname = dirent_name(d);

            // search = "directory/dname"
            let mut search: Vec<u8> = dir_bytes.clone();
            search.push(b'/');
            search.extend_from_slice(&dname);
            let search_c = match std::ffi::CString::new(search) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let mut st: libc::stat = core::mem::zeroed();
            if libc::stat(search_c.as_ptr(), &mut st) == -1 {
                continue;
            }
            let is_dir = st.st_mode & libc::S_IFDIR != 0;
            if (dironly != qfalse && !is_dir) || (dironly == qfalse && is_dir) {
                continue;
            }

            if ext_len != 0 {
                if dname.len() < ext_len
                    || !dname[dname.len() - ext_len..].eq_ignore_ascii_case(&ext)
                {
                    continue; // didn't match
                }
            }

            if list.len() == MAX_FOUND_FILES - 1 {
                break;
            }
            let dname_c = match std::ffi::CString::new(dname) {
                Ok(c) => c,
                Err(_) => continue,
            };
            list.push(libc::strdup(dname_c.as_ptr()));
        }

        libc::closedir(fdir);
        *numfiles = list.len() as c_int;
        finalize_list(list, numfiles)
    }
}

/// Copy the accumulated entries into a libc-`malloc`'d, NULL-terminated
/// `char**` (Raven's `listCopy` tail). Returns NULL for an empty list.
///
/// Source: `oracle/codemp/unix/unix_shared.cpp:243-253`
unsafe fn finalize_list(list: Vec<*mut c_char>, numfiles: *mut c_int) -> *mut *mut c_char {
    let n = list.len();
    if n == 0 {
        *numfiles = 0;
        return core::ptr::null_mut();
    }
    let bytes = (n + 1) * core::mem::size_of::<*mut c_char>();
    let arr = libc::malloc(bytes) as *mut *mut c_char;
    for (i, &p) in list.iter().enumerate() {
        *arr.add(i) = p;
    }
    *arr.add(n) = core::ptr::null_mut();
    arr
}

/// `Sys_FreeFileList` (unix): free each string, then the array (libc `free`,
/// pairing with `Sys_ListFiles`'s libc `malloc`/`strdup`).
///
/// Source: `oracle/codemp/unix/unix_shared.cpp:256-268`
pub fn Sys_FreeFileList(list: *mut *mut c_char) {
    if list.is_null() {
        return;
    }
    unsafe {
        let mut i = 0usize;
        while !(*list.add(i)).is_null() {
            libc::free(*list.add(i) as *mut c_void);
            i += 1;
        }
        libc::free(list as *mut c_void);
    }
}

// ===========================================================================
// Default paths
// ===========================================================================

/// `Sys_Cwd` (unix): the process working directory (Raven's static `cwd`).
///
/// Source: `oracle/codemp/unix/unix_shared.cpp:270-278`
fn sys_cwd() -> &'static CStr {
    static CWD: OnceLock<std::ffi::CString> = OnceLock::new();
    CWD.get_or_init(|| {
        let dir = std::env::current_dir()
            .ok()
            .and_then(|p| std::ffi::CString::new(p.into_os_string().into_encoded_bytes()).ok())
            .unwrap_or_default();
        dir
    })
}

/// `Sys_DefaultCDPath` (unix): the static `cdPath` — empty until the host
/// entrypoint lands (oracle `main()` seeds it from `argv[0]` via `Sys_SetDefaultCDPath`).
///
/// Source: `oracle/codemp/unix/unix_shared.cpp:285-288`
pub fn Sys_DefaultCDPath() -> *const c_char {
    c"".as_ptr()
}

/// `Sys_DefaultInstallPath` (unix): the static `installPath`, or `Sys_Cwd` when
/// unset (the dedicated path never sets it).
///
/// Source: `oracle/codemp/unix/unix_shared.cpp:295-301`
pub fn Sys_DefaultInstallPath() -> *const c_char {
    sys_cwd().as_ptr()
}

/// `Sys_DefaultHomePath` (unix): `$HOME` + platform suffix (macOS
/// `/Library/Application Support/Quake3`, else `/.ja`), `mkdir`'d; empty string
/// when `$HOME` is unset.
///
/// Source: `oracle/codemp/unix/unix_shared.cpp:308-329`
pub fn Sys_DefaultHomePath() -> *const c_char {
    static HOME: OnceLock<Option<std::ffi::CString>> = OnceLock::new();
    let cell = HOME.get_or_init(|| {
        let home = std::env::var_os("HOME")?;
        let mut bytes = home.into_encoded_bytes();
        #[cfg(target_os = "macos")]
        bytes.extend_from_slice(b"/Library/Application Support/Quake3");
        #[cfg(not(target_os = "macos"))]
        bytes.extend_from_slice(b"/.ja");
        let path = std::ffi::CString::new(bytes).ok()?;
        unsafe {
            libc::mkdir(path.as_ptr(), 0o777);
        }
        Some(path)
    });
    match cell {
        Some(p) => p.as_ptr(),
        None => c"".as_ptr(),
    }
}
