//! `mdxmHeader_t`/`mdxmSurfHierarchy_t`/`mdxmSurface_t`/`mdxmVertex_t` views
//! over the `.glm` block.
//!
//! Source: `oracle/codemp/renderer/mdx_format.h:151-334`

use core::ffi::c_void;
use core::slice;

use mp_qshared::shared::MAX_QPATH;
use native_string::latin1_to_string;

// mdxmHeader_t offsets (`mdx_format.h:153-172`). MAX_QPATH == 64. Every field is
// a 4-byte-aligned int/`char[64]`, so natural alignment adds no padding.
const OFS_ANIM_NAME: usize = 4 + 4 + MAX_QPATH; // ident,version,name[]
const OFS_ANIM_INDEX: usize = OFS_ANIM_NAME + MAX_QPATH; // ..animName[] -> 136
const OFS_NUM_BONES: usize = OFS_ANIM_INDEX + 4; // 140
const OFS_NUM_LODS: usize = OFS_ANIM_INDEX + 4 + 4; // ..animIndex,numBones -> 144
const OFS_OFS_LODS: usize = OFS_NUM_LODS + 4; // 148
const OFS_NUM_SURFACES: usize = OFS_OFS_LODS + 4; // 152
const OFS_OFS_SURF_HIERARCHY: usize = OFS_NUM_SURFACES + 4; // 156
const OFS_END: usize = OFS_OFS_SURF_HIERARCHY + 4; // 160
/// `sizeof(mdxmHeader_t)` — where the `mdxmHierarchyOffsets_t` table starts.
const HEADER_SIZE: usize = OFS_END + 4; // 164

// mdxmSurfHierarchy_t offsets (`mdx_format.h:187-195`).
const SH_OFS_FLAGS: usize = MAX_QPATH; // name[64] -> 64
const SH_OFS_SHADER: usize = SH_OFS_FLAGS + 4; // 68
const SH_OFS_SHADER_INDEX: usize = SH_OFS_SHADER + MAX_QPATH; // shader[64] -> 132
const SH_OFS_PARENT_INDEX: usize = SH_OFS_SHADER_INDEX + 4; // shaderIndex -> 136
const SH_OFS_NUM_CHILDREN: usize = SH_OFS_PARENT_INDEX + 4; // 140
const SH_OFS_CHILD_INDEXES: usize = SH_OFS_NUM_CHILDREN + 4; // 144

// mdxmSurface_t offsets (`mdx_format.h:219-243`).
const SURF_OFS_THIS_SURFACE_INDEX: usize = 4;
const SURF_OFS_NUM_VERTS: usize = 12;
const SURF_OFS_OFS_VERTS: usize = 16;
const SURF_OFS_NUM_TRIANGLES: usize = 20;
const SURF_OFS_OFS_TRIANGLES: usize = 24;
const SURF_OFS_OFS_BONE_REFERENCES: usize = 32;

/// `sizeof(mdxmLOD_t)` — a single `int ofsEnd` (`mdx_format.h:203-207`).
const LOD_HEADER_SIZE: usize = 4;
/// `sizeof(mdxmTriangle_t)` — `int indexes[3]` (`mdx_format.h:250-252`).
const TRIANGLE_SIZE: usize = 12;
/// `sizeof(mdxmVertex_t)` (non-`_XBOX`): `normal`(12) + `vertCoords`(12) +
/// packed(4) + `BoneWeightings[4]`(4) — "kept at 32 bytes" (`mdx_format.h:263`).
const VERTEX_SIZE: usize = 32;
/// `sizeof(mdxmVertexTexCoord_t)` — `vec2_t texCoords` (`mdx_format.h:328-334`).
const TEXCOORD_SIZE: usize = 8;

// mdxmVertex_t offsets (`mdx_format.h:263-281`).
const VERT_OFS_VERT_COORDS: usize = 12;
const VERT_OFS_PACKED: usize = 24;
const VERT_OFS_BONE_WEIGHTINGS: usize = 28;

// Vertex bone-weight bit-packing (`mdx_format.h:57-66,290-322`).
const IG2_BITS_PER_BONEREF: u32 = 5;
const IG2_BONEWEIGHT_TOPBITS_SHIFT: u32 = 12;
const IG2_BONEWEIGHT_TOPBITS_AND: u32 = 0x300;
const FG2_BONEWEIGHT_RECIPROCAL_MULT: f32 = 1.0 / 1023.0;

fn read_i32(b: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(b[off..off + 4].try_into().unwrap())
}

fn read_u32(b: &[u8], off: usize) -> u32 {
    u32::from_le_bytes(b[off..off + 4].try_into().unwrap())
}

