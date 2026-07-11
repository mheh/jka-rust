//! MP ICARUS `Q3_Registers.cpp` — the script variable store (§F idiomatic).
//!
//! The `varStrings`/`varFloats`/`varVectors` stores live on `Icarus`
//! (State-ownership table); these fns re-index those field borrows. The
//! `q3_get_*` fns are the `G_ICARUS_GET*VARIABLE` arm callees; `q3_variable_declared`
//! is `G_ICARUS_VARIABLEDECLARED`; `q3_declare_variable`/`q3_free_variable` have
//! NO arm — they are the outbound `I_DeclareVariable`/`I_FreeVariable` targets
//! (`Interface_Init`, `Q3_Interface.cpp:1002-1003`). Diagnostics gate on
//! `host.cvar_integer("developer")` (ruling 36).

#![allow(non_snake_case)]

use std::collections::HashMap;

use mp_host_interface::EngineHost;
use mp_qshared::shared::wl_e::WL_e;

use crate::Icarus;

/// Raven `MAX_VARIABLES`.
/// Source: `oracle/codemp/icarus/Q3_Registers.h:13`
pub const MAX_VARIABLES: i32 = 32;

/// Raven anonymous variable-type enum `VTYPE_NONE`.
/// Source: `oracle/codemp/icarus/Q3_Registers.h:4-11`
pub const VTYPE_NONE: i32 = 0;
/// Raven `VTYPE_FLOAT`.
/// Source: `oracle/codemp/icarus/Q3_Registers.h:4-11`
pub const VTYPE_FLOAT: i32 = 1;
/// Raven `VTYPE_STRING`.
/// Source: `oracle/codemp/icarus/Q3_Registers.h:4-11`
pub const VTYPE_STRING: i32 = 2;
/// Raven `VTYPE_VECTOR`.
/// Source: `oracle/codemp/icarus/Q3_Registers.h:4-11`
pub const VTYPE_VECTOR: i32 = 3;

// `Q3_DeclareVariable`'s `switch(type)` matches the *tokenizer's* token
// values, not the local `VTYPE_*` enum above: `tokenizer.h`'s base token enum
// plus `interpreter.h`'s user-token enum starting at `TK_USERDEF`.
// `tokenizer.h`/`interpreter.h` are untouched skeletons this run (out of
// scope), so the raw values are reproduced here rather than imported.
/// Raven `TK_STRING`.
/// Source: `oracle/codemp/icarus/tokenizer.h:63-73`
const TK_STRING: i32 = 4;
/// Raven `TK_FLOAT`.
/// Source: `oracle/codemp/icarus/tokenizer.h:63-73`
const TK_FLOAT: i32 = 6;
/// Raven `TK_VECTOR`.
/// Source: `oracle/codemp/icarus/interpreter.h:16-23`
const TK_VECTOR: i32 = 14;

/// Raven `Q3_InitVariables` — reset the variable stores.
/// Source: `oracle/codemp/icarus/Q3_Registers.cpp:192-202`
pub fn Q3_InitVariables(icarus: &mut Icarus, host: &mut dyn EngineHost) {
    icarus.var_strings.clear();
    icarus.var_floats.clear();
    icarus.var_vectors.clear();

    // Raven warns about any variables still present at reset time
    // (`Q3_DebugPrint( WL_WARNING, "%d residual variables found!\n", … )`);
    // the diagnostic is `developer`-gated through `Q3_DebugPrint` (ruling 36).
    if icarus.num_variables > 0 {
        let n = icarus.num_variables;
        crate::q3_interface::Q3_DebugPrint(
            icarus,
            host,
            WL_e::WL_WARNING as i32,
            &format!("{} residual variables found!\n", n),
        );
    }

    icarus.num_variables = 0;
}

/// Raven `Q3_DeclareVariable` — the `I_DeclareVariable` target.
/// Source: `oracle/codemp/icarus/Q3_Registers.cpp:50`
pub fn q3_declare_variable(
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    var_type: i32,
    name: &str,
) {
    // Cannot declare the same variable twice.
    if q3_variable_declared(icarus, host, name) != VTYPE_NONE {
        return;
    }

    if icarus.num_variables > MAX_VARIABLES {
        crate::q3_interface::Q3_DebugPrint(
            icarus,
            host,
            WL_e::WL_ERROR as i32,
            &format!(
                "too many variables already declared, maximum is {}\n",
                MAX_VARIABLES
            ),
        );
        return;
    }

    match var_type {
        TK_FLOAT => {
            icarus.var_floats.insert(name.to_string(), 0.0);
        }
        TK_STRING => {
            icarus
                .var_strings
                .insert(name.to_string(), "NULL".to_string());
        }
        TK_VECTOR => {
            icarus
                .var_vectors
                .insert(name.to_string(), "0.0 0.0 0.0".to_string());
        }
        _ => {
            crate::q3_interface::Q3_DebugPrint(
                icarus,
                host,
                WL_e::WL_ERROR as i32,
                "unknown 'type' for declare() function!\n",
            );
            return;
        }
    }

    icarus.num_variables += 1;
}

