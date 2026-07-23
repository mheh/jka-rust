#![allow(non_camel_case_types, non_snake_case)]

/// Raven `fuzzyseperator_t` — a node in a fuzzy-logic weight decision tree.
///
/// Redesigned (porting-rules §F17) from Raven's malloc'd `child`/`next` raw
/// pointers into an owned recursive shape: each node owns its `child` subtree
/// and its `next` sibling as `Option<Box<FuzzySeperator>>`. Raven builds the
/// tree as a pure ownership hierarchy (a sibling chain linked by `next`, each
/// node optionally rooting a `child` sub-switch) with no shared or back
/// pointers, so `Box` maps it exactly; the recursive walks
/// (`FuzzyWeight_r`/`Evolve`/`Scale`/`Interbreed`) become `&`/`&mut` tree walks
/// and `FreeFuzzySeperators_r` dissolves into `Drop`.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_weight.h:19-29`
#[derive(Default)]
pub struct FuzzySeperator {
    /// index in the inventory the case switches on
    pub index: i32,
    /// case value (or `MAX_INVENTORYVALUE` for `default`)
    pub value: i32,
    /// `WT_BALANCE` for a `balance(...)` leaf, else `0` (Raven `type`)
    pub type_: i32,
    pub weight: f32,
    pub minweight: f32,
    pub maxweight: f32,
    /// sub-switch evaluated when this case matches
    pub child: Option<Box<FuzzySeperator>>,
    /// next sibling case in the switch
    pub next: Option<Box<FuzzySeperator>>,
}