fn read_f32(b: &[u8], off: usize) -> f32 {
    f32::from_le_bytes(b[off..off + 4].try_into().unwrap())
}

/// A NUL-terminated `char[MAX_QPATH]` at `off`, decoded lossily.
fn name_bytes(b: &[u8], off: usize) -> &[u8] {
    let n = b[off..off + MAX_QPATH]
        .iter()
        .position(|&x| x == 0)
        .unwrap_or(MAX_QPATH);
    &b[off..off + n]
}

/// View over a `.glm` `mdxmHeader_t` block.
#[derive(Clone, Copy)]
pub struct MdxmView<'a> {
    bytes: &'a [u8],
}

impl<'a> MdxmView<'a> {
    /// # Safety
    /// `ptr` is a non-null `EngineHost::model_mdxm` block (self-sized by its
    /// `ofsEnd` field).
    pub unsafe fn from_block(ptr: *const c_void) -> Self {
        let base = ptr as *const u8;
        let ofs_end = unsafe { base.add(OFS_END).cast::<i32>().read_unaligned() };
        Self {
            bytes: unsafe { slice::from_raw_parts(base, ofs_end as usize) },
        }
    }

    /// `mdxmHeader_t->animIndex`.
    pub fn anim_index(&self) -> i32 {
        read_i32(self.bytes, OFS_ANIM_INDEX)
    }

    /// `mdxmHeader_t->animName` (empty when the buffer is NUL at byte 0).
    pub fn anim_name(&self) -> String {
        latin1_to_string(name_bytes(self.bytes, OFS_ANIM_NAME))
    }

    /// `mdxmHeader_t->numLODs`.
    pub fn num_lods(&self) -> i32 {
        read_i32(self.bytes, OFS_NUM_LODS)
    }

    /// `mdxmHeader_t->numSurfaces` (same per LOD).
    pub fn num_surfaces(&self) -> i32 {
        read_i32(self.bytes, OFS_NUM_SURFACES)
    }

    /// `mdxmHeader_t->ofsEnd` — the block's total self-describing size.
    pub fn ofs_end(&self) -> i32 {
        read_i32(self.bytes, OFS_END)
    }

    /// Sequential walk of the `numSurfaces` `mdxmSurfHierarchy_t` entries from
    /// `ofsSurfHierarchy` (each variable-sized by its `childIndexes[numChildren]`
    /// tail) — the name/shader searches (`G2_IsSurfaceLegal`,
    /// `G2API_SkinlessModel`, `G2_List_Model_Surfaces`) walk this.
    pub fn hierarchy_iter(&self) -> MdxmHierarchyIter<'a> {
        MdxmHierarchyIter {
            bytes: self.bytes,
            cursor: read_i32(self.bytes, OFS_OFS_SURF_HIERARCHY) as usize,
            remaining: self.num_surfaces(),
        }
    }

    /// The `mdxmSurfHierarchy_t` at `this_surface_index`, via the
    /// `mdxmHierarchyOffsets_t` table at `header + sizeof(mdxmHeader_t)`
    /// (`mdx_format.h:177-180`).
    pub fn surf_hierarchy(&self, this_surface_index: i32) -> MdxmSurfHierarchyView<'a> {
        let offset = read_i32(&self.bytes[HEADER_SIZE..], 4 * this_surface_index as usize) as usize;
        MdxmSurfHierarchyView {
            bytes: &self.bytes[HEADER_SIZE + offset..],
        }
    }

    /// Surface `index` within LOD `lod` — walk the `mdxmLOD_t` chain (each
    /// links to the next by its own `ofsEnd` at offset 0), step over the LOD
    /// header to the `mdxmLODSurfOffset_t` array, then index it
    /// (`G2_FindSurface`/`G2_FindSurface_BC`, `mdx_format.h:199-212`).
    pub fn find_surface(&self, index: i32, lod: i32) -> MdxmSurfaceView<'a> {
        let mut cursor = read_i32(self.bytes, OFS_OFS_LODS) as usize;
        for _ in 0..lod {
            cursor += read_i32(&self.bytes[cursor..], 0) as usize;
        }
        cursor += LOD_HEADER_SIZE;
        let offset = read_i32(&self.bytes[cursor..], 4 * index as usize) as usize;
        MdxmSurfaceView {
            bytes: &self.bytes[cursor + offset..],
        }
    }
}

/// The sequential `mdxmSurfHierarchy_t` walk of [`MdxmView::hierarchy_iter`].
pub struct MdxmHierarchyIter<'a> {
    bytes: &'a [u8],
    cursor: usize,
    remaining: i32,
}

