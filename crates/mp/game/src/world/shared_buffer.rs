//! `SharedBuffer`: the module's engine-registered shared-memory region (`gSharedBuffer`), plus one typed overlay accessor per `T_G_ICARUS_*` command.
//!
//! The engine registers this buffer via `trap_SV_RegisterSharedMemory` (`G_InitGame`).
//! It writes each ICARUS command's `T_G_ICARUS_*` payload into the buffer before dispatching.
//! Raven's `vmMain` switch overlay-casts the raw `gSharedBuffer` to the command's struct.
//! It reads and writes fields in place (`(T_G_ICARUS_X *)gSharedBuffer`).
//! Module and engine address the same bytes, so out-params land back in the engine's view.
//! Each accessor below reproduces that one cast behind a typed `&mut`, and confines the overlay `unsafe` to the seam.
//!
//! Source: `oracle/codemp/game/g_local.h:85-86` (buffer decl),
//! `oracle/codemp/game/g_main.c:881` (definition),
//! `oracle/codemp/game/g_main.c:557-670` (overlay switch).

use core::ffi::c_char;

use crate::g_local_consts::MAX_G_SHARED_BUFFER_SIZE;

use mp_qshared::common::mp::qcommon::t_g_icarus_getfloat::T_G_ICARUS_GETFLOAT;
use mp_qshared::common::mp::qcommon::t_g_icarus_getsetidforstring::T_G_ICARUS_GETSETIDFORSTRING;
use mp_qshared::common::mp::qcommon::t_g_icarus_getstring::T_G_ICARUS_GETSTRING;
use mp_qshared::common::mp::qcommon::t_g_icarus_gettag::T_G_ICARUS_GETTAG;
use mp_qshared::common::mp::qcommon::t_g_icarus_getvector::T_G_ICARUS_GETVECTOR;
use mp_qshared::common::mp::qcommon::t_g_icarus_kill::T_G_ICARUS_KILL;
use mp_qshared::common::mp::qcommon::t_g_icarus_lerp2_angles::T_G_ICARUS_LERP2ANGLES;
use mp_qshared::common::mp::qcommon::t_g_icarus_lerp2_end::T_G_ICARUS_LERP2END;
use mp_qshared::common::mp::qcommon::t_g_icarus_lerp2_origin::T_G_ICARUS_LERP2ORIGIN;
use mp_qshared::common::mp::qcommon::t_g_icarus_lerp2_pos::T_G_ICARUS_LERP2POS;
use mp_qshared::common::mp::qcommon::t_g_icarus_lerp2_start::T_G_ICARUS_LERP2START;
use mp_qshared::common::mp::qcommon::t_g_icarus_play::T_G_ICARUS_PLAY;
use mp_qshared::common::mp::qcommon::t_g_icarus_playsound::T_G_ICARUS_PLAYSOUND;
use mp_qshared::common::mp::qcommon::t_g_icarus_remove::T_G_ICARUS_REMOVE;
use mp_qshared::common::mp::qcommon::t_g_icarus_set::T_G_ICARUS_SET;
use mp_qshared::common::mp::qcommon::t_g_icarus_soundindex::T_G_ICARUS_SOUNDINDEX;
use mp_qshared::common::mp::qcommon::t_g_icarus_use::T_G_ICARUS_USE;

/// Raven `char gSharedBuffer[MAX_G_SHARED_BUFFER_SIZE]`: the module's engine-registered shared-memory region.
///
/// `#[repr(C, align(8))]` makes every overlay cast's alignment guaranteed rather than incidental.
/// The old `Box<[u8; N]>` only guaranteed align 1, while `align(8)` covers every `T_G_ICARUS_*` payload's alignment (asserted below).
/// Source: `oracle/codemp/game/g_local.h:85-86`, `oracle/codemp/game/g_main.c:881`
#[repr(C, align(8))]
pub struct SharedBuffer([u8; MAX_G_SHARED_BUFFER_SIZE]);

// All-zero bytes are a valid `SharedBuffer` (`#[repr(C)]` over a `u8` array), so
// `native_platform::zeroed_box::<SharedBuffer>()` builds it on the heap.
unsafe impl native_platform::ZeroValid for SharedBuffer {}

impl SharedBuffer {
    /// Raw `*mut c_char` for `trap_SV_RegisterSharedMemory` (`G_InitGame`).
    /// The engine writes each command's payload here before dispatching.
    #[inline]
    pub fn as_registration_ptr(&mut self) -> *mut c_char {
        self.0.as_mut_ptr() as *mut c_char
    }
}

