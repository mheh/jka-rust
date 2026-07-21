//! `mdxmHeader_t` view over the `.glm` block.
//!
//! Source: `oracle/codemp/renderer/mdx_format.h:151-172`

use core::ffi::c_void;
use core::slice;

use mp_qshared::shared::MAX_QPATH;

// mdxmHeader_t offsets (`mdx_format.h:153-172`). MAX_QPATH == 64.
const OFS_ANIM_INDEX: usize = 4 + 4 + MAX_QPATH + MAX_QPATH; // ident,version,name[],animName[]
const OFS_END: usize = OFS_ANIM_INDEX + 4 + 4 + 4 + 4 + 4 + 4; // ..numBones,numLODs,ofsLODs,numSurfaces,ofsSurfHierarchy,ofsEnd

/// View over a `.glm` `mdxmHeader_t` block.
#[derive(Clone, Copy)]
pub struct MdxmView<'a> {
    bytes: &'a [u8],
}

impl MdxmView<'_> {
    /// # Safety
    /// `ptr` is a non-null `EngineHost::model_mdxm` block (self-sized by its
    /// `ofsEnd` field).
    pub unsafe fn from_block(ptr: *const c_void) -> Self {
        let base = ptr as *const u8;
        let ofs_end = unsafe { base.add(OFS_END).cast::<i32>().read_unaligned() };
        Self { bytes: unsafe { slice::from_raw_parts(base, ofs_end as usize) } }
    }

    /// `mdxmHeader_t->animIndex`.
    pub fn anim_index(&self) -> i32 {
        i32::from_le_bytes(self.bytes[OFS_ANIM_INDEX..OFS_ANIM_INDEX + 4].try_into().unwrap())
    }
}
