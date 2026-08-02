#![allow(non_camel_case_types, non_snake_case)]

/// Raven's four `void (*VQ0/VQ1/VQNormal/VQBuffer)(byte *status, void *qdata)`
/// blitter slots in `cin_cache`.
///
/// The set is closed: `initRoQ` is the only writer and it installs
/// `blitVQQuad32fs` in every slot, so a function-pointer table becomes an enum
/// (porting-rules §8). The Rust blitter also needs `&mut Client`, which no bare
/// `fn` pointer can carry.
///
/// - `None`: the slot Raven leaves null before `initRoQ` runs.
/// - `BlitVQQuad32fs`: Raven `blitVQQuad32fs`.
///
/// Source: `oracle/codemp/client/cl_cin.cpp:97-100,866-874`
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VqBlitter {
    None = 0,
    BlitVQQuad32fs = 1,
}
