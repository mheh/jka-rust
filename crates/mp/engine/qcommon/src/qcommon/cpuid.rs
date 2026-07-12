#![allow(non_camel_case_types, non_snake_case)]

// Raven: any CPUID_XXXX defined as higher than CPUID_INTEL_MMX *must* have
// MMX support (eg like CPUID_AMD_3DNOW (0x30) has), this allows convenient
// MMX capability checking. Values returned by `Sys_GetProcessorId`.

/// Raven `CPUID_GENERIC` — any unrecognized processor.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:645`
pub const CPUID_GENERIC: i32 = 0;

/// Raven `CPUID_AXP`.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:647`
pub const CPUID_AXP: i32 = 0x10;

/// Raven `CPUID_INTEL_UNSUPPORTED` — Intel 386/486.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:649`
pub const CPUID_INTEL_UNSUPPORTED: i32 = 0x20;

/// Raven `CPUID_INTEL_PENTIUM` — Intel Pentium or PPro.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:650`
pub const CPUID_INTEL_PENTIUM: i32 = 0x21;

/// Raven `CPUID_INTEL_MMX` — Intel Pentium/MMX or P2/MMX.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:651`
pub const CPUID_INTEL_MMX: i32 = 0x22;

/// Raven `CPUID_INTEL_KATMAI`.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:652`
pub const CPUID_INTEL_KATMAI: i32 = 0x23;

/// Raven `CPUID_INTEL_WILLIAMETTE`.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:653`
pub const CPUID_INTEL_WILLIAMETTE: i32 = 0x24;

/// Raven `CPUID_AMD_3DNOW` — AMD K6 3DNOW!.
///
/// Source: `oracle/codemp/qcommon/qcommon.h:655`
pub const CPUID_AMD_3DNOW: i32 = 0x30;