/// One typed overlay accessor per `T_G_ICARUS_*` command: `(T_G_ICARUS_X *) gSharedBuffer` behind a typed `&mut`.
/// Size and alignment of every payload are statically asserted (`assert_fits!` below), so the reinterpret is sound.
macro_rules! overlay_accessors {
    ($( $(#[$doc:meta])* $name:ident => $ty:ty ),+ $(,)?) => {
        impl SharedBuffer {
            $(
                $(#[$doc])*
                #[inline]
                pub fn $name(&mut self) -> &mut $ty {
                    // SAFETY: engine-written POD payload.
                    // Size and alignment are statically asserted below.
                    unsafe { &mut *(self.0.as_mut_ptr() as *mut $ty) }
                }
            )+
        }
    };
}

overlay_accessors! {
    /// `GAME_ICARUS_PLAYSOUND` payload (`g_main.c:558-562`).
    playsound => T_G_ICARUS_PLAYSOUND,
    /// `GAME_ICARUS_SET` payload (`g_main.c:563-567`).
    set => T_G_ICARUS_SET,
    /// `GAME_ICARUS_LERP2POS` payload (`g_main.c:568-580`).
    lerp2pos => T_G_ICARUS_LERP2POS,
    /// `GAME_ICARUS_LERP2ORIGIN` payload (`g_main.c:581-586`).
    lerp2origin => T_G_ICARUS_LERP2ORIGIN,
    /// `GAME_ICARUS_LERP2ANGLES` payload (`g_main.c:587-592`).
    lerp2angles => T_G_ICARUS_LERP2ANGLES,
    /// `GAME_ICARUS_GETTAG` payload (`g_main.c:593-597`).
    gettag => T_G_ICARUS_GETTAG,
    /// `GAME_ICARUS_LERP2START` payload (`g_main.c:598-603`).
    lerp2start => T_G_ICARUS_LERP2START,
    /// `GAME_ICARUS_LERP2END` payload (`g_main.c:604-609`).
    lerp2end => T_G_ICARUS_LERP2END,
    /// `GAME_ICARUS_USE` payload (`g_main.c:610-615`).
    use_cmd => T_G_ICARUS_USE,
    /// `GAME_ICARUS_KILL` payload (`g_main.c:616-621`).
    kill => T_G_ICARUS_KILL,
    /// `GAME_ICARUS_REMOVE` payload (`g_main.c:622-627`).
    remove => T_G_ICARUS_REMOVE,
    /// `GAME_ICARUS_PLAY` payload (`g_main.c:628-633`).
    play => T_G_ICARUS_PLAY,
    /// `GAME_ICARUS_GETFLOAT` payload (`g_main.c:634-638`).
    getfloat => T_G_ICARUS_GETFLOAT,
    /// `GAME_ICARUS_GETVECTOR` payload (`g_main.c:639-643`).
    getvector => T_G_ICARUS_GETVECTOR,
    /// `GAME_ICARUS_GETSTRING` payload (`g_main.c:644-658`).
    getstring => T_G_ICARUS_GETSTRING,
    /// `GAME_ICARUS_SOUNDINDEX` payload (`g_main.c:659-664`).
    soundindex => T_G_ICARUS_SOUNDINDEX,
    /// `GAME_ICARUS_GETSETIDFORSTRING` payload (`g_main.c:665-669`).
    getsetidforstring => T_G_ICARUS_GETSETIDFORSTRING,
}

// Every overlay payload must fit within the buffer and be no more aligned than the buffer, or the reinterpret in each accessor would be unsound.
// Const asserts and fully-qualified paths are exempt from the import rule.
macro_rules! assert_fits {
    ($($ty:ty),+ $(,)?) => {
        $(
            const _: () = assert!(core::mem::size_of::<$ty>() <= MAX_G_SHARED_BUFFER_SIZE);
            const _: () =
                assert!(core::mem::align_of::<$ty>() <= core::mem::align_of::<SharedBuffer>());
        )+
    };
}

assert_fits! {
    T_G_ICARUS_PLAYSOUND,
    T_G_ICARUS_SET,
    T_G_ICARUS_LERP2POS,
    T_G_ICARUS_LERP2ORIGIN,
    T_G_ICARUS_LERP2ANGLES,
    T_G_ICARUS_GETTAG,
    T_G_ICARUS_LERP2START,
    T_G_ICARUS_LERP2END,
    T_G_ICARUS_USE,
    T_G_ICARUS_KILL,
    T_G_ICARUS_REMOVE,
    T_G_ICARUS_PLAY,
    T_G_ICARUS_GETFLOAT,
    T_G_ICARUS_GETVECTOR,
    T_G_ICARUS_GETSTRING,
    T_G_ICARUS_SOUNDINDEX,
    T_G_ICARUS_GETSETIDFORSTRING,
}