impl<'a> Iterator for MdxmHierarchyIter<'a> {
    type Item = MdxmSurfHierarchyView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining <= 0 {
            return None;
        }
        self.remaining -= 1;
        let view = MdxmSurfHierarchyView {
            bytes: &self.bytes[self.cursor..],
        };
        // Next entry: this one's fixed header + its variable `childIndexes` tail.
        self.cursor += SH_OFS_CHILD_INDEXES + 4 * view.num_children() as usize;
        Some(view)
    }
}

/// View over one `mdxmSurfHierarchy_t`.
#[derive(Clone, Copy)]
pub struct MdxmSurfHierarchyView<'a> {
    bytes: &'a [u8],
}

impl MdxmSurfHierarchyView<'_> {
    /// `!Q_stricmp(surf->name, name)`.
    pub fn name_matches(&self, name: &str) -> bool {
        name_bytes(self.bytes, 0).eq_ignore_ascii_case(name.as_bytes())
    }

    /// `surf->name`, decoded from Latin-1.
    pub fn name_lossy(&self) -> String {
        latin1_to_string(name_bytes(self.bytes, 0))
    }

    /// `surf->flags` (`unsigned int`, read as `i32` — same bit pattern).
    pub fn flags(&self) -> i32 {
        read_i32(self.bytes, SH_OFS_FLAGS)
    }

    /// First byte of `surf->shader` (Raven's `surf->shader[0]` shaderless test).
    pub fn shader_first_byte(&self) -> u8 {
        self.bytes[SH_OFS_SHADER]
    }

    /// `surf->shaderIndex` — DEC-42.2 stores the shader arena slot number here,
    /// so the render-side default-shader resolve reads it back through
    /// `Arena::handle_at_slot`.
    pub fn shader_index(&self) -> i32 {
        read_i32(self.bytes, SH_OFS_SHADER_INDEX)
    }

    /// `surf->parentIndex`.
    pub fn parent_index(&self) -> i32 {
        read_i32(self.bytes, SH_OFS_PARENT_INDEX)
    }

    /// `surf->numChildren`.
    pub fn num_children(&self) -> i32 {
        read_i32(self.bytes, SH_OFS_NUM_CHILDREN)
    }

    /// `surf->childIndexes[i]`.
    pub fn child(&self, i: i32) -> i32 {
        read_i32(self.bytes, SH_OFS_CHILD_INDEXES + 4 * i as usize)
    }
}

/// View over one `mdxmSurface_t` (inside a LOD).
#[derive(Clone, Copy)]
pub struct MdxmSurfaceView<'a> {
    bytes: &'a [u8],
}

impl<'a> MdxmSurfaceView<'a> {
    /// `surface->thisSurfaceIndex`.
    pub fn this_surface_index(&self) -> i32 {
        read_i32(self.bytes, SURF_OFS_THIS_SURFACE_INDEX)
    }

    /// `surface->numVerts`.
    pub fn num_verts(&self) -> i32 {
        read_i32(self.bytes, SURF_OFS_NUM_VERTS)
    }

    /// `surface->numTriangles`.
    pub fn num_triangles(&self) -> i32 {
        read_i32(self.bytes, SURF_OFS_NUM_TRIANGLES)
    }

    /// `surface->triangles[j].indexes` (`mdxmTriangle_t` at `ofsTriangles`).
    pub fn triangle(&self, j: i32) -> [i32; 3] {
        let base =
            read_i32(self.bytes, SURF_OFS_OFS_TRIANGLES) as usize + j as usize * TRIANGLE_SIZE;
        [
            read_i32(self.bytes, base),
            read_i32(self.bytes, base + 4),
            read_i32(self.bytes, base + 8),
        ]
    }

    /// `mdxmVertex_t` `j` (`ofsVerts + j*32`).
    pub fn vert(&self, j: i32) -> MdxmVertView<'a> {
        let ofs = read_i32(self.bytes, SURF_OFS_OFS_VERTS) as usize + j as usize * VERTEX_SIZE;
        MdxmVertView {
            bytes: &self.bytes[ofs..],
        }
    }

    /// `mdxmVertexTexCoord_t` `j` — the parallel array after the `numVerts`
    /// `mdxmVertex_t`s (`ofsVerts + numVerts*32 + j*8`, `mdx_format.h:324-334`).
    pub fn texcoord(&self, j: i32) -> [f32; 2] {
        let base = read_i32(self.bytes, SURF_OFS_OFS_VERTS) as usize
            + self.num_verts() as usize * VERTEX_SIZE
            + j as usize * TEXCOORD_SIZE;
        [read_f32(self.bytes, base), read_f32(self.bytes, base + 4)]
    }

    /// `surface->boneReferences[i]` (`ofsBoneReferences + i*4`).
    pub fn bone_ref(&self, i: i32) -> i32 {
        let base = read_i32(self.bytes, SURF_OFS_OFS_BONE_REFERENCES) as usize;
        read_i32(self.bytes, base + 4 * i as usize)
    }
}

