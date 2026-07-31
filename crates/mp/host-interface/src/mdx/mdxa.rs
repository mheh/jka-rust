//! `mdxaHeader_t`/`mdxaSkel_t`/`mdxaIndex_t` views over the `.gla` block.
//!
//! Source: `oracle/codemp/renderer/mdx_format.h:349-424`

use core::ffi::c_void;
use core::slice;

use mp_qshared::shared::{mdxaBone_t, MAX_QPATH};
use native_string::latin1_to_string;

// mdxaHeader_t offsets (`mdx_format.h:351-371`). MAX_QPATH == 64.
const OFS_NUM_FRAMES: usize = 4 + 4 + MAX_QPATH + 4; // ident,version,name[],fScale
const OFS_OFS_FRAMES: usize = OFS_NUM_FRAMES + 4;
const OFS_NUM_BONES: usize = OFS_OFS_FRAMES + 4;
const OFS_OFS_COMP_BONE_POOL: usize = OFS_NUM_BONES + 4;
const OFS_END: usize = OFS_OFS_COMP_BONE_POOL + 4 + 4; // ..ofsSkel, ofsEnd
/// `sizeof(mdxaHeader_t)`.
const HEADER_SIZE: usize = OFS_END + 4;

// mdxaSkel_t offsets (`mdx_format.h:388-396`).
const SKEL_OFS_PARENT: usize = MAX_QPATH + 4; // name[], flags
const SKEL_OFS_BASE_POSE_MAT: usize = SKEL_OFS_PARENT + 4;
const SKEL_OFS_BASE_POSE_MAT_INV: usize = SKEL_OFS_BASE_POSE_MAT + 48; // mdxaBone_t
const SKEL_OFS_NUM_CHILDREN: usize = SKEL_OFS_BASE_POSE_MAT_INV + 48;
const SKEL_OFS_CHILDREN: usize = SKEL_OFS_NUM_CHILDREN + 4;

/// `sizeof(mdxaCompQuatBone_t::Comp)` (`mdx_format.h:124`).
const COMP_QUAT_BONE_SIZE: usize = 14;

fn read_i32(b: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(b[off..off + 4].try_into().unwrap())
}

fn read_bone(b: &[u8], off: usize) -> mdxaBone_t {
    let mut m = [[0.0f32; 4]; 3];
    for (r, row) in m.iter_mut().enumerate() {
        for (c, v) in row.iter_mut().enumerate() {
            *v = f32::from_le_bytes(
                b[off + (r * 4 + c) * 4..off + (r * 4 + c) * 4 + 4]
                    .try_into()
                    .unwrap(),
            );
        }
    }
    mdxaBone_t { matrix: m }
}

/// View over a `.gla` `mdxaHeader_t` block.
#[derive(Clone, Copy)]
pub struct MdxaView<'a> {
    bytes: &'a [u8],
}

impl<'a> MdxaView<'a> {
    /// # Safety
    /// `ptr` is a non-null `EngineHost::model_mdxa` block (self-sized by its
    /// `ofsEnd` field).
    pub unsafe fn from_block(ptr: *const c_void) -> Self {
        let base = ptr as *const u8;
        let ofs_end = unsafe { base.add(OFS_END).cast::<i32>().read_unaligned() };
        Self {
            bytes: unsafe { slice::from_raw_parts(base, ofs_end as usize) },
        }
    }

    /// `mdxaHeader_t->numFrames`.
    pub fn num_frames(&self) -> i32 {
        read_i32(self.bytes, OFS_NUM_FRAMES)
    }

    /// `mdxaHeader_t->numBones`.
    pub fn num_bones(&self) -> i32 {
        read_i32(self.bytes, OFS_NUM_BONES)
    }

    /// `mdxaHeader_t->ofsEnd` — the block's total self-describing size (the
    /// reloaded-model size-change check in `G2_SetupModelPointers`/
    /// `G2_TestModelPointers`).
    pub fn ofs_end(&self) -> i32 {
        read_i32(self.bytes, OFS_END)
    }

