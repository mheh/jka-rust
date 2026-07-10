#![allow(non_upper_case_globals)]

use core::ffi::c_int;

/// Raven `MAX_TOKEN` — maximum token length.
/// Source: `oracle/codemp/botlib/l_script.h:23`
pub const MAX_TOKEN: usize = 1024;

/// Raven `BINARYNUMBERS` — unconditionally-defined feature guard allowing
/// `0b...`/`0B...` binary number literals. Ported as `bool` since Raven never
/// gives it a value, only tests it with `#ifdef`; see [`TT_BINARY`]'s doc
/// comment for this same guard's effect on the token-type table below.
///
/// Source: `oracle/codemp/botlib/l_script.h:16`
pub const BINARYNUMBERS: bool = true;

/// Raven `NUMBERVALUE` — unconditionally-defined feature guard enabling
/// `token.intvalue`/`token.floatvalue`. Ported as `bool` for the same reason
/// as [`BINARYNUMBERS`].
///
/// Source: `oracle/codemp/botlib/l_script.h:18`
pub const NUMBERVALUE: bool = true;

// Script flags — `SCFL_*` (anonymous `#define` family, not a named C enum).
/// Raven `SCFL_NOERRORS`.
/// Source: `oracle/codemp/botlib/l_script.h:31`
pub const SCFL_NOERRORS: c_int = 0x0001;
/// Raven `SCFL_NOWARNINGS`.
/// Source: `oracle/codemp/botlib/l_script.h:32`
pub const SCFL_NOWARNINGS: c_int = 0x0002;
/// Raven `SCFL_NOSTRINGWHITESPACES`.
/// Source: `oracle/codemp/botlib/l_script.h:33`
pub const SCFL_NOSTRINGWHITESPACES: c_int = 0x0004;
/// Raven `SCFL_NOSTRINGESCAPECHARS`.
/// Source: `oracle/codemp/botlib/l_script.h:34`
pub const SCFL_NOSTRINGESCAPECHARS: c_int = 0x0008;
/// Raven `SCFL_PRIMITIVE`.
/// Source: `oracle/codemp/botlib/l_script.h:35`
pub const SCFL_PRIMITIVE: c_int = 0x0010;
/// Raven `SCFL_NOBINARYNUMBERS`.
/// Source: `oracle/codemp/botlib/l_script.h:36`
pub const SCFL_NOBINARYNUMBERS: c_int = 0x0020;
/// Raven `SCFL_NONUMBERVALUES`.
/// Source: `oracle/codemp/botlib/l_script.h:37`
pub const SCFL_NONUMBERVALUES: c_int = 0x0040;

// Token types — `TT_*` (anonymous `#define` family).
/// Raven `TT_STRING` — string.
/// Source: `oracle/codemp/botlib/l_script.h:40`
pub const TT_STRING: c_int = 1;
/// Raven `TT_LITERAL` — literal.
/// Source: `oracle/codemp/botlib/l_script.h:41`
pub const TT_LITERAL: c_int = 2;
/// Raven `TT_NUMBER` — number.
/// Source: `oracle/codemp/botlib/l_script.h:42`
pub const TT_NUMBER: c_int = 3;
/// Raven `TT_NAME` — name.
/// Source: `oracle/codemp/botlib/l_script.h:43`
pub const TT_NAME: c_int = 4;
/// Raven `TT_PUNCTUATION` — punctuation.
/// Source: `oracle/codemp/botlib/l_script.h:44`
pub const TT_PUNCTUATION: c_int = 5;

// Number sub type — `TT_*` bit flags (anonymous `#define` family; `TT_BINARY`
// is only defined when `BINARYNUMBERS` is set, which `l_script.h:16` always
// defines unconditionally in this tree).
/// Raven `TT_DECIMAL` — decimal number.
/// Source: `oracle/codemp/botlib/l_script.h:54`
pub const TT_DECIMAL: c_int = 0x0008;
/// Raven `TT_HEX` — hexadecimal number.
/// Source: `oracle/codemp/botlib/l_script.h:55`
pub const TT_HEX: c_int = 0x0100;
/// Raven `TT_OCTAL` — octal number.
/// Source: `oracle/codemp/botlib/l_script.h:56`
pub const TT_OCTAL: c_int = 0x0200;
/// Raven `TT_BINARY` — binary number.
/// Source: `oracle/codemp/botlib/l_script.h:58`
pub const TT_BINARY: c_int = 0x0400;
/// Raven `TT_FLOAT` — floating point number.
/// Source: `oracle/codemp/botlib/l_script.h:60`
pub const TT_FLOAT: c_int = 0x0800;
/// Raven `TT_INTEGER` — integer number.
/// Source: `oracle/codemp/botlib/l_script.h:61`
pub const TT_INTEGER: c_int = 0x1000;
/// Raven `TT_LONG` — long number.
/// Source: `oracle/codemp/botlib/l_script.h:62`
pub const TT_LONG: c_int = 0x2000;
/// Raven `TT_UNSIGNED` — unsigned number.
/// Source: `oracle/codemp/botlib/l_script.h:63`
pub const TT_UNSIGNED: c_int = 0x4000;