/// View over one `mdxmVertex_t`.
#[derive(Clone, Copy)]
pub struct MdxmVertView<'a> {
    bytes: &'a [u8],
}

impl MdxmVertView<'_> {
    /// `vert->normal`.
    pub fn normal(&self) -> [f32; 3] {
        [
            read_f32(self.bytes, 0),
            read_f32(self.bytes, 4),
            read_f32(self.bytes, 8),
        ]
    }

    /// `vert->vertCoords`.
    pub fn vert_coords(&self) -> [f32; 3] {
        [
            read_f32(self.bytes, VERT_OFS_VERT_COORDS),
            read_f32(self.bytes, VERT_OFS_VERT_COORDS + 4),
            read_f32(self.bytes, VERT_OFS_VERT_COORDS + 8),
        ]
    }

    fn packed(&self) -> u32 {
        read_u32(self.bytes, VERT_OFS_PACKED)
    }

    /// `G2_GetVertWeights` — the packed weight count (1..4).
    pub fn num_weights(&self) -> i32 {
        ((self.packed() >> 30) + 1) as i32
    }

    /// `G2_GetVertBoneIndex` — the `weight_num`-th 5-bit bone reference.
    pub fn bone_index(&self, weight_num: i32) -> i32 {
        ((self.packed() >> (IG2_BITS_PER_BONEREF * weight_num as u32))
            & ((1 << IG2_BITS_PER_BONEREF) - 1)) as i32
    }

    /// `G2_GetVertBoneWeight` — the `weight_num`-th weight; the last one closes
    /// to `1.0 - total_weight`, the rest decode from the 8-bit `BoneWeightings`
    /// entry plus its 2-bit overflow in the packed int and accumulate into
    /// `total_weight`.
    pub fn bone_weight(&self, weight_num: i32, total_weight: &mut f32, num_weights: i32) -> f32 {
        if weight_num == num_weights - 1 {
            1.0 - *total_weight
        } else {
            let mut temp = self.bytes[VERT_OFS_BONE_WEIGHTINGS + weight_num as usize] as u32;
            temp |= (self.packed() >> (IG2_BONEWEIGHT_TOPBITS_SHIFT + (weight_num as u32 * 2)))
                & IG2_BONEWEIGHT_TOPBITS_AND;
            let bone_weight = FG2_BONEWEIGHT_RECIPROCAL_MULT * temp as f32;
            *total_weight += bone_weight;
            bone_weight
        }
    }

    /// `G2_GetVertBoneWeightNotSlow` — the raw `weight_num`-th weight, decoded
    /// from the 8-bit `BoneWeightings` entry plus its 2-bit overflow in the
    /// packed int. The shipped render arm uses this decode. It never closes the
    /// last weight and never accumulates a total, so the caller owns both.
    /// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:3628-3639`
    pub fn bone_weight_not_slow(&self, weight_num: i32) -> f32 {
        let mut temp = self.bytes[VERT_OFS_BONE_WEIGHTINGS + weight_num as usize] as u32;
        temp |= (self.packed() >> (IG2_BONEWEIGHT_TOPBITS_SHIFT + (weight_num as u32 * 2)))
            & IG2_BONEWEIGHT_TOPBITS_AND;
        FG2_BONEWEIGHT_RECIPROCAL_MULT * temp as f32
    }
}

/// One parsed `mdxmSurfHierarchy_t` (DEC-35 parse-once sidecar) — the
/// surface-hierarchy row the name/shader/child searches used to re-walk the
/// variable-stride hierarchy for. Accessors mirror [`MdxmSurfHierarchyView`]
/// exactly so consumers are unchanged.
///
/// Source: `oracle/codemp/renderer/mdx_format.h:187-195`
pub struct MdxmSurfHierarchy {
    /// `surf->name`, lossily decoded (surface names are ASCII).
    name: String,
    /// `surf->flags`.
    flags: i32,
    /// First byte of `surf->shader` (Raven's `surf->shader[0]` shaderless test).
    shader_first_byte: u8,
    /// `surf->shaderIndex` — DEC-42.2 shader arena slot number.
    shader_index: i32,
    /// `surf->parentIndex`.
    parent_index: i32,
    /// `surf->childIndexes[0..numChildren]`.
    children: Vec<i32>,
}

