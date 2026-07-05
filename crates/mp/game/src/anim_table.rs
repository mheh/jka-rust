//! MP `animTable` — animation-name lookup table (`stringID_table_t[]`).
//!
//! Raven: `stringID_table_t animTable[MAX_ANIMATIONS+1]` built via the
//! `ENUM2STRING` macro (`name` = stringified enumerator, `id` = the
//! `animNumber_t` value); consumed for anim-name <-> id lookups (obituary/
//! dismemberment logs, saber-move name printing, ICARUS anim commands).
//! Table entries and `animNumber_t` discriminants are 1:1 in declaration
//! order (verified against `anims.h`), so `animTable[n].id == n` for every
//! entry below. Raven callers (`GetIDForString`) walk the array until a
//! `NULL` `name`, so the trailing `{ NULL, -1 }` sentinel (`animtable.h:1789`)
//! is preserved as the last element here; the array is otherwise 1 element
//! shorter than the oracle's `[MAX_ANIMATIONS+1]` declaration (which
//! implicit-zero-fills unused tail slots past the sentinel — dead weight,
//! since no caller reads past the sentinel).
//!
//! Source: `oracle/oracle/codemp/cgame/animtable.h:9-1789`

#![allow(non_upper_case_globals)]

use core::ffi::{c_char, c_int};
use mp_bg::public::anim_number::animNumber_t;
use mp_qshared::shared::string_id_table::stringID_table_t;

