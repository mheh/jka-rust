#![allow(non_snake_case, non_camel_case_types, clippy::too_many_arguments)]
//! `vm.cpp` — the QVM module loader/interpreter dispatch (symbols, syscall
//! shuffle, alloc/free, profiling, `vmprofile`/`vminfo` console commands).
//!
//! DESTINATION NOTE: `vm.cpp`'s stem collides with the existing `vm/`
//! directory module, so this file lands at the `_fns` escape per
//! `_PREAMBLE.md`'s destination rule.
//!
//! Source: `oracle/codemp/qcommon/vm.cpp`

use core::ffi::{c_char, c_int};

use mp_host_interface::engine_host::EngineHost;
use mp_qshared::common::mp::qcommon::tags::memtag_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::ha_pref;
use mp_qshared::shared::limits::MAX_TOKEN_CHARS;
use native_types::{qfalse, qtrue, MAX_QPATH};

use crate::common::engine_host_view::EngineHostView;
use crate::common::Common;
use crate::qcommon::vm_interpret_t::vmInterpret_t;
use crate::qfiles::vm_header_t::vmHeader_t;
use crate::vm::module_registry::MAX_VM;
use crate::vm::vm_s::vm_t;
use crate::vm::vm_symbol_s::vmSymbol_t;
use crate::vm::vmptr_t::vmptr_t;

use crate::cmd::Cmd_AddCommand;
use crate::cvar_fns::Cvar_VariableValue;
use crate::sys_engine::Sys_LoadDll;
use crate::z_memman_pc::Hunk_Alloc;
use crate::z_memman_pc::{Z_Free, Z_Malloc};
use mp_qshared::shared::q_string::{COM_Parse, COM_StripExtension};
use mp_qshared::shared::swap::LittleLong;
use native_platform::Sys_UnloadDll;

/// `VM_VM2C`.
///
/// Raven: on native-dylib builds `vmptr_t` already IS the host pointer, so the
/// "VM pointer to C pointer" shift is a no-op cast.
/// Source: `oracle/codemp/qcommon/vm.cpp:37-39`
pub fn VM_VM2C(p: vmptr_t, _length: c_int) -> *mut () {
    p as *mut ()
}

/// `VM_Debug`.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:41-43`
pub fn VM_Debug(common: &mut Common, level: c_int) {
    common.vm_debugLevel = level;
}

/// `VM_ValueToSymbol`.
///
/// Raven: `CRAZY_SYMBOL_MAP` is never defined in this build (dead `#ifdef`
/// branch) — the always-compiled linear-scan path is transcribed; the
/// `g_symbolMap` fast-path cache is intentionally unreachable here, matching
/// retail.
/// Source: `oracle/codemp/qcommon/vm.cpp:70-104`
pub fn VM_ValueToSymbol(common: &mut Common, vm: *mut vm_t, value: c_int) -> *const c_char {
    unsafe {
        let mut sym = (*vm).symbols;
        if sym.is_null() {
            return c"NO SYMBOLS".as_ptr();
        }

        // find the symbol
        while !(*sym).next.is_null() && (*(*sym).next).symValue <= value {
            sym = (*sym).next;
        }

        if value == (*sym).symValue {
            return (*sym).symName.as_ptr();
        }

        let name = std::ffi::CStr::from_ptr((*sym).symName.as_ptr()).to_string_lossy();
        let text = format!("{}+{}", name, value - (*sym).symValue);
        let bytes = text.as_bytes();
        let n = bytes.len().min(MAX_TOKEN_CHARS - 1);
        for (i, b) in bytes[..n].iter().enumerate() {
            common.vm_value_to_symbol_buf[i] = *b as c_char;
        }
        common.vm_value_to_symbol_buf[n] = 0;
        common.vm_value_to_symbol_buf.as_ptr()
    }
}

/// `VM_ValueToFunctionSymbol`.
///
/// Raven: `CRAZY_SYMBOL_MAP` is never defined in this build (dead `#ifdef`
/// branch); the always-compiled linear-scan path is transcribed.
/// Source: `oracle/codemp/qcommon/vm.cpp:113-140`
pub fn VM_ValueToFunctionSymbol(
    common: &mut Common,
    vm: *mut vm_t,
    value: c_int,
) -> *mut vmSymbol_t {
    unsafe {
        let mut sym = (*vm).symbols;
        if sym.is_null() {
            // Raven's `static vmSymbol_t nullSym` (a zero-valued sentinel) —
            // the three-kind rule's rotating/cross-frame case: hosted on
            // `Common` since the caller receives a persistent pointer.
            return &mut common.vm_value_to_function_symbol_null_sym as *mut vmSymbol_t;
        }

        while !(*sym).next.is_null() && (*(*sym).next).symValue <= value {
            sym = (*sym).next;
        }

        sym
    }
}