impl MdxmSurfHierarchy {
    /// `!Q_stricmp(surf->name, name)` — byte-identical to
    /// [`MdxmSurfHierarchyView::name_matches`].
    pub fn name_matches(&self, name: &str) -> bool {
        self.name.as_bytes().eq_ignore_ascii_case(name.as_bytes())
    }

    /// `surf->name`, lossily decoded.
    pub fn name_lossy(&self) -> String {
        self.name.clone()
    }

    /// `surf->flags`.
    pub fn flags(&self) -> i32 {
        self.flags
    }

    /// First byte of `surf->shader`.
    pub fn shader_first_byte(&self) -> u8 {
        self.shader_first_byte
    }

    /// `surf->shaderIndex` — DEC-42.2 shader arena slot number.
    pub fn shader_index(&self) -> i32 {
        self.shader_index
    }

    /// `surf->parentIndex`.
    pub fn parent_index(&self) -> i32 {
        self.parent_index
    }

    /// `surf->numChildren`.
    pub fn num_children(&self) -> i32 {
        self.children.len() as i32
    }

    /// `surf->childIndexes[i]`.
    pub fn child(&self, i: i32) -> i32 {
        self.children[i as usize]
    }
}

/// Parse-once index over an `mdxmHeader_t` block (DEC-35) — the header
/// constants, the surface-hierarchy table (killing the linear hierarchy walks),
/// and a per-`(lod, surface)` byte-offset table so `find_surface` is an O(1)
/// lookup instead of a re-walk of the LOD chain. Vertices/triangles/texcoords
/// stay view-based (per-frame bulk data).
///
/// Source: `oracle/codemp/renderer/mdx_format.h:151-334`
pub struct MdxmParsed {
    /// `mdxmHeader_t->numBones`.
    pub num_bones: i32,
    /// `mdxmHeader_t->numLODs`.
    pub num_lods: i32,
    /// `mdxmHeader_t->numSurfaces` (same per LOD).
    pub num_surfaces: i32,
    /// `mdxmHeader_t->animIndex`.
    pub anim_index: i32,
    /// `mdxmHeader_t->animName`.
    pub anim_name: String,
    /// The `mdxmSurfHierarchy_t` table, in `thisSurfaceIndex` order.
    pub hierarchy: Vec<MdxmSurfHierarchy>,
    /// Absolute block byte offset of each surface's `mdxmSurface_t`, indexed
    /// `[lod][surface]` — `find_surface(surface, lod)`'s O(1) table.
    lod_surface_offsets: Vec<Vec<usize>>,
}

impl MdxmParsed {
    /// Decode the header constants, surface hierarchy, and per-LOD surface
    /// offsets from `view` once. Pure over the block bytes.
    pub fn parse(view: MdxmView) -> Self {
        let num_bones = read_i32(view.bytes, OFS_NUM_BONES);
        let num_lods = view.num_lods();
        let num_surfaces = view.num_surfaces();
        let anim_index = view.anim_index();
        let anim_name = view.anim_name();

        let hierarchy = view
            .hierarchy_iter()
            .map(|s| MdxmSurfHierarchy {
                name: s.name_lossy(),
                flags: s.flags(),
                shader_first_byte: s.shader_first_byte(),
                shader_index: s.shader_index(),
                parent_index: s.parent_index(),
                children: (0..s.num_children()).map(|i| s.child(i)).collect(),
            })
            .collect();

        // Mirror `MdxmView::find_surface`'s LOD-chain walk once, recording each
        // surface's absolute offset per LOD.
        let mut lod_surface_offsets = Vec::with_capacity(num_lods as usize);
        let mut cursor = read_i32(view.bytes, OFS_OFS_LODS) as usize;
        for _ in 0..num_lods {
            let surf_base = cursor + LOD_HEADER_SIZE;
            let offs = (0..num_surfaces as usize)
                .map(|index| surf_base + read_i32(&view.bytes[surf_base..], 4 * index) as usize)
                .collect();
            lod_surface_offsets.push(offs);
            cursor += read_i32(&view.bytes[cursor..], 0) as usize;
        }

        Self {
            num_bones,
            num_lods,
            num_surfaces,
            anim_index,
            anim_name,
            hierarchy,
            lod_surface_offsets,
        }
    }
}

