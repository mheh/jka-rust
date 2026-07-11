#![allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_assignments,
    unused_parens,
    clippy::too_many_arguments
)]

//! MP botlib `l_memory.cpp` — the botlib memory-allocation wrappers that
//! stamp a magic tag ahead of the returned block and hand off to the
//! `botimport` allocator/free callbacks.
//!
//! Source: `oracle/codemp/botlib/l_memory.cpp`

use core::ffi::c_ulong;

use crate::l_memory::memory_consts::{HUNK_ID, MEM_ID};
use crate::BotLib;

use mp_engine_qcommon::common_fns::Com_Memset;

/// Raven `GetMemory`.
///
/// Source: `oracle/codemp/botlib/l_memory.cpp:325-336`
pub fn GetMemory(bot: &mut BotLib, size: c_ulong) -> *mut () {
    unsafe {
        let ptr = bot.botimport.GetMemory.unwrap()(
            (size + core::mem::size_of::<c_ulong>() as c_ulong) as core::ffi::c_int,
        );
        if ptr.is_null() {
            return core::ptr::null_mut();
        }
        let memid = ptr as *mut c_ulong;
        *memid = MEM_ID as c_ulong;
        (ptr as *mut u8).add(core::mem::size_of::<c_ulong>()) as *mut ()
    }
}

/// Raven `GetHunkMemory`.
///
/// Source: `oracle/codemp/botlib/l_memory.cpp:367-378`
pub fn GetHunkMemory(bot: &mut BotLib, size: c_ulong) -> *mut () {
    unsafe {
        let ptr = bot.botimport.HunkAlloc.unwrap()(
            (size + core::mem::size_of::<c_ulong>() as c_ulong) as core::ffi::c_int,
        );
        if ptr.is_null() {
            return core::ptr::null_mut();
        }
        let memid = ptr as *mut c_ulong;
        *memid = HUNK_ID as c_ulong;
        (ptr as *mut u8).add(core::mem::size_of::<c_ulong>()) as *mut ()
    }
}

/// Raven `FreeMemory`.
///
/// Source: `oracle/codemp/botlib/l_memory.cpp:406-416`
pub fn FreeMemory(bot: &mut BotLib, ptr: *mut ()) {
    unsafe {
        let memid = (ptr as *mut u8).sub(core::mem::size_of::<c_ulong>()) as *mut c_ulong;

        if *memid == MEM_ID as c_ulong {
            bot.botimport.FreeMemory.unwrap()(memid as *mut core::ffi::c_void);
        } //end if
    }
}

/// Raven `AvailableMemory`.
///
/// Source: `oracle/codemp/botlib/l_memory.cpp:423-426`
pub fn AvailableMemory(bot: &mut BotLib) -> core::ffi::c_int {
    unsafe { bot.botimport.AvailableMemory.unwrap()() }
}

/// Raven `PrintUsedMemorySize`.
///
/// Source: `oracle/codemp/botlib/l_memory.cpp:433-435`
pub fn PrintUsedMemorySize() {}

/// Raven `PrintMemoryLabels`.
///
/// Source: `oracle/codemp/botlib/l_memory.cpp:442-444`
pub fn PrintMemoryLabels() {}

/// Raven `GetClearedMemory`.
///
/// Source: `oracle/codemp/botlib/l_memory.cpp:346-357`
pub fn GetClearedMemory(bot: &mut BotLib, size: c_ulong) -> *mut () {
    let ptr = GetMemory(bot, size);
    Com_Memset(ptr, 0, size as usize);
    ptr
}

/// Raven `GetClearedHunkMemory`.
///
/// Source: `oracle/codemp/botlib/l_memory.cpp:388-399`
pub fn GetClearedHunkMemory(bot: &mut BotLib, size: c_ulong) -> *mut () {
    let ptr = GetHunkMemory(bot, size);
    Com_Memset(ptr, 0, size as usize);
    ptr
}
