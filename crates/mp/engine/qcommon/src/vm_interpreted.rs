#![allow(non_snake_case, non_camel_case_types)]

//! Ported from `oracle/codemp/qcommon/vm_interpreted.cpp`.

use std::os::raw::{c_char, c_int};

use mp_host_interface::EngineHost;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::ha_pref::ha_pref;

use crate::collision_world::CollisionWorld;
use crate::common::com_error;
use crate::common::com_printf;
use crate::common::Common;
use crate::qfiles::vm_header_t::vmHeader_t;
use crate::vm::opcode_t::opcode_t;
use crate::vm::vm_s::vm_t;
use crate::vm_fns::VM_ValueToSymbol;
use crate::z_memman_pc::Hunk_Alloc;

// PORT-NOTE(rm-types): `RenderModels` is the state-receiver type pinned by
// the engine-fork-discovery preamble's receiver order (rmg-terrain.md /
// tr-model.md own its real shape); not importable here yet. Referenced by
// its exact resolved-signature name per the no-stub rule (z_memman_pc.rs
// precedent); reported as a missing symbol for the finisher.
#[allow(dead_code)]
struct RenderModels;

/// Raven `loadWord` — file-static helper, non-`_BIG_ENDIAN_PPC_` macro arm
/// (`*((int *)addr)`; oracle's PPC `__lwbrx` arm doesn't apply to our target).
///
/// Source: `oracle/codemp/qcommon/vm_interpreted.cpp:110`
unsafe fn loadWord(addr: *mut u8) -> c_int {
    *(addr as *mut c_int)
}

/// Raven `VM_Indent`.
///
/// Source: `oracle/codemp/qcommon/vm_interpreted.cpp:113-119`
pub fn VM_Indent(_common: &mut Common, vm: *mut vm_t) -> *mut c_char {
    // Rotating scratch: Raven's `static char *string` is a const string
    // literal (never mutated), so it collapses to a `const` per the
    // fork-3 three-kind rule.
    const STRING: &[u8] = b"                                        \0";
    unsafe {
        let base = STRING.as_ptr() as *mut c_char;
        if (*vm).callLevel > 20 {
            return base;
        }
        base.add((2 * (20 - (*vm).callLevel)) as usize)
    }
}

/// Raven `VM_StackTrace`.
///
/// Source: `oracle/codemp/qcommon/vm_interpreted.cpp:121-131`
pub fn VM_StackTrace(
    common: &mut Common,
    vm: *mut vm_t,
    programCounter: c_int,
    programStack: c_int,
) {
    let mut program_counter = programCounter;
    let mut program_stack = programStack;
    let mut count = 0;
    unsafe {
        loop {
            let sym = VM_ValueToSymbol(common, vm, program_counter);
            let sym_str = std::ffi::CStr::from_ptr(sym).to_string_lossy();
            com_printf(common, &format!("{}\n", sym_str));
            program_stack = *((*vm).dataBase.offset((program_stack + 4) as isize) as *const i32);
            program_counter = *((*vm).dataBase.offset(program_stack as isize) as *const i32);
            count += 1;
            if !(program_counter != -1 && count < 32) {
                break;
            }
        }
    }
}