/// `VM_SymbolToValue`.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:148-157`
pub fn VM_SymbolToValue(vm: *mut vm_t, symbol: *const c_char) -> c_int {
    unsafe {
        let mut sym = (*vm).symbols;
        while !sym.is_null() {
            if libc::strcmp(symbol, (*sym).symName.as_ptr()) == 0 {
                return (*sym).symValue;
            }
            sym = (*sym).next;
        }
    }
    0
}

/// `ParseHex`.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:197-218`
pub fn ParseHex(text: *const c_char) -> c_int {
    let mut value: c_int = 0;
    unsafe {
        let mut p = text;
        loop {
            let c = *p as c_int;
            p = p.add(1);
            if c == 0 {
                break;
            }
            if (b'0' as c_int..=b'9' as c_int).contains(&c) {
                value = value * 16 + c - b'0' as c_int;
                continue;
            }
            if (b'a' as c_int..=b'f' as c_int).contains(&c) {
                value = value * 16 + 10 + c - b'a' as c_int;
                continue;
            }
            if (b'A' as c_int..=b'F' as c_int).contains(&c) {
                value = value * 16 + 10 + c - b'A' as c_int;
                continue;
            }
        }
    }
    value
}

/// `VM_DllSyscall`.
///
/// Raven: the `__linux__ && __powerpc__` variadic-shuffle branch is dead on
/// every host this port targets; the `#else` "original id code" path
/// (`return currentVM->systemCall( &arg )`) is the always-compiled one.
/// Source: `oracle/codemp/qcommon/vm.cpp:363-381`
pub fn VM_DllSyscall(common: &mut Common, arg: c_int) -> c_int {
    unsafe {
        if common.currentVM.is_null() {
            // §19: Raven derefs `currentVM` unconditionally here; guarding a
            // null pointer instead of UB.
            return 0;
        }
        match (*common.currentVM).systemCall {
            Some(f) => f(&arg as *const c_int as *mut c_int),
            None => 0,
        }
    }
}

/// `VM_ArgPtr`.
///
/// `common` is a shared borrow: the body only reads `currentVM` (Raven's
/// file-scope global), and the dispatcher call sites resolve `VMA(n)` args
/// while `common` is reserved mutably for the outer call.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:640-654`
pub fn VM_ArgPtr(common: &Common, intValue: c_int) -> *mut () {
    if intValue == 0 {
        return core::ptr::null_mut();
    }
    // bk001220 - currentVM is missing on reconnect
    if common.currentVM.is_null() {
        return core::ptr::null_mut();
    }
    unsafe {
        if !(*common.currentVM).entryPoint.is_none() {
            ((*common.currentVM).dataBase as isize + intValue as isize) as *mut ()
        } else {
            ((*common.currentVM).dataBase as isize
                + (intValue & (*common.currentVM).dataMask) as isize) as *mut ()
        }
    }
}

/// `VM_ArgPtr` at the AbiWord-widened width — the inbound dispatcher's `VMA(n)`
/// resolves arg words as `isize` (state-ownership.md, vmMain pair), so the
/// value is used at full width instead of being truncated to `c_int`. The
/// narrow [`VM_ArgPtr`] original is kept untouched for its other callers.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:640-654`
pub fn VM_ArgPtrWord(common: &Common, value: isize) -> *mut () {
    if value == 0 {
        return core::ptr::null_mut();
    }
    // bk001220 - currentVM is missing on reconnect
    if common.currentVM.is_null() {
        return core::ptr::null_mut();
    }
    unsafe {
        if !(*common.currentVM).entryPoint.is_none() {
            ((*common.currentVM).dataBase as isize + value) as *mut ()
        } else {
            ((*common.currentVM).dataBase as isize
                + (value & (*common.currentVM).dataMask as isize)) as *mut ()
        }
    }
}

