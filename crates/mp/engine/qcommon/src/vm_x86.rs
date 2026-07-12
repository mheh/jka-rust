#![allow(non_snake_case, non_camel_case_types, clippy::too_many_arguments)]
//! `vm_x86.cpp` — the x86 QVM JIT compiler/emitter (ruling 6: data-faithful
//! emitter; executes only on x86 hosts, same as Raven).
//!
//! DESTINATION NOTE: `vm_x86.cpp`'s stem collides with the existing `vm/`
//! directory module, so this file lands at the `_fns`-style escape named
//! `vm_x86.rs` per the packet's own DESTINATION line.
//!
//! Source: `oracle/codemp/qcommon/vm_x86.cpp`

use core::ffi::{c_char, c_int};
use std::ffi::CStr;

use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::ha_pref;
use native_types::{byte, qboolean, qfalse, qtrue};

use crate::common::engine_host_view::EngineHostView;
use crate::common::{com_error, com_printf, Common};
use crate::common_fns::{Com_Memcpy, Com_Memset};
use crate::qfiles::vm_header_t::vmHeader_t;
use crate::vm::elastcommand::ELastCommand;
use crate::vm::opcode_t::opcode_t;
use crate::vm::vm_s::vm_t;

// Real in-crate callee imported (sweep: extern forward-declares eliminated).
use crate::z_memman_pc::Hunk_Alloc;
// Genuinely-unported callees referenced at their canonical future homes.
use crate::z_memman_pc::{Z_Free, Z_Malloc};

/// `callAsmCall`.
///
/// Raven: save the stack to allow recursive VM entry.
/// Source: `oracle/codemp/qcommon/vm_x86.cpp:159-165`
pub extern "C" fn callAsmCall(common: &mut Common) {
    // PORT-NOTE(vm-x86-statics): `currentVM`/`callOpStack`/`callProgramStack`/
    // `callSyscallNum` are vm_x86.cpp file-scope globals/statics (ruling 3,
    // genuine cross-frame state) with no home field on `Common` yet; referenced
    // by their obvious snake_case names and reported as missing symbols.
    unsafe {
        (*common.current_vm).programStack = common.call_program_stack - 4;
        let data_base = (*common.current_vm).dataBase;
        *(data_base.offset((common.call_program_stack + 4) as isize) as *mut c_int) =
            common.call_syscall_num;
        //VM_LogSyscalls(  (int *)((byte *)currentVM->dataBase + programStack + 4) );
        let sys_call = (*common.current_vm).systemCall.unwrap();
        *(common.call_op_stack.offset(1)) =
            sys_call(data_base.offset((common.call_program_stack + 4) as isize) as *mut c_int);
    }
}

/// `AsmCall`.
///
/// Raven: the hand-written x86 asm trampoline between the compiled VM code and
/// `callAsmCall`. Transcribed as an inline-asm block matching the oracle's
/// clobbers/operands 1:1 (data-faithful emitter, ruling 6).
/// Source: `oracle/codemp/qcommon/vm_x86.cpp:167-199`
// Ruling 6: vm_x86 executes only on x86 hosts, same as Raven; the inline asm
// below is 32-bit-only (`movl`/`%eax`-family mnemonics), so it is gated to
// `target_arch = "x86"` and stubbed elsewhere rather than compiled unconditionally.
#[cfg(target_arch = "x86")]
pub extern "C" fn AsmCall(common: &mut Common) {
    // PORT-NOTE(vm-x86-asm): GNU inline asm syntax differs from Rust's; the
    // operand/clobber list is transcribed 1:1 from the oracle block. `callMask`
    // is an oracle asm-local symbol with no rosetta row — reported as a missing
    // symbol.
    unsafe {
        core::arch::asm!(
            "doAsmCall:",
            "movl (%edi),%eax",
            "subl $4,%edi",
            "orl %eax,%eax",
            "jl systemCall",
            "shll $2,%eax",
            "addl {instr},%eax",
            "call *(%eax)",
            "movl (%edi),%eax",
            "andl callMask, %eax",
            "jmp doret",
            "systemCall:",
            "negl %eax",
            "decl %eax",
            "movl %eax,{syscall_out}",
            "movl %esi,{prog_stack_out}",
            "movl %edi,{op_stack_out}",
            "pushl %ecx",
            "pushl %esi",
            "pushl %edi",
            "call callAsmCall",
            "popl %edi",
            "popl %esi",
            "popl %ecx",
            "addl $4,%edi",
            "doret:",
            "ret",
            instr = in(reg) common.instruction_pointers,
            syscall_out = out(reg) common.call_syscall_num,
            prog_stack_out = out(reg) common.call_program_stack,
            op_stack_out = out(reg) common.call_op_stack,
            out("eax") _, out("edi") _, out("esi") _, out("ecx") _,
            options(att_syntax),
        );
    }
}

/// Non-x86 stub: Raven's `AsmCall` never runs on non-x86 hosts either (the
/// vm_x86 JIT emitter is x86-only); calling it off-target is a build/config
/// error, not a runtime path any target actually exercises.
#[cfg(not(target_arch = "x86"))]
pub extern "C" fn AsmCall(_common: &mut Common) {
    unreachable!("vm_x86::AsmCall is x86-only (ruling 6)");
}

/// `Constant4`.
///
/// Source: `oracle/codemp/qcommon/vm_x86.cpp:202-208`
pub fn Constant4(common: &mut Common) -> c_int {
    let v: c_int;
    unsafe {
        v = *common.code.offset(common.pc as isize) as c_int
            | ((*common.code.offset((common.pc + 1) as isize) as c_int) << 8)
            | ((*common.code.offset((common.pc + 2) as isize) as c_int) << 16)
            | ((*common.code.offset((common.pc + 3) as isize) as c_int) << 24);
    }
    common.pc += 4;
    v
}

/// `Constant1`.
///
/// Source: `oracle/codemp/qcommon/vm_x86.cpp:210-216`
pub fn Constant1(common: &mut Common) -> c_int {
    let v: c_int;
    unsafe {
        v = *common.code.offset(common.pc as isize) as c_int;
    }
    common.pc += 1;
    v
}

/// `Emit1`.
///
/// Source: `oracle/codemp/qcommon/vm_x86.cpp:218-224`
pub fn Emit1(common: &mut Common, v: c_int) {
    unsafe {
        *common.buf.offset(common.compiled_ofs as isize) = v as byte;
    }
    common.compiled_ofs += 1;

    common.last_command = ELastCommand::LAST_COMMAND_NONE;
}

