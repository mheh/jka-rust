use core::ffi::c_int;

use crate::common::mp::trace_t::trace_t;
use crate::ffi::GameImport;
use crate::shared::vec3_t;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `G_TRACE`.
///
/// `results` is the engine out-param: the engine writes the resulting [`trace_t`]
/// through this pointer, so it stays a pointer here rather than becoming an
/// `Output`. `start`/`mins`/`maxs`/`end` are the swept box geometry. The C
/// `trap_Trace` wrapper hard-codes the two trailing ghoul2 args (`g2TraceType` 0,
/// `traceLod` 10); they are reproduced as literals in `encode_syscall` rather than
/// carried as caller inputs (see `G_G2TRACE` to set them explicitly).
#[derive(Debug)]
pub struct GTraceArgs {
    results: *mut trace_t,
    start: *const vec3_t,
    mins: *const vec3_t,
    maxs: *const vec3_t,
    end: *const vec3_t,
    pass_entity_num: c_int,
    contentmask: c_int,
}

impl GTraceArgs {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        results: *mut trace_t,
        start: *const vec3_t,
        mins: *const vec3_t,
        maxs: *const vec3_t,
        end: *const vec3_t,
        pass_entity_num: c_int,
        contentmask: c_int,
    ) -> Self {
        Self {
            results,
            start,
            mins,
            maxs,
            end,
            pass_entity_num,
            contentmask,
        }
    }

    pub const fn results(&self) -> *mut trace_t {
        self.results
    }

    pub const fn start(&self) -> *const vec3_t {
        self.start
    }

    pub const fn mins(&self) -> *const vec3_t {
        self.mins
    }

    pub const fn maxs(&self) -> *const vec3_t {
        self.maxs
    }

    pub const fn end(&self) -> *const vec3_t {
        self.end
    }

    pub const fn pass_entity_num(&self) -> c_int {
        self.pass_entity_num
    }

    pub const fn contentmask(&self) -> c_int {
        self.contentmask
    }
}

/// `G_TRACE` MP game imports syscall ABI token.
///
/// Raven: ( trace_t *results, const vec3_t start, const vec3_t mins, const vec3_t maxs, const vec3_t end, int passEntityNum, int contentmask );
/// Raven: collision detection against all linked entities
/// Source: `oracle/oracle/codemp/game/g_public.h:182`
pub struct GTrace;

impl OutboundSysCall for GTrace {
    type Import = GameImport;
    type Args = GTraceArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_TRACE;
}

impl EncodeSysCall for GTrace {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.results()),
            ptr_to_word(args.start()),
            ptr_to_word(args.mins()),
            ptr_to_word(args.maxs()),
            ptr_to_word(args.end()),
            args.pass_entity_num() as isize,
            args.contentmask() as isize,
            // ghoul2 g2TraceType: trap_Trace hard-codes 0 (no g2 collision).
            0,
            // ghoul2 traceLod: trap_Trace hard-codes 10.
            10,
        ])
    }
}

impl DecodeSysCallReturn for GTrace {
    // `trap_Trace` is `void`; the engine writes its result through `results`.
    fn decode_return(_word: isize) -> Self::Output {}
}