// Punctuation sub type — `P_*` (anonymous `#define` family).
/// Raven `P_RSHIFT_ASSIGN`.
/// Source: `oracle/codemp/botlib/l_script.h:66`
pub const P_RSHIFT_ASSIGN: c_int = 1;
/// Raven `P_LSHIFT_ASSIGN`.
/// Source: `oracle/codemp/botlib/l_script.h:67`
pub const P_LSHIFT_ASSIGN: c_int = 2;
/// Raven `P_PARMS`.
/// Source: `oracle/codemp/botlib/l_script.h:68`
pub const P_PARMS: c_int = 3;
/// Raven `P_PRECOMPMERGE`.
/// Source: `oracle/codemp/botlib/l_script.h:69`
pub const P_PRECOMPMERGE: c_int = 4;

/// Raven `P_LOGIC_AND`.
/// Source: `oracle/codemp/botlib/l_script.h:71`
pub const P_LOGIC_AND: c_int = 5;
/// Raven `P_LOGIC_OR`.
/// Source: `oracle/codemp/botlib/l_script.h:72`
pub const P_LOGIC_OR: c_int = 6;
/// Raven `P_LOGIC_GEQ`.
/// Source: `oracle/codemp/botlib/l_script.h:73`
pub const P_LOGIC_GEQ: c_int = 7;
/// Raven `P_LOGIC_LEQ`.
/// Source: `oracle/codemp/botlib/l_script.h:74`
pub const P_LOGIC_LEQ: c_int = 8;
/// Raven `P_LOGIC_EQ`.
/// Source: `oracle/codemp/botlib/l_script.h:75`
pub const P_LOGIC_EQ: c_int = 9;
/// Raven `P_LOGIC_UNEQ`.
/// Source: `oracle/codemp/botlib/l_script.h:76`
pub const P_LOGIC_UNEQ: c_int = 10;

/// Raven `P_MUL_ASSIGN`.
/// Source: `oracle/codemp/botlib/l_script.h:78`
pub const P_MUL_ASSIGN: c_int = 11;
/// Raven `P_DIV_ASSIGN`.
/// Source: `oracle/codemp/botlib/l_script.h:79`
pub const P_DIV_ASSIGN: c_int = 12;
/// Raven `P_MOD_ASSIGN`.
/// Source: `oracle/codemp/botlib/l_script.h:80`
pub const P_MOD_ASSIGN: c_int = 13;
/// Raven `P_ADD_ASSIGN`.
/// Source: `oracle/codemp/botlib/l_script.h:81`
pub const P_ADD_ASSIGN: c_int = 14;
/// Raven `P_SUB_ASSIGN`.
/// Source: `oracle/codemp/botlib/l_script.h:82`
pub const P_SUB_ASSIGN: c_int = 15;
/// Raven `P_INC`.
/// Source: `oracle/codemp/botlib/l_script.h:83`
pub const P_INC: c_int = 16;
/// Raven `P_DEC`.
/// Source: `oracle/codemp/botlib/l_script.h:84`
pub const P_DEC: c_int = 17;

/// Raven `P_BIN_AND_ASSIGN`.
/// Source: `oracle/codemp/botlib/l_script.h:86`
pub const P_BIN_AND_ASSIGN: c_int = 18;
/// Raven `P_BIN_OR_ASSIGN`.
/// Source: `oracle/codemp/botlib/l_script.h:87`
pub const P_BIN_OR_ASSIGN: c_int = 19;
/// Raven `P_BIN_XOR_ASSIGN`.
/// Source: `oracle/codemp/botlib/l_script.h:88`
pub const P_BIN_XOR_ASSIGN: c_int = 20;
/// Raven `P_RSHIFT`.
/// Source: `oracle/codemp/botlib/l_script.h:89`
pub const P_RSHIFT: c_int = 21;
/// Raven `P_LSHIFT`.
/// Source: `oracle/codemp/botlib/l_script.h:90`
pub const P_LSHIFT: c_int = 22;