/// `Emit4`.
///
/// Source: `oracle/codemp/qcommon/vm_x86.cpp:233-238`
pub fn Emit4(common: &mut Common, v: c_int) {
    Emit1(common, v & 255);
    Emit1(common, (v >> 8) & 255);
    Emit1(common, (v >> 16) & 255);
    Emit1(common, (v >> 24) & 255);
}

/// `Hex`.
///
/// Source: `oracle/codemp/qcommon/vm_x86.cpp:240-254`
pub fn Hex(_view: &mut EngineHostView, c: c_int) -> c_int {
    if c >= b'a' as c_int && c <= b'f' as c_int {
        return 10 + c - b'a' as c_int;
    }
    if c >= b'A' as c_int && c <= b'F' as c_int {
        return 10 + c - b'A' as c_int;
    }
    if (b'0' as c_int..=b'9' as c_int).contains(&c) {
        return c - b'0' as c_int;
    }

    // Raven `Com_Error( ERR_DROP, "Hex: bad char '%c'", c )` — the engine's
    // longjmp error path is the diverging `com_error` panic (ruling 1).
    com_error(
        errorParm_t::ERR_DROP,
        format!("Hex: bad char '{}'", c as u8 as char),
    )
}

/// `EmitString`.
///
/// Source: `oracle/codemp/qcommon/vm_x86.cpp:255-271`
pub fn EmitString(view: &mut EngineHostView, string: *const c_char) {
    unsafe {
        let mut string = string;
        loop {
            let c1 = *string as c_int;
            let c2 = *string.offset(1) as c_int;

            let v = (Hex(view, c1) << 4) | Hex(view, c2);
            Emit1(view.common, v);

            if *string.offset(2) == 0 {
                break;
            }
            string = string.offset(3);
        }
    }
}

/// `EmitCommand`.
///
/// Source: `oracle/codemp/qcommon/vm_x86.cpp:275-292`
pub fn EmitCommand(view: &mut EngineHostView, command: ELastCommand) {
    match command {
        ELastCommand::LAST_COMMAND_MOV_EDI_EAX => {
            EmitString(view, c"89 07".as_ptr()); // mov dword ptr [edi], eax
        }
        ELastCommand::LAST_COMMAND_SUB_DI_4 => {
            EmitString(view, c"83 EF 04".as_ptr()); // sub edi, 4
        }
        ELastCommand::LAST_COMMAND_SUB_DI_8 => {
            EmitString(view, c"83 EF 08".as_ptr()); // sub edi, 8
        }
        _ => {}
    }
    view.common.last_command = command;
}

/// `EmitAddEDI4`.
///
/// Source: `oracle/codemp/qcommon/vm_x86.cpp:294-309`
pub fn EmitAddEDI4(view: &mut EngineHostView, vm: *mut vm_t) {
    unsafe {
        if view.common.last_command == ELastCommand::LAST_COMMAND_SUB_DI_4
            && *view.common.jused.offset((view.common.instruction - 1) as isize) == 0
        {
            // sub di,4
            view.common.compiled_ofs -= 3;
            *(*vm)
                .instructionPointers
                .offset((view.common.instruction - 1) as isize) = view.common.compiled_ofs;
            return;
        }
        if view.common.last_command == ELastCommand::LAST_COMMAND_SUB_DI_8
            && *view.common.jused.offset((view.common.instruction - 1) as isize) == 0
        {
            // sub di,8
            view.common.compiled_ofs -= 3;
            *(*vm)
                .instructionPointers
                .offset((view.common.instruction - 1) as isize) = view.common.compiled_ofs;
            EmitString(view, c"83 EF 04".as_ptr()); //	sub edi,4
            return;
        }
    }
    EmitString(view, c"83 C7 04".as_ptr()); //	add edi,4
}

/// `EmitMovEAXEDI`.
///
/// Source: `oracle/codemp/qcommon/vm_x86.cpp:311-332`
pub fn EmitMovEAXEDI(view: &mut EngineHostView, vm: *mut vm_t) {
    unsafe {
        if view.common.last_command == ELastCommand::LAST_COMMAND_MOV_EDI_EAX {
            // mov [edi], eax
            view.common.compiled_ofs -= 2;
            *(*vm)
                .instructionPointers
                .offset((view.common.instruction - 1) as isize) = view.common.compiled_ofs;
            return;
        }
        if view.common.pop1 == opcode_t::OP_DIVI as c_int
            || view.common.pop1 == opcode_t::OP_DIVU as c_int
            || view.common.pop1 == opcode_t::OP_MULI as c_int
            || view.common.pop1 == opcode_t::OP_MULU as c_int
            || view.common.pop1 == opcode_t::OP_STORE4 as c_int
            || view.common.pop1 == opcode_t::OP_STORE2 as c_int
            || view.common.pop1 == opcode_t::OP_STORE1 as c_int
        {
            return;
        }
        if view.common.pop1 == opcode_t::OP_CONST as c_int
            && *view.common.buf.offset((view.common.compiled_ofs - 6) as isize) == 0xC7
            && *view.common.buf.offset((view.common.compiled_ofs - 5) as isize) == 0x07
        {
            // mov edi, 0x123456
            view.common.compiled_ofs -= 6;
            *(*vm)
                .instructionPointers
                .offset((view.common.instruction - 1) as isize) = view.common.compiled_ofs;
            EmitString(view, c"B8".as_ptr()); // mov	eax, 0x12345678
            Emit4(view.common, view.common.last_const);
            return;
        }
    }
    EmitString(view, c"8B 07".as_ptr()); // mov eax, dword ptr [edi]
}

/// `EmitMovEBXEDI`.
///
/// Source: `oracle/codemp/qcommon/vm_x86.cpp:334-363`
pub fn EmitMovEBXEDI(view: &mut EngineHostView, vm: *mut vm_t, andit: c_int) -> qboolean {
    unsafe {
        if view.common.last_command == ELastCommand::LAST_COMMAND_MOV_EDI_EAX {
            // mov [edi], eax
            view.common.compiled_ofs -= 2;
            *(*vm)
                .instructionPointers
                .offset((view.common.instruction - 1) as isize) = view.common.compiled_ofs;
            EmitString(view, c"8B D8".as_ptr()); // mov bx, eax
            return qfalse;
        }
        if view.common.pop1 == opcode_t::OP_DIVI as c_int
            || view.common.pop1 == opcode_t::OP_DIVU as c_int
            || view.common.pop1 == opcode_t::OP_MULI as c_int
            || view.common.pop1 == opcode_t::OP_MULU as c_int
            || view.common.pop1 == opcode_t::OP_STORE4 as c_int
            || view.common.pop1 == opcode_t::OP_STORE2 as c_int
            || view.common.pop1 == opcode_t::OP_STORE1 as c_int
        {
            EmitString(view, c"8B D8".as_ptr()); // mov bx, eax
            return qfalse;
        }
        if view.common.pop1 == opcode_t::OP_CONST as c_int
            && *view.common.buf.offset((view.common.compiled_ofs - 6) as isize) == 0xC7
            && *view.common.buf.offset((view.common.compiled_ofs - 5) as isize) == 0x07
        {
            // mov edi, 0x123456
            view.common.compiled_ofs -= 6;
            *(*vm)
                .instructionPointers
                .offset((view.common.instruction - 1) as isize) = view.common.compiled_ofs;
            EmitString(view, c"BB".as_ptr()); // mov	ebx, 0x12345678
            if andit != 0 {
                Emit4(view.common, view.common.last_const & andit);
            } else {
                Emit4(view.common, view.common.last_const);
            }
            return qtrue;
        }
    }

    EmitString(view, c"8B 1F".as_ptr()); // mov ebx, dword ptr [edi]
    qfalse
}