/// `BotVMShift`.
///
/// Raven: `gvm` — "always using the game vm here."
///
/// Raven's `int ptr` is 32-bit-era; on LP64 the native-dll arm carries a full
/// pointer word (game waypoint addresses), so `isize`.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:657-677`
pub fn BotVMShift(gvm: *mut vm_t, ptr: isize) -> *mut () {
    if ptr == 0 {
        return core::ptr::null_mut();
    }
    if gvm.is_null() {
        return core::ptr::null_mut();
    }
    unsafe {
        if !(*gvm).entryPoint.is_none() {
            ((*gvm).dataBase as isize + ptr) as *mut ()
        } else {
            ((*gvm).dataBase as isize + (ptr & (*gvm).dataMask as isize)) as *mut ()
        }
    }
}

/// `VM_ExplicitArgPtr`.
///
/// Raven's `int intValue` is 32-bit-era; on LP64 the native-dll arm carries a
/// full pointer word (the `GAME_CLIENT_CONNECT` denied string), so `isize`.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:742-758`
pub fn VM_ExplicitArgPtr(common: &mut Common, vm: *mut vm_t, intValue: isize) -> *mut () {
    if intValue == 0 {
        return core::ptr::null_mut();
    }
    // bk010124 - currentVM is missing on reconnect here as well?
    if common.currentVM.is_null() {
        return core::ptr::null_mut();
    }
    unsafe {
        if !(*vm).entryPoint.is_none() {
            ((*vm).dataBase as isize + intValue) as *mut ()
        } else {
            ((*vm).dataBase as isize + (intValue & (*vm).dataMask as isize)) as *mut ()
        }
    }
}

/// `VM_ProfileSort`.
///
/// Raven: `qsort` comparator over `vmSymbol_t *` entries by `profileCount`.
/// Source: `oracle/codemp/qcommon/vm.cpp:833-846`
pub fn VM_ProfileSort(a: *const (), b: *const ()) -> c_int {
    unsafe {
        let sa = *(a as *const *mut vmSymbol_t);
        let sb = *(b as *const *mut vmSymbol_t);
        if (*sa).profileCount < (*sb).profileCount {
            return -1;
        }
        if (*sa).profileCount > (*sb).profileCount {
            return 1;
        }
    }
    0
}

/// `VM_LogSyscalls`.
///
/// Raven: `callnum`/`f` are genuine cross-frame state (fork-3 case 3) —
/// hosted on `Common` with Raven's verbatim names.
/// Source: `oracle/codemp/qcommon/vm.cpp:934-944`
pub fn VM_LogSyscalls(common: &mut Common, args: *mut c_int) {
    unsafe {
        if common.vm_log_syscalls_f.is_null() {
            common.vm_log_syscalls_f = libc::fopen(c"syscalls.log".as_ptr(), c"w".as_ptr());
        }
        common.vm_log_syscalls_callnum += 1;
        libc::fprintf(
            common.vm_log_syscalls_f,
            c"%i: %i (%i) = %i %i %i %i\n".as_ptr(),
            common.vm_log_syscalls_callnum,
            (args as isize - (*common.currentVM).dataBase as isize) / 4,
            *args.offset(0),
            *args.offset(1),
            *args.offset(2),
            *args.offset(3),
            *args.offset(4),
        );
    }
}

/// `VM_SymbolForCompiledPointer`.
///
/// Raven: `CRAZY_SYMBOL_MAP` is never defined in this build; the
/// `VM_SetSymbolMap` fast-path is intentionally unreachable here.
/// Source: `oracle/codemp/qcommon/vm.cpp:165-188`
pub fn VM_SymbolForCompiledPointer(
    common: &mut Common,
    vm: *mut vm_t,
    code: *mut (),
) -> *const c_char {
    unsafe {
        if (code as usize) < (*vm).codeBase as usize {
            return c"Before code block".as_ptr();
        }
        if code as usize >= (*vm).codeBase as usize + (*vm).codeLength as usize {
            return c"After code block".as_ptr();
        }

        // find which original instruction it is after
        let mut i: c_int = 0;
        while i < (*vm).codeLength {
            if !(*vm).instructionPointers.is_null()
                && (*(*vm).instructionPointers.offset(i as isize)) as usize > code as usize
            {
                break;
            }
            i += 1;
        }
        i -= 1;

        VM_ValueToSymbol(common, vm, i)
    }
}

/// `VM_Free`.
///
/// Raven: the `#if 0` manual-free block is dead (hunk auto-frees); only the
/// dll-unload + zero-out path is live.
/// Source: `oracle/codemp/qcommon/vm.cpp:605-626`
pub fn VM_Free(common: &mut Common, vm: *mut vm_t) {
    unsafe {
        if !(*vm).dllHandle.is_null() {
            Sys_UnloadDll((*vm).dllHandle);
            *vm = core::mem::zeroed();
        }

        *vm = core::mem::zeroed();
    }

    common.currentVM = core::ptr::null_mut();
    common.lastVM = core::ptr::null_mut();
}