/// Raven `P_POINTERREF`.
/// Source: `oracle/codemp/botlib/l_script.h:92`
pub const P_POINTERREF: c_int = 23;
/// Raven `P_CPP1`.
/// Source: `oracle/codemp/botlib/l_script.h:93`
pub const P_CPP1: c_int = 24;
/// Raven `P_CPP2`.
/// Source: `oracle/codemp/botlib/l_script.h:94`
pub const P_CPP2: c_int = 25;
/// Raven `P_MUL`.
/// Source: `oracle/codemp/botlib/l_script.h:95`
pub const P_MUL: c_int = 26;
/// Raven `P_DIV`.
/// Source: `oracle/codemp/botlib/l_script.h:96`
pub const P_DIV: c_int = 27;
/// Raven `P_MOD`.
/// Source: `oracle/codemp/botlib/l_script.h:97`
pub const P_MOD: c_int = 28;
/// Raven `P_ADD`.
/// Source: `oracle/codemp/botlib/l_script.h:98`
pub const P_ADD: c_int = 29;
/// Raven `P_SUB`.
/// Source: `oracle/codemp/botlib/l_script.h:99`
pub const P_SUB: c_int = 30;
/// Raven `P_ASSIGN`.
/// Source: `oracle/codemp/botlib/l_script.h:100`
pub const P_ASSIGN: c_int = 31;

/// Raven `P_BIN_AND`.
/// Source: `oracle/codemp/botlib/l_script.h:102`
pub const P_BIN_AND: c_int = 32;
/// Raven `P_BIN_OR`.
/// Source: `oracle/codemp/botlib/l_script.h:103`
pub const P_BIN_OR: c_int = 33;
/// Raven `P_BIN_XOR`.
/// Source: `oracle/codemp/botlib/l_script.h:104`
pub const P_BIN_XOR: c_int = 34;
/// Raven `P_BIN_NOT`.
/// Source: `oracle/codemp/botlib/l_script.h:105`
pub const P_BIN_NOT: c_int = 35;

/// Raven `P_LOGIC_NOT`.
/// Source: `oracle/codemp/botlib/l_script.h:107`
pub const P_LOGIC_NOT: c_int = 36;
/// Raven `P_LOGIC_GREATER`.
/// Source: `oracle/codemp/botlib/l_script.h:108`
pub const P_LOGIC_GREATER: c_int = 37;
/// Raven `P_LOGIC_LESS`.
/// Source: `oracle/codemp/botlib/l_script.h:109`
pub const P_LOGIC_LESS: c_int = 38;

/// Raven `P_REF`.
/// Source: `oracle/codemp/botlib/l_script.h:111`
pub const P_REF: c_int = 39;
/// Raven `P_COMMA`.
/// Source: `oracle/codemp/botlib/l_script.h:112`
pub const P_COMMA: c_int = 40;
/// Raven `P_SEMICOLON`.
/// Source: `oracle/codemp/botlib/l_script.h:113`
pub const P_SEMICOLON: c_int = 41;
/// Raven `P_COLON`.
/// Source: `oracle/codemp/botlib/l_script.h:114`
pub const P_COLON: c_int = 42;
/// Raven `P_QUESTIONMARK`.
/// Source: `oracle/codemp/botlib/l_script.h:115`
pub const P_QUESTIONMARK: c_int = 43;

/// Raven `P_PARENTHESESOPEN`.
/// Source: `oracle/codemp/botlib/l_script.h:117`
pub const P_PARENTHESESOPEN: c_int = 44;
/// Raven `P_PARENTHESESCLOSE`.
/// Source: `oracle/codemp/botlib/l_script.h:118`
pub const P_PARENTHESESCLOSE: c_int = 45;
/// Raven `P_BRACEOPEN`.
/// Source: `oracle/codemp/botlib/l_script.h:119`
pub const P_BRACEOPEN: c_int = 46;
/// Raven `P_BRACECLOSE`.
/// Source: `oracle/codemp/botlib/l_script.h:120`
pub const P_BRACECLOSE: c_int = 47;
/// Raven `P_SQBRACKETOPEN`.
/// Source: `oracle/codemp/botlib/l_script.h:121`
pub const P_SQBRACKETOPEN: c_int = 48;
/// Raven `P_SQBRACKETCLOSE`.
/// Source: `oracle/codemp/botlib/l_script.h:122`
pub const P_SQBRACKETCLOSE: c_int = 49;
/// Raven `P_BACKSLASH`.
/// Source: `oracle/codemp/botlib/l_script.h:123`
pub const P_BACKSLASH: c_int = 50;

/// Raven `P_PRECOMP`.
/// Source: `oracle/codemp/botlib/l_script.h:125`
pub const P_PRECOMP: c_int = 51;
/// Raven `P_DOLLAR`.
/// Source: `oracle/codemp/botlib/l_script.h:126`
pub const P_DOLLAR: c_int = 52;
/// Raven `P_ATSIGN`.
/// Source: `oracle/codemp/botlib/l_script.h:127`
pub const P_ATSIGN: c_int = 53;
