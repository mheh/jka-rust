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

use std::ffi::CStr;
use std::sync::OnceLock;

use native_string::filter::Com_FilterPathBytes;

/// `MAX_FOUND_FILES` — Raven's stack list cap in `Sys_ListFiles`.
/// Source: `oracle/codemp/unix/unix_shared.cpp:103`
const MAX_FOUND_FILES: usize = 0x1000;

// ===========================================================================
// Directory scanning
// ===========================================================================

/// `Sys_Mkdir` (unix): `mkdir(path, 0777)`, ignoring the result.
///
/// Source: `oracle/codemp/unix/unix_shared.cpp:84-87`
pub fn Sys_Mkdir(path: &str) {
    let Ok(path_c) = std::ffi::CString::new(path) else {
        return;
    };
    unsafe {
        libc::mkdir(path_c.as_ptr(), 0o777);
    }
}

// The libc path-call chokepoints for owned strings (not Raven fns — the
// single CString conversions the FS string migration funnels through; an
// interior-NUL path fails as a nonexistent one).

/// libc `fopen` over an owned path.
pub fn sys_fopen(path: &str, mode: &CStr) -> *mut libc::FILE {
    let Ok(path_c) = std::ffi::CString::new(path) else {
        return core::ptr::null_mut();
    };
    unsafe { libc::fopen(path_c.as_ptr(), mode.as_ptr()) }
}

/// libc `rename` over owned paths.
pub fn sys_rename(from: &str, to: &str) -> core::ffi::c_int {
    let (Ok(from_c), Ok(to_c)) = (std::ffi::CString::new(from), std::ffi::CString::new(to)) else {
        return -1;
    };
    unsafe { libc::rename(from_c.as_ptr(), to_c.as_ptr()) }
}

/// libc `remove` over an owned path.
pub fn sys_remove(path: &str) {
    let Ok(path_c) = std::ffi::CString::new(path) else {
        return;
    };
    unsafe {
        libc::remove(path_c.as_ptr());
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
    basedir: &str,
    subdirs: &[u8],
    filter: &str,
    list: &mut Vec<String>,
) {
    if list.len() >= MAX_FOUND_FILES - 1 {
        return;
    }

    // search = subdirs ? "basedir/subdirs" : "basedir"
    let mut search: Vec<u8> = Vec::new();
    search.extend_from_slice(basedir.as_bytes());
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
        if !Com_FilterPathBytes(filter.as_bytes(), &relname, false) {
            continue;
        }
        list.push(String::from_utf8_lossy(&relname).into_owned());
    }

    libc::closedir(fdir);
}

/// `Sys_ListFiles` (unix): directory scan with an extension filter (or a glob
/// `filter`, or dirs-only when `wantsubs` / `extension == "/"`). Raven's
/// libc-`malloc`'d `char**`/`numfiles` return (freed by `Sys_FreeFileList`)
/// becomes an owned `Vec<String>` (string-data migration, DEC-32).
///
/// Source: `oracle/codemp/unix/unix_shared.cpp:159-254`
pub fn Sys_ListFiles(
    directory: &str,
    extension: Option<&str>,
    filter: Option<&str>,
    wantsubs: bool,
) -> Vec<String> {
    unsafe {
        let mut dironly = wantsubs;
        let mut list: Vec<String> = Vec::new();

        if let Some(filter) = filter {
            Sys_ListFilteredFiles(directory, b"", filter, &mut list);
            return list;
        }

        // extension = extension ? extension : ""
        let mut ext: &[u8] = extension.map_or(b"", |e| e.as_bytes());
        // "/" alone means "directories only, no extension match"
        if ext == b"/" {
            ext = b"";
            dironly = true;
        }
        let ext_len = ext.len();

        let directory_c = match std::ffi::CString::new(directory) {
            Ok(c) => c,
            Err(_) => return list,
        };
        let fdir = libc::opendir(directory_c.as_ptr());
        if fdir.is_null() {
            return list;
        }

        loop {
            let d = libc::readdir(fdir);
            if d.is_null() {
                break;
            }
            let dname = dirent_name(d);

            // search = "directory/dname"
            let mut search: Vec<u8> = directory.as_bytes().to_vec();
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
            if (dironly && !is_dir) || (!dironly && is_dir) {
                continue;
            }

            if ext_len != 0 {
                if dname.len() < ext_len
                    || !dname[dname.len() - ext_len..].eq_ignore_ascii_case(ext)
                {
                    continue; // didn't match
                }
            }

            if list.len() == MAX_FOUND_FILES - 1 {
                break;
            }
            list.push(String::from_utf8_lossy(&dname).into_owned());
        }

        libc::closedir(fdir);
        list
    }
}

// ===========================================================================
// Default paths
// ===========================================================================

/// `Sys_Cwd` (unix): the process working directory (Raven's static `cwd`).
///
/// Source: `oracle/codemp/unix/unix_shared.cpp:270-278`
fn sys_cwd() -> &'static str {
    static CWD: OnceLock<String> = OnceLock::new();
    CWD.get_or_init(|| {
        std::env::current_dir()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default()
    })
}

/// `Sys_DefaultCDPath` (unix): the static `cdPath` — empty until the host
/// entrypoint lands (oracle `main()` seeds it from `argv[0]` via `Sys_SetDefaultCDPath`).
///
/// Source: `oracle/codemp/unix/unix_shared.cpp:285-288`
pub fn Sys_DefaultCDPath() -> &'static str {
    ""
}

/// `Sys_DefaultInstallPath` (unix): the static `installPath`, or `Sys_Cwd` when
/// unset (the dedicated path never sets it).
///
/// Source: `oracle/codemp/unix/unix_shared.cpp:295-301`
pub fn Sys_DefaultInstallPath() -> &'static str {
    sys_cwd()
}

/// `Sys_DefaultHomePath` (unix): `$HOME` + platform suffix (macOS
/// `/Library/Application Support/Quake3`, else `/.ja`), `mkdir`'d; empty string
/// when `$HOME` is unset.
///
/// Source: `oracle/codemp/unix/unix_shared.cpp:308-329`
pub fn Sys_DefaultHomePath() -> &'static str {
    static HOME: OnceLock<Option<String>> = OnceLock::new();
    let cell = HOME.get_or_init(|| {
        let home = std::env::var_os("HOME")?;
        let mut path = home.to_string_lossy().into_owned();
        #[cfg(target_os = "macos")]
        path.push_str("/Library/Application Support/Quake3");
        #[cfg(not(target_os = "macos"))]
        path.push_str("/.ja");
        Sys_Mkdir(&path);
        Some(path)
    });
    cell.as_deref().unwrap_or("")
}