/// Raven `Q3_FreeVariable` — the `I_FreeVariable` target.
/// Source: `oracle/codemp/icarus/Q3_Registers.cpp:91`
pub fn q3_free_variable(icarus: &mut Icarus, host: &mut dyn EngineHost, name: &str) {
    let _ = host;

    // Check the strings.
    if icarus.var_strings.remove(name).is_some() {
        icarus.num_variables -= 1;
        return;
    }

    // Check the floats.
    if icarus.var_floats.remove(name).is_some() {
        icarus.num_variables -= 1;
        return;
    }

    // Check the vectors.
    if icarus.var_vectors.remove(name).is_some() {
        icarus.num_variables -= 1;
        return;
    }
}

/// Raven `Q3_VariableDeclared` — the `G_ICARUS_VARIABLEDECLARED` arm callee.
/// Source: `oracle/codemp/icarus/Q3_Registers.cpp:21`
pub fn q3_variable_declared(icarus: &mut Icarus, host: &mut dyn EngineHost, name: &str) -> i32 {
    let _ = host;

    // Check the strings.
    if icarus.var_strings.contains_key(name) {
        return VTYPE_STRING;
    }

    // Check the floats.
    if icarus.var_floats.contains_key(name) {
        return VTYPE_FLOAT;
    }

    // Check the vectors.
    if icarus.var_vectors.contains_key(name) {
        return VTYPE_VECTOR;
    }

    VTYPE_NONE
}

/// Raven `Q3_GetFloatVariable` — the `G_ICARUS_GETFLOATVARIABLE` arm callee
/// (out-param → `Option`, §C7).
/// Source: `oracle/codemp/icarus/Q3_Registers.cpp:130`
pub fn q3_get_float_variable(
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    name: &str,
) -> Option<f32> {
    let _ = host;

    icarus.var_floats.get(name).copied()
}

/// Raven `Q3_GetStringVariable` — the `G_ICARUS_GETSTRINGVARIABLE` arm callee.
/// Source: `oracle/codemp/icarus/Q3_Registers.cpp:150`
pub fn q3_get_string_variable(
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    name: &str,
) -> Option<String> {
    let _ = host;

    icarus.var_strings.get(name).cloned()
}

/// Raven `Q3_GetVectorVariable` — the `G_ICARUS_GETVECTORVARIABLE` arm callee.
/// Source: `oracle/codemp/icarus/Q3_Registers.cpp:170`
pub fn q3_get_vector_variable(
    icarus: &mut Icarus,
    host: &mut dyn EngineHost,
    name: &str,
) -> Option<[f32; 3]> {
    let _ = host;

    let stored = icarus.var_vectors.get(name)?;

    // Raven `sscanf(str, "%f %f %f", &value[0..2])` leaves a component sscanf
    // fails to match untouched (uninitialized in C); ported as `0.0` since
    // this fn always builds a fresh array (porting-rules §19).
    let mut value = [0.0f32; 3];
    for (slot, token) in value.iter_mut().zip(stored.split_whitespace()) {
        if let Ok(parsed) = token.parse::<f32>() {
            *slot = parsed;
        }
    }

    Some(value)
}

/// Raven `Q3_SetFloatVariable`.
/// Source: `oracle/codemp/icarus/Q3_Registers.cpp`
pub fn Q3_SetFloatVariable(icarus: &mut Icarus, name: &str, value: f32) -> i32 {
    // Raven's not-found branch `return VTYPE_FLOAT;` is a stray value, not
    // `false` — both branches are truthy in C, so this always reports
    // success; transcribed as-is (porting-rules §2: faithful before clean).
    match icarus.var_floats.get_mut(name) {
        Some(slot) => {
            *slot = value;
            1
        }
        None => VTYPE_FLOAT,
    }
}

/// Raven `Q3_SetStringVariable`.
/// Source: `oracle/codemp/icarus/Q3_Registers.cpp`
pub fn Q3_SetStringVariable(icarus: &mut Icarus, name: &str, value: &str) -> i32 {
    match icarus.var_strings.get_mut(name) {
        Some(slot) => {
            *slot = value.to_string();
            1
        }
        None => 0,
    }
}

/// Raven `Q3_SetVectorVariable`.
/// Source: `oracle/codemp/icarus/Q3_Registers.cpp`
pub fn Q3_SetVectorVariable(icarus: &mut Icarus, name: &str, value: &str) -> i32 {
    match icarus.var_vectors.get_mut(name) {
        Some(slot) => {
            *slot = value.to_string();
            1
        }
        None => 0,
    }
}

