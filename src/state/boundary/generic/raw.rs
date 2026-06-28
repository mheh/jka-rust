use core::ffi::c_int;

/// Empty ABI payload for calls whose C signature is `( void )`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NoArgs;

/// Single `int` payload for the common one-scalar boundary calls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OneIntArg {
    value: c_int,
}

impl OneIntArg {
    pub const fn new(value: c_int) -> Self {
        Self { value }
    }

    pub const fn value(self) -> c_int {
        self.value
    }
}

/// Two `int` payload for simple scalar queries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TwoIntArgs {
    first: c_int,
    second: c_int,
}

impl TwoIntArgs {
    pub const fn new(first: c_int, second: c_int) -> Self {
        Self { first, second }
    }

    pub const fn first(self) -> c_int {
        self.first
    }

    pub const fn second(self) -> c_int {
        self.second
    }
}

/// Three `int` payload for simple scalar calls that need one more value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThreeIntArgs {
    first: c_int,
    second: c_int,
    third: c_int,
}

impl ThreeIntArgs {
    pub const fn new(first: c_int, second: c_int, third: c_int) -> Self {
        Self {
            first,
            second,
            third,
        }
    }

    pub const fn first(self) -> c_int {
        self.first
    }

    pub const fn second(self) -> c_int {
        self.second
    }

    pub const fn third(self) -> c_int {
        self.third
    }
}

/// Raw executable-to-game `vmMain` payload.
///
/// This is for named vmcalls whose safe payload type has not been carved out
/// yet. It keeps the incoming `intptr_t` arguments verbatim.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawVmCallArgs {
    args: [isize; 12],
}

impl RawVmCallArgs {
    pub const fn new(args: [isize; 12]) -> Self {
        Self { args }
    }

    pub const fn args(self) -> [isize; 12] {
        self.args
    }
}

/// Raw game-to-engine syscall payload.
///
/// This is for named syscalls whose final safe payload type is still opaque,
/// pointer-heavy, variadic, or subsystem-specific. The router can still keep
/// order and identity without losing the ABI words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawSysCallArgs {
    args: Box<[isize]>,
}

impl RawSysCallArgs {
    pub fn new(args: impl Into<Box<[isize]>>) -> Self {
        Self { args: args.into() }
    }

    pub fn args(&self) -> &[isize] {
        &self.args
    }
}
