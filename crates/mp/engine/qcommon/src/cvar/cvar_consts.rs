#![allow(non_camel_case_types, non_snake_case)]

/// Raven `MAX_CVARS` — capacity of the engine's static `cvar_indexes` table.
/// Source: oracle/codemp/qcommon/cvar.cpp:10
pub const MAX_CVARS: usize = 1224;

/// Raven `FILE_HASH_SIZE` — bucket count of the cvar-name `hashTable`.
/// Source: oracle/codemp/qcommon/cvar.cpp:14
pub const FILE_HASH_SIZE: usize = 256;
