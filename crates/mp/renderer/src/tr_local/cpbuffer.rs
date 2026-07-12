#![allow(non_camel_case_types, non_snake_case)]
use core::ffi::c_int;

/// Raven `CPBUFFER` — Win32 WGL pixel-buffer render target (render/device
/// contexts, dimensions, and the display texture).
///
/// Raven: Pixel Buffer Rendering and Device Contexts.
/// Type definition source: `oracle/codemp/renderer/tr_local.h:1156-1197`
#[repr(C)]
pub struct CPBUFFER {
    // Pixel Buffer Rendering and Device Contexts.
    // HGLRC/HDC/HPBUFFERARB are win32 opaque handles (pointer-sized); they are
    // platform types, not Raven types, so they stay `*mut c_void` permanently.
    pub m_hRC: *mut core::ffi::c_void,
    pub m_hDC: *mut core::ffi::c_void,

    // The render and device contexts for the previous render target.
    pub m_hOldRC: *mut core::ffi::c_void,
    pub m_hOldDC: *mut core::ffi::c_void,

    // Buffer handle.
    pub m_hBuffer: *mut core::ffi::c_void,

    // Buffer Dimensions.
    pub m_iWidth: c_int,
    pub m_iHeight: c_int,

    // Color, depth, and stencil bits for this buffer.
    pub m_iColorBits: c_int,
    pub m_iDepthBits: c_int,
    pub m_iStencilBits: c_int,

    // Texture used for displaying the pbuffer.
    pub m_uiPBufferTexture: u32,
}

const _: () = assert!(core::mem::offset_of!(CPBUFFER, m_hRC) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<CPBUFFER>() == 64);
    assert!(core::mem::offset_of!(CPBUFFER, m_hDC) == 8);
    assert!(core::mem::offset_of!(CPBUFFER, m_hOldRC) == 16);
    assert!(core::mem::offset_of!(CPBUFFER, m_hOldDC) == 24);
    assert!(core::mem::offset_of!(CPBUFFER, m_hBuffer) == 32);
    assert!(core::mem::offset_of!(CPBUFFER, m_iWidth) == 40);
    assert!(core::mem::offset_of!(CPBUFFER, m_iHeight) == 44);
    assert!(core::mem::offset_of!(CPBUFFER, m_iColorBits) == 48);
    assert!(core::mem::offset_of!(CPBUFFER, m_iDepthBits) == 52);
    assert!(core::mem::offset_of!(CPBUFFER, m_iStencilBits) == 56);
    assert!(core::mem::offset_of!(CPBUFFER, m_uiPBufferTexture) == 60);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<CPBUFFER>() == 44);
    assert!(core::mem::offset_of!(CPBUFFER, m_hDC) == 4);
    assert!(core::mem::offset_of!(CPBUFFER, m_hOldRC) == 8);
    assert!(core::mem::offset_of!(CPBUFFER, m_hOldDC) == 12);
    assert!(core::mem::offset_of!(CPBUFFER, m_hBuffer) == 16);
    assert!(core::mem::offset_of!(CPBUFFER, m_iWidth) == 20);
    assert!(core::mem::offset_of!(CPBUFFER, m_iHeight) == 24);
    assert!(core::mem::offset_of!(CPBUFFER, m_iColorBits) == 28);
    assert!(core::mem::offset_of!(CPBUFFER, m_iDepthBits) == 32);
    assert!(core::mem::offset_of!(CPBUFFER, m_iStencilBits) == 36);
    assert!(core::mem::offset_of!(CPBUFFER, m_uiPBufferTexture) == 40);
};