/// Copy façade pairing the parse-once [`MdxmParsed`] index with the live
/// [`MdxmView`] over the same `.glm` block (DEC-35). Header constants and the
/// hierarchy come from `parsed`; `find_surface` is an O(1) table lookup over
/// `parsed` returning the same [`MdxmSurfaceView`] into `view` bytes;
/// vertex/triangle reads stay on that surface view (per-frame bulk data). Same
/// `'static` soundness contract as the bare view (see [`MdxaRef`]).
///
/// [`MdxaRef`]: crate::mdx::mdxa::MdxaRef
#[derive(Clone, Copy)]
pub struct MdxmRef<'a> {
    pub parsed: &'a MdxmParsed,
    pub view: MdxmView<'a>,
}

impl<'a> MdxmRef<'a> {
    /// `mdxmHeader_t->animIndex`.
    pub fn anim_index(&self) -> i32 {
        self.parsed.anim_index
    }

    /// `mdxmHeader_t->animName`.
    pub fn anim_name(&self) -> String {
        self.parsed.anim_name.clone()
    }

    /// `mdxmHeader_t->numLODs`.
    pub fn num_lods(&self) -> i32 {
        self.parsed.num_lods
    }

    /// `mdxmHeader_t->numSurfaces` (same per LOD).
    pub fn num_surfaces(&self) -> i32 {
        self.parsed.num_surfaces
    }

    /// `mdxmHeader_t->ofsEnd` — read off the block (a setup-path size read).
    pub fn ofs_end(&self) -> i32 {
        self.view.ofs_end()
    }

