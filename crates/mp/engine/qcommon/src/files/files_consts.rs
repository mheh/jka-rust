#![allow(non_camel_case_types, non_snake_case)]

/// Raven `BASEGAME` — default base-game directory name.
/// Source: `oracle/codemp/qcommon/files.cpp:188`
pub const BASEGAME: &str = "base";

/// Raven `DEMOGAME` — demo-release game directory name.
/// Source: `oracle/codemp/qcommon/files.cpp:189`
pub const DEMOGAME: &str = "demo";

/// Raven `DEMO_PAK_CHECKSUM` — checksum of the demo `.pk3`, updated whenever a
/// new demo pak is built.
/// Source: `oracle/codemp/qcommon/files.cpp:193`
pub const DEMO_PAK_CHECKSUM: u32 = 437558517;

/// Raven `MAX_ZPATH` — max path length inside a `.pk3`/zip.
/// Source: `oracle/codemp/qcommon/files.cpp:203`
pub const MAX_ZPATH: usize = 256;

/// Raven `MAX_SEARCH_PATHS` — max entries on the filesystem search-path chain.
/// Source: `oracle/codemp/qcommon/files.cpp:204`
pub const MAX_SEARCH_PATHS: usize = 4096;

/// Raven `MAX_FILEHASH_SIZE` — bucket count for the pak filename hash table.
/// Source: `oracle/codemp/qcommon/files.cpp:205`
pub const MAX_FILEHASH_SIZE: usize = 1024;

/// Raven `MAX_FOUND_FILES` — max results returned by a `FS_ListFiles`-style
/// directory scan.
/// Source: `oracle/codemp/qcommon/files.cpp:1982`
pub const MAX_FOUND_FILES: usize = 0x1000;

/// Raven `MAX_PAKFILES` — max `.pk3` files scanned per game directory.
/// Source: `oracle/codemp/qcommon/files.cpp:2661`
pub const MAX_PAKFILES: usize = 1024;
