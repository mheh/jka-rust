#![allow(non_camel_case_types, non_snake_case)]

/// Raven `opcode_t` — VM instruction opcodes.
///
/// Type definition source: `oracle/codemp/qcommon/vm_local.h:10-95`
#[repr(i32)]
pub enum opcode_t {
    OP_UNDEF = 0,

    OP_IGNORE = 1,

    OP_BREAK = 2,

    OP_ENTER = 3,
    OP_LEAVE = 4,
    OP_CALL = 5,
    OP_PUSH = 6,
    OP_POP = 7,

    OP_CONST = 8,
    OP_LOCAL = 9,

    OP_JUMP = 10,

    OP_EQ = 11,
    OP_NE = 12,

    OP_LTI = 13,
    OP_LEI = 14,
    OP_GTI = 15,
    OP_GEI = 16,

    OP_LTU = 17,
    OP_LEU = 18,
    OP_GTU = 19,
    OP_GEU = 20,

    OP_EQF = 21,
    OP_NEF = 22,

    OP_LTF = 23,
    OP_LEF = 24,
    OP_GTF = 25,
    OP_GEF = 26,

    OP_LOAD1 = 27,
    OP_LOAD2 = 28,
    OP_LOAD4 = 29,
    OP_STORE1 = 30,
    OP_STORE2 = 31,
    OP_STORE4 = 32,
    OP_ARG = 33,

    OP_BLOCK_COPY = 34,

    OP_SEX8 = 35,
    OP_SEX16 = 36,

    OP_NEGI = 37,
    OP_ADD = 38,
    OP_SUB = 39,
    OP_DIVI = 40,
    OP_DIVU = 41,
    OP_MODI = 42,
    OP_MODU = 43,
    OP_MULI = 44,
    OP_MULU = 45,

    OP_BAND = 46,
    OP_BOR = 47,
    OP_BXOR = 48,
    OP_BCOM = 49,

    OP_LSH = 50,
    OP_RSHI = 51,
    OP_RSHU = 52,

    OP_NEGF = 53,
    OP_ADDF = 54,
    OP_SUBF = 55,
    OP_DIVF = 56,
    OP_MULF = 57,

    OP_CVIF = 58,
    OP_CVFI = 59,
}
