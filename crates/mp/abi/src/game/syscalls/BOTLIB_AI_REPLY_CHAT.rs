use core::ffi::{c_char, c_int};

use super::super::MpGameImport;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_REPLY_CHAT` outbound game-to-engine syscall.
///
/// Mirrors: `int trap_BotReplyChat(int chatstate, char *message, int mcontext, int vcontext,
///     char *var0, char *var1, char *var2, char *var3, char *var4, char *var5, char *var6, char *var7)`
#[derive(Debug)]
pub struct BotlibAiReplyChatArgs {
    chatstate: c_int,
    message: *const c_char,
    mcontext: c_int,
    vcontext: c_int,
    var0: *const c_char,
    var1: *const c_char,
    var2: *const c_char,
    var3: *const c_char,
    var4: *const c_char,
    var5: *const c_char,
    var6: *const c_char,
    var7: *const c_char,
}

impl BotlibAiReplyChatArgs {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        chatstate: c_int,
        message: *const c_char,
        mcontext: c_int,
        vcontext: c_int,
        var0: *const c_char,
        var1: *const c_char,
        var2: *const c_char,
        var3: *const c_char,
        var4: *const c_char,
        var5: *const c_char,
        var6: *const c_char,
        var7: *const c_char,
    ) -> Self {
        Self {
            chatstate,
            message,
            mcontext,
            vcontext,
            var0,
            var1,
            var2,
            var3,
            var4,
            var5,
            var6,
            var7,
        }
    }

    pub fn chatstate(&self) -> c_int {
        self.chatstate
    }
    pub fn message(&self) -> *const c_char {
        self.message
    }
    pub fn mcontext(&self) -> c_int {
        self.mcontext
    }
    pub fn vcontext(&self) -> c_int {
        self.vcontext
    }
    pub fn var0(&self) -> *const c_char {
        self.var0
    }
    pub fn var1(&self) -> *const c_char {
        self.var1
    }
    pub fn var2(&self) -> *const c_char {
        self.var2
    }
    pub fn var3(&self) -> *const c_char {
        self.var3
    }
    pub fn var4(&self) -> *const c_char {
        self.var4
    }
    pub fn var5(&self) -> *const c_char {
        self.var5
    }
    pub fn var6(&self) -> *const c_char {
        self.var6
    }
    pub fn var7(&self) -> *const c_char {
        self.var7
    }
}

/// `BOTLIB_AI_REPLY_CHAT` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:428`
pub struct BotlibAiReplyChat;

impl OutboundSysCall for BotlibAiReplyChat {
    type Import = MpGameImport;
    type Args = BotlibAiReplyChatArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_REPLY_CHAT;
}

impl EncodeSysCall for BotlibAiReplyChat {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.chatstate as isize,
            ptr_to_word(a.message),
            a.mcontext as isize,
            a.vcontext as isize,
            ptr_to_word(a.var0),
            ptr_to_word(a.var1),
            ptr_to_word(a.var2),
            ptr_to_word(a.var3),
            ptr_to_word(a.var4),
            ptr_to_word(a.var5),
            ptr_to_word(a.var6),
            ptr_to_word(a.var7),
        ])
    }
}

impl DecodeSysCallReturn for BotlibAiReplyChat {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
