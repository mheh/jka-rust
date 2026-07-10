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
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::limits::MAX_TOKEN_CHARS;
use native_types::MAX_QPATH;

use crate::collision_world::CollisionWorld;
use crate::common::Common;
use crate::qcommon::vm_interpret_t::vmInterpret_t;
use crate::qfiles::vm_header_t::vmHeader_t;
use crate::vm::module_registry::MAX_VM;
use crate::vm::vm_s::vm_t;
use crate::vm::vm_symbol_s::vmSymbol_t;
use crate::vm::vmptr_t::vmptr_t;

// PORT-NOTE(rm-types): `RenderModels`/`RmManager`/`Server` are state-receiver
// types pinned by the engine-fork-discovery preamble's receiver order
// (rmg-terrain.md own their real shape); none has landed in this crate yet.
// Referenced by their exact resolved-signature names per the no-stub rule
// (`common_fns.rs`/`vm_x86.rs` precedent); reported as missing symbols for
// the finisher to replace with the real imports once they land.
#[allow(dead_code)]
struct RenderModels;
#[allow(dead_code)]
struct RmManager;
#[allow(dead_code)]
struct Server;

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
/// Source: `oracle/codemp/qcommon/vm.cpp:640-654`
pub fn VM_ArgPtr(common: &mut Common, intValue: c_int) -> *mut () {
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

/// `BotVMShift`.
///
/// Raven: `gvm` — "always using the game vm here."
/// Source: `oracle/codemp/qcommon/vm.cpp:657-677`
pub fn BotVMShift(common: &mut Common, ptr: c_int) -> *mut () {
    if ptr == 0 {
        return core::ptr::null_mut();
    }
    if common.gvm.is_null() {
        return core::ptr::null_mut();
    }
    unsafe {
        if !(*common.gvm).entryPoint.is_none() {
            ((*common.gvm).dataBase as isize + ptr as isize) as *mut ()
        } else {
            ((*common.gvm).dataBase as isize + (ptr & (*common.gvm).dataMask) as isize) as *mut ()
        }
    }
}

/// `VM_ExplicitArgPtr`.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:742-758`
pub fn VM_ExplicitArgPtr(common: &mut Common, vm: *mut vm_t, intValue: c_int) -> *mut () {
    if intValue == 0 {
        return core::ptr::null_mut();
    }
    // bk010124 - currentVM is missing on reconnect here as well?
    if common.currentVM.is_null() {
        return core::ptr::null_mut();
    }
    unsafe {
        if !(*vm).entryPoint.is_none() {
            ((*vm).dataBase as isize + intValue as isize) as *mut ()
        } else {
            ((*vm).dataBase as isize + (intValue & (*vm).dataMask) as isize) as *mut ()
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
            // PORT-NOTE(sys-dll): `Sys_UnloadDll` is the platform dylib-unload
            // external; not yet exposed on a receiver in this shard —
            // reported as a missing symbol.
            crate::qcommon::sys_dll::Sys_UnloadDll((*vm).dllHandle);
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
                crate::qcommon::sys_dll::Sys_UnloadDll(common.vmTable[i].dllHandle);
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
pub fn VM_Shifted_Alloc(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    ptr: *mut *mut (),
    size: c_int,
) {
    unsafe {
        if common.currentVM.is_null() {
            debug_assert!(false);
            *ptr = core::ptr::null_mut();
            return;
        }

        // first allocate our desired memory, up front
        let mem = crate::z_memman_pc::Z_Malloc(
            common,
            cm,
            rm,
            host,
            size + 1,
            mp_qshared::common::mp::qcommon::tags::memtag_t::TAG_VM_ALLOCATED,
            false,
        );

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
        *ptr = (mem as isize - (*common.currentVM).dataBase as isize) as *mut ();
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

        crate::z_memman_pc::Z_Free(common, mem);
        *ptr = core::ptr::null_mut(); // go ahead and clear the pointer for the game.
    }
}

/// `VM_VmProfile_f`.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:854-893`
pub fn VM_VmProfile_f(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    unsafe {
        if common.lastVM.is_null() {
            return;
        }

        let vm = common.lastVM;

        if (*vm).numSymbols == 0 {
            return;
        }

        let sorted = crate::z_memman_pc::Z_Malloc(
            common,
            cm,
            rm,
            host,
            (*vm).numSymbols as usize * core::mem::size_of::<*mut vmSymbol_t>(),
            mp_qshared::common::mp::qcommon::tags::memtag_t::TAG_VM,
            true,
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
            host.print(&format!("{perc:2}% {:9} {name}\n", (*sym).profileCount));
            (*sym).profileCount = 0;
        }

        host.print(&format!("    {total:9.0} total\n"));

        crate::z_memman_pc::Z_Free(common, sorted as *mut ());
    }
}

/// `VM_VmInfo_f`.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:901-925`
pub fn VM_VmInfo_f(common: &mut Common) {
    // PORT-NOTE(host-print): the packet's resolved signature doesn't carry
    // `host`, but `Com_Printf` in the tree resolves to the receiverless
    // `crate::common::com_printf` helper (common_fns.rs precedent) rather
    // than a raw external — used here to stay within the printed signature.
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
            if (*vm).compiled != mp_qshared::shared::qboolean::qfalse {
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
pub fn VM_Init(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    // default to DLLs now instead. Our VMs are getting too HUGE.
    host.cvar_register(
        "vm_cgame",
        "0",
        (mp_game::q_shared_cvar_flags::CVAR_SYSTEMINFO | mp_game::q_shared_cvar_flags::CVAR_ARCHIVE)
            as c_int,
    );
    host.cvar_register(
        "vm_game",
        "0",
        (mp_game::q_shared_cvar_flags::CVAR_SYSTEMINFO | mp_game::q_shared_cvar_flags::CVAR_ARCHIVE)
            as c_int,
    );
    host.cvar_register(
        "vm_ui",
        "0",
        (mp_game::q_shared_cvar_flags::CVAR_SYSTEMINFO | mp_game::q_shared_cvar_flags::CVAR_ARCHIVE)
            as c_int,
    );
    // client wants to know if the server is using vm's for certain modules,
    // so if pure we can force the same method (be it vm or dll) -rww

    // PORT-NOTE(cmd-table): `Cmd_AddCommand`'s resolved receiver-taking
    // signature isn't landed in this shard (dispatch-table wave, ruling 5);
    // referenced by its exact resolved name/receivers per the no-stub rule,
    // reported as a missing symbol.
    crate::cmd_pc::Cmd_AddCommand(
        common,
        cm,
        rm,
        host,
        c"vmprofile".as_ptr(),
        VM_VmProfile_f as _,
    );
    crate::cmd_pc::Cmd_AddCommand(common, cm, rm, host, c"vminfo".as_ptr(), VM_VmInfo_f as _);

    common.vmTable = unsafe { core::mem::zeroed() };
}

/// `VM_Alloc`.
///
/// Raven: the `Z_Malloc` alternative is commented out — `Hunk_Alloc` is the
/// live path.
/// Source: `oracle/codemp/qcommon/vm.cpp:227-231`
pub fn VM_Alloc(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    size: c_int,
) -> *mut () {
    crate::miniheap::cmini_heap::Hunk_Alloc(
        common,
        cm,
        rm,
        host,
        size,
        mp_qshared::shared::ha_pref::ha_pref::h_high,
    )
}

/// `VM_LoadSymbols`.
///
/// Raven: `CRAZY_SYMBOL_MAP` is never defined in this build; the
/// `g_symbolMap` write-through cache is intentionally unreachable here.
/// Source: `oracle/codemp/qcommon/vm.cpp:238-323`
pub fn VM_LoadSymbols(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    vm: *mut vm_t,
) {
    // don't load symbols if not developer
    if host.cvar_integer("developer") == 0 {
        return;
    }

    unsafe {
        let mut name = [0u8; MAX_QPATH as usize];
        let vm_name = std::ffi::CStr::from_ptr((*vm).name.as_ptr()).to_string_lossy();
        let stripped = crate::qcommon::com_string::COM_StripExtension(&vm_name);
        let n = stripped.len().min(name.len() - 1);
        name[..n].copy_from_slice(&stripped.as_bytes()[..n]);

        let symbols = format!("vm/{}.map", String::from_utf8_lossy(&name[..n]));

        let mapfile = match host.fs_read_file(&symbols) {
            Some(bytes) => bytes,
            None => {
                host.print(&format!("Couldn't load symbol file: {symbols}\n"));
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
            let (token, rest) = crate::qcommon::com_string::COM_Parse(cursor);
            cursor = rest;
            if token.is_empty() {
                break;
            }
            let token_c = std::ffi::CString::new(token.clone()).unwrap();
            let segment = ParseHex(token_c.as_ptr());
            if segment != 0 {
                let (_, rest) = crate::qcommon::com_string::COM_Parse(cursor);
                cursor = rest;
                let (_, rest) = crate::qcommon::com_string::COM_Parse(cursor);
                cursor = rest;
                continue; // only load code segment values
            }

            let (token, rest) = crate::qcommon::com_string::COM_Parse(cursor);
            cursor = rest;
            if token.is_empty() {
                host.print("WARNING: incomplete line at end of file\n");
                break;
            }
            let token_c = std::ffi::CString::new(token).unwrap();
            let mut value = ParseHex(token_c.as_ptr());

            let (token, rest) = crate::qcommon::com_string::COM_Parse(cursor);
            cursor = rest;
            if token.is_empty() {
                host.print("WARNING: incomplete line at end of file\n");
                break;
            }
            let chars = token.len();

            let sym = VM_Alloc(
                common,
                cm,
                rm,
                host,
                (core::mem::size_of::<vmSymbol_t>() + chars) as c_int,
            ) as *mut vmSymbol_t;
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
        host.print(&format!("{count} symbols parsed from {symbols}\n"));
    }
}

/// `VM_Create`.
///
/// Raven: `systemCalls`'s C fn-pointer type isn't in the rosetta
/// (escalated); ported as an `Option<extern "C" fn(*mut c_int) -> c_int>`
/// matching `vm_t::systemCall`'s already-landed shape.
/// Source: `oracle/codemp/qcommon/vm.cpp:471-597`
pub fn VM_Create(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    module: *const c_char,
    systemCalls: Option<extern "C" fn(*mut c_int) -> c_int>,
    mut interpret: vmInterpret_t,
) -> *mut vm_t {
    unsafe {
        let module_str = std::ffi::CStr::from_ptr(module).to_string_lossy();
        if module.is_null() || module_str.is_empty() || systemCalls.is_none() {
            host.error(errorParm_t::ERR_FATAL, "VM_Create: bad parms");
        }

        // see if we already have the VM
        for i in 0..MAX_VM {
            let name = std::ffi::CStr::from_ptr(common.vmTable[i].name.as_ptr()).to_string_lossy();
            if name.eq_ignore_ascii_case(&module_str) {
                return &mut common.vmTable[i] as *mut vm_t;
            }
        }

        // find a free vm
        let mut i = 0;
        while i < MAX_VM {
            if common.vmTable[i].name[0] == 0 {
                break;
            }
            i += 1;
        }

        if i == MAX_VM {
            host.error(errorParm_t::ERR_FATAL, "VM_Create: no free vm_t");
        }

        let vm = &mut common.vmTable[i] as *mut vm_t;

        let name_bytes = module_str.as_bytes();
        let n = name_bytes.len().min((*vm).name.len() - 1);
        for (j, b) in name_bytes[..n].iter().enumerate() {
            (*vm).name[j] = *b as c_char;
        }
        (*vm).name[n] = 0;
        (*vm).systemCall = systemCalls;

        // never allow dll loading with a demo
        if interpret == vmInterpret_t::VMI_NATIVE {
            if crate::cvar_fns::Cvar_VariableValue(common, cm, rm, host, c"fs_restrict".as_ptr())
                != 0.0
            {
                interpret = vmInterpret_t::VMI_COMPILED;
            }
        }

        if interpret == vmInterpret_t::VMI_NATIVE {
            // try to load as a system dll
            let vm_name = std::ffi::CStr::from_ptr((*vm).name.as_ptr()).to_string_lossy();
            host.print(&format!("Loading dll file {vm_name}.\n"));
            (*vm).dllHandle = crate::qcommon::sys_dll::Sys_LoadDll(
                module,
                &mut (*vm).entryPoint,
                VM_DllSyscall as _,
            );
            if !(*vm).dllHandle.is_null() {
                return vm;
            }

            host.print("Failed to load dll, looking for qvm.\n");
            interpret = vmInterpret_t::VMI_COMPILED;
        }

        // load the image
        let vm_name = std::ffi::CStr::from_ptr((*vm).name.as_ptr()).to_string_lossy();
        let filename = format!("vm/{vm_name}.qvm");
        host.print(&format!("Loading vm file {filename}.\n"));
        let file_bytes = host.fs_read_file(&filename);
        let header = match file_bytes {
            Some(bytes) if !bytes.is_empty() => bytes.as_ptr() as *mut vmHeader_t,
            _ => {
                host.print("Failed.\n");
                VM_Free(common, vm);
                return core::ptr::null_mut();
            }
        };
        // PORT-NOTE(fs-buf): `header` above aliases the returned `Vec<u8>`'s
        // storage; kept alive via `file_bytes` for the header's lifetime in
        // this fn (the `Vec` is dropped at scope end, mirroring `FS_FreeFile`
        // further down).
        let _keep_alive = file_bytes;

        // byte swap the header
        for j in 0..(core::mem::size_of::<vmHeader_t>() / 4) {
            let p = (header as *mut c_int).add(j);
            *p = crate::qcommon::byteswap::LittleLong(*p);
        }

        // validate
        if (*header).vmMagic != crate::qfiles::vm_magic::VM_MAGIC
            || (*header).bssLength < 0
            || (*header).dataLength < 0
            || (*header).litLength < 0
            || (*header).codeLength <= 0
        {
            VM_Free(common, vm);
            host.error(
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
        (*vm).dataBase = VM_Alloc(common, cm, rm, host, dataLength) as *mut u8;
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
            *p = crate::qcommon::byteswap::LittleLong(*p);
            k += 4;
        }

        // allocate space for the jump targets, which will be filled in by
        // the compile/prep functions
        (*vm).instructionPointersLength = (*header).instructionCount * 4;
        (*vm).instructionPointers =
            VM_Alloc(common, cm, rm, host, (*vm).instructionPointersLength) as *mut c_int;

        // copy or compile the instructions
        (*vm).codeLength = (*header).codeLength;

        if interpret as c_int >= vmInterpret_t::VMI_COMPILED as c_int {
            (*vm).compiled = mp_qshared::shared::qboolean::qtrue;
            crate::vm_x86::VM_Compile(common, cm, rm, host, vm, header);
        } else {
            (*vm).compiled = mp_qshared::shared::qboolean::qfalse;
            crate::vm_interpreted::VM_PrepareInterpreter(common, cm, rm, host, vm, header);
        }

        // load the map file
        VM_LoadSymbols(common, cm, rm, host, vm);

        // the stack is implicitly at the end of the image
        (*vm).programStack = (*vm).dataMask + 1;
        (*vm).stackBottom = (*vm).programStack - crate::vm::vm_stack_consts::STACK_SIZE;

        vm
    }
}

/// `VM_Restart`.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:391-458`
pub fn VM_Restart(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    vm: *mut vm_t,
) -> *mut vm_t {
    unsafe {
        // DLL's can't be restarted in place
        if !(*vm).dllHandle.is_null() {
            let systemCall = (*vm).systemCall;
            let name = std::ffi::CStr::from_ptr((*vm).name.as_ptr())
                .to_string_lossy()
                .into_owned();

            VM_Free(common, vm);

            let name_c = std::ffi::CString::new(name).unwrap();
            return VM_Create(
                common,
                cm,
                rm,
                host,
                name_c.as_ptr(),
                systemCall,
                vmInterpret_t::VMI_NATIVE,
            );
        }

        // load the image
        host.print("VM_Restart()\n");
        let vm_name = std::ffi::CStr::from_ptr((*vm).name.as_ptr()).to_string_lossy();
        let filename = format!("vm/{vm_name}.qvm");
        host.print(&format!("Loading vm file {filename}.\n"));
        let file_bytes = host.fs_read_file(&filename);
        let header = match &file_bytes {
            Some(bytes) if !bytes.is_empty() => bytes.as_ptr() as *mut vmHeader_t,
            _ => {
                host.error(errorParm_t::ERR_DROP, "VM_Restart failed.\n");
            }
        };

        // byte swap the header
        for j in 0..(core::mem::size_of::<vmHeader_t>() / 4) {
            let p = (header as *mut c_int).add(j);
            *p = crate::qcommon::byteswap::LittleLong(*p);
        }

        // validate
        if (*header).vmMagic != crate::qfiles::vm_magic::VM_MAGIC
            || (*header).bssLength < 0
            || (*header).dataLength < 0
            || (*header).litLength < 0
            || (*header).codeLength <= 0
        {
            VM_Free(common, vm);
            host.error(
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
            *p = crate::qcommon::byteswap::LittleLong(*p);
            k += 4;
        }

        let _keep_alive = file_bytes;

        vm
    }
}