    /// Bone `i`'s `mdxaSkel_t`, via the `mdxaSkelOffsets_t` table at
    /// `header + sizeof(mdxaHeader_t)` (`mdx_format.h:376-379`).
    pub fn skel(&self, i: i32) -> MdxaSkelView<'a> {
        let rel = read_i32(self.bytes, HEADER_SIZE + i as usize * 4);
        MdxaSkelView {
            bytes: &self.bytes[HEADER_SIZE + rel as usize..],
        }
    }

    /// `G2_GetBonePoolIndex` — the compressed-bone pool index for
    /// `<frame, bone>`, AND'd to 24 bits (`mdx_format.h:405-413`;
    /// `tr_ghoul2.cpp:1148-1155`).
    pub fn frame_bone_pool_index(&self, frame: i32, bone: i32) -> i32 {
        let num_bones = self.num_bones();
        let ofs_frames = read_i32(self.bytes, OFS_OFS_FRAMES);
        let ofs = (frame * num_bones * 3) + (bone * 3);
        read_i32(self.bytes, ofs_frames as usize + ofs as usize) & 0x00FF_FFFF
    }

    /// The 14-byte `mdxaCompQuatBone_t` at `pool_index` in the pool at
    /// `ofsCompBonePool` (`mdx_format.h:366,420-424`).
    pub fn comp_bone(&self, pool_index: i32) -> &'a [u8] {
        let ofs_pool = read_i32(self.bytes, OFS_OFS_COMP_BONE_POOL) as usize;
        let start = ofs_pool + pool_index as usize * COMP_QUAT_BONE_SIZE;
        &self.bytes[start..start + COMP_QUAT_BONE_SIZE]
    }
}

/// View over one `mdxaSkel_t`.
#[derive(Clone, Copy)]
pub struct MdxaSkelView<'a> {
    bytes: &'a [u8],
}

impl<'a> MdxaSkelView<'a> {
    fn name_bytes(&self) -> &'a [u8] {
        let n = self.bytes[..MAX_QPATH]
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(MAX_QPATH);
        &self.bytes[..n]
    }

    /// `!Q_stricmp(skel->name, name)`.
    pub fn name_matches(&self, name: &str) -> bool {
        self.name_bytes().eq_ignore_ascii_case(name.as_bytes())
    }

    /// `skel->name`, decoded from Latin-1.
    pub fn name_lossy(&self) -> String {
        latin1_to_string(self.name_bytes())
    }

    /// `skel->parent`.
    pub fn parent(&self) -> i32 {
        read_i32(self.bytes, SKEL_OFS_PARENT)
    }

    /// `skel->numChildren`.
    pub fn num_children(&self) -> i32 {
        read_i32(self.bytes, SKEL_OFS_NUM_CHILDREN)
    }

    /// `skel->children[i]`.
    pub fn child(&self, i: usize) -> i32 {
        read_i32(self.bytes, SKEL_OFS_CHILDREN + i * 4)
    }

    /// `skel->BasePoseMat`.
    pub fn base_pose_mat(&self) -> mdxaBone_t {
        read_bone(self.bytes, SKEL_OFS_BASE_POSE_MAT)
    }

    /// `skel->BasePoseMatInv`.
    pub fn base_pose_mat_inv(&self) -> mdxaBone_t {
        read_bone(self.bytes, SKEL_OFS_BASE_POSE_MAT_INV)
    }

    /// Pointer to `skel->BasePoseMat` in the loader block — for the callers
    /// that alias it out by reference (`G2_GetBoneMatrixLow`/`GetBoneBasepose`).
    pub fn base_pose_mat_ptr(&self) -> *const mdxaBone_t {
        self.bytes[SKEL_OFS_BASE_POSE_MAT..].as_ptr().cast()
    }

    /// Pointer to `skel->BasePoseMatInv` — see [`Self::base_pose_mat_ptr`].
    pub fn base_pose_mat_inv_ptr(&self) -> *const mdxaBone_t {
        self.bytes[SKEL_OFS_BASE_POSE_MAT_INV..].as_ptr().cast()
    }
}

/// One parsed `mdxaSkel_t` (DEC-35 parse-once sidecar) — the bone-hierarchy
/// data the transform/ragdoll paths re-decode every frame, decoded ONCE at
/// model load. Indexed by bone number. Plain fields; only [`Self::name_matches`]
/// carries the byte-exact case-insensitive semantics [`MdxaSkelView::name_matches`]
/// had.
///
/// Source: `oracle/codemp/renderer/mdx_format.h:388-396`
pub struct MdxaSkel {
    /// `skel->name`, lossily decoded (bone names are ASCII, so lossless here).
    pub name: String,
    /// `skel->parent`.
    pub parent: i32,
    /// `skel->children[0..numChildren]`.
    pub children: Vec<i32>,
    /// `skel->BasePoseMat`.
    pub base_pose_mat: mdxaBone_t,
    /// `skel->BasePoseMatInv`.
    pub base_pose_mat_inv: mdxaBone_t,
}

impl MdxaSkel {
    /// `!Q_stricmp(skel->name, name)` — byte-identical to
    /// [`MdxaSkelView::name_matches`] (case-insensitive over the NUL-stripped
    /// name bytes; `name` is ASCII so the lossy `String` matches raw bytes).
    pub fn name_matches(&self, name: &str) -> bool {
        self.name.as_bytes().eq_ignore_ascii_case(name.as_bytes())
    }
}