pub static animTable: [stringID_table_t; 1544] = [
    stringID_table_t {
        name: c"FACE_TALK0".as_ptr() as *mut c_char,
        id: animNumber_t::FACE_TALK0 as c_int,
    },
    stringID_table_t {
        name: c"FACE_TALK1".as_ptr() as *mut c_char,
        id: animNumber_t::FACE_TALK1 as c_int,
    },
    stringID_table_t {
        name: c"FACE_TALK2".as_ptr() as *mut c_char,
        id: animNumber_t::FACE_TALK2 as c_int,
    },
    stringID_table_t {
        name: c"FACE_TALK3".as_ptr() as *mut c_char,
        id: animNumber_t::FACE_TALK3 as c_int,
    },
    stringID_table_t {
        name: c"FACE_TALK4".as_ptr() as *mut c_char,
        id: animNumber_t::FACE_TALK4 as c_int,
    },
    stringID_table_t {
        name: c"FACE_ALERT".as_ptr() as *mut c_char,
        id: animNumber_t::FACE_ALERT as c_int,
    },
    stringID_table_t {
        name: c"FACE_SMILE".as_ptr() as *mut c_char,
        id: animNumber_t::FACE_SMILE as c_int,
    },
    stringID_table_t {
        name: c"FACE_FROWN".as_ptr() as *mut c_char,
        id: animNumber_t::FACE_FROWN as c_int,
    },
    stringID_table_t {
        name: c"FACE_DEAD".as_ptr() as *mut c_char,
        id: animNumber_t::FACE_DEAD as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH3".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH3 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH4".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH4 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH5".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH5 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH6".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH6 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH7".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH7 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH8".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH8 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH9".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH9 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH10".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH10 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH11".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH11 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH12".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH12 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH13".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH13 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH14".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH14 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH15".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH15 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH16".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH16 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH17".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH17 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH18".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH18 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH19".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH19 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH20".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH20 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH21".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH21 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH22".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH22 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH23".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH23 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH24".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH24 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH25".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH25 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATHFORWARD1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATHFORWARD1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATHFORWARD2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATHFORWARD2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATHFORWARD3".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATHFORWARD3 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATHBACKWARD1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATHBACKWARD1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATHBACKWARD2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATHBACKWARD2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH1IDLE".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH1IDLE as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LYINGDEATH1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LYINGDEATH1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STUMBLEDEATH1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STUMBLEDEATH1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FALLDEATH1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FALLDEATH1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FALLDEATH1INAIR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FALLDEATH1INAIR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FALLDEATH1LAND".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FALLDEATH1LAND as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH_ROLL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH_ROLL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH_FLIP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH_FLIP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH_SPIN_90_R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH_SPIN_90_R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH_SPIN_90_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH_SPIN_90_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH_SPIN_180".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH_SPIN_180 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH_LYING_UP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH_LYING_UP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH_LYING_DN".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH_LYING_DN as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH_FALLING_DN".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH_FALLING_DN as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH_FALLING_UP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH_FALLING_UP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH_CROUCHED".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH_CROUCHED as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEAD1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEAD1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEAD2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEAD2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEAD3".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEAD3 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEAD4".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEAD4 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEAD5".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEAD5 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEAD6".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEAD6 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEAD7".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEAD7 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEAD8".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEAD8 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEAD9".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEAD9 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEAD10".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEAD10 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEAD11".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEAD11 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEAD12".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEAD12 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEAD13".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEAD13 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEAD14".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEAD14 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEAD15".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEAD15 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEAD16".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEAD16 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEAD17".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEAD17 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEAD18".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEAD18 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEAD19".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEAD19 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEAD20".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEAD20 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEAD21".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEAD21 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEAD22".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEAD22 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEAD23".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEAD23 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEAD24".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEAD24 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEAD25".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEAD25 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEADFORWARD1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEADFORWARD1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEADFORWARD2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEADFORWARD2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEADBACKWARD1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEADBACKWARD1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEADBACKWARD2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEADBACKWARD2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LYINGDEAD1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LYINGDEAD1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STUMBLEDEAD1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STUMBLEDEAD1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FALLDEAD1LAND".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FALLDEAD1LAND as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEADFLOP1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEADFLOP1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEADFLOP2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEADFLOP2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DISMEMBER_HEAD1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DISMEMBER_HEAD1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DISMEMBER_TORSO1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DISMEMBER_TORSO1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DISMEMBER_LLEG".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DISMEMBER_LLEG as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DISMEMBER_RLEG".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DISMEMBER_RLEG as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DISMEMBER_RARM".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DISMEMBER_RARM as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DISMEMBER_LARM".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DISMEMBER_LARM as c_int,
    },
    stringID_table_t {
        name: c"BOTH_PAIN1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_PAIN1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_PAIN2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_PAIN2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_PAIN3".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_PAIN3 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_PAIN4".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_PAIN4 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_PAIN5".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_PAIN5 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_PAIN6".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_PAIN6 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_PAIN7".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_PAIN7 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_PAIN8".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_PAIN8 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_PAIN9".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_PAIN9 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_PAIN10".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_PAIN10 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_PAIN11".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_PAIN11 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_PAIN12".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_PAIN12 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_PAIN13".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_PAIN13 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_PAIN14".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_PAIN14 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_PAIN15".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_PAIN15 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_PAIN16".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_PAIN16 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_PAIN17".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_PAIN17 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_PAIN18".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_PAIN18 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ATTACK1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ATTACK1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ATTACK2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ATTACK2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ATTACK3".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ATTACK3 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ATTACK4".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ATTACK4 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ATTACK5".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ATTACK5 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ATTACK6".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ATTACK6 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ATTACK7".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ATTACK7 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ATTACK10".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ATTACK10 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ATTACK11".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ATTACK11 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_MELEE1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_MELEE1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_MELEE2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_MELEE2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_THERMAL_READY".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_THERMAL_READY as c_int,
    },
    stringID_table_t {
        name: c"BOTH_THERMAL_THROW".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_THERMAL_THROW as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A1_T__B_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A1_T__B_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A1__L__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A1__L__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A1__R__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A1__R__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A1_TL_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A1_TL_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A1_BR_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A1_BR_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A1_BL_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A1_BL_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A1_TR_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A1_TR_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_BR__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_BR__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_BR_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_BR_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_BR__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_BR__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_BR_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_BR_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1__R_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1__R_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1__R_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1__R_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1__R__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1__R__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1__R_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1__R_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_TR_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_TR_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_TR_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_TR_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_TR__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_TR__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_TR_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_TR_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_T__BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_T__BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_T___R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_T___R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_T__TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_T__TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_T__TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_T__TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_T___L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_T___L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_T__BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_T__BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_TL_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_TL_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_TL_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_TL_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1__L_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1__L_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1__L__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1__L__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1__L_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1__L_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_BL_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_BL_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_BL__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_BL__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_BL_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_BL_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_BL__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_BL__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_BR_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_BR_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_BR_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_BR_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1__R_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1__R_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1__R_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1__R_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_TR__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_TR__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_TR_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_TR_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_TL__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_TL__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_TL_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_TL_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_TL_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_TL_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_TL__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_TL__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1__L_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1__L_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1__L_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1__L_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1__L_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1__L_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_BL_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_BL_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T1_BL_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T1_BL_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S1_S1_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S1_S1_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S1_S1__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S1_S1__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S1_S1__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S1_S1__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S1_S1_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S1_S1_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S1_S1_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S1_S1_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S1_S1_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S1_S1_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S1_S1_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S1_S1_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R1_B__S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R1_B__S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R1__L_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R1__L_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R1__R_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R1__R_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R1_TL_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R1_TL_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R1_BR_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R1_BR_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R1_BL_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R1_BL_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R1_TR_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R1_TR_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B1_BR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B1_BR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B1__R___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B1__R___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B1_TR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B1_TR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B1_T____".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B1_T____ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B1_TL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B1_TL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B1__L___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B1__L___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B1_BL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B1_BL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D1_BR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D1_BR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D1__R___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D1__R___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D1_TR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D1_TR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D1_TL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D1_TL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D1__L___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D1__L___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D1_BL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D1_BL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D1_B____".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D1_B____ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A2_T__B_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A2_T__B_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A2__L__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A2__L__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A2__R__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A2__R__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A2_TL_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A2_TL_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A2_BR_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A2_BR_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A2_BL_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A2_BL_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A2_TR_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A2_TR_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_BR__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_BR__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_BR_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_BR_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_BR__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_BR__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_BR_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_BR_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2__R_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2__R_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2__R_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2__R_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2__R__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2__R__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2__R_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2__R_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_TR_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_TR_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_TR_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_TR_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_TR__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_TR__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_TR_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_TR_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_T__BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_T__BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_T___R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_T___R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_T__TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_T__TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_T__TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_T__TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_T___L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_T___L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_T__BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_T__BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_TL_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_TL_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_TL_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_TL_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2__L_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2__L_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2__L__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2__L__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2__L_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2__L_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_BL_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_BL_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_BL__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_BL__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_BL_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_BL_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_BL__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_BL__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_BR_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_BR_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_BR_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_BR_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2__R_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2__R_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2__R_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2__R_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_TR__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_TR__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_TR_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_TR_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_TL__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_TL__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_TL_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_TL_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_TL_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_TL_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_TL__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_TL__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2__L_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2__L_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2__L_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2__L_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2__L_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2__L_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_BL_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_BL_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T2_BL_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T2_BL_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S2_S1_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S2_S1_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S2_S1__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S2_S1__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S2_S1__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S2_S1__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S2_S1_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S2_S1_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S2_S1_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S2_S1_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S2_S1_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S2_S1_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S2_S1_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S2_S1_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R2_B__S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R2_B__S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R2__L_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R2__L_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R2__R_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R2__R_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R2_TL_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R2_TL_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R2_BR_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R2_BR_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R2_BL_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R2_BL_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R2_TR_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R2_TR_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B2_BR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B2_BR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B2__R___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B2__R___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B2_TR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B2_TR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B2_T____".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B2_T____ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B2_TL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B2_TL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B2__L___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B2__L___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B2_BL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B2_BL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D2_BR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D2_BR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D2__R___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D2__R___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D2_TR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D2_TR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D2_TL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D2_TL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D2__L___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D2__L___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D2_BL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D2_BL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D2_B____".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D2_B____ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A3_T__B_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A3_T__B_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A3__L__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A3__L__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A3__R__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A3__R__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A3_TL_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A3_TL_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A3_BR_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A3_BR_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A3_BL_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A3_BL_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A3_TR_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A3_TR_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_BR__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_BR__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_BR_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_BR_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_BR__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_BR__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_BR_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_BR_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3__R_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3__R_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3__R_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3__R_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3__R__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3__R__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3__R_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3__R_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_TR_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_TR_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_TR_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_TR_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_TR__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_TR__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_TR_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_TR_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_T__BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_T__BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_T___R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_T___R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_T__TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_T__TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_T__TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_T__TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_T___L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_T___L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_T__BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_T__BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_TL_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_TL_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_TL_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_TL_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3__L_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3__L_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3__L__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3__L__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3__L_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3__L_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_BL_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_BL_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_BL__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_BL__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_BL_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_BL_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_BL__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_BL__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_BR_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_BR_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_BR_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_BR_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3__R_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3__R_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3__R_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3__R_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_TR__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_TR__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_TR_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_TR_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_TL__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_TL__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_TL_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_TL_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_TL_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_TL_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_TL__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_TL__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3__L_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3__L_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3__L_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3__L_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3__L_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3__L_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_BL_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_BL_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T3_BL_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T3_BL_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S3_S1_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S3_S1_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S3_S1__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S3_S1__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S3_S1__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S3_S1__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S3_S1_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S3_S1_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S3_S1_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S3_S1_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S3_S1_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S3_S1_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S3_S1_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S3_S1_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R3_B__S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R3_B__S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R3__L_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R3__L_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R3__R_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R3__R_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R3_TL_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R3_TL_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R3_BR_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R3_BR_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R3_BL_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R3_BL_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R3_TR_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R3_TR_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B3_BR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B3_BR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B3__R___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B3__R___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B3_TR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B3_TR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B3_T____".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B3_T____ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B3_TL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B3_TL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B3__L___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B3__L___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B3_BL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B3_BL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D3_BR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D3_BR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D3__R___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D3__R___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D3_TR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D3_TR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D3_TL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D3_TL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D3__L___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D3__L___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D3_BL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D3_BL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D3_B____".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D3_B____ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A4_T__B_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A4_T__B_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A4__L__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A4__L__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A4__R__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A4__R__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A4_TL_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A4_TL_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A4_BR_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A4_BR_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A4_BL_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A4_BL_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A4_TR_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A4_TR_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_BR__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_BR__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_BR_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_BR_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_BR__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_BR__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_BR_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_BR_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4__R_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4__R_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4__R_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4__R_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4__R__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4__R__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4__R_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4__R_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_TR_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_TR_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_TR_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_TR_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_TR__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_TR__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_TR_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_TR_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_T__BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_T__BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_T___R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_T___R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_T__TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_T__TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_T__TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_T__TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_T___L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_T___L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_T__BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_T__BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_TL_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_TL_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_TL_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_TL_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4__L_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4__L_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4__L__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4__L__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4__L_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4__L_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_BL_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_BL_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_BL__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_BL__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_BL_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_BL_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_BL__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_BL__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_BR_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_BR_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_BR_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_BR_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4__R_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4__R_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4__R_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4__R_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_TR__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_TR__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_TR_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_TR_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_TL__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_TL__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_TL_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_TL_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_TL_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_TL_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_TL__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_TL__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4__L_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4__L_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4__L_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4__L_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4__L_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4__L_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_BL_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_BL_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T4_BL_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T4_BL_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S4_S1_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S4_S1_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S4_S1__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S4_S1__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S4_S1__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S4_S1__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S4_S1_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S4_S1_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S4_S1_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S4_S1_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S4_S1_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S4_S1_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S4_S1_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S4_S1_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R4_B__S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R4_B__S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R4__L_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R4__L_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R4__R_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R4__R_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R4_TL_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R4_TL_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R4_BR_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R4_BR_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R4_BL_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R4_BL_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R4_TR_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R4_TR_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B4_BR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B4_BR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B4__R___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B4__R___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B4_TR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B4_TR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B4_T____".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B4_T____ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B4_TL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B4_TL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B4__L___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B4__L___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B4_BL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B4_BL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D4_BR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D4_BR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D4__R___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D4__R___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D4_TR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D4_TR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D4_TL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D4_TL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D4__L___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D4__L___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D4_BL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D4_BL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D4_B____".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D4_B____ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A5_T__B_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A5_T__B_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A5__L__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A5__L__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A5__R__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A5__R__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A5_TL_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A5_TL_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A5_BR_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A5_BR_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A5_BL_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A5_BL_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A5_TR_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A5_TR_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_BR__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_BR__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_BR_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_BR_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_BR__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_BR__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_BR_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_BR_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5__R_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5__R_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5__R_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5__R_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5__R__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5__R__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5__R_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5__R_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_TR_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_TR_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_TR_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_TR_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_TR__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_TR__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_TR_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_TR_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_T__BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_T__BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_T___R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_T___R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_T__TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_T__TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_T__TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_T__TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_T___L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_T___L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_T__BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_T__BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_TL_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_TL_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_TL_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_TL_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5__L_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5__L_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5__L__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5__L__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5__L_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5__L_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_BL_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_BL_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_BL__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_BL__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_BL_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_BL_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_BL__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_BL__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_BR_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_BR_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_BR_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_BR_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5__R_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5__R_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5__R_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5__R_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_TR__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_TR__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_TR_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_TR_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_TL__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_TL__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_TL_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_TL_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_TL_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_TL_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_TL__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_TL__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5__L_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5__L_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5__L_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5__L_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5__L_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5__L_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_BL_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_BL_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T5_BL_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T5_BL_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S5_S1_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S5_S1_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S5_S1__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S5_S1__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S5_S1__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S5_S1__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S5_S1_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S5_S1_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S5_S1_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S5_S1_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S5_S1_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S5_S1_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S5_S1_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S5_S1_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R5_B__S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R5_B__S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R5__L_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R5__L_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R5__R_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R5__R_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R5_TL_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R5_TL_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R5_BR_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R5_BR_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R5_BL_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R5_BL_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R5_TR_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R5_TR_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B5_BR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B5_BR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B5__R___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B5__R___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B5_TR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B5_TR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B5_T____".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B5_T____ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B5_TL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B5_TL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B5__L___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B5__L___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B5_BL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B5_BL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D5_BR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D5_BR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D5__R___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D5__R___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D5_TR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D5_TR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D5_TL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D5_TL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D5__L___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D5__L___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D5_BL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D5_BL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D5_B____".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D5_B____ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A6_T__B_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A6_T__B_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A6__L__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A6__L__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A6__R__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A6__R__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A6_TL_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A6_TL_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A6_BR_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A6_BR_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A6_BL_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A6_BL_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A6_TR_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A6_TR_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_BR__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_BR__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_BR_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_BR_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_BR__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_BR__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_BR_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_BR_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6__R_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6__R_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6__R_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6__R_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6__R__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6__R__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6__R_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6__R_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_TR_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_TR_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_TR_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_TR_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_TR__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_TR__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_TR_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_TR_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_T__BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_T__BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_T___R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_T___R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_T__TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_T__TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_T__TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_T__TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_T___L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_T___L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_T__BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_T__BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_TL_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_TL_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_TL_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_TL_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6__L_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6__L_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6__L__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6__L__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6__L_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6__L_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_BL_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_BL_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_BL__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_BL__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_BL_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_BL_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_BL__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_BL__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_BR_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_BR_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_BR_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_BR_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6__R_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6__R_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6__R_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6__R_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_TR__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_TR__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_TR_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_TR_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_TL__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_TL__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_TL_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_TL_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_TL_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_TL_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_TL__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_TL__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6__L_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6__L_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6__L_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6__L_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6__L_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6__L_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_BL_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_BL_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T6_BL_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T6_BL_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S6_S6_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S6_S6_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S6_S6__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S6_S6__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S6_S6__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S6_S6__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S6_S6_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S6_S6_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S6_S6_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S6_S6_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S6_S6_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S6_S6_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S6_S6_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S6_S6_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R6_B__S6".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R6_B__S6 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R6__L_S6".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R6__L_S6 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R6__R_S6".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R6__R_S6 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R6_TL_S6".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R6_TL_S6 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R6_BR_S6".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R6_BR_S6 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R6_BL_S6".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R6_BL_S6 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R6_TR_S6".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R6_TR_S6 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B6_BR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B6_BR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B6__R___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B6__R___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B6_TR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B6_TR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B6_T____".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B6_T____ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B6_TL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B6_TL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B6__L___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B6__L___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B6_BL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B6_BL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D6_BR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D6_BR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D6__R___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D6__R___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D6_TR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D6_TR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D6_TL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D6_TL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D6__L___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D6__L___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D6_BL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D6_BL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D6_B____".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D6_B____ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A7_T__B_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A7_T__B_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A7__L__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A7__L__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A7__R__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A7__R__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A7_TL_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A7_TL_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A7_BR_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A7_BR_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A7_BL_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A7_BL_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A7_TR_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A7_TR_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_BR__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_BR__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_BR_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_BR_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_BR__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_BR__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_BR_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_BR_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7__R_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7__R_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7__R_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7__R_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7__R__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7__R__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7__R_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7__R_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_TR_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_TR_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_TR_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_TR_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_TR__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_TR__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_TR_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_TR_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_T__BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_T__BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_T___R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_T___R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_T__TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_T__TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_T__TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_T__TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_T___L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_T___L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_T__BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_T__BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_TL_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_TL_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_TL_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_TL_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7__L_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7__L_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7__L__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7__L__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7__L_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7__L_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_BL_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_BL_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_BL__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_BL__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_BL_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_BL_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_BL__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_BL__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_BR_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_BR_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_BR_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_BR_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7__R_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7__R_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7__R_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7__R_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_TR__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_TR__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_TR_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_TR_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_TL__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_TL__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_TL_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_TL_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_TL_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_TL_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_TL__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_TL__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7__L_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7__L_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7__L_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7__L_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7__L_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7__L_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_BL_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_BL_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_T7_BL_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_T7_BL_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S7_S7_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S7_S7_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S7_S7__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S7_S7__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S7_S7__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S7_S7__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S7_S7_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S7_S7_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S7_S7_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S7_S7_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S7_S7_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S7_S7_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S7_S7_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S7_S7_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R7_B__S7".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R7_B__S7 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R7__L_S7".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R7__L_S7 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R7__R_S7".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R7__R_S7 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R7_TL_S7".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R7_TL_S7 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R7_BR_S7".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R7_BR_S7 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R7_BL_S7".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R7_BL_S7 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_R7_TR_S7".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_R7_TR_S7 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B7_BR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B7_BR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B7__R___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B7__R___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B7_TR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B7_TR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B7_T____".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B7_T____ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B7_TL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B7_TL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B7__L___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B7__L___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_B7_BL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_B7_BL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D7_BR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D7_BR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D7__R___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D7__R___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D7_TR___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D7_TR___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D7_TL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D7_TL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D7__L___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D7__L___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D7_BL___".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D7_BL___ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_D7_B____".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_D7_B____ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_P1_S1_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_P1_S1_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_P1_S1_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_P1_S1_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_P1_S1_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_P1_S1_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_P1_S1_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_P1_S1_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_P1_S1_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_P1_S1_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_K1_S1_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_K1_S1_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_K1_S1_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_K1_S1_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_K1_S1_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_K1_S1_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_K1_S1_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_K1_S1_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_K1_S1_B_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_K1_S1_B_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_K1_S1_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_K1_S1_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_V1_BR_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_V1_BR_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_V1__R_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_V1__R_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_V1_TR_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_V1_TR_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_V1_T__S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_V1_T__S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_V1_TL_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_V1_TL_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_V1__L_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_V1__L_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_V1_BL_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_V1_BL_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_V1_B__S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_V1_B__S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_H1_S1_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_H1_S1_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_H1_S1_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_H1_S1_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_H1_S1_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_H1_S1_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_H1_S1_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_H1_S1_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_H1_S1_B_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_H1_S1_B_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_H1_S1_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_H1_S1_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_P6_S6_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_P6_S6_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_P6_S6_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_P6_S6_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_P6_S6_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_P6_S6_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_P6_S6_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_P6_S6_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_P6_S6_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_P6_S6_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_K6_S6_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_K6_S6_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_K6_S6_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_K6_S6_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_K6_S6_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_K6_S6_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_K6_S6_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_K6_S6_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_K6_S6_B_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_K6_S6_B_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_K6_S6_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_K6_S6_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_V6_BR_S6".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_V6_BR_S6 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_V6__R_S6".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_V6__R_S6 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_V6_TR_S6".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_V6_TR_S6 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_V6_T__S6".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_V6_T__S6 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_V6_TL_S6".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_V6_TL_S6 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_V6__L_S6".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_V6__L_S6 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_V6_BL_S6".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_V6_BL_S6 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_V6_B__S6".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_V6_B__S6 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_H6_S6_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_H6_S6_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_H6_S6_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_H6_S6_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_H6_S6_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_H6_S6_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_H6_S6_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_H6_S6_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_H6_S6_B_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_H6_S6_B_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_H6_S6_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_H6_S6_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_P7_S7_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_P7_S7_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_P7_S7_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_P7_S7_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_P7_S7_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_P7_S7_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_P7_S7_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_P7_S7_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_P7_S7_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_P7_S7_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_K7_S7_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_K7_S7_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_K7_S7_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_K7_S7_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_K7_S7_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_K7_S7_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_K7_S7_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_K7_S7_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_K7_S7_B_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_K7_S7_B_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_K7_S7_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_K7_S7_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_V7_BR_S7".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_V7_BR_S7 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_V7__R_S7".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_V7__R_S7 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_V7_TR_S7".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_V7_TR_S7 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_V7_T__S7".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_V7_T__S7 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_V7_TL_S7".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_V7_TL_S7 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_V7__L_S7".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_V7__L_S7 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_V7_BL_S7".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_V7_BL_S7 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_V7_B__S7".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_V7_B__S7 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_H7_S7_T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_H7_S7_T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_H7_S7_TR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_H7_S7_TR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_H7_S7_TL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_H7_S7_TL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_H7_S7_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_H7_S7_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_H7_S7_B_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_H7_S7_B_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_H7_S7_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_H7_S7_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_DL_S_B_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_DL_S_B_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_DL_S_B_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_DL_S_B_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_DL_S_L_1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_DL_S_L_1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_DL_S_SB_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_DL_S_SB_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_DL_S_SB_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_DL_S_SB_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_DL_T_B_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_DL_T_B_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_DL_T_B_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_DL_T_B_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_DL_T_L_1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_DL_T_L_1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_DL_T_SB_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_DL_T_SB_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_DL_T_SB_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_DL_T_SB_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_ST_S_B_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_ST_S_B_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_ST_S_B_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_ST_S_B_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_ST_S_L_1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_ST_S_L_1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_ST_S_SB_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_ST_S_SB_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_ST_S_SB_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_ST_S_SB_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_ST_T_B_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_ST_T_B_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_ST_T_B_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_ST_T_B_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_ST_T_L_1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_ST_T_L_1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_ST_T_SB_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_ST_T_SB_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_ST_T_SB_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_ST_T_SB_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_S_S_B_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_S_S_B_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_S_S_B_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_S_S_B_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_S_S_L_1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_S_S_L_1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_S_S_SB_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_S_S_SB_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_S_S_SB_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_S_S_SB_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_S_T_B_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_S_T_B_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_S_T_B_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_S_T_B_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_S_T_L_1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_S_T_L_1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_S_T_SB_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_S_T_SB_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_S_T_SB_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_S_T_SB_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_DL_S_B_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_DL_S_B_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_DL_S_B_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_DL_S_B_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_DL_S_L_1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_DL_S_L_1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_DL_S_SB_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_DL_S_SB_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_DL_S_SB_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_DL_S_SB_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_DL_T_B_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_DL_T_B_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_DL_T_B_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_DL_T_B_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_DL_T_L_1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_DL_T_L_1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_DL_T_SB_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_DL_T_SB_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_DL_T_SB_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_DL_T_SB_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_ST_S_B_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_ST_S_B_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_ST_S_B_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_ST_S_B_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_ST_S_L_1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_ST_S_L_1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_ST_S_SB_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_ST_S_SB_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_ST_S_SB_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_ST_S_SB_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_ST_T_B_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_ST_T_B_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_ST_T_B_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_ST_T_B_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_ST_T_L_1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_ST_T_L_1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_ST_T_SB_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_ST_T_SB_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_ST_T_SB_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_ST_T_SB_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_S_S_B_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_S_S_B_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_S_S_B_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_S_S_B_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_S_S_L_1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_S_S_L_1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_S_S_SB_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_S_S_SB_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_S_S_SB_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_S_S_SB_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_S_T_B_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_S_T_B_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_S_T_B_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_S_T_B_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_S_T_L_1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_S_T_L_1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_S_T_SB_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_S_T_SB_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_S_T_SB_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_S_T_SB_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_DL_S_B_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_DL_S_B_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_DL_S_B_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_DL_S_B_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_DL_S_L_1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_DL_S_L_1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_DL_S_SB_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_DL_S_SB_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_DL_S_SB_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_DL_S_SB_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_DL_T_B_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_DL_T_B_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_DL_T_B_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_DL_T_B_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_DL_T_L_1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_DL_T_L_1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_DL_T_SB_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_DL_T_SB_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_DL_T_SB_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_DL_T_SB_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_ST_S_B_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_ST_S_B_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_ST_S_B_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_ST_S_B_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_ST_S_L_1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_ST_S_L_1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_ST_S_SB_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_ST_S_SB_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_ST_S_SB_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_ST_S_SB_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_ST_T_B_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_ST_T_B_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_ST_T_B_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_ST_T_B_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_ST_T_L_1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_ST_T_L_1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_ST_T_SB_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_ST_T_SB_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_ST_T_SB_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_ST_T_SB_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_S_S_B_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_S_S_B_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_S_S_B_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_S_S_B_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_S_S_L_1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_S_S_L_1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_S_S_SB_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_S_S_SB_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_S_S_SB_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_S_S_SB_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_S_T_B_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_S_T_B_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_S_T_B_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_S_T_B_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_S_T_L_1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_S_T_L_1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_S_T_SB_1_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_S_T_SB_1_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_S_T_SB_1_W".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_S_T_SB_1_W as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_S_S_L_2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_S_S_L_2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_S_S_T_L_2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_S_S_T_L_2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_DL_S_L_2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_DL_S_L_2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_DL_DL_T_L_2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_DL_DL_T_L_2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_ST_S_L_2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_ST_S_L_2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LK_ST_ST_T_L_2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LK_ST_ST_T_L_2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_BF2RETURN".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_BF2RETURN as c_int,
    },
    stringID_table_t {
        name: c"BOTH_BF2BREAK".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_BF2BREAK as c_int,
    },
    stringID_table_t {
        name: c"BOTH_BF2LOCK".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_BF2LOCK as c_int,
    },
    stringID_table_t {
        name: c"BOTH_BF1RETURN".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_BF1RETURN as c_int,
    },
    stringID_table_t {
        name: c"BOTH_BF1BREAK".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_BF1BREAK as c_int,
    },
    stringID_table_t {
        name: c"BOTH_BF1LOCK".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_BF1LOCK as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CWCIRCLE_R2__R_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CWCIRCLE_R2__R_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CCWCIRCLE_R2__L_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CCWCIRCLE_R2__L_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CWCIRCLE_A2__L__R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CWCIRCLE_A2__L__R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CCWCIRCLE_A2__R__L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CCWCIRCLE_A2__R__L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CWCIRCLEBREAK".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CWCIRCLEBREAK as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CCWCIRCLEBREAK".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CCWCIRCLEBREAK as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CWCIRCLELOCK".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CWCIRCLELOCK as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CCWCIRCLELOCK".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CCWCIRCLELOCK as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SABERFAST_STANCE".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SABERFAST_STANCE as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SABERSLOW_STANCE".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SABERSLOW_STANCE as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SABERDUAL_STANCE".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SABERDUAL_STANCE as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SABERSTAFF_STANCE".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SABERSTAFF_STANCE as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A2_STABBACK1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A2_STABBACK1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ATTACK_BACK".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ATTACK_BACK as c_int,
    },
    stringID_table_t {
        name: c"BOTH_JUMPFLIPSLASHDOWN1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_JUMPFLIPSLASHDOWN1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_JUMPFLIPSTABDOWN".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_JUMPFLIPSTABDOWN as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCELEAP2_T__B_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCELEAP2_T__B_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LUNGE2_B__T_".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LUNGE2_B__T_ as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CROUCHATTACKBACK1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CROUCHATTACKBACK1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_JUMPATTACK6".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_JUMPATTACK6 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_JUMPATTACK7".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_JUMPATTACK7 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SPINATTACK6".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SPINATTACK6 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SPINATTACK7".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SPINATTACK7 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S1_S6".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S1_S6 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S6_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S6_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S1_S7".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S1_S7 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_S7_S1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_S7_S1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCELONGLEAP_START".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCELONGLEAP_START as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCELONGLEAP_ATTACK".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCELONGLEAP_ATTACK as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCELONGLEAP_LAND".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCELONGLEAP_LAND as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEWALLRUNFLIP_START".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEWALLRUNFLIP_START as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEWALLRUNFLIP_END".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEWALLRUNFLIP_END as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEWALLRUNFLIP_ALT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEWALLRUNFLIP_ALT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEWALLREBOUND_FORWARD".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEWALLREBOUND_FORWARD as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEWALLREBOUND_LEFT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEWALLREBOUND_LEFT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEWALLREBOUND_BACK".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEWALLREBOUND_BACK as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEWALLREBOUND_RIGHT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEWALLREBOUND_RIGHT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEWALLHOLD_FORWARD".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEWALLHOLD_FORWARD as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEWALLHOLD_LEFT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEWALLHOLD_LEFT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEWALLHOLD_BACK".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEWALLHOLD_BACK as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEWALLHOLD_RIGHT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEWALLHOLD_RIGHT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEWALLRELEASE_FORWARD".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEWALLRELEASE_FORWARD as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEWALLRELEASE_LEFT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEWALLRELEASE_LEFT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEWALLRELEASE_BACK".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEWALLRELEASE_BACK as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEWALLRELEASE_RIGHT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEWALLRELEASE_RIGHT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A7_KICK_F".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A7_KICK_F as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A7_KICK_B".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A7_KICK_B as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A7_KICK_R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A7_KICK_R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A7_KICK_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A7_KICK_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A7_KICK_S".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A7_KICK_S as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A7_KICK_BF".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A7_KICK_BF as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A7_KICK_BF_STOP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A7_KICK_BF_STOP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A7_KICK_RL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A7_KICK_RL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A7_KICK_F_AIR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A7_KICK_F_AIR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A7_KICK_B_AIR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A7_KICK_B_AIR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A7_KICK_R_AIR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A7_KICK_R_AIR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A7_KICK_L_AIR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A7_KICK_L_AIR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FLIP_ATTACK7".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FLIP_ATTACK7 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FLIP_HOLD7".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FLIP_HOLD7 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FLIP_LAND".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FLIP_LAND as c_int,
    },
    stringID_table_t {
        name: c"BOTH_PULL_IMPALE_STAB".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_PULL_IMPALE_STAB as c_int,
    },
    stringID_table_t {
        name: c"BOTH_PULL_IMPALE_SWING".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_PULL_IMPALE_SWING as c_int,
    },
    stringID_table_t {
        name: c"BOTH_PULLED_INAIR_B".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_PULLED_INAIR_B as c_int,
    },
    stringID_table_t {
        name: c"BOTH_PULLED_INAIR_F".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_PULLED_INAIR_F as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STABDOWN".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STABDOWN as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STABDOWN_STAFF".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STABDOWN_STAFF as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STABDOWN_DUAL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STABDOWN_DUAL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A6_SABERPROTECT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A6_SABERPROTECT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A7_SOULCAL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A7_SOULCAL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A1_SPECIAL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A1_SPECIAL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A2_SPECIAL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A2_SPECIAL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A3_SPECIAL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A3_SPECIAL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ROLL_STAB".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ROLL_STAB as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND1IDLE1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND1IDLE1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND2IDLE1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND2IDLE1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND2IDLE2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND2IDLE2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND3".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND3 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND3IDLE1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND3IDLE1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND4".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND4 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND5".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND5 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND5IDLE1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND5IDLE1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND6".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND6 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND8".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND8 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND1TO2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND1TO2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND2TO1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND2TO1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND2TO4".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND2TO4 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND4TO2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND4TO2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND4TOATTACK2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND4TOATTACK2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STANDUP2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STANDUP2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND5TOSIT3".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND5TOSIT3 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND1TOSTAND5".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND1TOSTAND5 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND5TOSTAND1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND5TOSTAND1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND5TOAIM".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND5TOAIM as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND5STARTLEDLOOKLEFT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND5STARTLEDLOOKLEFT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STARTLEDLOOKLEFTTOSTAND5".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STARTLEDLOOKLEFTTOSTAND5 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND5TOSTAND8".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND5TOSTAND8 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND7TOSTAND8".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND7TOSTAND8 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND8TOSTAND5".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND8TOSTAND5 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND9".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND9 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND9IDLE1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND9IDLE1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND5SHIFTWEIGHT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND5SHIFTWEIGHT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND5SHIFTWEIGHTSTART".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND5SHIFTWEIGHTSTART as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND5SHIFTWEIGHTSTOP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND5SHIFTWEIGHTSTOP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND5TURNLEFTSTART".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND5TURNLEFTSTART as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND5TURNLEFTSTOP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND5TURNLEFTSTOP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND5TURNRIGHTSTART".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND5TURNRIGHTSTART as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND5TURNRIGHTSTOP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND5TURNRIGHTSTOP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND5LOOK180LEFTSTART".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND5LOOK180LEFTSTART as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND5LOOK180LEFTSTOP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND5LOOK180LEFTSTOP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CONSOLE1START".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CONSOLE1START as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CONSOLE1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CONSOLE1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CONSOLE1STOP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CONSOLE1STOP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CONSOLE2START".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CONSOLE2START as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CONSOLE2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CONSOLE2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CONSOLE2STOP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CONSOLE2STOP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CONSOLE2HOLDCOMSTART".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CONSOLE2HOLDCOMSTART as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CONSOLE2HOLDCOMSTOP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CONSOLE2HOLDCOMSTOP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_GUARD_LOOKAROUND1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_GUARD_LOOKAROUND1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_GUARD_IDLE1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_GUARD_IDLE1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_GESTURE1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_GESTURE1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_GESTURE2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_GESTURE2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_WALK1TALKCOMM1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_WALK1TALKCOMM1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_TALK1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_TALK1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_TALK2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_TALK2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_TALKCOMM1START".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_TALKCOMM1START as c_int,
    },
    stringID_table_t {
        name: c"BOTH_TALKCOMM1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_TALKCOMM1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_TALKCOMM1STOP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_TALKCOMM1STOP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_TALKGESTURE1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_TALKGESTURE1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_HEADTILTLSTART".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_HEADTILTLSTART as c_int,
    },
    stringID_table_t {
        name: c"BOTH_HEADTILTLSTOP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_HEADTILTLSTOP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_HEADTILTRSTART".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_HEADTILTRSTART as c_int,
    },
    stringID_table_t {
        name: c"BOTH_HEADTILTRSTOP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_HEADTILTRSTOP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_HEADNOD".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_HEADNOD as c_int,
    },
    stringID_table_t {
        name: c"BOTH_HEADSHAKE".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_HEADSHAKE as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SIT2HEADTILTLSTART".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SIT2HEADTILTLSTART as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SIT2HEADTILTLSTOP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SIT2HEADTILTLSTOP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_REACH1START".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_REACH1START as c_int,
    },
    stringID_table_t {
        name: c"BOTH_REACH1STOP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_REACH1STOP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_COME_ON1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_COME_ON1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STEADYSELF1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STEADYSELF1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STEADYSELF1END".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STEADYSELF1END as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SILENCEGESTURE1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SILENCEGESTURE1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_REACHFORSABER1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_REACHFORSABER1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SABERKILLER1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SABERKILLER1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SABERKILLEE1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SABERKILLEE1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_HUGGER1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_HUGGER1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_HUGGERSTOP1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_HUGGERSTOP1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_HUGGEE1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_HUGGEE1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_HUGGEESTOP1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_HUGGEESTOP1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SABERTHROW1START".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SABERTHROW1START as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SABERTHROW1STOP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SABERTHROW1STOP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SABERTHROW2START".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SABERTHROW2START as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SABERTHROW2STOP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SABERTHROW2STOP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SIT1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SIT1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SIT2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SIT2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SIT3".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SIT3 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SIT2TOSTAND5".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SIT2TOSTAND5 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND5TOSIT2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND5TOSIT2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SIT2TOSIT4".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SIT2TOSIT4 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SIT3TOSTAND5".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SIT3TOSTAND5 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CROUCH1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CROUCH1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CROUCH1IDLE".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CROUCH1IDLE as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CROUCH1WALK".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CROUCH1WALK as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CROUCH1WALKBACK".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CROUCH1WALKBACK as c_int,
    },
    stringID_table_t {
        name: c"BOTH_UNCROUCH1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_UNCROUCH1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CROUCH2TOSTAND1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CROUCH2TOSTAND1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CROUCH3".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CROUCH3 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_UNCROUCH3".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_UNCROUCH3 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CROUCH4".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CROUCH4 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_UNCROUCH4".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_UNCROUCH4 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_GUNSIT1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_GUNSIT1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_MOUNT_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_MOUNT_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_DISMOUNT_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_DISMOUNT_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_MOUNT_R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_MOUNT_R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_DISMOUNT_R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_DISMOUNT_R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_MOUNTJUMP_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_MOUNTJUMP_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_MOUNTTHROW".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_MOUNTTHROW as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_MOUNTTHROW_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_MOUNTTHROW_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_MOUNTTHROW_R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_MOUNTTHROW_R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_MOUNTTHROWEE".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_MOUNTTHROWEE as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_LOOKLEFT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_LOOKLEFT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_LOOKRIGHT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_LOOKRIGHT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_TURBO".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_TURBO as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_REV".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_REV as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_AIR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_AIR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_AIR_G".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_AIR_G as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_AIR_SL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_AIR_SL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_AIR_SR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_AIR_SR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_LAND".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_LAND as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_LAND_G".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_LAND_G as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_LAND_SL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_LAND_SL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_LAND_SR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_LAND_SR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_IDLE".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_IDLE as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_IDLE_G".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_IDLE_G as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_IDLE_SL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_IDLE_SL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_IDLE_SR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_IDLE_SR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_LEANL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_LEANL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_LEANL_G".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_LEANL_G as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_LEANL_SL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_LEANL_SL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_LEANL_SR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_LEANL_SR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_LEANR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_LEANR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_LEANR_G".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_LEANR_G as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_LEANR_SL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_LEANR_SL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_LEANR_SR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_LEANR_SR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_ATL_S".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_ATL_S as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_ATR_S".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_ATR_S as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_ATR_TO_L_S".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_ATR_TO_L_S as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_ATL_TO_R_S".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_ATL_TO_R_S as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_ATR_G".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_ATR_G as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_ATL_G".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_ATL_G as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_ATF_G".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_ATF_G as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VS_PAIN1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VS_PAIN1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_MOUNT_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_MOUNT_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_MOUNT_R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_MOUNT_R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_MOUNT_B".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_MOUNT_B as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_DISMOUNT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_DISMOUNT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_DISMOUNT_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_DISMOUNT_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_DISMOUNT_R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_DISMOUNT_R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_WALK_FWD".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_WALK_FWD as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_WALK_REV".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_WALK_REV as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_WALK_FWD_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_WALK_FWD_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_WALK_FWD_R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_WALK_FWD_R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_RUN_FWD".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_RUN_FWD as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_RUN_REV".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_RUN_REV as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_RUN_FWD_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_RUN_FWD_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_RUN_FWD_R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_RUN_FWD_R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_SLIDEF".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_SLIDEF as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_AIR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_AIR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_ATB".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_ATB as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_PAIN1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_PAIN1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_DEATH1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_DEATH1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_STAND".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_STAND as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_BUCK".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_BUCK as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_LAND".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_LAND as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_TURBO".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_TURBO as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_IDLE_SL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_IDLE_SL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_IDLE_SR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_IDLE_SR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_IDLE".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_IDLE as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_IDLE1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_IDLE1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_IDLE_S".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_IDLE_S as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_IDLE_G".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_IDLE_G as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_IDLE_T".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_IDLE_T as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_ATL_S".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_ATL_S as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_ATR_S".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_ATR_S as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_ATR_TO_L_S".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_ATR_TO_L_S as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_ATL_TO_R_S".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_ATL_TO_R_S as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_ATR_G".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_ATR_G as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_ATL_G".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_ATL_G as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VT_ATF_G".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VT_ATF_G as c_int,
    },
    stringID_table_t {
        name: c"BOTH_GEARS_OPEN".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_GEARS_OPEN as c_int,
    },
    stringID_table_t {
        name: c"BOTH_GEARS_CLOSE".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_GEARS_CLOSE as c_int,
    },
    stringID_table_t {
        name: c"BOTH_WINGS_OPEN".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_WINGS_OPEN as c_int,
    },
    stringID_table_t {
        name: c"BOTH_WINGS_CLOSE".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_WINGS_CLOSE as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH14_UNGRIP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH14_UNGRIP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEATH14_SITUP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEATH14_SITUP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_KNEES1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_KNEES1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_KNEES2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_KNEES2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_KNEES2TO1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_KNEES2TO1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_WALK1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_WALK1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_WALK2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_WALK2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_WALK_STAFF".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_WALK_STAFF as c_int,
    },
    stringID_table_t {
        name: c"BOTH_WALKBACK_STAFF".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_WALKBACK_STAFF as c_int,
    },
    stringID_table_t {
        name: c"BOTH_WALK_DUAL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_WALK_DUAL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_WALKBACK_DUAL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_WALKBACK_DUAL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_WALK5".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_WALK5 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_WALK6".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_WALK6 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_WALK7".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_WALK7 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_RUN1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_RUN1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_RUN1START".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_RUN1START as c_int,
    },
    stringID_table_t {
        name: c"BOTH_RUN1STOP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_RUN1STOP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_RUN2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_RUN2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_RUN1TORUN2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_RUN1TORUN2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_RUN2TORUN1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_RUN2TORUN1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_RUN4".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_RUN4 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_RUN_STAFF".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_RUN_STAFF as c_int,
    },
    stringID_table_t {
        name: c"BOTH_RUNBACK_STAFF".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_RUNBACK_STAFF as c_int,
    },
    stringID_table_t {
        name: c"BOTH_RUN_DUAL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_RUN_DUAL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_RUNBACK_DUAL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_RUNBACK_DUAL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STRAFE_LEFT1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STRAFE_LEFT1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STRAFE_RIGHT1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STRAFE_RIGHT1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_RUNSTRAFE_LEFT1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_RUNSTRAFE_LEFT1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_RUNSTRAFE_RIGHT1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_RUNSTRAFE_RIGHT1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_TURN_LEFT1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_TURN_LEFT1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_TURN_RIGHT1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_TURN_RIGHT1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_TURNSTAND1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_TURNSTAND1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_TURNSTAND2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_TURNSTAND2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_TURNSTAND3".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_TURNSTAND3 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_TURNSTAND4".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_TURNSTAND4 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_TURNSTAND5".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_TURNSTAND5 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_TURNCROUCH1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_TURNCROUCH1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_WALKBACK1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_WALKBACK1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_WALKBACK2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_WALKBACK2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_RUNBACK1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_RUNBACK1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_RUNBACK2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_RUNBACK2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_JUMP1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_JUMP1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_INAIR1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_INAIR1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LAND1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LAND1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LAND2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LAND2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_JUMPBACK1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_JUMPBACK1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_INAIRBACK1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_INAIRBACK1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LANDBACK1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LANDBACK1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_JUMPLEFT1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_JUMPLEFT1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_INAIRLEFT1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_INAIRLEFT1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LANDLEFT1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LANDLEFT1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_JUMPRIGHT1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_JUMPRIGHT1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_INAIRRIGHT1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_INAIRRIGHT1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LANDRIGHT1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LANDRIGHT1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEJUMP1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEJUMP1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEINAIR1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEINAIR1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCELAND1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCELAND1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEJUMPBACK1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEJUMPBACK1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEINAIRBACK1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEINAIRBACK1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCELANDBACK1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCELANDBACK1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEJUMPLEFT1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEJUMPLEFT1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEINAIRLEFT1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEINAIRLEFT1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCELANDLEFT1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCELANDLEFT1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEJUMPRIGHT1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEJUMPRIGHT1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEINAIRRIGHT1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEINAIRRIGHT1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCELANDRIGHT1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCELANDRIGHT1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FLIP_F".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FLIP_F as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FLIP_B".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FLIP_B as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FLIP_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FLIP_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FLIP_R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FLIP_R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ROLL_F".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ROLL_F as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ROLL_B".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ROLL_B as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ROLL_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ROLL_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ROLL_R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ROLL_R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_HOP_F".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_HOP_F as c_int,
    },
    stringID_table_t {
        name: c"BOTH_HOP_B".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_HOP_B as c_int,
    },
    stringID_table_t {
        name: c"BOTH_HOP_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_HOP_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_HOP_R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_HOP_R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DODGE_FL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DODGE_FL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DODGE_FR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DODGE_FR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DODGE_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DODGE_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DODGE_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DODGE_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DODGE_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DODGE_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DODGE_R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DODGE_R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DODGE_HOLD_FL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DODGE_HOLD_FL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DODGE_HOLD_FR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DODGE_HOLD_FR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DODGE_HOLD_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DODGE_HOLD_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DODGE_HOLD_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DODGE_HOLD_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DODGE_HOLD_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DODGE_HOLD_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DODGE_HOLD_R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DODGE_HOLD_R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ENGAGETAUNT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ENGAGETAUNT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_BOW".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_BOW as c_int,
    },
    stringID_table_t {
        name: c"BOTH_MEDITATE".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_MEDITATE as c_int,
    },
    stringID_table_t {
        name: c"BOTH_MEDITATE_END".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_MEDITATE_END as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SHOWOFF_FAST".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SHOWOFF_FAST as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SHOWOFF_MEDIUM".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SHOWOFF_MEDIUM as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SHOWOFF_STRONG".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SHOWOFF_STRONG as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SHOWOFF_DUAL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SHOWOFF_DUAL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SHOWOFF_STAFF".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SHOWOFF_STAFF as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VICTORY_FAST".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VICTORY_FAST as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VICTORY_MEDIUM".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VICTORY_MEDIUM as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VICTORY_STRONG".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VICTORY_STRONG as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VICTORY_DUAL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VICTORY_DUAL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_VICTORY_STAFF".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_VICTORY_STAFF as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ARIAL_LEFT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ARIAL_LEFT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ARIAL_RIGHT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ARIAL_RIGHT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CARTWHEEL_LEFT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CARTWHEEL_LEFT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CARTWHEEL_RIGHT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CARTWHEEL_RIGHT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FLIP_LEFT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FLIP_LEFT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FLIP_BACK1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FLIP_BACK1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FLIP_BACK2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FLIP_BACK2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FLIP_BACK3".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FLIP_BACK3 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_BUTTERFLY_LEFT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_BUTTERFLY_LEFT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_BUTTERFLY_RIGHT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_BUTTERFLY_RIGHT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_WALL_RUN_RIGHT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_WALL_RUN_RIGHT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_WALL_RUN_RIGHT_FLIP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_WALL_RUN_RIGHT_FLIP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_WALL_RUN_RIGHT_STOP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_WALL_RUN_RIGHT_STOP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_WALL_RUN_LEFT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_WALL_RUN_LEFT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_WALL_RUN_LEFT_FLIP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_WALL_RUN_LEFT_FLIP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_WALL_RUN_LEFT_STOP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_WALL_RUN_LEFT_STOP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_WALL_FLIP_RIGHT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_WALL_FLIP_RIGHT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_WALL_FLIP_LEFT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_WALL_FLIP_LEFT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_KNOCKDOWN1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_KNOCKDOWN1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_KNOCKDOWN2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_KNOCKDOWN2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_KNOCKDOWN3".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_KNOCKDOWN3 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_KNOCKDOWN4".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_KNOCKDOWN4 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_KNOCKDOWN5".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_KNOCKDOWN5 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_GETUP1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_GETUP1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_GETUP2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_GETUP2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_GETUP3".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_GETUP3 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_GETUP4".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_GETUP4 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_GETUP5".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_GETUP5 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_GETUP_CROUCH_F1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_GETUP_CROUCH_F1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_GETUP_CROUCH_B1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_GETUP_CROUCH_B1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCE_GETUP_F1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCE_GETUP_F1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCE_GETUP_F2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCE_GETUP_F2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCE_GETUP_B1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCE_GETUP_B1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCE_GETUP_B2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCE_GETUP_B2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCE_GETUP_B3".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCE_GETUP_B3 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCE_GETUP_B4".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCE_GETUP_B4 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCE_GETUP_B5".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCE_GETUP_B5 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCE_GETUP_B6".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCE_GETUP_B6 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_GETUP_BROLL_B".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_GETUP_BROLL_B as c_int,
    },
    stringID_table_t {
        name: c"BOTH_GETUP_BROLL_F".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_GETUP_BROLL_F as c_int,
    },
    stringID_table_t {
        name: c"BOTH_GETUP_BROLL_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_GETUP_BROLL_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_GETUP_BROLL_R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_GETUP_BROLL_R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_GETUP_FROLL_B".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_GETUP_FROLL_B as c_int,
    },
    stringID_table_t {
        name: c"BOTH_GETUP_FROLL_F".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_GETUP_FROLL_F as c_int,
    },
    stringID_table_t {
        name: c"BOTH_GETUP_FROLL_L".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_GETUP_FROLL_L as c_int,
    },
    stringID_table_t {
        name: c"BOTH_GETUP_FROLL_R".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_GETUP_FROLL_R as c_int,
    },
    stringID_table_t {
        name: c"BOTH_WALL_FLIP_BACK1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_WALL_FLIP_BACK1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_WALL_FLIP_BACK2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_WALL_FLIP_BACK2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SPIN1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SPIN1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CEILING_CLING".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CEILING_CLING as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CEILING_DROP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CEILING_DROP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FJSS_TR_BL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FJSS_TR_BL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FJSS_TL_BR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FJSS_TL_BR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_RIGHTHANDCHOPPEDOFF".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_RIGHTHANDCHOPPEDOFF as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DEFLECTSLASH__R__L_FIN".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DEFLECTSLASH__R__L_FIN as c_int,
    },
    stringID_table_t {
        name: c"BOTH_BASHED1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_BASHED1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ARIAL_F1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ARIAL_F1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_BUTTERFLY_FR1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_BUTTERFLY_FR1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_BUTTERFLY_FL1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_BUTTERFLY_FL1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_BACK_FLIP_UP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_BACK_FLIP_UP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LOSE_SABER".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LOSE_SABER as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAFF_TAUNT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAFF_TAUNT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_DUAL_TAUNT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_DUAL_TAUNT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A6_FB".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A6_FB as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A6_LR".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A6_LR as c_int,
    },
    stringID_table_t {
        name: c"BOTH_A7_HILT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_A7_HILT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ALORA_SPIN".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ALORA_SPIN as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ALORA_FLIP_1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ALORA_FLIP_1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ALORA_FLIP_2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ALORA_FLIP_2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ALORA_FLIP_3".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ALORA_FLIP_3 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ALORA_FLIP_B".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ALORA_FLIP_B as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ALORA_SPIN_THROW".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ALORA_SPIN_THROW as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ALORA_SPIN_SLASH".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ALORA_SPIN_SLASH as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ALORA_TAUNT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ALORA_TAUNT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ROSH_PAIN".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ROSH_PAIN as c_int,
    },
    stringID_table_t {
        name: c"BOTH_ROSH_HEAL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_ROSH_HEAL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_TAVION_SCEPTERGROUND".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_TAVION_SCEPTERGROUND as c_int,
    },
    stringID_table_t {
        name: c"BOTH_TAVION_SWORDPOWER".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_TAVION_SWORDPOWER as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SCEPTER_START".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SCEPTER_START as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SCEPTER_HOLD".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SCEPTER_HOLD as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SCEPTER_STOP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SCEPTER_STOP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_KYLE_GRAB".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_KYLE_GRAB as c_int,
    },
    stringID_table_t {
        name: c"BOTH_KYLE_MISS".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_KYLE_MISS as c_int,
    },
    stringID_table_t {
        name: c"BOTH_KYLE_PA_1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_KYLE_PA_1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_PLAYER_PA_1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_PLAYER_PA_1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_KYLE_PA_2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_KYLE_PA_2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_PLAYER_PA_2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_PLAYER_PA_2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_PLAYER_PA_FLY".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_PLAYER_PA_FLY as c_int,
    },
    stringID_table_t {
        name: c"BOTH_KYLE_PA_3".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_KYLE_PA_3 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_PLAYER_PA_3".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_PLAYER_PA_3 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_PLAYER_PA_3_FLY".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_PLAYER_PA_3_FLY as c_int,
    },
    stringID_table_t {
        name: c"BOTH_BUCK_RIDER".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_BUCK_RIDER as c_int,
    },
    stringID_table_t {
        name: c"BOTH_HOLD_START".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_HOLD_START as c_int,
    },
    stringID_table_t {
        name: c"BOTH_HOLD_MISS".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_HOLD_MISS as c_int,
    },
    stringID_table_t {
        name: c"BOTH_HOLD_IDLE".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_HOLD_IDLE as c_int,
    },
    stringID_table_t {
        name: c"BOTH_HOLD_END".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_HOLD_END as c_int,
    },
    stringID_table_t {
        name: c"BOTH_HOLD_ATTACK".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_HOLD_ATTACK as c_int,
    },
    stringID_table_t {
        name: c"BOTH_HOLD_SNIFF".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_HOLD_SNIFF as c_int,
    },
    stringID_table_t {
        name: c"BOTH_HOLD_DROP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_HOLD_DROP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_GRABBED".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_GRABBED as c_int,
    },
    stringID_table_t {
        name: c"BOTH_RELEASED".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_RELEASED as c_int,
    },
    stringID_table_t {
        name: c"BOTH_HANG_IDLE".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_HANG_IDLE as c_int,
    },
    stringID_table_t {
        name: c"BOTH_HANG_ATTACK".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_HANG_ATTACK as c_int,
    },
    stringID_table_t {
        name: c"BOTH_HANG_PAIN".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_HANG_PAIN as c_int,
    },
    stringID_table_t {
        name: c"BOTH_HIT1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_HIT1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LADDER_UP1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LADDER_UP1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LADDER_DWN1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LADDER_DWN1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_LADDER_IDLE".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_LADDER_IDLE as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FLY_SHIELDED".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FLY_SHIELDED as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SWIM_IDLE1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SWIM_IDLE1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SWIMFORWARD".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SWIMFORWARD as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SWIMBACKWARD".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SWIMBACKWARD as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SLEEP1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SLEEP1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SLEEP6START".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SLEEP6START as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SLEEP6STOP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SLEEP6STOP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SLEEP1GETUP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SLEEP1GETUP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SLEEP1GETUP2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SLEEP1GETUP2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CHOKE1START".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CHOKE1START as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CHOKE1STARTHOLD".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CHOKE1STARTHOLD as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CHOKE1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CHOKE1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CHOKE2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CHOKE2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CHOKE3".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CHOKE3 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_POWERUP1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_POWERUP1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_TURNON".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_TURNON as c_int,
    },
    stringID_table_t {
        name: c"BOTH_TURNOFF".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_TURNOFF as c_int,
    },
    stringID_table_t {
        name: c"BOTH_BUTTON1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_BUTTON1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_BUTTON2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_BUTTON2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_BUTTON_HOLD".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_BUTTON_HOLD as c_int,
    },
    stringID_table_t {
        name: c"BOTH_BUTTON_RELEASE".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_BUTTON_RELEASE as c_int,
    },
    stringID_table_t {
        name: c"BOTH_RESISTPUSH".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_RESISTPUSH as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEPUSH".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEPUSH as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEPULL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEPULL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_MINDTRICK1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_MINDTRICK1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_MINDTRICK2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_MINDTRICK2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCELIGHTNING".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCELIGHTNING as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCELIGHTNING_START".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCELIGHTNING_START as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCELIGHTNING_HOLD".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCELIGHTNING_HOLD as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCELIGHTNING_RELEASE".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCELIGHTNING_RELEASE as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEHEAL_START".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEHEAL_START as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEHEAL_STOP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEHEAL_STOP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEHEAL_QUICK".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEHEAL_QUICK as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SABERPULL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SABERPULL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEGRIP1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEGRIP1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEGRIP3".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEGRIP3 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEGRIP3THROW".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEGRIP3THROW as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEGRIP_HOLD".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEGRIP_HOLD as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCEGRIP_RELEASE".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCEGRIP_RELEASE as c_int,
    },
    stringID_table_t {
        name: c"BOTH_TOSS1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_TOSS1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_TOSS2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_TOSS2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCE_RAGE".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCE_RAGE as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCE_2HANDEDLIGHTNING".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCE_2HANDEDLIGHTNING as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCE_2HANDEDLIGHTNING_START".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCE_2HANDEDLIGHTNING_START as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCE_2HANDEDLIGHTNING_HOLD".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCE_2HANDEDLIGHTNING_HOLD as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCE_2HANDEDLIGHTNING_RELEASE".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCE_2HANDEDLIGHTNING_RELEASE as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCE_DRAIN".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCE_DRAIN as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCE_DRAIN_START".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCE_DRAIN_START as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCE_DRAIN_HOLD".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCE_DRAIN_HOLD as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCE_DRAIN_RELEASE".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCE_DRAIN_RELEASE as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCE_DRAIN_GRAB_START".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCE_DRAIN_GRAB_START as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCE_DRAIN_GRAB_HOLD".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCE_DRAIN_GRAB_HOLD as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCE_DRAIN_GRAB_END".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCE_DRAIN_GRAB_END as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCE_DRAIN_GRABBED".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCE_DRAIN_GRABBED as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCE_ABSORB".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCE_ABSORB as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCE_ABSORB_START".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCE_ABSORB_START as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCE_ABSORB_END".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCE_ABSORB_END as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCE_PROTECT".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCE_PROTECT as c_int,
    },
    stringID_table_t {
        name: c"BOTH_FORCE_PROTECT_FAST".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_FORCE_PROTECT_FAST as c_int,
    },
    stringID_table_t {
        name: c"BOTH_WIND".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_WIND as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND_TO_KNEEL".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND_TO_KNEEL as c_int,
    },
    stringID_table_t {
        name: c"BOTH_KNEEL_TO_STAND".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_KNEEL_TO_STAND as c_int,
    },
    stringID_table_t {
        name: c"BOTH_TUSKENATTACK1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_TUSKENATTACK1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_TUSKENATTACK2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_TUSKENATTACK2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_TUSKENATTACK3".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_TUSKENATTACK3 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_TUSKENLUNGE1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_TUSKENLUNGE1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_TUSKENTAUNT1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_TUSKENTAUNT1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_COWER1_START".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_COWER1_START as c_int,
    },
    stringID_table_t {
        name: c"BOTH_COWER1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_COWER1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_COWER1_STOP".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_COWER1_STOP as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SONICPAIN_START".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SONICPAIN_START as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SONICPAIN_HOLD".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SONICPAIN_HOLD as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SONICPAIN_END".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SONICPAIN_END as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND10".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND10 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND10_TALK1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND10_TALK1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND10_TALK2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND10_TALK2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND10TOSTAND1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND10TOSTAND1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND1_TALK1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND1_TALK1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND1_TALK2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND1_TALK2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_STAND1_TALK3".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_STAND1_TALK3 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SIT4".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SIT4 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SIT5".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SIT5 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SIT5_TALK1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SIT5_TALK1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SIT5_TALK2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SIT5_TALK2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SIT5_TALK3".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SIT5_TALK3 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SIT6".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SIT6 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_SIT7".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_SIT7 as c_int,
    },
    stringID_table_t {
        name: c"TORSO_DROPWEAP1".as_ptr() as *mut c_char,
        id: animNumber_t::TORSO_DROPWEAP1 as c_int,
    },
    stringID_table_t {
        name: c"TORSO_DROPWEAP4".as_ptr() as *mut c_char,
        id: animNumber_t::TORSO_DROPWEAP4 as c_int,
    },
    stringID_table_t {
        name: c"TORSO_RAISEWEAP1".as_ptr() as *mut c_char,
        id: animNumber_t::TORSO_RAISEWEAP1 as c_int,
    },
    stringID_table_t {
        name: c"TORSO_RAISEWEAP4".as_ptr() as *mut c_char,
        id: animNumber_t::TORSO_RAISEWEAP4 as c_int,
    },
    stringID_table_t {
        name: c"TORSO_WEAPONREADY1".as_ptr() as *mut c_char,
        id: animNumber_t::TORSO_WEAPONREADY1 as c_int,
    },
    stringID_table_t {
        name: c"TORSO_WEAPONREADY2".as_ptr() as *mut c_char,
        id: animNumber_t::TORSO_WEAPONREADY2 as c_int,
    },
    stringID_table_t {
        name: c"TORSO_WEAPONREADY3".as_ptr() as *mut c_char,
        id: animNumber_t::TORSO_WEAPONREADY3 as c_int,
    },
    stringID_table_t {
        name: c"TORSO_WEAPONREADY4".as_ptr() as *mut c_char,
        id: animNumber_t::TORSO_WEAPONREADY4 as c_int,
    },
    stringID_table_t {
        name: c"TORSO_WEAPONREADY10".as_ptr() as *mut c_char,
        id: animNumber_t::TORSO_WEAPONREADY10 as c_int,
    },
    stringID_table_t {
        name: c"TORSO_WEAPONIDLE2".as_ptr() as *mut c_char,
        id: animNumber_t::TORSO_WEAPONIDLE2 as c_int,
    },
    stringID_table_t {
        name: c"TORSO_WEAPONIDLE3".as_ptr() as *mut c_char,
        id: animNumber_t::TORSO_WEAPONIDLE3 as c_int,
    },
    stringID_table_t {
        name: c"TORSO_WEAPONIDLE4".as_ptr() as *mut c_char,
        id: animNumber_t::TORSO_WEAPONIDLE4 as c_int,
    },
    stringID_table_t {
        name: c"TORSO_WEAPONIDLE10".as_ptr() as *mut c_char,
        id: animNumber_t::TORSO_WEAPONIDLE10 as c_int,
    },
    stringID_table_t {
        name: c"TORSO_SURRENDER_START".as_ptr() as *mut c_char,
        id: animNumber_t::TORSO_SURRENDER_START as c_int,
    },
    stringID_table_t {
        name: c"TORSO_SURRENDER_STOP".as_ptr() as *mut c_char,
        id: animNumber_t::TORSO_SURRENDER_STOP as c_int,
    },
    stringID_table_t {
        name: c"TORSO_CHOKING1".as_ptr() as *mut c_char,
        id: animNumber_t::TORSO_CHOKING1 as c_int,
    },
    stringID_table_t {
        name: c"TORSO_HANDSIGNAL1".as_ptr() as *mut c_char,
        id: animNumber_t::TORSO_HANDSIGNAL1 as c_int,
    },
    stringID_table_t {
        name: c"TORSO_HANDSIGNAL2".as_ptr() as *mut c_char,
        id: animNumber_t::TORSO_HANDSIGNAL2 as c_int,
    },
    stringID_table_t {
        name: c"TORSO_HANDSIGNAL3".as_ptr() as *mut c_char,
        id: animNumber_t::TORSO_HANDSIGNAL3 as c_int,
    },
    stringID_table_t {
        name: c"TORSO_HANDSIGNAL4".as_ptr() as *mut c_char,
        id: animNumber_t::TORSO_HANDSIGNAL4 as c_int,
    },
    stringID_table_t {
        name: c"TORSO_HANDSIGNAL5".as_ptr() as *mut c_char,
        id: animNumber_t::TORSO_HANDSIGNAL5 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_TURN1".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_TURN1 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_TURN2".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_TURN2 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_LEAN_LEFT1".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_LEAN_LEFT1 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_LEAN_RIGHT1".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_LEAN_RIGHT1 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_CHOKING1".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_CHOKING1 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_LEFTUP1".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_LEFTUP1 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_LEFTUP2".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_LEFTUP2 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_LEFTUP3".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_LEFTUP3 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_LEFTUP4".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_LEFTUP4 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_LEFTUP5".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_LEFTUP5 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_RIGHTUP1".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_RIGHTUP1 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_RIGHTUP2".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_RIGHTUP2 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_RIGHTUP3".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_RIGHTUP3 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_RIGHTUP4".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_RIGHTUP4 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_RIGHTUP5".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_RIGHTUP5 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S1_LUP1".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S1_LUP1 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S1_LUP2".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S1_LUP2 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S1_LUP3".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S1_LUP3 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S1_LUP4".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S1_LUP4 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S1_LUP5".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S1_LUP5 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S1_RUP1".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S1_RUP1 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S1_RUP2".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S1_RUP2 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S1_RUP3".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S1_RUP3 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S1_RUP4".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S1_RUP4 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S1_RUP5".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S1_RUP5 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S3_LUP1".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S3_LUP1 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S3_LUP2".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S3_LUP2 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S3_LUP3".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S3_LUP3 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S3_LUP4".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S3_LUP4 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S3_LUP5".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S3_LUP5 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S3_RUP1".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S3_RUP1 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S3_RUP2".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S3_RUP2 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S3_RUP3".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S3_RUP3 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S3_RUP4".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S3_RUP4 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S3_RUP5".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S3_RUP5 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S4_LUP1".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S4_LUP1 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S4_LUP2".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S4_LUP2 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S4_LUP3".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S4_LUP3 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S4_LUP4".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S4_LUP4 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S4_LUP5".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S4_LUP5 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S4_RUP1".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S4_RUP1 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S4_RUP2".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S4_RUP2 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S4_RUP3".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S4_RUP3 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S4_RUP4".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S4_RUP4 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S4_RUP5".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S4_RUP5 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S5_LUP1".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S5_LUP1 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S5_LUP2".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S5_LUP2 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S5_LUP3".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S5_LUP3 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S5_LUP4".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S5_LUP4 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S5_LUP5".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S5_LUP5 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S5_RUP1".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S5_RUP1 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S5_RUP2".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S5_RUP2 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S5_RUP3".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S5_RUP3 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S5_RUP4".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S5_RUP4 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S5_RUP5".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S5_RUP5 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S6_LUP1".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S6_LUP1 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S6_LUP2".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S6_LUP2 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S6_LUP3".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S6_LUP3 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S6_LUP4".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S6_LUP4 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S6_LUP5".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S6_LUP5 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S6_RUP1".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S6_RUP1 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S6_RUP2".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S6_RUP2 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S6_RUP3".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S6_RUP3 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S6_RUP4".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S6_RUP4 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S6_RUP5".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S6_RUP5 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S7_LUP1".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S7_LUP1 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S7_LUP2".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S7_LUP2 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S7_LUP3".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S7_LUP3 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S7_LUP4".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S7_LUP4 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S7_LUP5".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S7_LUP5 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S7_RUP1".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S7_RUP1 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S7_RUP2".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S7_RUP2 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S7_RUP3".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S7_RUP3 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S7_RUP4".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S7_RUP4 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_S7_RUP5".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_S7_RUP5 as c_int,
    },
    stringID_table_t {
        name: c"LEGS_TURN180".as_ptr() as *mut c_char,
        id: animNumber_t::LEGS_TURN180 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_1".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_1 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_2".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_2 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_3".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_3 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_4".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_4 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_5".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_5 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_6".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_6 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_7".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_7 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_8".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_8 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_9".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_9 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_10".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_10 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_11".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_11 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_12".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_12 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_13".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_13 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_14".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_14 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_15".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_15 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_16".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_16 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_17".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_17 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_18".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_18 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_19".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_19 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_20".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_20 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_21".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_21 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_22".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_22 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_23".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_23 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_24".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_24 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_25".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_25 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_26".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_26 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_27".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_27 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_28".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_28 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_29".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_29 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_30".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_30 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_31".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_31 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_32".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_32 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_33".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_33 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_34".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_34 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_35".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_35 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_36".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_36 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_37".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_37 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_38".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_38 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_39".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_39 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_40".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_40 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_41".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_41 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_42".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_42 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_43".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_43 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_44".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_44 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_45".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_45 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_46".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_46 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_47".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_47 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_48".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_48 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_49".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_49 as c_int,
    },
    stringID_table_t {
        name: c"BOTH_CIN_50".as_ptr() as *mut c_char,
        id: animNumber_t::BOTH_CIN_50 as c_int,
    },
    // Raven's terminator sentinel (`animtable.h:1789`): `NULL, -1`.
    stringID_table_t {
        name: core::ptr::null_mut(),
        id: -1,
    },
];