/// `VM_Clear`.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:628-638`
pub fn VM_Clear(common: &mut Common) {
    for i in 0..MAX_VM {
        unsafe {
            if !common.vmTable[i].dllHandle.is_null() {
                Sys_UnloadDll(common.vmTable[i].dllHandle);
            }
            common.vmTable[i] = core::mem::zeroed();
        }
    }
    common.currentVM = core::ptr::null_mut();
    common.lastVM = core::ptr::null_mut();
}

/// `VM_Shifted_Alloc`.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:679-717`
pub fn VM_Shifted_Alloc(view: &mut EngineHostView, ptr: *mut *mut (), size: c_int) {
    unsafe {
        if view.common.currentVM.is_null() {
            debug_assert!(false);
            *ptr = core::ptr::null_mut();
            return;
        }

        // first allocate our desired memory, up front
        let mem = Z_Malloc(view, size + 1, memtag_t::TAG_VM_ALLOCATED, qfalse, 0);

        if mem.is_null() {
            debug_assert!(false);
            *ptr = core::ptr::null_mut();
            return;
        }

        core::ptr::write_bytes(mem as *mut u8, 0, (size + 1) as usize);

        // Alright, subtract the database from the memory pointer to get a
        // memory address relative to the VM. When the VM modifies it it
        // should be modifying the same chunk of memory we have allocated in
        // the engine.
        *ptr = (mem as isize - (*view.common.currentVM).dataBase as isize) as *mut ();
    }
}

/// `VM_Shifted_Free`.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:719-740`
pub fn VM_Shifted_Free(common: &mut Common, ptr: *mut *mut ()) {
    unsafe {
        if common.currentVM.is_null() {
            debug_assert!(false);
            return;
        }

        // Shift the VM memory pointer back to get the same pointer we
        // initially allocated in real memory space.
        let mem = ((*common.currentVM).dataBase as isize + (*ptr) as isize) as *mut ();

        if mem.is_null() {
            debug_assert!(false);
            return;
        }

        Z_Free(common, mem);
        *ptr = core::ptr::null_mut(); // go ahead and clear the pointer for the game.
    }
}

/// `VM_VmProfile_f`.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:854-893`
pub fn VM_VmProfile_f(view: &mut EngineHostView) {
    unsafe {
        if view.common.lastVM.is_null() {
            return;
        }

        let vm = view.common.lastVM;

        if (*vm).numSymbols == 0 {
            return;
        }

        let sorted = Z_Malloc(
            view,
            ((*vm).numSymbols as usize * core::mem::size_of::<*mut vmSymbol_t>()) as c_int,
            memtag_t::TAG_VM,
            qtrue,
            0,
        ) as *mut *mut vmSymbol_t;

        *sorted.offset(0) = (*vm).symbols;
        let mut total: f64 = (*(*sorted.offset(0))).profileCount as f64;
        for i in 1..(*vm).numSymbols as isize {
            *sorted.offset(i) = (*(*sorted.offset(i - 1))).next;
            total += (*(*sorted.offset(i))).profileCount as f64;
        }

        libc::qsort(
            sorted as *mut core::ffi::c_void,
            (*vm).numSymbols as usize,
            core::mem::size_of::<*mut vmSymbol_t>(),
            Some(core::mem::transmute::<
                fn(*const (), *const ()) -> c_int,
                unsafe extern "C" fn(*const core::ffi::c_void, *const core::ffi::c_void) -> c_int,
            >(VM_ProfileSort)),
        );

        for i in 0..(*vm).numSymbols as isize {
            let sym = *sorted.offset(i);
            let perc = (100.0 * (*sym).profileCount as f32 / total as f32) as c_int;
            let name = std::ffi::CStr::from_ptr((*sym).symName.as_ptr()).to_string_lossy();
            view.print(&format!("{perc:2}% {:9} {name}\n", (*sym).profileCount));
            (*sym).profileCount = 0;
        }

        view.print(&format!("    {total:9.0} total\n"));

        Z_Free(view.common, sorted as *mut ());
    }
}

