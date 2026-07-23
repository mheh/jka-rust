//! `native_string` — the canonical home of the C string-runtime family
//! (DEC-32), in idiomatic shapes: `&[u8]`/`&str` in, `bool`/`i32`/`String`
//! out. No `c_char`, no pointers — the ABI seam wrappers live at consumer
//! tiers (`mp_bg::cstr_util`, `native_platform`), which re-export from here
//! exactly like `native_math`'s q_math surface.
#![allow(non_snake_case)]
#![forbid(unsafe_code)]

pub mod atof;
pub mod atoi;
pub mod cstr;
pub mod ctype;
pub mod filter;
pub mod gp2_tokenizer;
pub mod info;
pub mod q_string;
pub mod q_strncpyz;
pub mod sscanf;
pub mod stricmp;

// The whole fn/const surface, flat at the crate root (module paths stay valid).
pub use atof::{atof, atof_bytes};
pub use atoi::{atoi, atoi_bytes};
pub use cstr::{buf_to_string, cstr, latin1_to_string, string_to_latin1, strncpyz_string};
pub use ctype::{
    isdigit, isdigit_byte, islower, islower_byte, isspace, isspace_byte, isupper, isupper_byte,
    tolower, tolower_byte, toupper, toupper_byte,
};
pub use filter::{
    Com_Filter, Com_FilterBytes, Com_FilterPath, Com_FilterPathBytes, Com_StringContains,
    Com_StringContainsBytes,
};
pub use gp2_tokenizer::{Tokenizer, MAX_TOKEN_SIZE};
pub use info::{
    Info_RemoveKey, Info_RemoveKey_Big, Info_SetValueForKey, Info_SetValueForKey_Big,
    Info_Validate, Info_ValueForKey, InfoSetResult, BIG_INFO_STRING, MAX_INFO_STRING,
};
pub use q_string::{
    strcat_string, Q_stricmp, Q_stricmpBytes, Q_stricmpn, Q_stricmpnBytes, Q_strcat, Q_strcmp,
    Q_strcmpBytes, Q_strlwr, Q_strncmp, Q_strncmpBytes, Q_CleanStr,
};
pub use q_strncpyz::{Q_strncpyz, Q_strncpyzBytes};
pub use sscanf::sscanf_f32s;
pub use stricmp::stricmp;
