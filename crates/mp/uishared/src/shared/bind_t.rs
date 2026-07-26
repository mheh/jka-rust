//! `Bind` — Raven `bind_t`.

use core::ffi::c_int;

/// Raven `bind_t` — one row of the controls-menu key-binding table.
///
/// The table is seeded from a static command list but is **not** read-only:
/// `Controls_GetConfig` writes the live `bind1`/`bind2` back into every row, so
/// the rows are [`MenuSystem`](super::menu_system::MenuSystem) state, not a
/// `const`. `command` keeps `&'static str` — it is the compiled-in console
/// command name and is never rewritten.
///
/// Type definition source: `oracle/codemp/ui/ui_shared.c:5173-5180`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[doc(alias = "bind_t")]
#[allow(non_snake_case)]
pub struct Bind {
    pub command: &'static str,
    pub id: c_int,
    pub defaultbind1: c_int,
    pub defaultbind2: c_int,
    pub bind1: c_int,
    pub bind2: c_int,
}

// `A_*` keycode literals (`oracle/codemp/ui/keycodes.h`'s `fakeAscii_t`) needed
// by `default_bindings` below. Only the subset `g_bindings` actually uses.
const A_SHIFT: c_int = 1;
const A_CTRL: c_int = 2;
const A_ALT: c_int = 3;
const A_TAB: c_int = 9;
const A_ENTER: c_int = 10;
const A_F1: c_int = 28;
const A_F2: c_int = 29;
const A_F3: c_int = 30;
const A_F4: c_int = 31;
const A_SPACE: c_int = 32;
const A_DELETE: c_int = 127;
const A_F5: c_int = 132;
const A_F6: c_int = 133;
const A_F7: c_int = 134;
const A_INSERT: c_int = 143;
const A_END: c_int = 157;
const A_PAGE_DOWN: c_int = 158;
const A_CURSOR_UP: c_int = 170;
const A_CURSOR_DOWN: c_int = 171;
const A_CURSOR_LEFT: c_int = 172;
const A_CURSOR_RIGHT: c_int = 173;

/// One `g_bindings` row: `defaultbind1`/`defaultbind2` are always `-1` in
/// Raven's initializer, and the C brace-init leaves the unlisted `bind2`
/// implicitly zeroed (`bind1`'s explicit `-1` is the only listed sentinel);
/// both are overwritten by the first `Controls_GetConfig` regardless.
const fn row(command: &'static str, id: c_int) -> Bind {
    Bind {
        command,
        id,
        defaultbind1: -1,
        defaultbind2: -1,
        bind1: -1,
        bind2: 0,
    }
}

/// Raven `static bind_t g_bindings[]` — the compiled-in controls table.
///
/// Raven's array is file-scope static data, populated unconditionally at
/// program load; the exact same guarantee here is that every
/// [`MenuSystem`](super::menu_system::MenuSystem) is constructed with this
/// table already seeded (its `Default` impl calls this), never with an empty
/// `Vec` a caller would have to remember to fill in.
///
/// Source: `oracle/codemp/ui/ui_shared.c:5190-5292`
pub fn default_bindings() -> Vec<Bind> {
    vec![
        row("+scores", A_TAB),
        row("+button2", A_ENTER),
        row("+speed", A_SHIFT),
        row("+forward", A_CURSOR_UP),
        row("+back", A_CURSOR_DOWN),
        row("+moveleft", ',' as c_int),
        row("+moveright", '.' as c_int),
        row("+moveup", A_SPACE),
        row("+movedown", 'c' as c_int),
        row("+left", A_CURSOR_LEFT),
        row("+right", A_CURSOR_RIGHT),
        row("+strafe", A_ALT),
        row("+lookup", A_PAGE_DOWN),
        row("+lookdown", A_DELETE),
        row("+mlook", '/' as c_int),
        row("centerview", A_END),
        //	{"+zoom", 			 -1,				-1,		-1, -1},
        row("weapon 1", '1' as c_int),
        row("weapon 2", '2' as c_int),
        row("weapon 3", '3' as c_int),
        row("weapon 4", '4' as c_int),
        row("weapon 5", '5' as c_int),
        row("weapon 6", '6' as c_int),
        row("weapon 7", '7' as c_int),
        row("weapon 8", '8' as c_int),
        row("weapon 9", '9' as c_int),
        row("weapon 10", '0' as c_int),
        row("saberAttackCycle", 'l' as c_int),
        row("weapon 11", -1),
        row("weapon 12", -1),
        row("weapon 13", -1),
        row("+attack", A_CTRL),
        row("+altattack", -1),
        row("+use", -1),
        row("engage_duel", 'h' as c_int),
        row("taunt", 'u' as c_int),
        row("bow", -1),
        row("meditate", -1),
        row("flourish", -1),
        row("gloat", -1),
        row("weapprev", '[' as c_int),
        row("weapnext", ']' as c_int),
        row("prevTeamMember", 'w' as c_int),
        row("nextTeamMember", 'r' as c_int),
        row("nextOrder", 't' as c_int),
        row("confirmOrder", 'y' as c_int),
        row("denyOrder", 'n' as c_int),
        row("taskOffense", 'o' as c_int),
        row("taskDefense", 'd' as c_int),
        row("taskPatrol", 'p' as c_int),
        row("taskCamp", 'c' as c_int),
        row("taskFollow", 'f' as c_int),
        row("taskRetrieve", 'v' as c_int),
        row("taskEscort", 'e' as c_int),
        row("taskOwnFlag", 'i' as c_int),
        row("taskSuicide", 'k' as c_int),
        row("tauntKillInsult", -1),
        row("tauntPraise", -1),
        row("tauntTaunt", -1),
        row("tauntDeathInsult", -1),
        row("tauntGauntlet", -1),
        row("scoresUp", A_INSERT),
        row("scoresDown", A_DELETE),
        row("messagemode", -1),
        row("messagemode2", -1),
        row("messagemode3", -1),
        row("messagemode4", -1),
        row("+use", -1),
        row("+force_jump", -1),
        row("force_throw", A_F1),
        row("force_pull", A_F2),
        row("force_speed", A_F3),
        row("force_distract", A_F4),
        row("force_heal", A_F5),
        row("+force_grip", A_F6),
        row("+force_lightning", A_F7),
        row("+force_drain", -1),
        row("force_rage", -1),
        row("force_protect", -1),
        row("force_absorb", -1),
        row("force_healother", -1),
        row("force_forcepowerother", -1),
        row("force_seeing", -1),
        row("+useforce", -1),
        row("forcenext", -1),
        row("forceprev", -1),
        row("invnext", -1),
        row("invprev", -1),
        row("use_seeker", -1),
        row("use_field", -1),
        row("use_bacta", -1),
        row("use_electrobinoculars", -1),
        row("use_sentry", -1),
        row("cg_thirdperson !", -1),
        row("automap_button", -1),
        row("automap_toggle", -1),
        row("voicechat", -1),
    ]
}
