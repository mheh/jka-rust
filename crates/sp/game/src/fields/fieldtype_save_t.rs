#![allow(non_camel_case_types, non_snake_case)]

/// Raven `fieldtypeSAVE_t` — save field type enumeration.
///
/// Type definition source: `oracle/code/game/fields.h:31-53`
#[repr(i32)]
pub enum fieldtypeSAVE_t {
    F_STRING = 0,  // string
    F_NULL = 1,    // A ptr to null out
    F_ITEM = 2,    // Item pointer handling
    F_GCLIENT = 3, // Client pointer handling
    F_GENTITY = 4, // gentity_t ptr handling
    F_BOOLPTR = 5, // Generic pointer that is recreated later, could be left alone, but clearer if only 0/1 rather than 0/alloc

    F_BEHAVIORSET = 6,  // special scripting string ptr array handler
    F_ALERTEVENT = 7,   // special handler for alertevent struct in level_locals_t
    F_AIGROUPS = 8,     // some AI grouping stuff of Mike's
    F_ANIMFILESETS = 9, // animfileset animevent strings

    F_GROUP = 10,
    F_VEHINFO = 11,
    F_IGNORE = 12,
}
