//! `mdx` — byte-offset views over the loader's opaque Ghoul2 model blocks:
//! `.glm`/`mdxm` mesh (`model_mdxm`), `.gla`/`mdxa` animation (`model_mdxa`),
//! both bare `*const c_void`. One home for the header offsets ~7 files used to
//! re-derive (dedup cluster 23).
//!
//! A view holds only a `&[u8]`; construction parses nothing. The one `unsafe`
//! per block is `from_block`: read the self-describing `ofsEnd` (total size),
//! `slice::from_raw_parts`. All accessors are safe slice reads.
//!
//! `G2SV-D5`: no `#[repr(C)]` mirror of the mdx header structs — these offsets
//! are the only copy. §19: an accessor panics on a truncated block where
//! Raven's pointer walk was UB; valid reads are byte-identical.
//!
//! Source: `oracle/codemp/renderer/mdx_format.h:151-413`

pub mod mdxa;
pub mod mdxm;
