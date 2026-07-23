#![allow(non_camel_case_types, non_snake_case)]

use super::fuzzyseperator_s::FuzzySeperator;

/// Raven `weight_t` — a named fuzzy weight rooting a separator tree.
///
/// Redesigned (porting-rules §F17): Raven's malloc'd `char *name` +
/// `fuzzyseperator_t *firstseperator` become an owned `String` and an owned
/// `Option<Box<FuzzySeperator>>` tree root.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_weight.h:32-36`
#[derive(Default)]
pub struct Weight {
    pub name: String,
    pub firstseperator: Option<Box<FuzzySeperator>>,
}
