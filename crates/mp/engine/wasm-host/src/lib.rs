//! `mp_engine_wasm_host` — host-side wasm pointer shape + memory accessors
//! (LOAD-D3/D10). Isolates the wasmtime dep so native-only builds carry no wasm
//! toolchain.

use core::marker::PhantomData;

/// A guest linear-memory offset interpreted against a module `Memory` (LOAD-D3).
/// A `u32`, **never** a host address.
///
/// Source: `docs/architecture/module-loading.md` § Host-side wasm pointer shape (LOAD-D3).
#[repr(transparent)]
pub struct WasmPtr<T>(pub u32, PhantomData<T>);

/// Bounds-checked, per-access accessors bound to one module's linear memory. The
/// `Memory` handle (wasmtime or equivalent) is re-read every call; no base is
/// cached (survives `memory.grow`). `None` = out of bounds (the sandbox fence).
///
/// Source: `docs/architecture/module-loading.md` § Host-side wasm pointer shape (LOAD-D3).
pub trait ModuleMemory {
    fn read<T: bytemuck::Pod>(&self, p: WasmPtr<T>) -> Option<T>;
    fn write<T: bytemuck::Pod>(&self, p: WasmPtr<T>, v: T) -> Option<()>;
    fn bytes(&self, p: WasmPtr<u8>, len: u32) -> Option<&[u8]>;
    fn bytes_mut(&mut self, p: WasmPtr<u8>, len: u32) -> Option<&mut [u8]>;
}
