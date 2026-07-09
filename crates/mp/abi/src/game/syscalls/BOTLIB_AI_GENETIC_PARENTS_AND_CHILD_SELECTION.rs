use core::ffi::c_int;

use super::super::MpGameImport;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_GENETIC_PARENTS_AND_CHILD_SELECTION` outbound game-to-engine syscall.
///
/// C signature: `int trap_GeneticParentsAndChildSelection(int numranks, float *ranks, int *parent1, int *parent2, int *child)`
#[derive(Debug)]
pub struct BotlibAiGeneticParentsAndChildSelectionArgs {
    numranks: c_int,
    ranks: *mut f32,
    parent1: *mut c_int,
    parent2: *mut c_int,
    child: *mut c_int,
}

impl BotlibAiGeneticParentsAndChildSelectionArgs {
    pub fn new(
        numranks: c_int,
        ranks: *mut f32,
        parent1: *mut c_int,
        parent2: *mut c_int,
        child: *mut c_int,
    ) -> Self {
        Self {
            numranks,
            ranks,
            parent1,
            parent2,
            child,
        }
    }

    pub fn numranks(&self) -> c_int {
        self.numranks
    }
    pub fn ranks(&self) -> *mut f32 {
        self.ranks
    }
    pub fn parent1(&self) -> *mut c_int {
        self.parent1
    }
    pub fn parent2(&self) -> *mut c_int {
        self.parent2
    }
    pub fn child(&self) -> *mut c_int {
        self.child
    }
}

/// `BOTLIB_AI_GENETIC_PARENTS_AND_CHILD_SELECTION` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:482`
pub struct BotlibAiGeneticParentsAndChildSelection;

impl OutboundSysCall for BotlibAiGeneticParentsAndChildSelection {
    type Import = MpGameImport;
    type Args = BotlibAiGeneticParentsAndChildSelectionArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_GENETIC_PARENTS_AND_CHILD_SELECTION;
}

impl EncodeSysCall for BotlibAiGeneticParentsAndChildSelection {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.numranks as isize,
            ptr_to_word(a.ranks),
            ptr_to_word(a.parent1),
            ptr_to_word(a.parent2),
            ptr_to_word(a.child),
        ])
    }
}

impl DecodeSysCallReturn for BotlibAiGeneticParentsAndChildSelection {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