/// `VM_VmInfo_f`.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:901-925`
pub fn VM_VmInfo_f(common: &mut Common) {
    unsafe {
        crate::common::com_printf(common, "Registered virtual machines:\n");
        for i in 0..MAX_VM {
            let vm = &common.vmTable[i] as *const vm_t;
            if (*vm).name[0] == 0 {
                break;
            }
            let name = std::ffi::CStr::from_ptr((*vm).name.as_ptr()).to_string_lossy();
            crate::common::com_printf(common, &format!("{name} : "));
            if !(*vm).dllHandle.is_null() {
                crate::common::com_printf(common, "native\n");
                continue;
            }
            if (*vm).compiled != qfalse {
                crate::common::com_printf(common, "compiled on load\n");
            } else {
                crate::common::com_printf(common, "interpreted\n");
            }
            crate::common::com_printf(
                common,
                &format!("    code length : {:7}\n", (*vm).codeLength),
            );
            crate::common::com_printf(
                common,
                &format!("    table length: {:7}\n", (*vm).instructionPointersLength),
            );
            crate::common::com_printf(
                common,
                &format!("    data length : {:7}\n", (*vm).dataMask + 1),
            );
        }
    }
}

/// `VM_Init`.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:50-61`
pub fn VM_Init(view: &mut EngineHostView) {
    // default to DLLs now instead. Our VMs are getting too HUGE.
    view.cvar_register(
        "vm_cgame",
        "0",
        (mp_qshared::shared::cvar::CVAR_SYSTEMINFO | mp_qshared::shared::cvar::CVAR_ARCHIVE)
            as c_int,
    );
    view.cvar_register(
        "vm_game",
        "0",
        (mp_qshared::shared::cvar::CVAR_SYSTEMINFO | mp_qshared::shared::cvar::CVAR_ARCHIVE)
            as c_int,
    );
    view.cvar_register(
        "vm_ui",
        "0",
        (mp_qshared::shared::cvar::CVAR_SYSTEMINFO | mp_qshared::shared::cvar::CVAR_ARCHIVE)
            as c_int,
    );
    // client wants to know if the server is using vm's for certain modules,
    // so if pure we can force the same method (be it vm or dll) -rww

    Cmd_AddCommand(view, "vmprofile", Some(|view| VM_VmProfile_f(view)));
    Cmd_AddCommand(view, "vminfo", Some(|view| VM_VmInfo_f(view.common)));

    view.common.vmTable = unsafe { core::mem::zeroed() };
}

/// `VM_Alloc`.
///
/// Raven: the `Z_Malloc` alternative is commented out — `Hunk_Alloc` is the
/// live path.
/// Source: `oracle/codemp/qcommon/vm.cpp:227-231`
pub fn VM_Alloc(view: &mut EngineHostView, size: c_int) -> *mut () {
    Hunk_Alloc(view, size, ha_pref::h_high)
}

/// `VM_LoadSymbols`.
///
/// Raven: `CRAZY_SYMBOL_MAP` is never defined in this build; the
/// `g_symbolMap` write-through cache is intentionally unreachable here.
/// Source: `oracle/codemp/qcommon/vm.cpp:238-323`
pub fn VM_LoadSymbols(view: &mut EngineHostView, vm: *mut vm_t) {
    // don't load symbols if not developer
    if view.cvar_integer("developer") == 0 {
        return;
    }

    unsafe {
        let mut name = [0u8; MAX_QPATH as usize];
        let vm_name = std::ffi::CStr::from_ptr((*vm).name.as_ptr()).to_string_lossy();
        let stripped = COM_StripExtension(&vm_name);
        let n = stripped.len().min(name.len() - 1);
        name[..n].copy_from_slice(&stripped.as_bytes()[..n]);

        let symbols = format!("vm/{}.map", String::from_utf8_lossy(&name[..n]));

        let mapfile = match view.fs_read_file(&symbols) {
            Some(bytes) => bytes,
            None => {
                view.print(&format!("Couldn't load symbol file: {symbols}\n"));
                return;
            }
        };

        let numInstructions = (*vm).instructionPointersLength >> 2;

        // parse the symbols
        let text = String::from_utf8_lossy(&mapfile).into_owned();
        let mut cursor = text.as_str();
        let mut prev: *mut *mut vmSymbol_t = &mut (*vm).symbols;
        let mut count: c_int = 0;

        loop {
            let (token, rest) = COM_Parse(cursor);
            cursor = rest;
            if token.is_empty() {
                break;
            }
            let token_c = std::ffi::CString::new(token.clone()).unwrap();
            let segment = ParseHex(token_c.as_ptr());
            if segment != 0 {
                let (_, rest) = COM_Parse(cursor);
                cursor = rest;
                let (_, rest) = COM_Parse(cursor);
                cursor = rest;
                continue; // only load code segment values
            }

            let (token, rest) = COM_Parse(cursor);
            cursor = rest;
            if token.is_empty() {
                view.print("WARNING: incomplete line at end of file\n");
                break;
            }
            let token_c = std::ffi::CString::new(token).unwrap();
            let mut value = ParseHex(token_c.as_ptr());

            let (token, rest) = COM_Parse(cursor);
            cursor = rest;
            if token.is_empty() {
                view.print("WARNING: incomplete line at end of file\n");
                break;
            }
            let chars = token.len();

            let sym = VM_Alloc(view, (core::mem::size_of::<vmSymbol_t>() + chars) as c_int)
                as *mut vmSymbol_t;
            *prev = sym;
            prev = &mut (*sym).next;
            (*sym).next = core::ptr::null_mut();

            // convert value from an instruction number to a code offset
            if value >= 0 && value < numInstructions {
                value = *(*vm).instructionPointers.offset(value as isize);
            }

            (*sym).symValue = value;
            let name_bytes = token.as_bytes();
            let dst = core::slice::from_raw_parts_mut((*sym).symName.as_mut_ptr(), chars + 1);
            for (i, b) in name_bytes.iter().enumerate() {
                dst[i] = *b as c_char;
            }
            dst[chars] = 0;

            count += 1;
        }

        (*vm).numSymbols = count;
        view.print(&format!("{count} symbols parsed from {symbols}\n"));
    }
}

