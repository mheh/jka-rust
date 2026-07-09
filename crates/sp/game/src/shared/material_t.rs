#![allow(non_camel_case_types, non_snake_case)]

/// Raven `material_t` — material type for chunk generation.
///
/// Type definition source: `oracle/code/game/g_shared.h:37-58`
#[repr(i32)]
pub enum material_t {
    MAT_METAL = 0,           // scorched blue-grey metal
    MAT_GLASS = 1,           // not a real chunk type, just plays an effect with glass sprites
    MAT_ELECTRICAL = 2,      // sparks only
    MAT_ELEC_METAL = 3,      // sparks/electrical type metal
    MAT_DRK_STONE = 4,       // brown
    MAT_LT_STONE = 5,        // tan
    MAT_GLASS_METAL = 6,     // glass sprites and METAL chunk
    MAT_METAL2 = 7,          // electrical metal type
    MAT_NONE = 8,            // no chunks
    MAT_GREY_STONE = 9,      // grey
    MAT_METAL3 = 10,         // METAL and METAL2 chunks
    MAT_CRATE1 = 11,         // yellow multi-colored crate chunks
    MAT_GRATE1 = 12,         // grate chunks
    MAT_ROPE = 13,           // for yavin trial...no chunks, just wispy bits
    MAT_CRATE2 = 14,         // red multi-colored crate chunks
    MAT_WHITE_METAL = 15,    // white angular chunks
    NUM_MATERIALS = 16,
}
