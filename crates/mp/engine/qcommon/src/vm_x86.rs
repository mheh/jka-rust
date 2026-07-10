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

use mp_host_interface::engine_host::EngineHost;
use mp_qshared::shared::error_parm::errorParm_t;
use native_types::{byte, qboolean, qfalse, qtrue};

use crate::collision_world::CollisionWorld;
use crate::common::Common;
use crate::qfiles::vm_header_t::vmHeader_t;
use crate::vm::elastcommand::ELastCommand;
use crate::vm::opcode_t::opcode_t;
use crate::vm::vm_s::vm_t;

// PORT-NOTE(rm-types): `RenderModels`/`RmManager` are state-receiver types
// pinned by the engine-fork-discovery preamble's receiver order
// (rmg-terrain.md/tr-model.md own their real shape); neither has landed in
// this crate yet. Referenced by their exact resolved-signature names per the
// no-stub rule (common_fns.rs precedent); reported as missing symbols for the
// finisher to replace with the real imports once they land.
#[allow(dead_code)]
struct RenderModels;
#[allow(dead_code)]
struct RmManager;

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
            out("ax") _, out("di") _, out("si") _, out("cx") _,
            options(att_syntax),
        );
    }
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
pub fn Hex(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    c: c_int,
) -> c_int {
    if c >= b'a' as c_int && c <= b'f' as c_int {
        return 10 + c - b'a' as c_int;
    }
    if c >= b'A' as c_int && c <= b'F' as c_int {
        return 10 + c - b'A' as c_int;
    }
    if (b'0' as c_int..=b'9' as c_int).contains(&c) {
        return c - b'0' as c_int;
    }

    // PORT-NOTE(missing-callee): `Com_Error` is part of the still-unlanded
    // cm_load.cpp cyclic unit (qcommon__1592_CM_DeleteCachedMap.md); called
    // with the exact resolved receivers and reported as a missing symbol.
    Com_Error(
        common,
        cm,
        rm,
        rmg,
        host,
        errorParm_t::ERR_DROP as c_int,
        c"Hex: bad char '%c'".as_ptr(),
        c,
    );

    0
}