/// `VM_Compile`.
///
/// Raven: compiles a loaded QVM module's bytecode to native x86 machine code
/// in two passes (peephole-optimizing pass 0, final-length pass 1), then
/// copies the result to an exact-size Hunk allocation. Data-faithful emitter
/// (ruling 6): executes only on x86 hosts, same restriction as the oracle.
/// Source: `oracle/codemp/qcommon/vm_x86.cpp:370-1056`
pub fn VM_Compile(view: &mut EngineHostView, vm: *mut vm_t, header: *mut vmHeader_t) {
    let mut opt: qboolean;

    unsafe {
        // allocate a very large temp buffer, we will shrink it later
        let max_length = (*header).codeLength * 8;
        view.common.buf = Z_Malloc(
            view,
            max_length,
            mp_qshared::common::mp::qcommon::tags::memtag_t::TAG_VM,
            qtrue,
            0,
        ) as *mut byte;
        view.common.jused = Z_Malloc(
            view,
            (*header).instructionCount + 2,
            mp_qshared::common::mp::qcommon::tags::memtag_t::TAG_VM,
            qtrue,
            0,
        ) as *mut byte;

        Com_Memset(
            view.common.jused as *mut (),
            0,
            ((*header).instructionCount + 2) as usize,
        );

        for pass_ in 0..2 {
            view.common.pass = pass_;
            view.common.oc0 = -23423;
            view.common.oc1 = -234354;
            view.common.pop0 = -43435;
            view.common.pop1 = -545455;

            // translate all instructions
            view.common.pc = 0;
            view.common.instruction = 0;
            view.common.code = (header as *mut byte).offset((*header).codeOffset as isize);
            view.common.compiled_ofs = 0;

            view.common.last_command = ELastCommand::LAST_COMMAND_NONE;

            while view.common.instruction < (*header).instructionCount {
                if view.common.compiled_ofs > max_length - 16 {
                    com_error(
                        errorParm_t::ERR_FATAL,
                        "VM_CompileX86: maxLength exceeded".into(),
                    );
                }

                *(*vm)
                    .instructionPointers
                    .offset(view.common.instruction as isize) = view.common.compiled_ofs;
                view.common.instruction += 1;

                if view.common.pc > (*header).codeLength {
                    com_error(
                        errorParm_t::ERR_FATAL,
                        "VM_CompileX86: pc > header->codeLength".into(),
                    );
                }

                let op = *view.common.code.offset(view.common.pc as isize);
                view.common.pc += 1;
                match opcode_from_i32(op as c_int) {
                    0 => {}
                    x if x == opcode_t::OP_BREAK as c_int => {
                        EmitString(view, c"CC".as_ptr());
                        // int 3
                    }
                    x if x == opcode_t::OP_ENTER as c_int => {
                        EmitString(view, c"81 EE".as_ptr()); // sub	esi, 0x12345678
                        let c4 = Constant4(view.common);
                        Emit4(view.common, c4);
                    }
                    x if x == opcode_t::OP_CONST as c_int => {
                        if *view.common.code.offset((view.common.pc + 4) as isize) as c_int
                            == opcode_t::OP_LOAD4 as c_int
                        {
                            EmitAddEDI4(view, vm);
                            EmitString(view, c"BB".as_ptr()); // mov	ebx, 0x12345678
                            let c4 = Constant4(view.common);
                            Emit4(view.common, (c4 & (*vm).dataMask) + (*vm).dataBase as c_int);
                            EmitString(view, c"8B 03".as_ptr()); // mov	eax, dword ptr [ebx]
                            EmitCommand(
                                view,
                                ELastCommand::LAST_COMMAND_MOV_EDI_EAX,
                            ); // mov dword ptr [edi], eax
                            view.common.pc += 1; // OP_LOAD4
                            view.common.instruction += 1;
                        } else if *view.common.code.offset((view.common.pc + 4) as isize) as c_int
                            == opcode_t::OP_LOAD2 as c_int
                        {
                            EmitAddEDI4(view, vm);
                            EmitString(view, c"BB".as_ptr()); // mov	ebx, 0x12345678
                            let c4 = Constant4(view.common);
                            Emit4(view.common, (c4 & (*vm).dataMask) + (*vm).dataBase as c_int);
                            EmitString(view, c"0F B7 03".as_ptr()); // movzx	eax, word ptr [ebx]
                            EmitCommand(
                                view,
                                ELastCommand::LAST_COMMAND_MOV_EDI_EAX,
                            ); // mov dword ptr [edi], eax
                            view.common.pc += 1; // OP_LOAD4
                            view.common.instruction += 1;
                        } else if *view.common.code.offset((view.common.pc + 4) as isize) as c_int
                            == opcode_t::OP_LOAD1 as c_int
                        {
                            EmitAddEDI4(view, vm);
                            EmitString(view, c"BB".as_ptr()); // mov	ebx, 0x12345678
                            let c4 = Constant4(view.common);
                            Emit4(view.common, (c4 & (*vm).dataMask) + (*vm).dataBase as c_int);
                            EmitString(view, c"0F B6 03".as_ptr()); // movzx	eax, byte ptr [ebx]
                            EmitCommand(
                                view,
                                ELastCommand::LAST_COMMAND_MOV_EDI_EAX,
                            ); // mov dword ptr [edi], eax
                            view.common.pc += 1; // OP_LOAD4
                            view.common.instruction += 1;
                        } else if *view.common.code.offset((view.common.pc + 4) as isize) as c_int
                            == opcode_t::OP_STORE4 as c_int
                        {
                            opt = EmitMovEBXEDI(
                                view,
                                vm,
                                (*vm).dataMask & !3,
                            );
                            let _ = opt;
                            EmitString(view, c"B8".as_ptr()); // mov	eax, 0x12345678
                            let c4 = Constant4(view.common);
                            Emit4(view.common, c4);
                            EmitString(view, c"89 83".as_ptr()); // mov dword ptr [ebx+0x12345678], eax
                            Emit4(view.common, (*vm).dataBase as c_int);
                            EmitCommand(
                                view,
                                ELastCommand::LAST_COMMAND_SUB_DI_4,
                            ); // sub edi, 4
                            view.common.pc += 1; // OP_STORE4
                            view.common.instruction += 1;
                        } else if *view.common.code.offset((view.common.pc + 4) as isize) as c_int
                            == opcode_t::OP_STORE2 as c_int
                        {
                            opt = EmitMovEBXEDI(
                                view,
                                vm,
                                (*vm).dataMask & !1,
                            );
                            let _ = opt;
                            EmitString(view, c"B8".as_ptr()); // mov	eax, 0x12345678
                            let c4 = Constant4(view.common);
                            Emit4(view.common, c4);
                            EmitString(view, c"66 89 83".as_ptr()); // mov word ptr [ebx+0x12345678], eax
                            Emit4(view.common, (*vm).dataBase as c_int);
                            EmitCommand(
                                view,
                                ELastCommand::LAST_COMMAND_SUB_DI_4,
                            ); // sub edi, 4
                            view.common.pc += 1; // OP_STORE4
                            view.common.instruction += 1;
                        } else if *view.common.code.offset((view.common.pc + 4) as isize) as c_int
                            == opcode_t::OP_STORE1 as c_int
                        {
                            opt = EmitMovEBXEDI(view, vm, (*vm).dataMask);
                            let _ = opt;
                            EmitString(view, c"B8".as_ptr()); // mov	eax, 0x12345678
                            let c4 = Constant4(view.common);
                            Emit4(view.common, c4);
                            EmitString(view, c"88 83".as_ptr()); // mov byte ptr [ebx+0x12345678], eax
                            Emit4(view.common, (*vm).dataBase as c_int);
                            EmitCommand(
                                view,
                                ELastCommand::LAST_COMMAND_SUB_DI_4,
                            ); // sub edi, 4
                            view.common.pc += 1; // OP_STORE4
                            view.common.instruction += 1;
                        } else if *view.common.code.offset((view.common.pc + 4) as isize) as c_int
                            == opcode_t::OP_ADD as c_int
                        {
                            EmitString(view, c"81 07".as_ptr()); // add dword ptr [edi], 0x1234567
                            let c4 = Constant4(view.common);
                            Emit4(view.common, c4);
                            view.common.pc += 1; // OP_ADD
                            view.common.instruction += 1;
                        } else if *view.common.code.offset((view.common.pc + 4) as isize) as c_int
                            == opcode_t::OP_SUB as c_int
                        {
                            EmitString(view, c"81 2F".as_ptr()); // sub dword ptr [edi], 0x1234567
                            let c4 = Constant4(view.common);
                            Emit4(view.common, c4);
                            view.common.pc += 1; // OP_ADD
                            view.common.instruction += 1;
                        } else {
                            EmitAddEDI4(view, vm);
                            EmitString(view, c"C7 07".as_ptr()); // mov	dword ptr [edi], 0x12345678
                            view.common.last_const = Constant4(view.common);
                            Emit4(view.common, view.common.last_const);
                            if *view.common.code.offset(view.common.pc as isize) as c_int
                                == opcode_t::OP_JUMP as c_int
                            {
                                *view.common.jused.offset(view.common.last_const as isize) = 1;
                            }
                        }
                    }
                    x if x == opcode_t::OP_LOCAL as c_int => {
                        EmitAddEDI4(view, vm);
                        EmitString(view, c"8D 86".as_ptr()); // lea eax, [0x12345678 + esi]
                        view.common.oc0 = view.common.oc1;
                        view.common.oc1 = Constant4(view.common);
                        Emit4(view.common, view.common.oc1);
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_MOV_EDI_EAX,
                        ); // mov dword ptr [edi], eax
                    }
                    x if x == opcode_t::OP_ARG as c_int => {
                        EmitMovEAXEDI(view, vm); // mov	eax,dword ptr [edi]
                        EmitString(view, c"89 86".as_ptr()); // mov	dword ptr [esi+database],eax
                                                                                       // FIXME: range check
                        let c1 = Constant1(view.common);
                        Emit4(view.common, c1 + (*vm).dataBase as c_int);
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_CALL as c_int => {
                        EmitString(view, c"C7 86".as_ptr()); // mov dword ptr [esi+database],0x12345678
                        Emit4(view.common, (*vm).dataBase as c_int);
                        Emit4(view.common, view.common.pc);
                        EmitString(view, c"FF 15".as_ptr()); // call asmCallPtr
                        Emit4(view.common, core::ptr::addr_of!(view.common.asm_call_ptr) as c_int);
                    }
                    x if x == opcode_t::OP_PUSH as c_int => {
                        EmitAddEDI4(view, vm);
                    }
                    x if x == opcode_t::OP_POP as c_int => {
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_LEAVE as c_int => {
                        let v = Constant4(view.common);
                        EmitString(view, c"81 C6".as_ptr()); // add	esi, 0x12345678
                        Emit4(view.common, v);
                        EmitString(view, c"C3".as_ptr());
                        // ret
                    }
                    x if x == opcode_t::OP_LOAD4 as c_int => {
                        if *view.common.code.offset(view.common.pc as isize) as c_int
                            == opcode_t::OP_CONST as c_int
                            && *view.common.code.offset((view.common.pc + 5) as isize) as c_int
                                == opcode_t::OP_ADD as c_int
                            && *view.common.code.offset((view.common.pc + 6) as isize) as c_int
                                == opcode_t::OP_STORE4 as c_int
                        {
                            if view.common.oc0 == view.common.oc1
                                && view.common.pop0 == opcode_t::OP_LOCAL as c_int
                                && view.common.pop1 == opcode_t::OP_LOCAL as c_int
                            {
                                view.common.compiled_ofs -= 11;
                                *(*vm)
                                    .instructionPointers
                                    .offset((view.common.instruction - 1) as isize) =
                                    view.common.compiled_ofs;
                            }
                            view.common.pc += 1; // OP_CONST
                            let v = Constant4(view.common);
                            EmitMovEBXEDI(view, vm, (*vm).dataMask);
                            if v == 1
                                && view.common.oc0 == view.common.oc1
                                && view.common.pop0 == opcode_t::OP_LOCAL as c_int
                                && view.common.pop1 == opcode_t::OP_LOCAL as c_int
                            {
                                EmitString(view, c"FF 83".as_ptr()); // inc dword ptr [ebx + 0x12345678]
                                Emit4(view.common, (*vm).dataBase as c_int);
                            } else {
                                EmitString(view, c"8B 83".as_ptr()); // mov	eax, dword ptr [ebx + 0x12345678]
                                Emit4(view.common, (*vm).dataBase as c_int);
                                EmitString(view, c"05".as_ptr()); // add eax, const
                                Emit4(view.common, v);
                                if view.common.oc0 == view.common.oc1
                                    && view.common.pop0 == opcode_t::OP_LOCAL as c_int
                                    && view.common.pop1 == opcode_t::OP_LOCAL as c_int
                                {
                                    EmitString(view, c"89 83".as_ptr()); // mov dword ptr [ebx+0x12345678], eax
                                    Emit4(view.common, (*vm).dataBase as c_int);
                                } else {
                                    EmitCommand(
                                        view,
                                        ELastCommand::LAST_COMMAND_SUB_DI_4,
                                    ); // sub edi, 4
                                    EmitString(view, c"8B 1F".as_ptr()); // mov	ebx, dword ptr [edi]
                                    EmitString(view, c"89 83".as_ptr()); // mov dword ptr [ebx+0x12345678], eax
                                    Emit4(view.common, (*vm).dataBase as c_int);
                                }
                            }
                            EmitCommand(
                                view,
                                ELastCommand::LAST_COMMAND_SUB_DI_4,
                            ); // sub edi, 4
                            view.common.pc += 1; // OP_ADD
                            view.common.pc += 1; // OP_STORE
                            view.common.instruction += 3;
                        } else if *view.common.code.offset(view.common.pc as isize) as c_int
                            == opcode_t::OP_CONST as c_int
                            && *view.common.code.offset((view.common.pc + 5) as isize) as c_int
                                == opcode_t::OP_SUB as c_int
                            && *view.common.code.offset((view.common.pc + 6) as isize) as c_int
                                == opcode_t::OP_STORE4 as c_int
                        {
                            if view.common.oc0 == view.common.oc1
                                && view.common.pop0 == opcode_t::OP_LOCAL as c_int
                                && view.common.pop1 == opcode_t::OP_LOCAL as c_int
                            {
                                view.common.compiled_ofs -= 11;
                                *(*vm)
                                    .instructionPointers
                                    .offset((view.common.instruction - 1) as isize) =
                                    view.common.compiled_ofs;
                            }
                            EmitMovEBXEDI(view, vm, (*vm).dataMask);
                            EmitString(view, c"8B 83".as_ptr()); // mov	eax, dword ptr [ebx + 0x12345678]
                            Emit4(view.common, (*vm).dataBase as c_int);
                            view.common.pc += 1; // OP_CONST
                            let v = Constant4(view.common);
                            if v == 1
                                && view.common.oc0 == view.common.oc1
                                && view.common.pop0 == opcode_t::OP_LOCAL as c_int
                                && view.common.pop1 == opcode_t::OP_LOCAL as c_int
                            {
                                EmitString(view, c"FF 8B".as_ptr()); // dec dword ptr [ebx + 0x12345678]
                                Emit4(view.common, (*vm).dataBase as c_int);
                            } else {
                                EmitString(view, c"2D".as_ptr()); // sub eax, const
                                Emit4(view.common, v);
                                if view.common.oc0 == view.common.oc1
                                    && view.common.pop0 == opcode_t::OP_LOCAL as c_int
                                    && view.common.pop1 == opcode_t::OP_LOCAL as c_int
                                {
                                    EmitString(view, c"89 83".as_ptr()); // mov dword ptr [ebx+0x12345678], eax
                                    Emit4(view.common, (*vm).dataBase as c_int);
                                } else {
                                    EmitCommand(
                                        view,
                                        ELastCommand::LAST_COMMAND_SUB_DI_4,
                                    ); // sub edi, 4
                                    EmitString(view, c"8B 1F".as_ptr()); // mov	ebx, dword ptr [edi]
                                    EmitString(view, c"89 83".as_ptr()); // mov dword ptr [ebx+0x12345678], eax
                                    Emit4(view.common, (*vm).dataBase as c_int);
                                }
                            }
                            EmitCommand(
                                view,
                                ELastCommand::LAST_COMMAND_SUB_DI_4,
                            ); // sub edi, 4
                            view.common.pc += 1; // OP_SUB
                            view.common.pc += 1; // OP_STORE
                            view.common.instruction += 3;
                        } else if *view.common.buf.offset((view.common.compiled_ofs - 2) as isize) == 0x89
                            && *view.common.buf.offset((view.common.compiled_ofs - 1) as isize) == 0x07
                        {
                            view.common.compiled_ofs -= 2;
                            *(*vm)
                                .instructionPointers
                                .offset((view.common.instruction - 1) as isize) = view.common.compiled_ofs;
                            EmitString(view, c"8B 80".as_ptr()); // mov eax, dword ptr [eax + 0x1234567]
                            Emit4(view.common, (*vm).dataBase as c_int);
                            EmitCommand(
                                view,
                                ELastCommand::LAST_COMMAND_MOV_EDI_EAX,
                            ); // mov dword ptr [edi], eax
                        } else {
                            EmitMovEBXEDI(view, vm, (*vm).dataMask);
                            EmitString(view, c"8B 83".as_ptr()); // mov	eax, dword ptr [ebx + 0x12345678]
                            Emit4(view.common, (*vm).dataBase as c_int);
                            EmitCommand(
                                view,
                                ELastCommand::LAST_COMMAND_MOV_EDI_EAX,
                            ); // mov dword ptr [edi], eax
                        }
                    }
                    x if x == opcode_t::OP_LOAD2 as c_int => {
                        EmitMovEBXEDI(view, vm, (*vm).dataMask);
                        EmitString(view, c"0F B7 83".as_ptr()); // movzx	eax, word ptr [ebx + 0x12345678]
                        Emit4(view.common, (*vm).dataBase as c_int);
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_MOV_EDI_EAX,
                        ); // mov dword ptr [edi], eax
                    }
                    x if x == opcode_t::OP_LOAD1 as c_int => {
                        EmitMovEBXEDI(view, vm, (*vm).dataMask);
                        EmitString(view, c"0F B6 83".as_ptr()); // movzx eax, byte ptr [ebx + 0x12345678]
                        Emit4(view.common, (*vm).dataBase as c_int);
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_MOV_EDI_EAX,
                        ); // mov dword ptr [edi], eax
                    }
                    x if x == opcode_t::OP_STORE4 as c_int => {
                        EmitMovEAXEDI(view, vm);
                        EmitString(view, c"8B 5F FC".as_ptr()); // mov	ebx, dword ptr [edi-4]
                        EmitString(view, c"89 83".as_ptr()); // mov dword ptr [ebx+0x12345678], eax
                        Emit4(view.common, (*vm).dataBase as c_int);
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                    }
                    x if x == opcode_t::OP_STORE2 as c_int => {
                        EmitMovEAXEDI(view, vm);
                        EmitString(view, c"8B 5F FC".as_ptr()); // mov	ebx, dword ptr [edi-4]
                        EmitString(view, c"66 89 83".as_ptr()); // mov word ptr [ebx+0x12345678], eax
                        Emit4(view.common, (*vm).dataBase as c_int);
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                    }
                    x if x == opcode_t::OP_STORE1 as c_int => {
                        EmitMovEAXEDI(view, vm);
                        EmitString(view, c"8B 5F FC".as_ptr()); // mov	ebx, dword ptr [edi-4]
                        EmitString(view, c"88 83".as_ptr()); // mov byte ptr [ebx+0x12345678], eax
                        Emit4(view.common, (*vm).dataBase as c_int);
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                    }
                    x if x == opcode_t::OP_EQ as c_int => {
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(view, c"8B 47 04".as_ptr()); // mov	eax, dword ptr [edi+4]
                        EmitString(view, c"3B 47 08".as_ptr()); // cmp	eax, dword ptr [edi+8]
                        EmitString(view, c"75 06".as_ptr()); // jne +6
                        EmitString(view, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(view.common);
                        *view.common.jused.offset(v as isize) = 1;
                        Emit4(
                            view.common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_NE as c_int => {
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(view, c"8B 47 04".as_ptr()); // mov	eax, dword ptr [edi+4]
                        EmitString(view, c"3B 47 08".as_ptr()); // cmp	eax, dword ptr [edi+8]
                        EmitString(view, c"74 06".as_ptr()); // je +6
                        EmitString(view, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(view.common);
                        *view.common.jused.offset(v as isize) = 1;
                        Emit4(
                            view.common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_LTI as c_int => {
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(view, c"8B 47 04".as_ptr()); // mov	eax, dword ptr [edi+4]
                        EmitString(view, c"3B 47 08".as_ptr()); // cmp	eax, dword ptr [edi+8]
                        EmitString(view, c"7D 06".as_ptr()); // jnl +6
                        EmitString(view, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(view.common);
                        *view.common.jused.offset(v as isize) = 1;
                        Emit4(
                            view.common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_LEI as c_int => {
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(view, c"8B 47 04".as_ptr()); // mov	eax, dword ptr [edi+4]
                        EmitString(view, c"3B 47 08".as_ptr()); // cmp	eax, dword ptr [edi+8]
                        EmitString(view, c"7F 06".as_ptr()); // jnle +6
                        EmitString(view, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(view.common);
                        *view.common.jused.offset(v as isize) = 1;
                        Emit4(
                            view.common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_GTI as c_int => {
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(view, c"8B 47 04".as_ptr()); // mov	eax, dword ptr [edi+4]
                        EmitString(view, c"3B 47 08".as_ptr()); // cmp	eax, dword ptr [edi+8]
                        EmitString(view, c"7E 06".as_ptr()); // jng +6
                        EmitString(view, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(view.common);
                        *view.common.jused.offset(v as isize) = 1;
                        Emit4(
                            view.common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_GEI as c_int => {
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(view, c"8B 47 04".as_ptr()); // mov	eax, dword ptr [edi+4]
                        EmitString(view, c"3B 47 08".as_ptr()); // cmp	eax, dword ptr [edi+8]
                        EmitString(view, c"7C 06".as_ptr()); // jnge +6
                        EmitString(view, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(view.common);
                        *view.common.jused.offset(v as isize) = 1;
                        Emit4(
                            view.common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_LTU as c_int => {
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(view, c"8B 47 04".as_ptr()); // mov	eax, dword ptr [edi+4]
                        EmitString(view, c"3B 47 08".as_ptr()); // cmp	eax, dword ptr [edi+8]
                        EmitString(view, c"73 06".as_ptr()); // jnb +6
                        EmitString(view, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(view.common);
                        *view.common.jused.offset(v as isize) = 1;
                        Emit4(
                            view.common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_LEU as c_int => {
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(view, c"8B 47 04".as_ptr()); // mov	eax, dword ptr [edi+4]
                        EmitString(view, c"3B 47 08".as_ptr()); // cmp	eax, dword ptr [edi+8]
                        EmitString(view, c"77 06".as_ptr()); // jnbe +6
                        EmitString(view, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(view.common);
                        *view.common.jused.offset(v as isize) = 1;
                        Emit4(
                            view.common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_GTU as c_int => {
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(view, c"8B 47 04".as_ptr()); // mov	eax, dword ptr [edi+4]
                        EmitString(view, c"3B 47 08".as_ptr()); // cmp	eax, dword ptr [edi+8]
                        EmitString(view, c"76 06".as_ptr()); // jna +6
                        EmitString(view, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(view.common);
                        *view.common.jused.offset(v as isize) = 1;
                        Emit4(
                            view.common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_GEU as c_int => {
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(view, c"8B 47 04".as_ptr()); // mov	eax, dword ptr [edi+4]
                        EmitString(view, c"3B 47 08".as_ptr()); // cmp	eax, dword ptr [edi+8]
                        EmitString(view, c"72 06".as_ptr()); // jnae +6
                        EmitString(view, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(view.common);
                        *view.common.jused.offset(v as isize) = 1;
                        Emit4(
                            view.common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_EQF as c_int => {
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(view, c"D9 47 04".as_ptr()); // fld dword ptr [edi+4]
                        EmitString(view, c"D8 5F 08".as_ptr()); // fcomp dword ptr [edi+8]
                        EmitString(view, c"DF E0".as_ptr()); // fnstsw ax
                        EmitString(view, c"F6 C4 40".as_ptr()); // test	ah,0x40
                        EmitString(view, c"74 06".as_ptr()); // je +6
                        EmitString(view, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(view.common);
                        *view.common.jused.offset(v as isize) = 1;
                        Emit4(
                            view.common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_NEF as c_int => {
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(view, c"D9 47 04".as_ptr()); // fld dword ptr [edi+4]
                        EmitString(view, c"D8 5F 08".as_ptr()); // fcomp dword ptr [edi+8]
                        EmitString(view, c"DF E0".as_ptr()); // fnstsw ax
                        EmitString(view, c"F6 C4 40".as_ptr()); // test	ah,0x40
                        EmitString(view, c"75 06".as_ptr()); // jne +6
                        EmitString(view, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(view.common);
                        *view.common.jused.offset(v as isize) = 1;
                        Emit4(
                            view.common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_LTF as c_int => {
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(view, c"D9 47 04".as_ptr()); // fld dword ptr [edi+4]
                        EmitString(view, c"D8 5F 08".as_ptr()); // fcomp dword ptr [edi+8]
                        EmitString(view, c"DF E0".as_ptr()); // fnstsw ax
                        EmitString(view, c"F6 C4 01".as_ptr()); // test	ah,0x01
                        EmitString(view, c"74 06".as_ptr()); // je +6
                        EmitString(view, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(view.common);
                        *view.common.jused.offset(v as isize) = 1;
                        Emit4(
                            view.common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_LEF as c_int => {
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(view, c"D9 47 04".as_ptr()); // fld dword ptr [edi+4]
                        EmitString(view, c"D8 5F 08".as_ptr()); // fcomp dword ptr [edi+8]
                        EmitString(view, c"DF E0".as_ptr()); // fnstsw ax
                        EmitString(view, c"F6 C4 41".as_ptr()); // test	ah,0x41
                        EmitString(view, c"74 06".as_ptr()); // je +6
                        EmitString(view, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(view.common);
                        *view.common.jused.offset(v as isize) = 1;
                        Emit4(
                            view.common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_GTF as c_int => {
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(view, c"D9 47 04".as_ptr()); // fld dword ptr [edi+4]
                        EmitString(view, c"D8 5F 08".as_ptr()); // fcomp dword ptr [edi+8]
                        EmitString(view, c"DF E0".as_ptr()); // fnstsw ax
                        EmitString(view, c"F6 C4 41".as_ptr()); // test	ah,0x41
                        EmitString(view, c"75 06".as_ptr()); // jne +6
                        EmitString(view, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(view.common);
                        *view.common.jused.offset(v as isize) = 1;
                        Emit4(
                            view.common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_GEF as c_int => {
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(view, c"D9 47 04".as_ptr()); // fld dword ptr [edi+4]
                        EmitString(view, c"D8 5F 08".as_ptr()); // fcomp dword ptr [edi+8]
                        EmitString(view, c"DF E0".as_ptr()); // fnstsw ax
                        EmitString(view, c"F6 C4 01".as_ptr()); // test	ah,0x01
                        EmitString(view, c"75 06".as_ptr()); // jne +6
                        EmitString(view, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(view.common);
                        *view.common.jused.offset(v as isize) = 1;
                        Emit4(
                            view.common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_NEGI as c_int => {
                        EmitString(view, c"F7 1F".as_ptr());
                        // neg dword ptr [edi]
                    }
                    x if x == opcode_t::OP_ADD as c_int => {
                        EmitMovEAXEDI(view, vm); // mov eax, dword ptr [edi]
                        EmitString(view, c"01 47 FC".as_ptr()); // add dword ptr [edi-4],eax
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_SUB as c_int => {
                        EmitMovEAXEDI(view, vm); // mov eax, dword ptr [edi]
                        EmitString(view, c"29 47 FC".as_ptr()); // sub dword ptr [edi-4],eax
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_DIVI as c_int => {
                        EmitString(view, c"8B 47 FC".as_ptr()); // mov eax,dword ptr [edi-4]
                        EmitString(view, c"99".as_ptr()); // cdq
                        EmitString(view, c"F7 3F".as_ptr()); // idiv dword ptr [edi]
                        EmitString(view, c"89 47 FC".as_ptr()); // mov dword ptr [edi-4],eax
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_DIVU as c_int => {
                        EmitString(view, c"8B 47 FC".as_ptr()); // mov eax,dword ptr [edi-4]
                        EmitString(view, c"33 D2".as_ptr()); // xor edx, edx
                        EmitString(view, c"F7 37".as_ptr()); // div dword ptr [edi]
                        EmitString(view, c"89 47 FC".as_ptr()); // mov dword ptr [edi-4],eax
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_MODI as c_int => {
                        EmitString(view, c"8B 47 FC".as_ptr()); // mov eax,dword ptr [edi-4]
                        EmitString(view, c"99".as_ptr()); // cdq
                        EmitString(view, c"F7 3F".as_ptr()); // idiv dword ptr [edi]
                        EmitString(view, c"89 57 FC".as_ptr()); // mov dword ptr [edi-4],edx
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_MODU as c_int => {
                        EmitString(view, c"8B 47 FC".as_ptr()); // mov eax,dword ptr [edi-4]
                        EmitString(view, c"33 D2".as_ptr()); // xor edx, edx
                        EmitString(view, c"F7 37".as_ptr()); // div dword ptr [edi]
                        EmitString(view, c"89 57 FC".as_ptr()); // mov dword ptr [edi-4],edx
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_MULI as c_int => {
                        EmitString(view, c"8B 47 FC".as_ptr()); // mov eax,dword ptr [edi-4]
                        EmitString(view, c"F7 2F".as_ptr()); // imul dword ptr [edi]
                        EmitString(view, c"89 47 FC".as_ptr()); // mov dword ptr [edi-4],eax
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_MULU as c_int => {
                        EmitString(view, c"8B 47 FC".as_ptr()); // mov eax,dword ptr [edi-4]
                        EmitString(view, c"F7 27".as_ptr()); // mul dword ptr [edi]
                        EmitString(view, c"89 47 FC".as_ptr()); // mov dword ptr [edi-4],eax
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_BAND as c_int => {
                        EmitMovEAXEDI(view, vm); // mov eax, dword ptr [edi]
                        EmitString(view, c"21 47 FC".as_ptr()); // and dword ptr [edi-4],eax
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_BOR as c_int => {
                        EmitMovEAXEDI(view, vm); // mov eax, dword ptr [edi]
                        EmitString(view, c"09 47 FC".as_ptr()); // or dword ptr [edi-4],eax
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_BXOR as c_int => {
                        EmitMovEAXEDI(view, vm); // mov eax, dword ptr [edi]
                        EmitString(view, c"31 47 FC".as_ptr()); // xor dword ptr [edi-4],eax
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_BCOM as c_int => {
                        EmitString(view, c"F7 17".as_ptr());
                        // not dword ptr [edi]
                    }
                    x if x == opcode_t::OP_LSH as c_int => {
                        EmitString(view, c"8B 0F".as_ptr()); // mov ecx, dword ptr [edi]
                        EmitString(view, c"D3 67 FC".as_ptr()); // shl dword ptr [edi-4], cl
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_RSHI as c_int => {
                        EmitString(view, c"8B 0F".as_ptr()); // mov ecx, dword ptr [edi]
                        EmitString(view, c"D3 7F FC".as_ptr()); // sar dword ptr [edi-4], cl
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_RSHU as c_int => {
                        EmitString(view, c"8B 0F".as_ptr()); // mov ecx, dword ptr [edi]
                        EmitString(view, c"D3 6F FC".as_ptr()); // shr dword ptr [edi-4], cl
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_NEGF as c_int => {
                        EmitString(view, c"D9 07".as_ptr()); // fld dword ptr [edi]
                        EmitString(view, c"D9 E0".as_ptr()); // fchs
                        EmitString(view, c"D9 1F".as_ptr());
                        // fstp dword ptr [edi]
                    }
                    x if x == opcode_t::OP_ADDF as c_int => {
                        EmitString(view, c"D9 47 FC".as_ptr()); // fld dword ptr [edi-4]
                        EmitString(view, c"D8 07".as_ptr()); // fadd dword ptr [edi]
                        EmitString(view, c"D9 5F FC".as_ptr()); // fstp dword ptr [edi-4]
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_SUBF as c_int => {
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                        EmitString(view, c"D9 07".as_ptr()); // fld dword ptr [edi]
                        EmitString(view, c"D8 67 04".as_ptr()); // fsub dword ptr [edi+4]
                        EmitString(view, c"D9 1F".as_ptr());
                        // fstp dword ptr [edi]
                    }
                    x if x == opcode_t::OP_DIVF as c_int => {
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                        EmitString(view, c"D9 07".as_ptr()); // fld dword ptr [edi]
                        EmitString(view, c"D8 77 04".as_ptr()); // fdiv dword ptr [edi+4]
                        EmitString(view, c"D9 1F".as_ptr());
                        // fstp dword ptr [edi]
                    }
                    x if x == opcode_t::OP_MULF as c_int => {
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                        EmitString(view, c"D9 07".as_ptr()); // fld dword ptr [edi]
                        EmitString(view, c"D8 4f 04".as_ptr()); // fmul dword ptr [edi+4]
                        EmitString(view, c"D9 1F".as_ptr());
                        // fstp dword ptr [edi]
                    }
                    x if x == opcode_t::OP_CVIF as c_int => {
                        EmitString(view, c"DB 07".as_ptr()); // fild dword ptr [edi]
                        EmitString(view, c"D9 1F".as_ptr());
                        // fstp dword ptr [edi]
                    }
                    x if x == opcode_t::OP_CVFI as c_int => {
                        // Raven: `#ifndef FTOL_PTR` selects the non-IEEE-compliant
                        // direct fistp path; FTOL_PTR is unset in the WinDed build
                        // this port targets (no rosetta row — reported missing).
                        EmitString(view, c"D9 07".as_ptr()); // fld dword ptr [edi]
                        EmitString(view, c"DB 1F".as_ptr());
                        // fistp dword ptr [edi]
                    }
                    x if x == opcode_t::OP_SEX8 as c_int => {
                        EmitString(view, c"0F BE 07".as_ptr()); // movsx eax, byte ptr [edi]
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_MOV_EDI_EAX,
                        ); // mov dword ptr [edi], eax
                    }
                    x if x == opcode_t::OP_SEX16 as c_int => {
                        EmitString(view, c"0F BF 07".as_ptr()); // movsx eax, word ptr [edi]
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_MOV_EDI_EAX,
                        ); // mov dword ptr [edi], eax
                    }
                    x if x == opcode_t::OP_BLOCK_COPY as c_int => {
                        // FIXME: range check
                        EmitString(view, c"56".as_ptr()); // push esi
                        EmitString(view, c"57".as_ptr()); // push edi
                        EmitString(view, c"8B 37".as_ptr()); // mov esi,[edi]
                        EmitString(view, c"8B 7F FC".as_ptr()); // mov edi,[edi-4]
                        EmitString(view, c"B9".as_ptr()); // mov ecx,0x12345678
                        let c4 = Constant4(view.common);
                        Emit4(view.common, c4 >> 2);
                        EmitString(view, c"B8".as_ptr()); // mov eax, datamask
                        Emit4(view.common, (*vm).dataMask);
                        EmitString(view, c"BB".as_ptr()); // mov ebx, database
                        Emit4(view.common, (*vm).dataBase as c_int);
                        EmitString(view, c"23 F0".as_ptr()); // and esi, eax
                        EmitString(view, c"03 F3".as_ptr()); // add esi, ebx
                        EmitString(view, c"23 F8".as_ptr()); // and edi, eax
                        EmitString(view, c"03 FB".as_ptr()); // add edi, ebx
                        EmitString(view, c"F3 A5".as_ptr()); // rep movsd
                        EmitString(view, c"5F".as_ptr()); // pop edi
                        EmitString(view, c"5E".as_ptr()); // pop esi
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                    }
                    x if x == opcode_t::OP_JUMP as c_int => {
                        EmitCommand(
                            view,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                        EmitString(view, c"8B 47 04".as_ptr()); // mov eax,dword ptr [edi+4]
                                                                                          // FIXME: range check
                        EmitString(view, c"FF 24 85".as_ptr()); // jmp dword ptr [instructionPointers + eax * 4]
                        Emit4(view.common, (*vm).instructionPointers as c_int);
                    }
                    _ => {
                        com_error(
                            errorParm_t::ERR_DROP,
                            format!("VM_CompileX86: bad opcode {} at offset {}", op, view.common.pc),
                        );
                    }
                }
                view.common.pop0 = view.common.pop1;
                view.common.pop1 = op as c_int;
            }
        }

        // copy to an exact size buffer on the hunk
        (*vm).codeLength = view.common.compiled_ofs;
        (*vm).codeBase =
            Hunk_Alloc(view, view.common.compiled_ofs, ha_pref::h_low) as *mut byte;
        Com_Memcpy(
            (*vm).codeBase as *mut (),
            view.common.buf as *const (),
            view.common.compiled_ofs as usize,
        );
        Z_Free(view.common, view.common.buf as *mut ());
        Z_Free(view.common, view.common.jused as *mut ());
        let vm_name = CStr::from_ptr((*vm).name.as_ptr()).to_string_lossy();
        let msg = format!(
            "VM file {} compiled to {} bytes of code\n",
            vm_name, view.common.compiled_ofs
        );
        com_printf(view.common, &msg);

        // offset all the instruction pointers for the new location
        for i in 0..(*header).instructionCount {
            *(*vm).instructionPointers.offset(i as isize) += (*vm).codeBase as c_int;
        }

        // Raven: `#if 0 // ndef _WIN32` — the mprotect(PROT_EXEC) block is
        // dead under `#if 0` in the oracle itself; not transcribed (§20-class,
        // Raven's own preprocessor drops it).
    }
}

/// Raven `opcode_t` cast helper — `op` is read as a raw byte off the wire and
/// switched on as an int (`switch (op)`), matching values that fall outside
/// the enum (Raven's `case 0:` arm) as well as in-range opcodes.
#[inline]
fn opcode_from_i32(op: c_int) -> c_int {
    op
}