/// `VM_Create`.
///
/// Raven: `systemCalls`'s C fn-pointer type isn't in the rosetta
/// (escalated); ported as an `Option<extern "C" fn(*mut c_int) -> c_int>`
/// matching `vm_t::systemCall`'s already-landed shape.
/// Source: `oracle/codemp/qcommon/vm.cpp:471-597`
pub fn VM_Create(
    view: &mut EngineHostView,
    module: &str,
    systemCalls: Option<extern "C" fn(*mut c_int) -> c_int>,
    mut interpret: vmInterpret_t,
) -> *mut vm_t {
    unsafe {
        if module.is_empty() || systemCalls.is_none() {
            view.error(errorParm_t::ERR_FATAL, "VM_Create: bad parms");
        }

        // see if we already have the VM
        for i in 0..MAX_VM {
            let name =
                std::ffi::CStr::from_ptr(view.common.vmTable[i].name.as_ptr()).to_string_lossy();
            if name.eq_ignore_ascii_case(module) {
                return &mut view.common.vmTable[i] as *mut vm_t;
            }
        }

        // find a free vm
        let mut i = 0;
        while i < MAX_VM {
            if view.common.vmTable[i].name[0] == 0 {
                break;
            }
            i += 1;
        }

        if i == MAX_VM {
            view.error(errorParm_t::ERR_FATAL, "VM_Create: no free vm_t");
        }

        let vm = &mut view.common.vmTable[i] as *mut vm_t;

        let name_bytes = module.as_bytes();
        let n = name_bytes.len().min((*vm).name.len() - 1);
        for (j, b) in name_bytes[..n].iter().enumerate() {
            (*vm).name[j] = *b as c_char;
        }
        (*vm).name[n] = 0;
        (*vm).systemCall = systemCalls;

        // never allow dll loading with a demo
        if interpret == vmInterpret_t::VMI_NATIVE {
            if Cvar_VariableValue(view, "fs_restrict") != 0.0 {
                interpret = vmInterpret_t::VMI_COMPILED;
            }
        }

        if interpret == vmInterpret_t::VMI_NATIVE {
            // try to load as a system dll
            let vm_name = std::ffi::CStr::from_ptr((*vm).name.as_ptr()).to_string_lossy();
            view.print(&format!("Loading dll file {vm_name}.\n"));
            // SEAM-D11: `game_syscall_trampoline` is the C-variadic entry that
            // unpacks the va_list and dispatches to the armed engine slot; the
            // Rust `VM_DllSyscall` is reached through slot arming, not directly.
            (*vm).dllHandle = Sys_LoadDll(
                view.common,
                module,
                &mut (*vm).entryPoint,
                Some(crate::vm::trampoline::game_syscall_trampoline),
            );
            if !(*vm).dllHandle.is_null() {
                return vm;
            }

            view.print("Failed to load dll, looking for qvm.\n");
            interpret = vmInterpret_t::VMI_COMPILED;
        }

        // load the image
        let vm_name = std::ffi::CStr::from_ptr((*vm).name.as_ptr()).to_string_lossy();
        let filename = format!("vm/{vm_name}.qvm");
        view.print(&format!("Loading vm file {filename}.\n"));
        let file_bytes = view.fs_read_file(&filename);
        let header = match file_bytes.as_ref() {
            Some(bytes) if !bytes.is_empty() => bytes.as_ptr() as *mut vmHeader_t,
            _ => {
                view.print("Failed.\n");
                VM_Free(view.common, vm);
                return core::ptr::null_mut();
            }
        };
        // `header` aliases `file_bytes`'s storage; kept alive via
        // `file_bytes` for the header's lifetime here (mirrors `FS_FreeFile`).
        let _keep_alive = file_bytes;

        // byte swap the header
        for j in 0..(core::mem::size_of::<vmHeader_t>() / 4) {
            let p = (header as *mut c_int).add(j);
            *p = LittleLong(*p);
        }

        // validate
        if (*header).vmMagic != crate::qfiles::vm_magic::VM_MAGIC
            || (*header).bssLength < 0
            || (*header).dataLength < 0
            || (*header).litLength < 0
            || (*header).codeLength <= 0
        {
            VM_Free(view.common, vm);
            view.error(
                errorParm_t::ERR_FATAL,
                &format!("{filename} has bad header"),
            );
        }

        // round up to next power of 2 so all data operations can be mask
        // protected
        let mut dataLength = (*header).dataLength + (*header).litLength + (*header).bssLength;
        let mut j = 0;
        while dataLength > (1 << j) {
            j += 1;
        }
        dataLength = 1 << j;

        // allocate zero filled space for initialized and uninitialized data
        (*vm).dataBase = VM_Alloc(view, dataLength) as *mut u8;
        (*vm).dataMask = dataLength - 1;

        // copy the intialized data
        crate::common_fns::Com_Memcpy(
            (*vm).dataBase as *mut (),
            ((header as *mut u8).add((*header).dataOffset as usize)) as *const (),
            ((*header).dataLength + (*header).litLength) as usize,
        );

        // byte swap the longs
        let mut k = 0;
        while k < (*header).dataLength {
            let p = (*vm).dataBase.add(k as usize) as *mut c_int;
            *p = LittleLong(*p);
            k += 4;
        }

        // allocate space for the jump targets, which will be filled in by
        // the compile/prep functions
        (*vm).instructionPointersLength = (*header).instructionCount * 4;
        (*vm).instructionPointers = VM_Alloc(view, (*vm).instructionPointersLength) as *mut c_int;

        // copy or compile the instructions
        (*vm).codeLength = (*header).codeLength;

        if interpret as c_int >= vmInterpret_t::VMI_COMPILED as c_int {
            (*vm).compiled = qtrue;
            crate::vm_x86::VM_Compile(view, vm, header);
        } else {
            (*vm).compiled = qfalse;
            crate::vm_interpreted::VM_PrepareInterpreter(view, vm, header);
        }

        // load the map file
        VM_LoadSymbols(view, vm);

        // the stack is implicitly at the end of the image
        (*vm).programStack = (*vm).dataMask + 1;
        (*vm).stackBottom = (*vm).programStack - crate::vm::vm_stack_consts::STACK_SIZE as i32;

        vm
    }
}