/// Raven `Q3_VariableSaveFloats` — inert in MP: the oracle body is an
/// unconditional `return;`, so the save-game append tail is dead and untranscribed.
/// Source: `oracle/codemp/icarus/Q3_Registers.cpp:267-288`
pub fn Q3_VariableSaveFloats(fmap: &HashMap<String, f32>) {
    let _ = fmap;
}

/// Raven `Q3_VariableSaveStrings` — inert in MP: the oracle body is an
/// unconditional `return;`, so the save-game append tail is dead and untranscribed.
/// Source: `oracle/codemp/icarus/Q3_Registers.cpp:296-320`
pub fn Q3_VariableSaveStrings(smap: &HashMap<String, String>) {
    let _ = smap;
}

/// Raven `Q3_VariableSave` — drives the (inert) per-map save helpers, returns qtrue.
/// Source: `oracle/codemp/icarus/Q3_Registers.cpp:328-335`
pub fn Q3_VariableSave(icarus: &Icarus) -> i32 {
    Q3_VariableSaveFloats(&icarus.var_floats);
    Q3_VariableSaveStrings(&icarus.var_strings);
    Q3_VariableSaveStrings(&icarus.var_vectors);

    1 // qtrue
}

/// Raven `Q3_VariableLoadFloats` — inert in MP: the oracle body is an
/// unconditional `return;`, so the save-game read tail is dead and untranscribed.
/// Source: `oracle/codemp/icarus/Q3_Registers.cpp:343-368`
pub fn Q3_VariableLoadFloats(fmap: &mut HashMap<String, f32>) {
    let _ = fmap;
}

/// Raven `Q3_VariableLoadStrings` — inert in MP: the oracle body is an
/// unconditional `return;`, so the save-game read tail is dead and untranscribed.
/// Source: `oracle/codemp/icarus/Q3_Registers.cpp:376-412`
pub fn Q3_VariableLoadStrings(var_type: i32, fmap: &mut HashMap<String, String>) {
    let _ = var_type;
    let _ = fmap;
}

/// Raven `Q3_VariableLoad` — resets the store then drives the (inert) per-map
/// load helpers, returns qfalse.
/// Source: `oracle/codemp/icarus/Q3_Registers.cpp:420-428`
pub fn Q3_VariableLoad(icarus: &mut Icarus, host: &mut dyn EngineHost) -> i32 {
    Q3_InitVariables(icarus, host);

    Q3_VariableLoadFloats(&mut icarus.var_floats);
    Q3_VariableLoadStrings(TK_STRING, &mut icarus.var_strings);
    Q3_VariableLoadStrings(TK_VECTOR, &mut icarus.var_vectors);

    0 // qfalse
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Q3_SetFloatVariable`'s not-found branch returns `VTYPE_FLOAT` (1), not
    /// `false` (0) — a Raven copy/paste quirk that still reads truthy in C.
    /// Guards against "fixing" it to `0` during a later cleanup pass.
    #[test]
    fn set_float_variable_not_found_returns_vtype_float_not_false() {
        let mut icarus = Icarus::default();
        assert_eq!(Q3_SetFloatVariable(&mut icarus, "nope", 1.0), VTYPE_FLOAT);
    }

    /// `Q3_GetVectorVariable`'s `sscanf` reads whitespace-separated floats out
    /// of the backing string store; a short/malformed string leaves the
    /// remaining components at the port's `0.0` fill value.
    #[test]
    fn get_vector_variable_parses_stored_string_and_defaults_short_input() {
        let mut icarus = Icarus::default();
        icarus
            .var_vectors
            .insert("v".to_string(), "1.0 2.0 3.0".to_string());
        icarus
            .var_vectors
            .insert("short".to_string(), "5.0".to_string());

        let mut host = mp_host_interface::mock::MockHost::default();

        assert_eq!(
            q3_get_vector_variable(&mut icarus, &mut host, "v"),
            Some([1.0, 2.0, 3.0])
        );
        assert_eq!(
            q3_get_vector_variable(&mut icarus, &mut host, "short"),
            Some([5.0, 0.0, 0.0])
        );
        assert_eq!(
            q3_get_vector_variable(&mut icarus, &mut host, "missing"),
            None
        );
    }

    /// `Q3_DeclareVariable`'s duplicate guard: declaring an already-declared
    /// name is a silent no-op (`Q3_VariableDeclared(name) != VTYPE_NONE` early
    /// return), it does not overwrite the existing value or bump the count.
    #[test]
    fn declare_variable_is_noop_on_already_declared_name() {
        let mut icarus = Icarus::default();
        icarus.var_floats.insert("f".to_string(), 42.0);
        icarus.num_variables = 1;

        let mut host = mp_host_interface::mock::MockHost::default();
        q3_declare_variable(&mut icarus, &mut host, TK_FLOAT, "f");

        assert_eq!(icarus.var_floats.get("f"), Some(&42.0));
        assert_eq!(icarus.num_variables, 1);
    }
}
