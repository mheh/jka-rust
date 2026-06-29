use core::ffi::{c_char, c_int};

use crate::ffi::GameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_INITIAL_CHAT` outbound game-to-engine syscall.
///
/// C signature:
/// ```c
/// void trap_BotInitialChat(int chatstate, char *type, int mcontext,
///     char *var0, char *var1, char *var2, char *var3,
///     char *var4, char *var5, char *var6, char *var7);
/// ```
#[derive(Debug)]
pub struct BotlibAiInitialChatArgs {
    pub chatstate: c_int,
    pub r#type: *const c_char,
    pub mcontext: c_int,
    pub var0: *const c_char,
    pub var1: *const c_char,
    pub var2: *const c_char,
    pub var3: *const c_char,
    pub var4: *const c_char,
    pub var5: *const c_char,
    pub var6: *const c_char,
    pub var7: *const c_char,
}

impl BotlibAiInitialChatArgs {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        chatstate: c_int,
        r#type: *const c_char,
        mcontext: c_int,
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
            r#type,
            mcontext,
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
    pub fn r#type(&self) -> *const c_char {
        self.r#type
    }
    pub fn mcontext(&self) -> c_int {
        self.mcontext
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

/// `BOTLIB_AI_INITIAL_CHAT` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:427`
pub struct BotlibAiInitialChat;

impl OutboundSysCall for BotlibAiInitialChat {
    type Import = GameImport;
    type Args = BotlibAiInitialChatArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_INITIAL_CHAT;
}

impl EncodeSysCall for BotlibAiInitialChat {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.chatstate as isize,
            ptr_to_word(a.r#type),
            a.mcontext as isize,
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

impl DecodeSysCallReturn for BotlibAiInitialChat {
    fn decode_return(_word: isize) -> Self::Output {}
}