/// Parse-once index over an `mdxaHeader_t` block (DEC-35) — the header constants
/// and the full `mdxaSkel_t` table, decoded once at model load instead of every
/// bone every frame. The compressed bone pool and per-frame indices stay
/// view-based (per-frame bulk data); this holds only the read-more-than-once
/// data plus the pool offsets [`MdxaRef`] needs so the hot-path accessors never
/// re-decode header constants.
///
/// Source: `oracle/codemp/renderer/mdx_format.h:349-424`
pub struct MdxaParsed {
    /// `mdxaHeader_t->numFrames`.
    pub num_frames: i32,
    /// `mdxaHeader_t->numBones`.
    pub num_bones: i32,
    /// `mdxaHeader_t->ofsFrames` — base of the `<frame,bone>` pool-index table.
    pub ofs_frames: i32,
    /// `mdxaHeader_t->ofsCompBonePool` — base of the 14-byte compressed bones.
    pub ofs_comp_bone_pool: i32,
    /// The `mdxaSkel_t` table, indexed by bone number.
    pub skel: Vec<MdxaSkel>,
}

impl MdxaParsed {
    /// Decode the header constants and `mdxaSkel_t` table from `view` once.
    /// Pure over the block bytes; the renderer calls it at ingest and the mock
    /// from its fixture bytes.
    pub fn parse(view: MdxaView) -> Self {
        let num_frames = view.num_frames();
        let num_bones = view.num_bones();
        let ofs_frames = read_i32(view.bytes, OFS_OFS_FRAMES);
        let ofs_comp_bone_pool = read_i32(view.bytes, OFS_OFS_COMP_BONE_POOL);
        // §19: a header-only/truncated block (whose `mdxaSkelOffsets_t` table
        // does not fit before `ofsEnd` — a loader-test fixture; a real `.gla`
        // always carries the full table) has no skel data. Raven's pointer walk
        // past the block would be UB; the defined behavior is an empty index.
        let mut skel = Vec::with_capacity(num_bones as usize);
        if view.bytes.len() >= HEADER_SIZE + num_bones as usize * 4 {
            for i in 0..num_bones {
                let s = view.skel(i);
                let children = (0..s.num_children()).map(|c| s.child(c as usize)).collect();
                skel.push(MdxaSkel {
                    name: s.name_lossy(),
                    parent: s.parent(),
                    children,
                    base_pose_mat: s.base_pose_mat(),
                    base_pose_mat_inv: s.base_pose_mat_inv(),
                });
            }
        }
        Self {
            num_frames,
            num_bones,
            ofs_frames,
            ofs_comp_bone_pool,
            skel,
        }
    }
}

/// Copy façade pairing the parse-once [`MdxaParsed`] index with the live
/// [`MdxaView`] over the same `.gla` block (DEC-35). Accessors keep the same
/// names [`MdxaView`] exposes, routed to the cheap source: header constants and
/// `skel` come from `parsed`; `frame_bone_pool_index`/`comp_bone` read the
/// per-frame pool bytes from `view` with `parsed` constants (no per-call header
/// decode). Same `'static` soundness contract as the bare view: valid until
/// model eviction, revalidated by `G2_SetupModelPointers` — and `parsed` is
/// owned by the same registry entry, dropped at eviction with the block.
#[derive(Clone, Copy)]
pub struct MdxaRef<'a> {
    pub parsed: &'a MdxaParsed,
    pub view: MdxaView<'a>,
}

impl<'a> MdxaRef<'a> {
    /// `mdxaHeader_t->numFrames`.
    pub fn num_frames(&self) -> i32 {
        self.parsed.num_frames
    }

    /// `mdxaHeader_t->numBones`.
    pub fn num_bones(&self) -> i32 {
        self.parsed.num_bones
    }

    /// `mdxaHeader_t->ofsEnd` — read off the block (the `G2_SetupModelPointers`
    /// size-change check, a setup-path read, not a per-frame one).
    pub fn ofs_end(&self) -> i32 {
        self.view.ofs_end()
    }

