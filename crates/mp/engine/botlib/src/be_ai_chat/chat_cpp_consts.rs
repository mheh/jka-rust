#![allow(non_camel_case_types, non_snake_case)]

/// Raven `ESCAPE_CHAR`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:33`
pub const ESCAPE_CHAR: u8 = 0x01;

// Raven: match-piece types for a parsed chat match template.

/// Raven `MT_VARIABLE` — variable match piece.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:41`
pub const MT_VARIABLE: i32 = 1;

/// Raven `MT_STRING` — string match piece.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:42`
pub const MT_STRING: i32 = 2;

// Raven: reply-chat key flags.

/// Raven `RCKFL_AND` — key must be present.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:44`
pub const RCKFL_AND: i32 = 1;

/// Raven `RCKFL_NOT` — key must be absent.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:45`
pub const RCKFL_NOT: i32 = 2;

/// Raven `RCKFL_NAME` — name of bot must be present.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:46`
pub const RCKFL_NAME: i32 = 4;

/// Raven `RCKFL_STRING` — key is a string.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:47`
pub const RCKFL_STRING: i32 = 8;

/// Raven `RCKFL_VARIABLES` — key is a match template.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:48`
pub const RCKFL_VARIABLES: i32 = 16;

/// Raven `RCKFL_BOTNAMES` — key is a series of botnames.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:49`
pub const RCKFL_BOTNAMES: i32 = 32;

/// Raven `RCKFL_GENDERFEMALE` — bot must be female.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:50`
pub const RCKFL_GENDERFEMALE: i32 = 64;

/// Raven `RCKFL_GENDERMALE` — bot must be male.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:51`
pub const RCKFL_GENDERMALE: i32 = 128;

/// Raven `RCKFL_GENDERLESS` — bot must be genderless.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:52`
pub const RCKFL_GENDERLESS: i32 = 256;

/// Raven `CHATMESSAGE_RECENTTIME`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:54`
pub const CHATMESSAGE_RECENTTIME: i32 = 20;
