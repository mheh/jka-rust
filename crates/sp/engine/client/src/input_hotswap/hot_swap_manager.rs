#![allow(non_camel_case_types, non_snake_case)]

/// Raven `HotSwapManager` — tracks a single hot-swap button's down/bind state
/// so a held force/weapon/item-select button can bind and later execute a
/// selection without needing a separate keybind per slot.
///
/// Type definition source: `oracle/code/client/cl_input_hotswap.h:13-56`
#[repr(C)]
pub struct HotSwapManager {
    /// Is the button down?
    down: bool,
    /// Don't execute the button's bind.
    noExec: bool,
    /// Don't bind the button.
    noBind: bool,
    /// Is a force power currently bound?
    forceBound: bool,
    /// How long the button has been held down.
    downTime: i32,
    /// How long the button has been down with the selection up.
    bindTime: i32,
    /// Unique ID for this button.
    uniqueID: i32,
}

const _: () = assert!(core::mem::size_of::<HotSwapManager>() == 16);
const _: () = assert!(core::mem::offset_of!(HotSwapManager, down) == 0);
const _: () = assert!(core::mem::offset_of!(HotSwapManager, noExec) == 1);
const _: () = assert!(core::mem::offset_of!(HotSwapManager, noBind) == 2);
const _: () = assert!(core::mem::offset_of!(HotSwapManager, forceBound) == 3);
const _: () = assert!(core::mem::offset_of!(HotSwapManager, downTime) == 4);
const _: () = assert!(core::mem::offset_of!(HotSwapManager, bindTime) == 8);
const _: () = assert!(core::mem::offset_of!(HotSwapManager, uniqueID) == 12);