/// `VM_Call` — engine→module dispatch.
///
/// Raven packs up to 16 `int` args into a stack frame and, on the native-dll
/// arm (`vm->entryPoint`), forwards `callnum` + all 16 words to the module's
/// `vmMain`; the callee's fixed parameter list silently drops unused extras
/// (`vm.cpp:806-816`). This port reproduces that native arm exactly. The QVM
/// `VM_CallCompiled`/`VM_CallInterpreted` arms (`vm.cpp:817-820`) are not part
/// of the native-dll build (MP ships native modules), so a non-native `vm` is a
/// fatal misconfiguration here.
///
/// Raven's `int` arg/return words are a 32-bit-era assumption; on LP64 they
/// truncate pointer-carrying words (`GAME_NAV_*` vec3 args, the
/// `GAME_CLIENT_CONNECT` denied-string return), so the words are `isize` here.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:787-819`
pub fn VM_Call(common: &mut Common, vm: *mut vm_t, callnum: c_int, args: &[isize]) -> isize {
    if vm.is_null() {
        crate::common::com_error(errorParm_t::ERR_FATAL, "VM_Call with NULL vm".to_string());
    }

    // SAFETY: `vm` is non-null (guarded above) and points at a live `vmTable`
    // slot; `entryPoint` is a module `vmMain` address filled by `Sys_LoadDll`
    // during `VM_Create`, valid for the duration of this synchronous call.
    unsafe {
        let oldVM = common.currentVM;
        common.currentVM = vm;
        common.lastVM = vm;

        if common.vm_debugLevel != 0 {
            crate::common::com_printf(common, &format!("VM_Call( {callnum} )\n"));
        }

        // if we have a dll loaded, call it directly (native arm, vm.cpp:806-816)
        let r = if let Some(entry) = (*vm).entryPoint {
            // Fixed-arity RawVmMain (the widened vmMain dual): command stays
            // c_int, the 12 arg words are AbiWord-width — matching the module
            // export register-for-register.
            let mut a = [0 as isize; 12];
            for (i, v) in args.iter().take(12).enumerate() {
                a[i] = *v;
            }
            entry(
                callnum, a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7], a[8], a[9], a[10], a[11],
            )
        } else {
            // The QVM `VM_CallCompiled`/`VM_CallInterpreted` arms are dead
            // surface in the native-dll build (§20): no bytecode VM is ever
            // loaded, so `entryPoint` is always set.
            crate::common::com_error(
                errorParm_t::ERR_FATAL,
                "VM_Call: non-native VM (QVM interpreter not built)".to_string(),
            );
        };

        // bk001220 - assert(currentVM!=NULL) for oldVM==NULL
        if !oldVM.is_null() {
            common.currentVM = oldVM;
        }
        r
    }
}