    /// Bone `i`'s parsed `mdxaSkel_t`.
    pub fn skel(&self, i: i32) -> &'a MdxaSkel {
        &self.parsed.skel[i as usize]
    }

    /// `G2_GetBonePoolIndex` — the compressed-bone pool index for
    /// `<frame, bone>`, AND'd to 24 bits (per-frame bulk read off `view` with
    /// the parsed `numBones`/`ofsFrames`, no header re-decode).
    pub fn frame_bone_pool_index(&self, frame: i32, bone: i32) -> i32 {
        let num_bones = self.parsed.num_bones;
        let ofs = (frame * num_bones * 3) + (bone * 3);
        read_i32(
            self.view.bytes,
            self.parsed.ofs_frames as usize + ofs as usize,
        ) & 0x00FF_FFFF
    }

    /// The 14-byte `mdxaCompQuatBone_t` at `pool_index` (per-frame bulk read off
    /// `view` with the parsed `ofsCompBonePool`).
    pub fn comp_bone(&self, pool_index: i32) -> &'a [u8] {
        let start =
            self.parsed.ofs_comp_bone_pool as usize + pool_index as usize * COMP_QUAT_BONE_SIZE;
        &self.view.bytes[start..start + COMP_QUAT_BONE_SIZE]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Header + one-entry offset table + one skel (name/parent/basepose).
    fn one_bone_mdxa(
        num_frames: i32,
        name: &str,
        base: mdxaBone_t,
        base_inv: mdxaBone_t,
    ) -> Vec<u8> {
        let mut buf = vec![0u8; HEADER_SIZE];
        buf[OFS_NUM_FRAMES..OFS_NUM_FRAMES + 4].copy_from_slice(&num_frames.to_le_bytes());
        buf[OFS_NUM_BONES..OFS_NUM_BONES + 4].copy_from_slice(&1i32.to_le_bytes());
        // offsets[0] = 4 (past the single-entry table).
        buf.extend_from_slice(&4i32.to_le_bytes());
        let mut skel = vec![0u8; MAX_QPATH];
        skel[..name.len()].copy_from_slice(name.as_bytes());
        skel.extend_from_slice(&0i32.to_le_bytes()); // flags
        skel.extend_from_slice(&(-1i32).to_le_bytes()); // parent
        for row in base.matrix {
            for v in row {
                skel.extend_from_slice(&v.to_le_bytes());
            }
        }
        for row in base_inv.matrix {
            for v in row {
                skel.extend_from_slice(&v.to_le_bytes());
            }
        }
        skel.extend_from_slice(&0i32.to_le_bytes()); // numChildren
        buf.extend_from_slice(&skel);
        let ofs_end = buf.len() as i32;
        buf[OFS_END..OFS_END + 4].copy_from_slice(&ofs_end.to_le_bytes());
        buf
    }

    #[test]
    fn reads_match_layout() {
        let base = mdxaBone_t {
            matrix: [
                [1.0, 2.0, 3.0, 4.0],
                [5.0, 6.0, 7.0, 8.0],
                [9.0, 10.0, 11.0, 12.0],
            ],
        };
        let base_inv = mdxaBone_t {
            matrix: [
                [21.0, 22.0, 23.0, 24.0],
                [25.0, 26.0, 27.0, 28.0],
                [29.0, 30.0, 31.0, 32.0],
            ],
        };
        let buf = one_bone_mdxa(42, "Pelvis", base, base_inv);
        let v = unsafe { MdxaView::from_block(buf.as_ptr() as *const c_void) };
        assert_eq!(v.num_frames(), 42);
        assert_eq!(v.num_bones(), 1);
        let skel = v.skel(0);
        assert!(skel.name_matches("PELVIS"));
        assert!(!skel.name_matches("head"));
        assert_eq!(skel.name_lossy(), "Pelvis");
        assert_eq!(skel.parent(), -1);
        assert_eq!(skel.base_pose_mat(), base);
        assert_eq!(skel.base_pose_mat_inv(), base_inv);
    }

    #[test]
    fn parsed_and_ref_match_the_view() {
        let base = mdxaBone_t {
            matrix: [
                [1.0, 2.0, 3.0, 4.0],
                [5.0, 6.0, 7.0, 8.0],
                [9.0, 10.0, 11.0, 12.0],
            ],
        };
        let base_inv = mdxaBone_t {
            matrix: [
                [21.0, 22.0, 23.0, 24.0],
                [25.0, 26.0, 27.0, 28.0],
                [29.0, 30.0, 31.0, 32.0],
            ],
        };
        let buf = one_bone_mdxa(42, "Pelvis", base, base_inv);
        let view = unsafe { MdxaView::from_block(buf.as_ptr() as *const c_void) };
        let parsed = MdxaParsed::parse(view);
        let r = MdxaRef {
            parsed: &parsed,
            view,
        };
        assert_eq!(r.num_frames(), 42);
        assert_eq!(r.num_bones(), 1);
        let skel = r.skel(0);
        assert!(skel.name_matches("PELVIS"));
        assert!(!skel.name_matches("head"));
        assert_eq!(skel.name, "Pelvis");
        assert_eq!(skel.parent, -1);
        assert_eq!(skel.base_pose_mat, base);
        assert_eq!(skel.base_pose_mat_inv, base_inv);
    }
}