    /// The parsed `mdxmSurfHierarchy_t` at `this_surface_index`.
    pub fn surf_hierarchy(&self, this_surface_index: i32) -> &'a MdxmSurfHierarchy {
        &self.parsed.hierarchy[this_surface_index as usize]
    }

    /// The parsed surface-hierarchy table walk (subsumes the sequential
    /// re-walk [`MdxmView::hierarchy_iter`] did).
    pub fn hierarchy_iter(&self) -> core::slice::Iter<'a, MdxmSurfHierarchy> {
        self.parsed.hierarchy.iter()
    }

    /// Surface `index` within LOD `lod` — an O(1) parsed-offset lookup
    /// returning the same [`MdxmSurfaceView`] over `view` bytes.
    pub fn find_surface(&self, index: i32, lod: i32) -> MdxmSurfaceView<'a> {
        let off = self.parsed.lod_surface_offsets[lod as usize][index as usize];
        MdxmSurfaceView {
            bytes: &self.view.bytes[off..],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `mdxmHeader_t` prefix with the header fields the accessors read.
    fn header(num_lods: i32, num_surfaces: i32, ofs_lods: i32, ofs_surf_hierarchy: i32) -> Vec<u8> {
        let mut buf = vec![0u8; HEADER_SIZE];
        buf[OFS_ANIM_NAME..OFS_ANIM_NAME + 5].copy_from_slice(b"anim\0");
        buf[OFS_ANIM_INDEX..OFS_ANIM_INDEX + 4].copy_from_slice(&7i32.to_le_bytes());
        buf[OFS_NUM_LODS..OFS_NUM_LODS + 4].copy_from_slice(&num_lods.to_le_bytes());
        buf[OFS_OFS_LODS..OFS_OFS_LODS + 4].copy_from_slice(&ofs_lods.to_le_bytes());
        buf[OFS_NUM_SURFACES..OFS_NUM_SURFACES + 4].copy_from_slice(&num_surfaces.to_le_bytes());
        buf[OFS_OFS_SURF_HIERARCHY..OFS_OFS_SURF_HIERARCHY + 4]
            .copy_from_slice(&ofs_surf_hierarchy.to_le_bytes());
        buf
    }

    fn push_surf_hier(buf: &mut Vec<u8>, name: &str, flags: i32, parent: i32, children: &[i32]) {
        let mut nb = [0u8; MAX_QPATH];
        nb[..name.len()].copy_from_slice(name.as_bytes());
        buf.extend_from_slice(&nb); // name[64]
        buf.extend_from_slice(&flags.to_le_bytes()); // flags
        let mut shader = [0u8; MAX_QPATH];
        // first entry ("root") carries a shader name, "child" is shaderless.
        if name == "root" {
            shader[0] = b'x';
        }
        buf.extend_from_slice(&shader); // shader[64]
        buf.extend_from_slice(&0i32.to_le_bytes()); // shaderIndex
        buf.extend_from_slice(&parent.to_le_bytes()); // parentIndex
        buf.extend_from_slice(&(children.len() as i32).to_le_bytes()); // numChildren
        for &c in children {
            buf.extend_from_slice(&c.to_le_bytes());
        }
    }

    #[test]
    fn header_and_hierarchy_reads_match_layout() {
        // Header, then the offset table (2 entries), then two hierarchy entries.
        let mut buf = header(1, 2, 0, HEADER_SIZE as i32 + 8);
        // mdxmHierarchyOffsets_t: offsets[] relative to HEADER_SIZE.
        buf.extend_from_slice(&8i32.to_le_bytes()); // offsets[0] -> first entry
        let entry0_size = MAX_QPATH + 4 + MAX_QPATH + 4 + 4 + 4; // no children
        buf.extend_from_slice(&(8 + entry0_size as i32).to_le_bytes()); // offsets[1]
        push_surf_hier(&mut buf, "root", 0, -1, &[]);
        push_surf_hier(&mut buf, "child", 0x100, 0, &[]);
        let ofs_end = buf.len() as i32;
        buf[OFS_END..OFS_END + 4].copy_from_slice(&ofs_end.to_le_bytes());

        let v = unsafe { MdxmView::from_block(buf.as_ptr() as *const c_void) };
        assert_eq!(v.num_lods(), 1);
        assert_eq!(v.num_surfaces(), 2);
        assert_eq!(v.anim_index(), 7);
        assert_eq!(v.anim_name(), "anim");
        assert_eq!(v.ofs_end(), buf.len() as i32);

        // Sequential walk.
        let names: Vec<String> = v.hierarchy_iter().map(|s| s.name_lossy()).collect();
        assert_eq!(names, ["root", "child"]);
        assert!(v.hierarchy_iter().next().unwrap().name_matches("ROOT"));

        // Indexed via the offset table.
        assert_eq!(v.surf_hierarchy(1).flags(), 0x100);
        assert_eq!(v.surf_hierarchy(1).parent_index(), 0);
        assert_eq!(v.surf_hierarchy(0).num_children(), 0);
        // shader_first_byte: "root" carries one, "child" is shaderless.
        assert_eq!(v.surf_hierarchy(0).shader_first_byte(), b'x');
        assert_eq!(v.surf_hierarchy(1).shader_first_byte(), 0);
    }

    #[test]
    fn surface_and_vertex_reads_match_layout() {
        // Header (1 LOD at ofsLODs), then a single-surface LOD.
        let ofs_lods = HEADER_SIZE + 8; // leave the hierarchy offset table (2 ints) empty
        let mut buf = header(1, 1, ofs_lods as i32, 0);
        buf.extend_from_slice(&0i32.to_le_bytes()); // offsets[0] (unused here)
        buf.extend_from_slice(&0i32.to_le_bytes()); // pad to ofs_lods
                                                    // mdxmLOD_t { ofsEnd } then mdxmLODSurfOffset_t { offsets[1] }.
        let lod_start = buf.len();
        buf.extend_from_slice(&0i32.to_le_bytes()); // mdxmLOD_t.ofsEnd (only LOD)
        let surf_offset_pos = buf.len();
        buf.extend_from_slice(&0i32.to_le_bytes()); // offsets[0], patched below
                                                    // mdxmSurface_t at the current end; offset is relative to surf_offset base.
        let surf_start = buf.len();
        let surf_off_rel = (surf_start - surf_offset_pos) as i32;
        buf[surf_offset_pos..surf_offset_pos + 4].copy_from_slice(&surf_off_rel.to_le_bytes());

        // mdxmSurface_t: ident, thisSurfaceIndex, ofsHeader, numVerts, ofsVerts,
        // numTriangles, ofsTriangles, numBoneReferences, ofsBoneReferences, ofsEnd.
        let num_verts = 1i32;
        // header is 40 bytes (10 ints); place verts/tris/bonerefs after it.
        let ofs_verts = 40i32;
        let ofs_tris = ofs_verts + num_verts * VERTEX_SIZE as i32 + TEXCOORD_SIZE as i32;
        let ofs_bone_refs = ofs_tris + TRIANGLE_SIZE as i32;
        let mut surf = Vec::new();
        surf.extend_from_slice(&0i32.to_le_bytes()); // ident
        surf.extend_from_slice(&3i32.to_le_bytes()); // thisSurfaceIndex
        surf.extend_from_slice(&0i32.to_le_bytes()); // ofsHeader
        surf.extend_from_slice(&num_verts.to_le_bytes()); // numVerts
        surf.extend_from_slice(&ofs_verts.to_le_bytes()); // ofsVerts
        surf.extend_from_slice(&1i32.to_le_bytes()); // numTriangles
        surf.extend_from_slice(&ofs_tris.to_le_bytes()); // ofsTriangles
        surf.extend_from_slice(&1i32.to_le_bytes()); // numBoneReferences
        surf.extend_from_slice(&ofs_bone_refs.to_le_bytes()); // ofsBoneReferences
        surf.extend_from_slice(&0i32.to_le_bytes()); // ofsEnd
                                                     // vertex 0: normal, vertCoords, packed(1 weight), boneWeightings.
        surf.extend_from_slice(&1.0f32.to_le_bytes());
        surf.extend_from_slice(&2.0f32.to_le_bytes());
        surf.extend_from_slice(&3.0f32.to_le_bytes());
        surf.extend_from_slice(&4.0f32.to_le_bytes());
        surf.extend_from_slice(&5.0f32.to_le_bytes());
        surf.extend_from_slice(&6.0f32.to_le_bytes());
        surf.extend_from_slice(&0u32.to_le_bytes()); // packed: 0 -> num_weights 1
        surf.extend_from_slice(&[0u8; 4]); // boneWeightings
                                           // texcoord 0.
        surf.extend_from_slice(&0.5f32.to_le_bytes());
        surf.extend_from_slice(&0.25f32.to_le_bytes());
        // triangle 0.
        surf.extend_from_slice(&10i32.to_le_bytes());
        surf.extend_from_slice(&11i32.to_le_bytes());
        surf.extend_from_slice(&12i32.to_le_bytes());
        // boneReferences[0].
        surf.extend_from_slice(&42i32.to_le_bytes());
        buf.extend_from_slice(&surf);

        // Patch ofsEnd so from_block sizes the whole buffer.
        let ofs_end = buf.len() as i32;
        buf[OFS_END..OFS_END + 4].copy_from_slice(&ofs_end.to_le_bytes());
        let _ = lod_start;

        let v = unsafe { MdxmView::from_block(buf.as_ptr() as *const c_void) };
        let s = v.find_surface(0, 0);
        assert_eq!(s.this_surface_index(), 3);
        assert_eq!(s.num_verts(), 1);
        assert_eq!(s.num_triangles(), 1);
        assert_eq!(s.triangle(0), [10, 11, 12]);
        assert_eq!(s.bone_ref(0), 42);
        assert_eq!(s.texcoord(0), [0.5, 0.25]);
        let vert = s.vert(0);
        assert_eq!(vert.normal(), [1.0, 2.0, 3.0]);
        assert_eq!(vert.vert_coords(), [4.0, 5.0, 6.0]);
        assert_eq!(vert.num_weights(), 1);

        // The parsed façade returns the same surface via its O(1) table.
        let parsed = MdxmParsed::parse(v);
        let r = MdxmRef {
            parsed: &parsed,
            view: v,
        };
        assert_eq!(r.num_surfaces(), 1);
        assert_eq!(r.num_lods(), 1);
        assert_eq!(r.anim_index(), 7);
        let rs = r.find_surface(0, 0);
        assert_eq!(rs.this_surface_index(), 3);
        assert_eq!(rs.triangle(0), [10, 11, 12]);
        assert_eq!(rs.bone_ref(0), 42);
    }

    #[test]
    fn parsed_hierarchy_matches_the_view() {
        let mut buf = header(1, 2, 0, HEADER_SIZE as i32 + 8);
        buf.extend_from_slice(&8i32.to_le_bytes());
        let entry0_size = MAX_QPATH + 4 + MAX_QPATH + 4 + 4 + 4;
        buf.extend_from_slice(&(8 + entry0_size as i32).to_le_bytes());
        push_surf_hier(&mut buf, "root", 0, -1, &[]);
        push_surf_hier(&mut buf, "child", 0x100, 0, &[]);
        let ofs_end = buf.len() as i32;
        buf[OFS_END..OFS_END + 4].copy_from_slice(&ofs_end.to_le_bytes());

        let v = unsafe { MdxmView::from_block(buf.as_ptr() as *const c_void) };
        let parsed = MdxmParsed::parse(v);
        let r = MdxmRef {
            parsed: &parsed,
            view: v,
        };
        let names: Vec<String> = r.hierarchy_iter().map(|s| s.name_lossy()).collect();
        assert_eq!(names, ["root", "child"]);
        assert!(r.surf_hierarchy(0).name_matches("ROOT"));
        assert_eq!(r.surf_hierarchy(1).flags(), 0x100);
        assert_eq!(r.surf_hierarchy(1).parent_index(), 0);
        assert_eq!(r.surf_hierarchy(0).shader_first_byte(), b'x');
        assert_eq!(r.surf_hierarchy(1).shader_first_byte(), 0);
    }
}