/// `VM_Restart`.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:391-458`
pub fn VM_Restart(view: &mut EngineHostView, vm: *mut vm_t) -> *mut vm_t {
    unsafe {
        // DLL's can't be restarted in place
        if !(*vm).dllHandle.is_null() {
            let systemCall = (*vm).systemCall;
            let name = std::ffi::CStr::from_ptr((*vm).name.as_ptr())
                .to_string_lossy()
                .into_owned();

            VM_Free(view.common, vm);

            return VM_Create(view, &name, systemCall, vmInterpret_t::VMI_NATIVE);
        }

        // load the image
        view.print("VM_Restart()\n");
        let vm_name = std::ffi::CStr::from_ptr((*vm).name.as_ptr()).to_string_lossy();
        let filename = format!("vm/{vm_name}.qvm");
        view.print(&format!("Loading vm file {filename}.\n"));
        let file_bytes = view.fs_read_file(&filename);
        let header = match &file_bytes {
            Some(bytes) if !bytes.is_empty() => bytes.as_ptr() as *mut vmHeader_t,
            _ => {
                view.error(errorParm_t::ERR_DROP, "VM_Restart failed.\n");
            }
        };

        // byte swap the header
        for j in 0..(core::mem::size_of::<vmHeader_t>() / 4) {
            let p = (header as *mut c_int).add(j);
            *p = LittleLong(*p);
        }

        // validate
        if (*header).vmMagic != crate::qfiles::vm_magic::VM_MAGIC
            || (*header).bssLength < 0
            || (*header).dataLength < 0
            || (*header).litLength < 0
            || (*header).codeLength <= 0
        {
            VM_Free(view.common, vm);
            view.error(
                errorParm_t::ERR_FATAL,
                &format!("{filename} has bad header"),
            );
        }

        // round up to next power of 2 so all data operations can be mask
        // protected
        let mut dataLength = (*header).dataLength + (*header).litLength + (*header).bssLength;
        let mut j = 0;
        while dataLength > (1 << j) {
            j += 1;
        }
        dataLength = 1 << j;

        // clear the data
        crate::common_fns::Com_Memset((*vm).dataBase as *mut (), 0, dataLength as usize);

        // copy the intialized data
        crate::common_fns::Com_Memcpy(
            (*vm).dataBase as *mut (),
            ((header as *mut u8).add((*header).dataOffset as usize)) as *const (),
            ((*header).dataLength + (*header).litLength) as usize,
        );

        // byte swap the longs
        let mut k = 0;
        while k < (*header).dataLength {
            let p = (*vm).dataBase.add(k as usize) as *mut c_int;
            *p = LittleLong(*p);
            k += 4;
        }

        let _keep_alive = file_bytes;

        vm
    }
}
