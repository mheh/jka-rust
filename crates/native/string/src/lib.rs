//! `native_string` — the canonical home of the C string-runtime family
//! (DEC-32), in idiomatic shapes: `&[u8]`/`&str` in, `bool`/`i32`/`String`
//! out. No `c_char`, no pointers — the ABI seam wrappers live at consumer
//! tiers (`mp_bg::cstr_util`, `native_platform`), which re-export from here
//! exactly like `native_math`'s q_math surface.
#![allow(non_snake_case)]
#![forbid(unsafe_code)]

pub mod atof;
pub mod atoi;
pub mod ctype;
pub mod filter;
pub mod gp2_tokenizer;
pub mod q_string;
pub mod q_strncpyz;
pub mod sscanf;
