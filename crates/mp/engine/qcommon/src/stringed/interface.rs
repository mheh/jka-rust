//! StringEd engine-side interface TU — the load/list services StringEd calls.
//!
//! Idiomatic reimplementation of `oracle/codemp/qcommon/stringed_interface.cpp`
//! (design frozen in `docs/subsystems/stringed.md`). Only the in-engine
//! `#ifndef _STRINGED` branches port; the `_STRINGED` editor-tool branches
//! (raw `fopen`/`filesize`/`malloc` in `SE_LoadFileData`, `BuildFileList` +
//! extern `strResult` in `SE_BuildFileList`) are the standalone StringEd-editor
//! build and are §20 zero-caller drops (SE-V1) — `_STRINGED` is undefined in the
//! WinDed/engine TU.
//!
//! File IO and directory listing are reached through
//! [`EngineHost`](mp_host_interface::EngineHost) (SE-D3): `fs_read_file` /
//! `fs_free_file` and the VFS/pk3-aware `fs_list_files` (RULING 55). `giFilesFound`
//! and the `';'`-delimited accumulator become returned values (RULING 3
//! three-kind; SE-D1(3)).

use mp_host_interface::EngineHost;

use super::SE_INGAME_FILE_EXTENSION;

/// Raven `SE_LoadFileData` — read a file's bytes into memory for the parser
/// (in-engine `FS_ReadFile` path only, SE-V1).
///
/// Raven returns `NULL` for a failed/empty load and writes the length through an
/// optional out-param; the idiomatic form returns `Some(bytes)` (length is
/// `Vec::len`) or `None`. Faithful to Raven's `iLen > 0` gate — a zero-length
/// read yields `None`.
///
/// Source: `oracle/codemp/qcommon/stringed_interface.cpp:41-104`
pub fn se_load_file_data(file_name: &str, host: &mut impl EngineHost) -> Option<Vec<u8>> {
    match host.fs_read_file(file_name) {
        Some(data) if !data.is_empty() => Some(data),
        _ => None,
    }
}

/// Raven `SE_FreeFileDataAfterLoad` — release the loaded file buffer after the
/// parse (in-engine `FS_FreeFile` path only, SE-V1).
///
/// Raven's `if (psLoadedFile) FS_FreeFile(...)` collapses into the by-value
/// `Vec` handed to `fs_free_file` (state-table `FS_FreeFile` row).
///
/// Source: `oracle/codemp/qcommon/stringed_interface.cpp:109-122`
pub fn se_free_file_data_after_load(data: Vec<u8>, host: &mut impl EngineHost) {
    host.fs_free_file(data);
}

/// Raven `SE_R_ListFiles` — recursively scan `dir` for `ext` files, appending
/// each `"dir/file;"` to a `';'`-delimited accumulator (in-engine only).
///
/// Quake's file-list code has no recursion flag, so Raven walks subdirectories
/// itself: `FS_ListFiles(dir, "/", …)` lists subdirs (the `ext == "/"`
/// convention, served by `fs_list_files`), each recursed into; then
/// `FS_ListFiles(dir, ext, …)` lists the matching files. Raven's mutating
/// `string &strResults` out-param + global `giFilesFound` become the returned
/// `(accumulator, count)` (SE-D1(3), RULING 3). Keeps Raven's `(extension, dir)`
/// arg order.
///
/// Source: `oracle/codemp/qcommon/stringed_interface.cpp:132-184`
pub fn se_r_list_files(ext: &str, dir: &str, host: &mut impl EngineHost) -> (String, i32) {
    let mut results = String::new();
    let mut found = 0;

    // Recurse subdirectories (skip blanks plus ".", ".." etc).
    let dir_files = host.fs_list_files(dir, "/", false);
    for d in &dir_files {
        if !d.is_empty() && !d.starts_with('.') {
            let mut sub_dir = format!("{dir}/{d}");
            // The quake filesystem now returns an extra trailing slash; strip it.
            if sub_dir.ends_with('/') {
                sub_dir.pop();
            }
            let (sub_results, sub_found) = se_r_list_files(ext, &sub_dir, host);
            results.push_str(&sub_results);
            found += sub_found;
        }
    }

    // List the matching files in this directory.
    let sys_files = host.fs_list_files(dir, ext, false);
    for f in &sys_files {
        results.push_str(&format!("{dir}/{f}"));
        results.push(';');
        found += 1;
    }

    (results, found)
}

/// Raven `SE_BuildFileList` — scan `dir` for `.str` files (in-engine only, SE-V1).
///
/// Resets Raven's `giFilesFound` + accumulator and delegates to
/// [`se_r_list_files`]; returns the `';'`-delimited results and the file count.
///
/// Source: `oracle/codemp/qcommon/stringed_interface.cpp:192-212`
pub fn se_build_file_list(dir: &str, host: &mut impl EngineHost) -> (String, i32) {
    se_r_list_files(SE_INGAME_FILE_EXTENSION, dir, host)
}
