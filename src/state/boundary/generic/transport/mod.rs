pub mod syscall;
pub mod vm_main;

pub use syscall::{
    c_int_to_word, ptr_to_word, DecodeSysCallReturn, EncodeSysCall, SysCallTransport,
};
pub use vm_main::{
    word_to_c_int, word_to_const_ptr, word_to_mut_ptr, DecodeVmMain, EncodeVmMainReturn,
    VmMainTransport,
};