/// `EmitString`.
///
/// Source: `oracle/codemp/qcommon/vm_x86.cpp:255-271`
pub fn EmitString(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    string: *const c_char,
) {
    unsafe {
        let mut string = string;
        loop {
            let c1 = *string as c_int;
            let c2 = *string.offset(1) as c_int;

            let v = (Hex(common, cm, rm, rmg, host, c1) << 4) | Hex(common, cm, rm, rmg, host, c2);
            Emit1(common, v);

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
pub fn EmitCommand(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    command: ELastCommand,
) {
    match command {
        ELastCommand::LAST_COMMAND_MOV_EDI_EAX => {
            EmitString(common, cm, rm, rmg, host, c"89 07".as_ptr()); // mov dword ptr [edi], eax
        }
        ELastCommand::LAST_COMMAND_SUB_DI_4 => {
            EmitString(common, cm, rm, rmg, host, c"83 EF 04".as_ptr()); // sub edi, 4
        }
        ELastCommand::LAST_COMMAND_SUB_DI_8 => {
            EmitString(common, cm, rm, rmg, host, c"83 EF 08".as_ptr()); // sub edi, 8
        }
        _ => {}
    }
    common.last_command = command;
}

/// `EmitAddEDI4`.
///
/// Source: `oracle/codemp/qcommon/vm_x86.cpp:294-309`
pub fn EmitAddEDI4(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    vm: *mut vm_t,
) {
    unsafe {
        if common.last_command == ELastCommand::LAST_COMMAND_SUB_DI_4
            && *common.jused.offset((common.instruction - 1) as isize) == 0
        {
            // sub di,4
            common.compiled_ofs -= 3;
            *(*vm)
                .instructionPointers
                .offset((common.instruction - 1) as isize) = common.compiled_ofs;
            return;
        }
        if common.last_command == ELastCommand::LAST_COMMAND_SUB_DI_8
            && *common.jused.offset((common.instruction - 1) as isize) == 0
        {
            // sub di,8
            common.compiled_ofs -= 3;
            *(*vm)
                .instructionPointers
                .offset((common.instruction - 1) as isize) = common.compiled_ofs;
            EmitString(common, cm, rm, rmg, host, c"83 EF 04".as_ptr()); //	sub edi,4
            return;
        }
    }
    EmitString(common, cm, rm, rmg, host, c"83 C7 04".as_ptr()); //	add edi,4
}

/// `EmitMovEAXEDI`.
///
/// Source: `oracle/codemp/qcommon/vm_x86.cpp:311-332`
pub fn EmitMovEAXEDI(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    vm: *mut vm_t,
) {
    unsafe {
        if common.last_command == ELastCommand::LAST_COMMAND_MOV_EDI_EAX {
            // mov [edi], eax
            common.compiled_ofs -= 2;
            *(*vm)
                .instructionPointers
                .offset((common.instruction - 1) as isize) = common.compiled_ofs;
            return;
        }
        if common.pop1 == opcode_t::OP_DIVI as c_int
            || common.pop1 == opcode_t::OP_DIVU as c_int
            || common.pop1 == opcode_t::OP_MULI as c_int
            || common.pop1 == opcode_t::OP_MULU as c_int
            || common.pop1 == opcode_t::OP_STORE4 as c_int
            || common.pop1 == opcode_t::OP_STORE2 as c_int
            || common.pop1 == opcode_t::OP_STORE1 as c_int
        {
            return;
        }
        if common.pop1 == opcode_t::OP_CONST as c_int
            && *common.buf.offset((common.compiled_ofs - 6) as isize) == 0xC7
            && *common.buf.offset((common.compiled_ofs - 5) as isize) == 0x07
        {
            // mov edi, 0x123456
            common.compiled_ofs -= 6;
            *(*vm)
                .instructionPointers
                .offset((common.instruction - 1) as isize) = common.compiled_ofs;
            EmitString(common, cm, rm, rmg, host, c"B8".as_ptr()); // mov	eax, 0x12345678
            Emit4(common, common.last_const);
            return;
        }
    }
    EmitString(common, cm, rm, rmg, host, c"8B 07".as_ptr()); // mov eax, dword ptr [edi]
}

/// `EmitMovEBXEDI`.
///
/// Source: `oracle/codemp/qcommon/vm_x86.cpp:334-363`
pub fn EmitMovEBXEDI(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    vm: *mut vm_t,
    andit: c_int,
) -> qboolean {
    unsafe {
        if common.last_command == ELastCommand::LAST_COMMAND_MOV_EDI_EAX {
            // mov [edi], eax
            common.compiled_ofs -= 2;
            *(*vm)
                .instructionPointers
                .offset((common.instruction - 1) as isize) = common.compiled_ofs;
            EmitString(common, cm, rm, rmg, host, c"8B D8".as_ptr()); // mov bx, eax
            return qfalse;
        }
        if common.pop1 == opcode_t::OP_DIVI as c_int
            || common.pop1 == opcode_t::OP_DIVU as c_int
            || common.pop1 == opcode_t::OP_MULI as c_int
            || common.pop1 == opcode_t::OP_MULU as c_int
            || common.pop1 == opcode_t::OP_STORE4 as c_int
            || common.pop1 == opcode_t::OP_STORE2 as c_int
            || common.pop1 == opcode_t::OP_STORE1 as c_int
        {
            EmitString(common, cm, rm, rmg, host, c"8B D8".as_ptr()); // mov bx, eax
            return qfalse;
        }
        if common.pop1 == opcode_t::OP_CONST as c_int
            && *common.buf.offset((common.compiled_ofs - 6) as isize) == 0xC7
            && *common.buf.offset((common.compiled_ofs - 5) as isize) == 0x07
        {
            // mov edi, 0x123456
            common.compiled_ofs -= 6;
            *(*vm)
                .instructionPointers
                .offset((common.instruction - 1) as isize) = common.compiled_ofs;
            EmitString(common, cm, rm, rmg, host, c"BB".as_ptr()); // mov	ebx, 0x12345678
            if andit != 0 {
                Emit4(common, common.last_const & andit);
            } else {
                Emit4(common, common.last_const);
            }
            return qtrue;
        }
    }

    EmitString(common, cm, rm, rmg, host, c"8B 1F".as_ptr()); // mov ebx, dword ptr [edi]
    qfalse
}

/// `VM_Compile`.
///
/// Raven: compiles a loaded QVM module's bytecode to native x86 machine code
/// in two passes (peephole-optimizing pass 0, final-length pass 1), then
/// copies the result to an exact-size Hunk allocation. Data-faithful emitter
/// (ruling 6): executes only on x86 hosts, same restriction as the oracle.
/// Source: `oracle/codemp/qcommon/vm_x86.cpp:370-1056`
pub fn VM_Compile(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    vm: *mut vm_t,
    header: *mut vmHeader_t,
) {
    // PORT-NOTE(rm-types): `RmManager` is required transitively by the
    // `Hex`/`EmitString`/`EmitCommand`/`EmitAddEDI4`/`EmitMovEAXEDI`/
    // `EmitMovEBXEDI` callees' resolved signatures but is not itself a
    // receiver of `VM_Compile` per the packet's printed signature; a local
    // placeholder default is threaded through to satisfy those callees until
    // the real `RmManager` state lands (missing-symbol reported).
    let mut rmg = RmManager;
    let mut opt: qboolean;

    unsafe {
        // allocate a very large temp buffer, we will shrink it later
        let max_length = (*header).codeLength * 8;
        common.buf = Z_Malloc(
            common,
            cm,
            rm,
            host,
            max_length,
            mp_qshared::common::mp::qcommon::tags::memtag_t::TAG_VM,
            qtrue,
            0,
        ) as *mut byte;
        common.jused = Z_Malloc(
            common,
            cm,
            rm,
            host,
            (*header).instructionCount + 2,
            mp_qshared::common::mp::qcommon::tags::memtag_t::TAG_VM,
            qtrue,
            0,
        ) as *mut byte;

        Com_Memset(
            common.jused as *mut (),
            0,
            ((*header).instructionCount + 2) as usize,
        );

        for pass_ in 0..2 {
            common.pass = pass_;
            common.oc0 = -23423;
            common.oc1 = -234354;
            common.pop0 = -43435;
            common.pop1 = -545455;

            // translate all instructions
            common.pc = 0;
            common.instruction = 0;
            common.code = (header as *mut byte).offset((*header).codeOffset as isize);
            common.compiled_ofs = 0;

            common.last_command = ELastCommand::LAST_COMMAND_NONE;

            while common.instruction < (*header).instructionCount {
                if common.compiled_ofs > max_length - 16 {
                    Com_Error(
                        common,
                        cm,
                        rm,
                        &mut rmg,
                        host,
                        errorParm_t::ERR_FATAL as c_int,
                        c"VM_CompileX86: maxLength exceeded".as_ptr(),
                    );
                }

                *(*vm)
                    .instructionPointers
                    .offset(common.instruction as isize) = common.compiled_ofs;
                common.instruction += 1;

                if common.pc > (*header).codeLength {
                    Com_Error(
                        common,
                        cm,
                        rm,
                        &mut rmg,
                        host,
                        errorParm_t::ERR_FATAL as c_int,
                        c"VM_CompileX86: pc > header->codeLength".as_ptr(),
                    );
                }

                let op = *common.code.offset(common.pc as isize);
                common.pc += 1;
                match opcode_from_i32(op as c_int) {
                    0 => {}
                    x if x == opcode_t::OP_BREAK as c_int => {
                        EmitString(common, cm, rm, &mut rmg, host, c"CC".as_ptr());
                        // int 3
                    }
                    x if x == opcode_t::OP_ENTER as c_int => {
                        EmitString(common, cm, rm, &mut rmg, host, c"81 EE".as_ptr()); // sub	esi, 0x12345678
                        let c4 = Constant4(common);
                        Emit4(common, c4);
                    }
                    x if x == opcode_t::OP_CONST as c_int => {
                        if *common.code.offset((common.pc + 4) as isize) as c_int
                            == opcode_t::OP_LOAD4 as c_int
                        {
                            EmitAddEDI4(common, cm, rm, &mut rmg, host, vm);
                            EmitString(common, cm, rm, &mut rmg, host, c"BB".as_ptr()); // mov	ebx, 0x12345678
                            let c4 = Constant4(common);
                            Emit4(common, (c4 & (*vm).dataMask) + (*vm).dataBase as c_int);
                            EmitString(common, cm, rm, &mut rmg, host, c"8B 03".as_ptr()); // mov	eax, dword ptr [ebx]
                            EmitCommand(
                                common,
                                cm,
                                rm,
                                &mut rmg,
                                host,
                                ELastCommand::LAST_COMMAND_MOV_EDI_EAX,
                            ); // mov dword ptr [edi], eax
                            common.pc += 1; // OP_LOAD4
                            common.instruction += 1;
                        } else if *common.code.offset((common.pc + 4) as isize) as c_int
                            == opcode_t::OP_LOAD2 as c_int
                        {
                            EmitAddEDI4(common, cm, rm, &mut rmg, host, vm);
                            EmitString(common, cm, rm, &mut rmg, host, c"BB".as_ptr()); // mov	ebx, 0x12345678
                            let c4 = Constant4(common);
                            Emit4(common, (c4 & (*vm).dataMask) + (*vm).dataBase as c_int);
                            EmitString(common, cm, rm, &mut rmg, host, c"0F B7 03".as_ptr()); // movzx	eax, word ptr [ebx]
                            EmitCommand(
                                common,
                                cm,
                                rm,
                                &mut rmg,
                                host,
                                ELastCommand::LAST_COMMAND_MOV_EDI_EAX,
                            ); // mov dword ptr [edi], eax
                            common.pc += 1; // OP_LOAD4
                            common.instruction += 1;
                        } else if *common.code.offset((common.pc + 4) as isize) as c_int
                            == opcode_t::OP_LOAD1 as c_int
                        {
                            EmitAddEDI4(common, cm, rm, &mut rmg, host, vm);
                            EmitString(common, cm, rm, &mut rmg, host, c"BB".as_ptr()); // mov	ebx, 0x12345678
                            let c4 = Constant4(common);
                            Emit4(common, (c4 & (*vm).dataMask) + (*vm).dataBase as c_int);
                            EmitString(common, cm, rm, &mut rmg, host, c"0F B6 03".as_ptr()); // movzx	eax, byte ptr [ebx]
                            EmitCommand(
                                common,
                                cm,
                                rm,
                                &mut rmg,
                                host,
                                ELastCommand::LAST_COMMAND_MOV_EDI_EAX,
                            ); // mov dword ptr [edi], eax
                            common.pc += 1; // OP_LOAD4
                            common.instruction += 1;
                        } else if *common.code.offset((common.pc + 4) as isize) as c_int
                            == opcode_t::OP_STORE4 as c_int
                        {
                            opt = EmitMovEBXEDI(
                                common,
                                cm,
                                rm,
                                &mut rmg,
                                host,
                                vm,
                                (*vm).dataMask & !3,
                            );
                            let _ = opt;
                            EmitString(common, cm, rm, &mut rmg, host, c"B8".as_ptr()); // mov	eax, 0x12345678
                            let c4 = Constant4(common);
                            Emit4(common, c4);
                            EmitString(common, cm, rm, &mut rmg, host, c"89 83".as_ptr()); // mov dword ptr [ebx+0x12345678], eax
                            Emit4(common, (*vm).dataBase as c_int);
                            EmitCommand(
                                common,
                                cm,
                                rm,
                                &mut rmg,
                                host,
                                ELastCommand::LAST_COMMAND_SUB_DI_4,
                            ); // sub edi, 4
                            common.pc += 1; // OP_STORE4
                            common.instruction += 1;
                        } else if *common.code.offset((common.pc + 4) as isize) as c_int
                            == opcode_t::OP_STORE2 as c_int
                        {
                            opt = EmitMovEBXEDI(
                                common,
                                cm,
                                rm,
                                &mut rmg,
                                host,
                                vm,
                                (*vm).dataMask & !1,
                            );
                            let _ = opt;
                            EmitString(common, cm, rm, &mut rmg, host, c"B8".as_ptr()); // mov	eax, 0x12345678
                            let c4 = Constant4(common);
                            Emit4(common, c4);
                            EmitString(common, cm, rm, &mut rmg, host, c"66 89 83".as_ptr()); // mov word ptr [ebx+0x12345678], eax
                            Emit4(common, (*vm).dataBase as c_int);
                            EmitCommand(
                                common,
                                cm,
                                rm,
                                &mut rmg,
                                host,
                                ELastCommand::LAST_COMMAND_SUB_DI_4,
                            ); // sub edi, 4
                            common.pc += 1; // OP_STORE4
                            common.instruction += 1;
                        } else if *common.code.offset((common.pc + 4) as isize) as c_int
                            == opcode_t::OP_STORE1 as c_int
                        {
                            opt = EmitMovEBXEDI(common, cm, rm, &mut rmg, host, vm, (*vm).dataMask);
                            let _ = opt;
                            EmitString(common, cm, rm, &mut rmg, host, c"B8".as_ptr()); // mov	eax, 0x12345678
                            let c4 = Constant4(common);
                            Emit4(common, c4);
                            EmitString(common, cm, rm, &mut rmg, host, c"88 83".as_ptr()); // mov byte ptr [ebx+0x12345678], eax
                            Emit4(common, (*vm).dataBase as c_int);
                            EmitCommand(
                                common,
                                cm,
                                rm,
                                &mut rmg,
                                host,
                                ELastCommand::LAST_COMMAND_SUB_DI_4,
                            ); // sub edi, 4
                            common.pc += 1; // OP_STORE4
                            common.instruction += 1;
                        } else if *common.code.offset((common.pc + 4) as isize) as c_int
                            == opcode_t::OP_ADD as c_int
                        {
                            EmitString(common, cm, rm, &mut rmg, host, c"81 07".as_ptr()); // add dword ptr [edi], 0x1234567
                            let c4 = Constant4(common);
                            Emit4(common, c4);
                            common.pc += 1; // OP_ADD
                            common.instruction += 1;
                        } else if *common.code.offset((common.pc + 4) as isize) as c_int
                            == opcode_t::OP_SUB as c_int
                        {
                            EmitString(common, cm, rm, &mut rmg, host, c"81 2F".as_ptr()); // sub dword ptr [edi], 0x1234567
                            let c4 = Constant4(common);
                            Emit4(common, c4);
                            common.pc += 1; // OP_ADD
                            common.instruction += 1;
                        } else {
                            EmitAddEDI4(common, cm, rm, &mut rmg, host, vm);
                            EmitString(common, cm, rm, &mut rmg, host, c"C7 07".as_ptr()); // mov	dword ptr [edi], 0x12345678
                            common.last_const = Constant4(common);
                            Emit4(common, common.last_const);
                            if *common.code.offset(common.pc as isize) as c_int
                                == opcode_t::OP_JUMP as c_int
                            {
                                *common.jused.offset(common.last_const as isize) = 1;
                            }
                        }
                    }
                    x if x == opcode_t::OP_LOCAL as c_int => {
                        EmitAddEDI4(common, cm, rm, &mut rmg, host, vm);
                        EmitString(common, cm, rm, &mut rmg, host, c"8D 86".as_ptr()); // lea eax, [0x12345678 + esi]
                        common.oc0 = common.oc1;
                        common.oc1 = Constant4(common);
                        Emit4(common, common.oc1);
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_MOV_EDI_EAX,
                        ); // mov dword ptr [edi], eax
                    }
                    x if x == opcode_t::OP_ARG as c_int => {
                        EmitMovEAXEDI(common, cm, rm, &mut rmg, host, vm); // mov	eax,dword ptr [edi]
                        EmitString(common, cm, rm, &mut rmg, host, c"89 86".as_ptr()); // mov	dword ptr [esi+database],eax
                                                                                       // FIXME: range check
                        let c1 = Constant1(common);
                        Emit4(common, c1 + (*vm).dataBase as c_int);
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_CALL as c_int => {
                        EmitString(common, cm, rm, &mut rmg, host, c"C7 86".as_ptr()); // mov dword ptr [esi+database],0x12345678
                        Emit4(common, (*vm).dataBase as c_int);
                        Emit4(common, common.pc);
                        EmitString(common, cm, rm, &mut rmg, host, c"FF 15".as_ptr()); // call asmCallPtr
                        Emit4(common, core::ptr::addr_of!(common.asm_call_ptr) as c_int);
                    }
                    x if x == opcode_t::OP_PUSH as c_int => {
                        EmitAddEDI4(common, cm, rm, &mut rmg, host, vm);
                    }
                    x if x == opcode_t::OP_POP as c_int => {
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_LEAVE as c_int => {
                        let v = Constant4(common);
                        EmitString(common, cm, rm, &mut rmg, host, c"81 C6".as_ptr()); // add	esi, 0x12345678
                        Emit4(common, v);
                        EmitString(common, cm, rm, &mut rmg, host, c"C3".as_ptr());
                        // ret
                    }
                    x if x == opcode_t::OP_LOAD4 as c_int => {
                        if *common.code.offset(common.pc as isize) as c_int
                            == opcode_t::OP_CONST as c_int
                            && *common.code.offset((common.pc + 5) as isize) as c_int
                                == opcode_t::OP_ADD as c_int
                            && *common.code.offset((common.pc + 6) as isize) as c_int
                                == opcode_t::OP_STORE4 as c_int
                        {
                            if common.oc0 == common.oc1
                                && common.pop0 == opcode_t::OP_LOCAL as c_int
                                && common.pop1 == opcode_t::OP_LOCAL as c_int
                            {
                                common.compiled_ofs -= 11;
                                *(*vm)
                                    .instructionPointers
                                    .offset((common.instruction - 1) as isize) =
                                    common.compiled_ofs;
                            }
                            common.pc += 1; // OP_CONST
                            let v = Constant4(common);
                            EmitMovEBXEDI(common, cm, rm, &mut rmg, host, vm, (*vm).dataMask);
                            if v == 1
                                && common.oc0 == common.oc1
                                && common.pop0 == opcode_t::OP_LOCAL as c_int
                                && common.pop1 == opcode_t::OP_LOCAL as c_int
                            {
                                EmitString(common, cm, rm, &mut rmg, host, c"FF 83".as_ptr()); // inc dword ptr [ebx + 0x12345678]
                                Emit4(common, (*vm).dataBase as c_int);
                            } else {
                                EmitString(common, cm, rm, &mut rmg, host, c"8B 83".as_ptr()); // mov	eax, dword ptr [ebx + 0x12345678]
                                Emit4(common, (*vm).dataBase as c_int);
                                EmitString(common, cm, rm, &mut rmg, host, c"05".as_ptr()); // add eax, const
                                Emit4(common, v);
                                if common.oc0 == common.oc1
                                    && common.pop0 == opcode_t::OP_LOCAL as c_int
                                    && common.pop1 == opcode_t::OP_LOCAL as c_int
                                {
                                    EmitString(common, cm, rm, &mut rmg, host, c"89 83".as_ptr()); // mov dword ptr [ebx+0x12345678], eax
                                    Emit4(common, (*vm).dataBase as c_int);
                                } else {
                                    EmitCommand(
                                        common,
                                        cm,
                                        rm,
                                        &mut rmg,
                                        host,
                                        ELastCommand::LAST_COMMAND_SUB_DI_4,
                                    ); // sub edi, 4
                                    EmitString(common, cm, rm, &mut rmg, host, c"8B 1F".as_ptr()); // mov	ebx, dword ptr [edi]
                                    EmitString(common, cm, rm, &mut rmg, host, c"89 83".as_ptr()); // mov dword ptr [ebx+0x12345678], eax
                                    Emit4(common, (*vm).dataBase as c_int);
                                }
                            }
                            EmitCommand(
                                common,
                                cm,
                                rm,
                                &mut rmg,
                                host,
                                ELastCommand::LAST_COMMAND_SUB_DI_4,
                            ); // sub edi, 4
                            common.pc += 1; // OP_ADD
                            common.pc += 1; // OP_STORE
                            common.instruction += 3;
                        } else if *common.code.offset(common.pc as isize) as c_int
                            == opcode_t::OP_CONST as c_int
                            && *common.code.offset((common.pc + 5) as isize) as c_int
                                == opcode_t::OP_SUB as c_int
                            && *common.code.offset((common.pc + 6) as isize) as c_int
                                == opcode_t::OP_STORE4 as c_int
                        {
                            if common.oc0 == common.oc1
                                && common.pop0 == opcode_t::OP_LOCAL as c_int
                                && common.pop1 == opcode_t::OP_LOCAL as c_int
                            {
                                common.compiled_ofs -= 11;
                                *(*vm)
                                    .instructionPointers
                                    .offset((common.instruction - 1) as isize) =
                                    common.compiled_ofs;
                            }
                            EmitMovEBXEDI(common, cm, rm, &mut rmg, host, vm, (*vm).dataMask);
                            EmitString(common, cm, rm, &mut rmg, host, c"8B 83".as_ptr()); // mov	eax, dword ptr [ebx + 0x12345678]
                            Emit4(common, (*vm).dataBase as c_int);
                            common.pc += 1; // OP_CONST
                            let v = Constant4(common);
                            if v == 1
                                && common.oc0 == common.oc1
                                && common.pop0 == opcode_t::OP_LOCAL as c_int
                                && common.pop1 == opcode_t::OP_LOCAL as c_int
                            {
                                EmitString(common, cm, rm, &mut rmg, host, c"FF 8B".as_ptr()); // dec dword ptr [ebx + 0x12345678]
                                Emit4(common, (*vm).dataBase as c_int);
                            } else {
                                EmitString(common, cm, rm, &mut rmg, host, c"2D".as_ptr()); // sub eax, const
                                Emit4(common, v);
                                if common.oc0 == common.oc1
                                    && common.pop0 == opcode_t::OP_LOCAL as c_int
                                    && common.pop1 == opcode_t::OP_LOCAL as c_int
                                {
                                    EmitString(common, cm, rm, &mut rmg, host, c"89 83".as_ptr()); // mov dword ptr [ebx+0x12345678], eax
                                    Emit4(common, (*vm).dataBase as c_int);
                                } else {
                                    EmitCommand(
                                        common,
                                        cm,
                                        rm,
                                        &mut rmg,
                                        host,
                                        ELastCommand::LAST_COMMAND_SUB_DI_4,
                                    ); // sub edi, 4
                                    EmitString(common, cm, rm, &mut rmg, host, c"8B 1F".as_ptr()); // mov	ebx, dword ptr [edi]
                                    EmitString(common, cm, rm, &mut rmg, host, c"89 83".as_ptr()); // mov dword ptr [ebx+0x12345678], eax
                                    Emit4(common, (*vm).dataBase as c_int);
                                }
                            }
                            EmitCommand(
                                common,
                                cm,
                                rm,
                                &mut rmg,
                                host,
                                ELastCommand::LAST_COMMAND_SUB_DI_4,
                            ); // sub edi, 4
                            common.pc += 1; // OP_SUB
                            common.pc += 1; // OP_STORE
                            common.instruction += 3;
                        } else if *common.buf.offset((common.compiled_ofs - 2) as isize) == 0x89
                            && *common.buf.offset((common.compiled_ofs - 1) as isize) == 0x07
                        {
                            common.compiled_ofs -= 2;
                            *(*vm)
                                .instructionPointers
                                .offset((common.instruction - 1) as isize) = common.compiled_ofs;
                            EmitString(common, cm, rm, &mut rmg, host, c"8B 80".as_ptr()); // mov eax, dword ptr [eax + 0x1234567]
                            Emit4(common, (*vm).dataBase as c_int);
                            EmitCommand(
                                common,
                                cm,
                                rm,
                                &mut rmg,
                                host,
                                ELastCommand::LAST_COMMAND_MOV_EDI_EAX,
                            ); // mov dword ptr [edi], eax
                        } else {
                            EmitMovEBXEDI(common, cm, rm, &mut rmg, host, vm, (*vm).dataMask);
                            EmitString(common, cm, rm, &mut rmg, host, c"8B 83".as_ptr()); // mov	eax, dword ptr [ebx + 0x12345678]
                            Emit4(common, (*vm).dataBase as c_int);
                            EmitCommand(
                                common,
                                cm,
                                rm,
                                &mut rmg,
                                host,
                                ELastCommand::LAST_COMMAND_MOV_EDI_EAX,
                            ); // mov dword ptr [edi], eax
                        }
                    }
                    x if x == opcode_t::OP_LOAD2 as c_int => {
                        EmitMovEBXEDI(common, cm, rm, &mut rmg, host, vm, (*vm).dataMask);
                        EmitString(common, cm, rm, &mut rmg, host, c"0F B7 83".as_ptr()); // movzx	eax, word ptr [ebx + 0x12345678]
                        Emit4(common, (*vm).dataBase as c_int);
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_MOV_EDI_EAX,
                        ); // mov dword ptr [edi], eax
                    }
                    x if x == opcode_t::OP_LOAD1 as c_int => {
                        EmitMovEBXEDI(common, cm, rm, &mut rmg, host, vm, (*vm).dataMask);
                        EmitString(common, cm, rm, &mut rmg, host, c"0F B6 83".as_ptr()); // movzx eax, byte ptr [ebx + 0x12345678]
                        Emit4(common, (*vm).dataBase as c_int);
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_MOV_EDI_EAX,
                        ); // mov dword ptr [edi], eax
                    }
                    x if x == opcode_t::OP_STORE4 as c_int => {
                        EmitMovEAXEDI(common, cm, rm, &mut rmg, host, vm);
                        EmitString(common, cm, rm, &mut rmg, host, c"8B 5F FC".as_ptr()); // mov	ebx, dword ptr [edi-4]
                        EmitString(common, cm, rm, &mut rmg, host, c"89 83".as_ptr()); // mov dword ptr [ebx+0x12345678], eax
                        Emit4(common, (*vm).dataBase as c_int);
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                    }
                    x if x == opcode_t::OP_STORE2 as c_int => {
                        EmitMovEAXEDI(common, cm, rm, &mut rmg, host, vm);
                        EmitString(common, cm, rm, &mut rmg, host, c"8B 5F FC".as_ptr()); // mov	ebx, dword ptr [edi-4]
                        EmitString(common, cm, rm, &mut rmg, host, c"66 89 83".as_ptr()); // mov word ptr [ebx+0x12345678], eax
                        Emit4(common, (*vm).dataBase as c_int);
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                    }
                    x if x == opcode_t::OP_STORE1 as c_int => {
                        EmitMovEAXEDI(common, cm, rm, &mut rmg, host, vm);
                        EmitString(common, cm, rm, &mut rmg, host, c"8B 5F FC".as_ptr()); // mov	ebx, dword ptr [edi-4]
                        EmitString(common, cm, rm, &mut rmg, host, c"88 83".as_ptr()); // mov byte ptr [ebx+0x12345678], eax
                        Emit4(common, (*vm).dataBase as c_int);
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                    }
                    x if x == opcode_t::OP_EQ as c_int => {
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(common, cm, rm, &mut rmg, host, c"8B 47 04".as_ptr()); // mov	eax, dword ptr [edi+4]
                        EmitString(common, cm, rm, &mut rmg, host, c"3B 47 08".as_ptr()); // cmp	eax, dword ptr [edi+8]
                        EmitString(common, cm, rm, &mut rmg, host, c"75 06".as_ptr()); // jne +6
                        EmitString(common, cm, rm, &mut rmg, host, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(common);
                        *common.jused.offset(v as isize) = 1;
                        Emit4(
                            common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_NE as c_int => {
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(common, cm, rm, &mut rmg, host, c"8B 47 04".as_ptr()); // mov	eax, dword ptr [edi+4]
                        EmitString(common, cm, rm, &mut rmg, host, c"3B 47 08".as_ptr()); // cmp	eax, dword ptr [edi+8]
                        EmitString(common, cm, rm, &mut rmg, host, c"74 06".as_ptr()); // je +6
                        EmitString(common, cm, rm, &mut rmg, host, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(common);
                        *common.jused.offset(v as isize) = 1;
                        Emit4(
                            common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_LTI as c_int => {
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(common, cm, rm, &mut rmg, host, c"8B 47 04".as_ptr()); // mov	eax, dword ptr [edi+4]
                        EmitString(common, cm, rm, &mut rmg, host, c"3B 47 08".as_ptr()); // cmp	eax, dword ptr [edi+8]
                        EmitString(common, cm, rm, &mut rmg, host, c"7D 06".as_ptr()); // jnl +6
                        EmitString(common, cm, rm, &mut rmg, host, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(common);
                        *common.jused.offset(v as isize) = 1;
                        Emit4(
                            common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_LEI as c_int => {
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(common, cm, rm, &mut rmg, host, c"8B 47 04".as_ptr()); // mov	eax, dword ptr [edi+4]
                        EmitString(common, cm, rm, &mut rmg, host, c"3B 47 08".as_ptr()); // cmp	eax, dword ptr [edi+8]
                        EmitString(common, cm, rm, &mut rmg, host, c"7F 06".as_ptr()); // jnle +6
                        EmitString(common, cm, rm, &mut rmg, host, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(common);
                        *common.jused.offset(v as isize) = 1;
                        Emit4(
                            common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_GTI as c_int => {
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(common, cm, rm, &mut rmg, host, c"8B 47 04".as_ptr()); // mov	eax, dword ptr [edi+4]
                        EmitString(common, cm, rm, &mut rmg, host, c"3B 47 08".as_ptr()); // cmp	eax, dword ptr [edi+8]
                        EmitString(common, cm, rm, &mut rmg, host, c"7E 06".as_ptr()); // jng +6
                        EmitString(common, cm, rm, &mut rmg, host, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(common);
                        *common.jused.offset(v as isize) = 1;
                        Emit4(
                            common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_GEI as c_int => {
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(common, cm, rm, &mut rmg, host, c"8B 47 04".as_ptr()); // mov	eax, dword ptr [edi+4]
                        EmitString(common, cm, rm, &mut rmg, host, c"3B 47 08".as_ptr()); // cmp	eax, dword ptr [edi+8]
                        EmitString(common, cm, rm, &mut rmg, host, c"7C 06".as_ptr()); // jnge +6
                        EmitString(common, cm, rm, &mut rmg, host, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(common);
                        *common.jused.offset(v as isize) = 1;
                        Emit4(
                            common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_LTU as c_int => {
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(common, cm, rm, &mut rmg, host, c"8B 47 04".as_ptr()); // mov	eax, dword ptr [edi+4]
                        EmitString(common, cm, rm, &mut rmg, host, c"3B 47 08".as_ptr()); // cmp	eax, dword ptr [edi+8]
                        EmitString(common, cm, rm, &mut rmg, host, c"73 06".as_ptr()); // jnb +6
                        EmitString(common, cm, rm, &mut rmg, host, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(common);
                        *common.jused.offset(v as isize) = 1;
                        Emit4(
                            common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_LEU as c_int => {
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(common, cm, rm, &mut rmg, host, c"8B 47 04".as_ptr()); // mov	eax, dword ptr [edi+4]
                        EmitString(common, cm, rm, &mut rmg, host, c"3B 47 08".as_ptr()); // cmp	eax, dword ptr [edi+8]
                        EmitString(common, cm, rm, &mut rmg, host, c"77 06".as_ptr()); // jnbe +6
                        EmitString(common, cm, rm, &mut rmg, host, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(common);
                        *common.jused.offset(v as isize) = 1;
                        Emit4(
                            common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_GTU as c_int => {
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(common, cm, rm, &mut rmg, host, c"8B 47 04".as_ptr()); // mov	eax, dword ptr [edi+4]
                        EmitString(common, cm, rm, &mut rmg, host, c"3B 47 08".as_ptr()); // cmp	eax, dword ptr [edi+8]
                        EmitString(common, cm, rm, &mut rmg, host, c"76 06".as_ptr()); // jna +6
                        EmitString(common, cm, rm, &mut rmg, host, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(common);
                        *common.jused.offset(v as isize) = 1;
                        Emit4(
                            common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_GEU as c_int => {
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(common, cm, rm, &mut rmg, host, c"8B 47 04".as_ptr()); // mov	eax, dword ptr [edi+4]
                        EmitString(common, cm, rm, &mut rmg, host, c"3B 47 08".as_ptr()); // cmp	eax, dword ptr [edi+8]
                        EmitString(common, cm, rm, &mut rmg, host, c"72 06".as_ptr()); // jnae +6
                        EmitString(common, cm, rm, &mut rmg, host, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(common);
                        *common.jused.offset(v as isize) = 1;
                        Emit4(
                            common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_EQF as c_int => {
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(common, cm, rm, &mut rmg, host, c"D9 47 04".as_ptr()); // fld dword ptr [edi+4]
                        EmitString(common, cm, rm, &mut rmg, host, c"D8 5F 08".as_ptr()); // fcomp dword ptr [edi+8]
                        EmitString(common, cm, rm, &mut rmg, host, c"DF E0".as_ptr()); // fnstsw ax
                        EmitString(common, cm, rm, &mut rmg, host, c"F6 C4 40".as_ptr()); // test	ah,0x40
                        EmitString(common, cm, rm, &mut rmg, host, c"74 06".as_ptr()); // je +6
                        EmitString(common, cm, rm, &mut rmg, host, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(common);
                        *common.jused.offset(v as isize) = 1;
                        Emit4(
                            common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_NEF as c_int => {
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(common, cm, rm, &mut rmg, host, c"D9 47 04".as_ptr()); // fld dword ptr [edi+4]
                        EmitString(common, cm, rm, &mut rmg, host, c"D8 5F 08".as_ptr()); // fcomp dword ptr [edi+8]
                        EmitString(common, cm, rm, &mut rmg, host, c"DF E0".as_ptr()); // fnstsw ax
                        EmitString(common, cm, rm, &mut rmg, host, c"F6 C4 40".as_ptr()); // test	ah,0x40
                        EmitString(common, cm, rm, &mut rmg, host, c"75 06".as_ptr()); // jne +6
                        EmitString(common, cm, rm, &mut rmg, host, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(common);
                        *common.jused.offset(v as isize) = 1;
                        Emit4(
                            common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_LTF as c_int => {
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(common, cm, rm, &mut rmg, host, c"D9 47 04".as_ptr()); // fld dword ptr [edi+4]
                        EmitString(common, cm, rm, &mut rmg, host, c"D8 5F 08".as_ptr()); // fcomp dword ptr [edi+8]
                        EmitString(common, cm, rm, &mut rmg, host, c"DF E0".as_ptr()); // fnstsw ax
                        EmitString(common, cm, rm, &mut rmg, host, c"F6 C4 01".as_ptr()); // test	ah,0x01
                        EmitString(common, cm, rm, &mut rmg, host, c"74 06".as_ptr()); // je +6
                        EmitString(common, cm, rm, &mut rmg, host, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(common);
                        *common.jused.offset(v as isize) = 1;
                        Emit4(
                            common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_LEF as c_int => {
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(common, cm, rm, &mut rmg, host, c"D9 47 04".as_ptr()); // fld dword ptr [edi+4]
                        EmitString(common, cm, rm, &mut rmg, host, c"D8 5F 08".as_ptr()); // fcomp dword ptr [edi+8]
                        EmitString(common, cm, rm, &mut rmg, host, c"DF E0".as_ptr()); // fnstsw ax
                        EmitString(common, cm, rm, &mut rmg, host, c"F6 C4 41".as_ptr()); // test	ah,0x41
                        EmitString(common, cm, rm, &mut rmg, host, c"74 06".as_ptr()); // je +6
                        EmitString(common, cm, rm, &mut rmg, host, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(common);
                        *common.jused.offset(v as isize) = 1;
                        Emit4(
                            common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_GTF as c_int => {
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(common, cm, rm, &mut rmg, host, c"D9 47 04".as_ptr()); // fld dword ptr [edi+4]
                        EmitString(common, cm, rm, &mut rmg, host, c"D8 5F 08".as_ptr()); // fcomp dword ptr [edi+8]
                        EmitString(common, cm, rm, &mut rmg, host, c"DF E0".as_ptr()); // fnstsw ax
                        EmitString(common, cm, rm, &mut rmg, host, c"F6 C4 41".as_ptr()); // test	ah,0x41
                        EmitString(common, cm, rm, &mut rmg, host, c"75 06".as_ptr()); // jne +6
                        EmitString(common, cm, rm, &mut rmg, host, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(common);
                        *common.jused.offset(v as isize) = 1;
                        Emit4(
                            common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_GEF as c_int => {
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                        EmitString(common, cm, rm, &mut rmg, host, c"D9 47 04".as_ptr()); // fld dword ptr [edi+4]
                        EmitString(common, cm, rm, &mut rmg, host, c"D8 5F 08".as_ptr()); // fcomp dword ptr [edi+8]
                        EmitString(common, cm, rm, &mut rmg, host, c"DF E0".as_ptr()); // fnstsw ax
                        EmitString(common, cm, rm, &mut rmg, host, c"F6 C4 01".as_ptr()); // test	ah,0x01
                        EmitString(common, cm, rm, &mut rmg, host, c"75 06".as_ptr()); // jne +6
                        EmitString(common, cm, rm, &mut rmg, host, c"FF 25".as_ptr()); // jmp	[0x12345678]
                        let v = Constant4(common);
                        *common.jused.offset(v as isize) = 1;
                        Emit4(
                            common,
                            (*vm).instructionPointers.offset((v * 4) as isize) as c_int,
                        );
                    }
                    x if x == opcode_t::OP_NEGI as c_int => {
                        EmitString(common, cm, rm, &mut rmg, host, c"F7 1F".as_ptr());
                        // neg dword ptr [edi]
                    }
                    x if x == opcode_t::OP_ADD as c_int => {
                        EmitMovEAXEDI(common, cm, rm, &mut rmg, host, vm); // mov eax, dword ptr [edi]
                        EmitString(common, cm, rm, &mut rmg, host, c"01 47 FC".as_ptr()); // add dword ptr [edi-4],eax
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_SUB as c_int => {
                        EmitMovEAXEDI(common, cm, rm, &mut rmg, host, vm); // mov eax, dword ptr [edi]
                        EmitString(common, cm, rm, &mut rmg, host, c"29 47 FC".as_ptr()); // sub dword ptr [edi-4],eax
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_DIVI as c_int => {
                        EmitString(common, cm, rm, &mut rmg, host, c"8B 47 FC".as_ptr()); // mov eax,dword ptr [edi-4]
                        EmitString(common, cm, rm, &mut rmg, host, c"99".as_ptr()); // cdq
                        EmitString(common, cm, rm, &mut rmg, host, c"F7 3F".as_ptr()); // idiv dword ptr [edi]
                        EmitString(common, cm, rm, &mut rmg, host, c"89 47 FC".as_ptr()); // mov dword ptr [edi-4],eax
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_DIVU as c_int => {
                        EmitString(common, cm, rm, &mut rmg, host, c"8B 47 FC".as_ptr()); // mov eax,dword ptr [edi-4]
                        EmitString(common, cm, rm, &mut rmg, host, c"33 D2".as_ptr()); // xor edx, edx
                        EmitString(common, cm, rm, &mut rmg, host, c"F7 37".as_ptr()); // div dword ptr [edi]
                        EmitString(common, cm, rm, &mut rmg, host, c"89 47 FC".as_ptr()); // mov dword ptr [edi-4],eax
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_MODI as c_int => {
                        EmitString(common, cm, rm, &mut rmg, host, c"8B 47 FC".as_ptr()); // mov eax,dword ptr [edi-4]
                        EmitString(common, cm, rm, &mut rmg, host, c"99".as_ptr()); // cdq
                        EmitString(common, cm, rm, &mut rmg, host, c"F7 3F".as_ptr()); // idiv dword ptr [edi]
                        EmitString(common, cm, rm, &mut rmg, host, c"89 57 FC".as_ptr()); // mov dword ptr [edi-4],edx
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_MODU as c_int => {
                        EmitString(common, cm, rm, &mut rmg, host, c"8B 47 FC".as_ptr()); // mov eax,dword ptr [edi-4]
                        EmitString(common, cm, rm, &mut rmg, host, c"33 D2".as_ptr()); // xor edx, edx
                        EmitString(common, cm, rm, &mut rmg, host, c"F7 37".as_ptr()); // div dword ptr [edi]
                        EmitString(common, cm, rm, &mut rmg, host, c"89 57 FC".as_ptr()); // mov dword ptr [edi-4],edx
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_MULI as c_int => {
                        EmitString(common, cm, rm, &mut rmg, host, c"8B 47 FC".as_ptr()); // mov eax,dword ptr [edi-4]
                        EmitString(common, cm, rm, &mut rmg, host, c"F7 2F".as_ptr()); // imul dword ptr [edi]
                        EmitString(common, cm, rm, &mut rmg, host, c"89 47 FC".as_ptr()); // mov dword ptr [edi-4],eax
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_MULU as c_int => {
                        EmitString(common, cm, rm, &mut rmg, host, c"8B 47 FC".as_ptr()); // mov eax,dword ptr [edi-4]
                        EmitString(common, cm, rm, &mut rmg, host, c"F7 27".as_ptr()); // mul dword ptr [edi]
                        EmitString(common, cm, rm, &mut rmg, host, c"89 47 FC".as_ptr()); // mov dword ptr [edi-4],eax
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_BAND as c_int => {
                        EmitMovEAXEDI(common, cm, rm, &mut rmg, host, vm); // mov eax, dword ptr [edi]
                        EmitString(common, cm, rm, &mut rmg, host, c"21 47 FC".as_ptr()); // and dword ptr [edi-4],eax
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_BOR as c_int => {
                        EmitMovEAXEDI(common, cm, rm, &mut rmg, host, vm); // mov eax, dword ptr [edi]
                        EmitString(common, cm, rm, &mut rmg, host, c"09 47 FC".as_ptr()); // or dword ptr [edi-4],eax
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_BXOR as c_int => {
                        EmitMovEAXEDI(common, cm, rm, &mut rmg, host, vm); // mov eax, dword ptr [edi]
                        EmitString(common, cm, rm, &mut rmg, host, c"31 47 FC".as_ptr()); // xor dword ptr [edi-4],eax
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_BCOM as c_int => {
                        EmitString(common, cm, rm, &mut rmg, host, c"F7 17".as_ptr());
                        // not dword ptr [edi]
                    }
                    x if x == opcode_t::OP_LSH as c_int => {
                        EmitString(common, cm, rm, &mut rmg, host, c"8B 0F".as_ptr()); // mov ecx, dword ptr [edi]
                        EmitString(common, cm, rm, &mut rmg, host, c"D3 67 FC".as_ptr()); // shl dword ptr [edi-4], cl
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_RSHI as c_int => {
                        EmitString(common, cm, rm, &mut rmg, host, c"8B 0F".as_ptr()); // mov ecx, dword ptr [edi]
                        EmitString(common, cm, rm, &mut rmg, host, c"D3 7F FC".as_ptr()); // sar dword ptr [edi-4], cl
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_RSHU as c_int => {
                        EmitString(common, cm, rm, &mut rmg, host, c"8B 0F".as_ptr()); // mov ecx, dword ptr [edi]
                        EmitString(common, cm, rm, &mut rmg, host, c"D3 6F FC".as_ptr()); // shr dword ptr [edi-4], cl
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_NEGF as c_int => {
                        EmitString(common, cm, rm, &mut rmg, host, c"D9 07".as_ptr()); // fld dword ptr [edi]
                        EmitString(common, cm, rm, &mut rmg, host, c"D9 E0".as_ptr()); // fchs
                        EmitString(common, cm, rm, &mut rmg, host, c"D9 1F".as_ptr());
                        // fstp dword ptr [edi]
                    }
                    x if x == opcode_t::OP_ADDF as c_int => {
                        EmitString(common, cm, rm, &mut rmg, host, c"D9 47 FC".as_ptr()); // fld dword ptr [edi-4]
                        EmitString(common, cm, rm, &mut rmg, host, c"D8 07".as_ptr()); // fadd dword ptr [edi]
                        EmitString(common, cm, rm, &mut rmg, host, c"D9 5F FC".as_ptr()); // fstp dword ptr [edi-4]
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                    }
                    x if x == opcode_t::OP_SUBF as c_int => {
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                        EmitString(common, cm, rm, &mut rmg, host, c"D9 07".as_ptr()); // fld dword ptr [edi]
                        EmitString(common, cm, rm, &mut rmg, host, c"D8 67 04".as_ptr()); // fsub dword ptr [edi+4]
                        EmitString(common, cm, rm, &mut rmg, host, c"D9 1F".as_ptr());
                        // fstp dword ptr [edi]
                    }
                    x if x == opcode_t::OP_DIVF as c_int => {
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                        EmitString(common, cm, rm, &mut rmg, host, c"D9 07".as_ptr()); // fld dword ptr [edi]
                        EmitString(common, cm, rm, &mut rmg, host, c"D8 77 04".as_ptr()); // fdiv dword ptr [edi+4]
                        EmitString(common, cm, rm, &mut rmg, host, c"D9 1F".as_ptr());
                        // fstp dword ptr [edi]
                    }
                    x if x == opcode_t::OP_MULF as c_int => {
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                        EmitString(common, cm, rm, &mut rmg, host, c"D9 07".as_ptr()); // fld dword ptr [edi]
                        EmitString(common, cm, rm, &mut rmg, host, c"D8 4f 04".as_ptr()); // fmul dword ptr [edi+4]
                        EmitString(common, cm, rm, &mut rmg, host, c"D9 1F".as_ptr());
                        // fstp dword ptr [edi]
                    }
                    x if x == opcode_t::OP_CVIF as c_int => {
                        EmitString(common, cm, rm, &mut rmg, host, c"DB 07".as_ptr()); // fild dword ptr [edi]
                        EmitString(common, cm, rm, &mut rmg, host, c"D9 1F".as_ptr());
                        // fstp dword ptr [edi]
                    }
                    x if x == opcode_t::OP_CVFI as c_int => {
                        // Raven: `#ifndef FTOL_PTR` selects the non-IEEE-compliant
                        // direct fistp path; FTOL_PTR is unset in the WinDed build
                        // this port targets (no rosetta row — reported missing).
                        EmitString(common, cm, rm, &mut rmg, host, c"D9 07".as_ptr()); // fld dword ptr [edi]
                        EmitString(common, cm, rm, &mut rmg, host, c"DB 1F".as_ptr());
                        // fistp dword ptr [edi]
                    }
                    x if x == opcode_t::OP_SEX8 as c_int => {
                        EmitString(common, cm, rm, &mut rmg, host, c"0F BE 07".as_ptr()); // movsx eax, byte ptr [edi]
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_MOV_EDI_EAX,
                        ); // mov dword ptr [edi], eax
                    }
                    x if x == opcode_t::OP_SEX16 as c_int => {
                        EmitString(common, cm, rm, &mut rmg, host, c"0F BF 07".as_ptr()); // movsx eax, word ptr [edi]
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_MOV_EDI_EAX,
                        ); // mov dword ptr [edi], eax
                    }
                    x if x == opcode_t::OP_BLOCK_COPY as c_int => {
                        // FIXME: range check
                        EmitString(common, cm, rm, &mut rmg, host, c"56".as_ptr()); // push esi
                        EmitString(common, cm, rm, &mut rmg, host, c"57".as_ptr()); // push edi
                        EmitString(common, cm, rm, &mut rmg, host, c"8B 37".as_ptr()); // mov esi,[edi]
                        EmitString(common, cm, rm, &mut rmg, host, c"8B 7F FC".as_ptr()); // mov edi,[edi-4]
                        EmitString(common, cm, rm, &mut rmg, host, c"B9".as_ptr()); // mov ecx,0x12345678
                        let c4 = Constant4(common);
                        Emit4(common, c4 >> 2);
                        EmitString(common, cm, rm, &mut rmg, host, c"B8".as_ptr()); // mov eax, datamask
                        Emit4(common, (*vm).dataMask);
                        EmitString(common, cm, rm, &mut rmg, host, c"BB".as_ptr()); // mov ebx, database
                        Emit4(common, (*vm).dataBase as c_int);
                        EmitString(common, cm, rm, &mut rmg, host, c"23 F0".as_ptr()); // and esi, eax
                        EmitString(common, cm, rm, &mut rmg, host, c"03 F3".as_ptr()); // add esi, ebx
                        EmitString(common, cm, rm, &mut rmg, host, c"23 F8".as_ptr()); // and edi, eax
                        EmitString(common, cm, rm, &mut rmg, host, c"03 FB".as_ptr()); // add edi, ebx
                        EmitString(common, cm, rm, &mut rmg, host, c"F3 A5".as_ptr()); // rep movsd
                        EmitString(common, cm, rm, &mut rmg, host, c"5F".as_ptr()); // pop edi
                        EmitString(common, cm, rm, &mut rmg, host, c"5E".as_ptr()); // pop esi
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_8,
                        ); // sub edi, 8
                    }
                    x if x == opcode_t::OP_JUMP as c_int => {
                        EmitCommand(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            ELastCommand::LAST_COMMAND_SUB_DI_4,
                        ); // sub edi, 4
                        EmitString(common, cm, rm, &mut rmg, host, c"8B 47 04".as_ptr()); // mov eax,dword ptr [edi+4]
                                                                                          // FIXME: range check
                        EmitString(common, cm, rm, &mut rmg, host, c"FF 24 85".as_ptr()); // jmp dword ptr [instructionPointers + eax * 4]
                        Emit4(common, (*vm).instructionPointers as c_int);
                    }
                    _ => {
                        Com_Error(
                            common,
                            cm,
                            rm,
                            &mut rmg,
                            host,
                            errorParm_t::ERR_DROP as c_int,
                            c"VM_CompileX86: bad opcode %i at offset %i".as_ptr(),
                            op,
                            common.pc,
                        );
                    }
                }
                common.pop0 = common.pop1;
                common.pop1 = op as c_int;
            }
        }

        // copy to an exact size buffer on the hunk
        (*vm).codeLength = common.compiled_ofs;
        (*vm).codeBase = Hunk_Alloc(common, cm, rm, host, common.compiled_ofs, h_low) as *mut byte;
        Com_Memcpy(
            (*vm).codeBase as *mut (),
            common.buf as *const (),
            common.compiled_ofs as usize,
        );
        Z_Free(common, common.buf as *mut ());
        Z_Free(common, common.jused as *mut ());
        Com_Printf(
            common,
            c"VM file %s compiled to %i bytes of code\n".as_ptr(),
            (*vm).name.as_ptr(),
            common.compiled_ofs,
        );

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
