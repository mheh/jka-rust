use core::ffi::c_int;

/// Raven `UNZ_OK` — success return code from vendored minizip `unz*` calls.
/// Source: `oracle/codemp/qcommon/unzip.h:110`
pub const UNZ_OK: c_int = 0;

/// Raven `UNZ_END_OF_LIST_OF_FILE` — no more files in the zip's central directory.
/// Source: `oracle/codemp/qcommon/unzip.h:111`
pub const UNZ_END_OF_LIST_OF_FILE: c_int = -100;

/// Raven `UNZ_ERRNO` — aliases zlib's `Z_DATA_ERROR`.
/// Source: `oracle/codemp/qcommon/unzip.h:112`
pub const UNZ_ERRNO: c_int = -3;

/// Raven `UNZ_EOF` — end of file reached.
/// Source: `oracle/codemp/qcommon/unzip.h:113`
pub const UNZ_EOF: c_int = 0;

/// Raven `UNZ_PARAMERROR` — invalid argument to an `unz*` call.
/// Source: `oracle/codemp/qcommon/unzip.h:114`
pub const UNZ_PARAMERROR: c_int = -102;

/// Raven `UNZ_BADZIPFILE` — malformed zip archive.
/// Source: `oracle/codemp/qcommon/unzip.h:115`
pub const UNZ_BADZIPFILE: c_int = -103;

/// Raven `UNZ_INTERNALERROR` — internal minizip inconsistency.
/// Source: `oracle/codemp/qcommon/unzip.h:116`
pub const UNZ_INTERNALERROR: c_int = -104;

/// Raven `UNZ_CRCERROR` — CRC-32 mismatch after decompression.
/// Source: `oracle/codemp/qcommon/unzip.h:117`
pub const UNZ_CRCERROR: c_int = -105;

/// Raven `UNZ_CASESENSITIVE` — filename comparison mode: case-sensitive.
/// Source: `oracle/codemp/qcommon/unzip.h:119`
pub const UNZ_CASESENSITIVE: c_int = 1;

/// Raven `UNZ_NOTCASESENSITIVE` — filename comparison mode: case-insensitive.
/// Source: `oracle/codemp/qcommon/unzip.h:120`
pub const UNZ_NOTCASESENSITIVE: c_int = 2;

/// Raven `UNZ_OSDEFAULTCASE` — filename comparison mode: OS default (case-sensitive
/// on Unix, case-insensitive on Windows).
/// Source: `oracle/codemp/qcommon/unzip.h:121`
pub const UNZ_OSDEFAULTCASE: c_int = 0;

/// Raven `UNZ_BUFSIZE` — read buffer size used while scanning a zip's central directory.
/// Source: `oracle/codemp/qcommon/unzip.cpp:235`
pub const UNZ_BUFSIZE: c_int = 65536;

/// Raven `UNZ_MAXFILENAMEINZIP` — max filename length considered while scanning.
/// Source: `oracle/codemp/qcommon/unzip.cpp:239`
pub const UNZ_MAXFILENAMEINZIP: c_int = 256;

/// Raven `SIZECENTRALDIRITEM` — byte size of one central-directory record header.
/// Source: `oracle/codemp/qcommon/unzip.cpp:249`
pub const SIZECENTRALDIRITEM: c_int = 0x2e;

/// Raven `SIZEZIPLOCALHEADER` — byte size of a local file header.
/// Source: `oracle/codemp/qcommon/unzip.cpp:250`
pub const SIZEZIPLOCALHEADER: c_int = 0x1e;

// Raven's `CASESENSITIVITYDEFAULTVALUE` resolves via `#ifdef CASESENSITIVITYDEFAULT_NO`
// (defined whenever `unix` is not defined, which is the engine's actual build config),
// selecting the `2` (case-insensitive) branch.
/// Raven `CASESENSITIVITYDEFAULTVALUE` — default `unzStringFileNameCompare` mode when
/// the caller passes 0.
/// Source: `oracle/codemp/qcommon/unzip.cpp:228-303`
pub const CASESENSITIVITYDEFAULTVALUE: c_int = 2;

/// Raven `BUFREADCOMMENT` — read-window size used while locating the end-of-central-directory
/// record.
/// Source: `oracle/codemp/qcommon/unzip.cpp:329`
pub const BUFREADCOMMENT: c_int = 0x400;
