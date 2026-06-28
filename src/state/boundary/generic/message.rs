use std::ffi::CString;

use super::outbound::{OutboundSysCall, OutboundSysCallExecutor};

/// Owned message payload shared by outbound message-style syscalls.
///
/// The message is owned so future queued/router code never holds a borrowed
/// string that could expire before the engine sees it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageArgs {
    message: CString,
}

impl MessageArgs {
    pub fn new(message: &str) -> Self {
        Self {
            message: CString::new(message)
                .unwrap_or_else(|_| CString::new(message.replace('\0', "")).unwrap_or_default()),
        }
    }

    pub fn format(args: std::fmt::Arguments<'_>) -> Self {
        Self::new(&args.to_string())
    }

    pub fn message(&self) -> &CString {
        &self.message
    }
}

/// Marker for outbound syscalls whose only payload is an owned message.
pub trait MessageOutboundSysCall: OutboundSysCall<Args = MessageArgs> {}

/// Route an outbound message syscall.
pub trait MessageOutboundSysCallExecutor: OutboundSysCallExecutor {
    fn call_message<C>(&self, args: std::fmt::Arguments<'_>) -> C::Output
    where
        C: MessageOutboundSysCall,
    {
        self.call_outbound::<C>(MessageArgs::format(args))
    }
}

impl<T> MessageOutboundSysCallExecutor for T where T: OutboundSysCallExecutor {}