/// Raven `VM_PrepareInterpreter`.
///
/// Source: `oracle/codemp/qcommon/vm_interpreted.cpp:139-266`
pub fn VM_PrepareInterpreter(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    vm: *mut vm_t,
    header: *mut vmHeader_t,
) {
    unsafe {
        (*vm).codeBase =
            Hunk_Alloc(common, cm, rm, host, (*vm).codeLength * 4, ha_pref::h_high).cast::<u8>();
        // memcpy( vm->codeBase, (byte *)header + header->codeOffset, vm->codeLength ); (Raven: commented out)

        // we don't need to translate the instructions, but we still need
        // to find each instructions starting point for jumps
        let mut pc: c_int = 0;
        let mut instruction: c_int = 0;
        let mut code = (header as *mut u8).offset((*header).codeOffset as isize);
        let mut code_base = (*vm).codeBase as *mut c_int;

        while instruction < (*header).instructionCount {
            *(*vm).instructionPointers.offset(instruction as isize) = pc;
            instruction += 1;

            let op = *code.offset(pc as isize);
            *code_base.offset(pc as isize) = op as c_int;
            if pc > (*header).codeLength {
                com_error(
                    errorParm_t::ERR_FATAL,
                    "VM_PrepareInterpreter: pc > header->codeLength".into(),
                );
            }

            pc += 1;

            // these are the only opcodes that aren't a single byte
            let op = op as c_int;
            if op == opcode_t::OP_ENTER as c_int
                || op == opcode_t::OP_CONST as c_int
                || op == opcode_t::OP_LOCAL as c_int
                || op == opcode_t::OP_LEAVE as c_int
                || op == opcode_t::OP_EQ as c_int
                || op == opcode_t::OP_NE as c_int
                || op == opcode_t::OP_LTI as c_int
                || op == opcode_t::OP_LEI as c_int
                || op == opcode_t::OP_GTI as c_int
                || op == opcode_t::OP_GEI as c_int
                || op == opcode_t::OP_LTU as c_int
                || op == opcode_t::OP_LEU as c_int
                || op == opcode_t::OP_GTU as c_int
                || op == opcode_t::OP_GEU as c_int
                || op == opcode_t::OP_EQF as c_int
                || op == opcode_t::OP_NEF as c_int
                || op == opcode_t::OP_LTF as c_int
                || op == opcode_t::OP_LEF as c_int
                || op == opcode_t::OP_GTF as c_int
                || op == opcode_t::OP_GEF as c_int
                || op == opcode_t::OP_BLOCK_COPY as c_int
            {
                *code_base.offset((pc + 0) as isize) = loadWord(code.offset(pc as isize));
                pc += 4;
            } else if op == opcode_t::OP_ARG as c_int {
                *code_base.offset((pc + 0) as isize) = *code.offset(pc as isize) as c_int;
                pc += 1;
            }
            // default: break; (nothing to do)
        }

        pc = 0;
        instruction = 0;
        code = (header as *mut u8).offset((*header).codeOffset as isize);
        code_base = (*vm).codeBase as *mut c_int;

        while instruction < (*header).instructionCount {
            let op = *code.offset(pc as isize) as c_int;
            instruction += 1;
            pc += 1;
            if op == opcode_t::OP_ENTER as c_int
                || op == opcode_t::OP_CONST as c_int
                || op == opcode_t::OP_LOCAL as c_int
                || op == opcode_t::OP_LEAVE as c_int
                || op == opcode_t::OP_EQ as c_int
                || op == opcode_t::OP_NE as c_int
                || op == opcode_t::OP_LTI as c_int
                || op == opcode_t::OP_LEI as c_int
                || op == opcode_t::OP_GTI as c_int
                || op == opcode_t::OP_GEI as c_int
                || op == opcode_t::OP_LTU as c_int
                || op == opcode_t::OP_LEU as c_int
                || op == opcode_t::OP_GTU as c_int
                || op == opcode_t::OP_GEU as c_int
                || op == opcode_t::OP_EQF as c_int
                || op == opcode_t::OP_NEF as c_int
                || op == opcode_t::OP_LTF as c_int
                || op == opcode_t::OP_LEF as c_int
                || op == opcode_t::OP_GTF as c_int
                || op == opcode_t::OP_GEF as c_int
                || op == opcode_t::OP_BLOCK_COPY as c_int
            {
                // inner switch(op): only the relational/equality ops (not
                // OP_BLOCK_COPY) rewrite the jump target through
                // instructionPointers.
                if op == opcode_t::OP_EQ as c_int
                    || op == opcode_t::OP_NE as c_int
                    || op == opcode_t::OP_LTI as c_int
                    || op == opcode_t::OP_LEI as c_int
                    || op == opcode_t::OP_GTI as c_int
                    || op == opcode_t::OP_GEI as c_int
                    || op == opcode_t::OP_LTU as c_int
                    || op == opcode_t::OP_LEU as c_int
                    || op == opcode_t::OP_GTU as c_int
                    || op == opcode_t::OP_GEU as c_int
                    || op == opcode_t::OP_EQF as c_int
                    || op == opcode_t::OP_NEF as c_int
                    || op == opcode_t::OP_LTF as c_int
                    || op == opcode_t::OP_LEF as c_int
                    || op == opcode_t::OP_GTF as c_int
                    || op == opcode_t::OP_GEF as c_int
                {
                    let target = *code_base.offset(pc as isize);
                    *code_base.offset(pc as isize) =
                        *(*vm).instructionPointers.offset(target as isize);
                }
                pc += 4;
            } else if op == opcode_t::OP_ARG as c_int {
                pc += 1;
            }
            // default: break; (nothing to do)
        }
    }
}
