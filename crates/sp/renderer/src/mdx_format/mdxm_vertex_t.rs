#![allow(non_camel_case_types, non_snake_case)]
use sp_qshared::shared::vec3_t;

/// Raven `mdxmVertex_t` — Ghoul2 mesh vertex (normal, position, packed bone weights).
///
/// Raven: `BoneWeightings` is sized by `iMAX_G2_BONEWEIGHTS_PER_VERT` (4).
/// Type definition source: `oracle/code/game/../game/../renderer/mdx_format.h:260-281`
#[repr(C)]
pub struct mdxmVertex_t {
    pub normal: vec3_t,
    pub vertCoords: vec3_t,
    // Raven: packed int...
    // 32 bits.  format:
    // 31 & 30:  0..3 (= 1..4) weight count
    // 29 & 28 (spare)
    //  2 bit pairs at 20,22,24,26 are 2-bit overflows from 4 BonWeights below (20=[0], 22=[1]) etc)
    //  5-bits each (iG2_BITS_PER_BONEREF) for boneweights
    // effectively a packed int, each bone weight converted from 0..1 float to 0..255 int...
    //  promote each entry to float and multiply by fG2_BONEWEIGHT_RECIPROCAL_MULT to convert.
    pub uiNmWeightsAndBoneIndexes: u32,
    pub BoneWeightings: [u8; 4],
}
const _: () = assert!(core::mem::size_of::<mdxmVertex_t>() == 32);
const _: () = assert!(core::mem::offset_of!(mdxmVertex_t, normal) == 0);
const _: () = assert!(core::mem::offset_of!(mdxmVertex_t, vertCoords) == 12);
const _: () = assert!(core::mem::offset_of!(mdxmVertex_t, uiNmWeightsAndBoneIndexes) == 24);
const _: () = assert!(core::mem::offset_of!(mdxmVertex_t, BoneWeightings) == 28);
