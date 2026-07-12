//! Raven `RMG_CreateSeed` — a procedural-name generator that also folds the
//! result into a 32-bit hash "seed" (`cm_randomterrain.cpp:1008-1084`).
//!
//! **GOLDEN-ONLY** (RMG-D4f, `docs/subsystems/rmg-terrain.md` roster row for
//! `crates/mp/engine/qcommon/src/cm_randomterrain.rs`): `RMG_CreateSeed` has
//! **zero live callers** anywhere in `codemp/` — no live RMG path under
//! `DEDICATED` draws it — but it is kept because it is the harness's golden
//! #1, pinning `Engine.common.rng` (`mp_qshared::QRand`) through
//! `EngineHost::flrand`/`irand` (ruling 21 part 3). Its sole private helper,
//! `FindPiece` (`cm_randomterrain.cpp:960-1005`), and the `ECPType` enum /
//! `TCharacterPiece` table shape it walks port alongside it per §21 (private
//! helpers colocate with their caller).
//!
//! **§20-dropped** (RMG-D1 generation path, not represented in this file):
//! the whole `CRandomTerrain`/`CPathInfo` class (`Generate`/`Smooth`/
//! `ParseGenerate`/`DrawPath`/…, `cm_randomterrain.h`,
//! `cm_randomterrain.cpp:1-825`) and the dead Perlin-noise path
//! (`noiseTable`/`noisePerm`/`CM_NoiseInit`, `cm_randomterrain.cpp:14-28`) —
//! both unreachable under `DEDICATED` (sole constructor `CreateRandomTerrain`
//! is in the `#else` of `#ifdef DEDICATED`, `cm_terrain.cpp:170-188`).
//!
//! Source: `oracle/codemp/qcommon/cm_randomterrain.cpp`

use mp_host_interface::EngineHost;

/// Raven `ECPType` — the character-piece category `FindPiece` draws from.
///
/// `typedef enum {...} ECPType` -> `#[repr(i32)] enum` per porting-rules
/// enum-vs-alias fidelity (never flattened to a bare `i32`).
///
/// Source: `oracle/codemp/qcommon/cm_randomterrain.cpp:829-839`
// `None`/`NumPieces` are faithful Raven enumerators kept for enum-vs-alias
// fidelity (never flatten a named enum); on the non-test build they are only
// *matched* (the defensive `default` arm of `find_piece`, `:966-968`), never
// *constructed*, so `dead_code` fires on them without this allow.
#[allow(dead_code)]
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EcpType {
    None = -1,
    Consonant = 0,
    ComplexConsonant = 1,
    Vowel = 2,
    ComplexVowel = 3,
    Ending = 4,
    NumPieces = 5,
}

/// Raven `TCharacterPiece` (`struct SCharacterPiece`) — one weighted
/// name-generation syllable.
///
/// Internal-only shape (never crosses the ABI seam): the raw `char *mPiece`
/// becomes an idiomatic `&'static str` (porting-rules §12); the sentinel
/// `{ 0, 0 }` terminator Raven's linear scan relies on is dropped in favor of
/// a plain Rust slice (`FindPiece` sums/walks `.len()`, not a null check).
///
/// Source: `oracle/codemp/qcommon/cm_randomterrain.cpp:841-844`
struct CharacterPiece {
    piece: &'static str,
    commonality: i32,
}

/// Raven `Consonants[]`.
///
/// Source: `oracle/codemp/qcommon/cm_randomterrain.cpp:847-869`
const CONSONANTS: &[CharacterPiece] = &[
    CharacterPiece {
        piece: "b",
        commonality: 6,
    },
    CharacterPiece {
        piece: "c",
        commonality: 8,
    },
    CharacterPiece {
        piece: "d",
        commonality: 6,
    },
    CharacterPiece {
        piece: "f",
        commonality: 5,
    },
    CharacterPiece {
        piece: "g",
        commonality: 4,
    },
    CharacterPiece {
        piece: "h",
        commonality: 5,
    },
    CharacterPiece {
        piece: "j",
        commonality: 2,
    },
    CharacterPiece {
        piece: "k",
        commonality: 4,
    },
    CharacterPiece {
        piece: "l",
        commonality: 4,
    },
    CharacterPiece {
        piece: "m",
        commonality: 7,
    },
    CharacterPiece {
        piece: "n",
        commonality: 7,
    },
    CharacterPiece {
        piece: "r",
        commonality: 6,
    },
    CharacterPiece {
        piece: "s",
        commonality: 10,
    },
    CharacterPiece {
        piece: "t",
        commonality: 10,
    },
    CharacterPiece {
        piece: "v",
        commonality: 1,
    },
    CharacterPiece {
        piece: "w",
        commonality: 2,
    },
    CharacterPiece {
        piece: "x",
        commonality: 1,
    },
    CharacterPiece {
        piece: "z",
        commonality: 1,
    },
];

/// Raven `ComplexConsonants[]`.
///
/// Source: `oracle/codemp/qcommon/cm_randomterrain.cpp:871-896`
const COMPLEX_CONSONANTS: &[CharacterPiece] = &[
    CharacterPiece {
        piece: "st",
        commonality: 10,
    },
    CharacterPiece {
        piece: "ck",
        commonality: 10,
    },
    CharacterPiece {
        piece: "ss",
        commonality: 10,
    },
    CharacterPiece {
        piece: "tt",
        commonality: 7,
    },
    CharacterPiece {
        piece: "ll",
        commonality: 8,
    },
    CharacterPiece {
        piece: "nd",
        commonality: 10,
    },
    CharacterPiece {
        piece: "rn",
        commonality: 6,
    },
    CharacterPiece {
        piece: "nc",
        commonality: 6,
    },
    CharacterPiece {
        piece: "mp",
        commonality: 4,
    },
    CharacterPiece {
        piece: "sc",
        commonality: 10,
    },
    CharacterPiece {
        piece: "sl",
        commonality: 10,
    },
    CharacterPiece {
        piece: "tch",
        commonality: 6,
    },
    CharacterPiece {
        piece: "th",
        commonality: 4,
    },
    CharacterPiece {
        piece: "rn",
        commonality: 5,
    },
    CharacterPiece {
        piece: "cl",
        commonality: 10,
    },
    CharacterPiece {
        piece: "sp",
        commonality: 10,
    },
    CharacterPiece {
        piece: "st",
        commonality: 10,
    },
    CharacterPiece {
        piece: "fl",
        commonality: 4,
    },
    CharacterPiece {
        piece: "sh",
        commonality: 7,
    },
    CharacterPiece {
        piece: "ng",
        commonality: 4,
    },
];

/// Raven `Vowels[]`.
///
/// Source: `oracle/codemp/qcommon/cm_randomterrain.cpp:898-908`
const VOWELS: &[CharacterPiece] = &[
    CharacterPiece {
        piece: "a",
        commonality: 10,
    },
    CharacterPiece {
        piece: "e",
        commonality: 10,
    },
    CharacterPiece {
        piece: "i",
        commonality: 10,
    },
    CharacterPiece {
        piece: "o",
        commonality: 10,
    },
    CharacterPiece {
        piece: "u",
        commonality: 2,
    },
];

/// Raven `ComplexVowels[]`.
///
/// Source: `oracle/codemp/qcommon/cm_randomterrain.cpp:910-927`
const COMPLEX_VOWELS: &[CharacterPiece] = &[
    CharacterPiece {
        piece: "ea",
        commonality: 10,
    },
    CharacterPiece {
        piece: "ue",
        commonality: 3,
    },
    CharacterPiece {
        piece: "oi",
        commonality: 10,
    },
    CharacterPiece {
        piece: "ai",
        commonality: 8,
    },
    CharacterPiece {
        piece: "oo",
        commonality: 10,
    },
    CharacterPiece {
        piece: "io",
        commonality: 10,
    },
    CharacterPiece {
        piece: "oe",
        commonality: 10,
    },
    CharacterPiece {
        piece: "au",
        commonality: 3,
    },
    CharacterPiece {
        piece: "ee",
        commonality: 7,
    },
    CharacterPiece {
        piece: "ei",
        commonality: 7,
    },
    CharacterPiece {
        piece: "ou",
        commonality: 7,
    },
    CharacterPiece {
        piece: "ia",
        commonality: 4,
    },
];

/// Raven `Endings[]`.
///
/// Source: `oracle/codemp/qcommon/cm_randomterrain.cpp:929-957`
const ENDINGS: &[CharacterPiece] = &[
    CharacterPiece {
        piece: "ing",
        commonality: 10,
    },
    CharacterPiece {
        piece: "ed",
        commonality: 10,
    },
    CharacterPiece {
        piece: "ute",
        commonality: 10,
    },
    CharacterPiece {
        piece: "ance",
        commonality: 10,
    },
    CharacterPiece {
        piece: "ey",
        commonality: 10,
    },
    CharacterPiece {
        piece: "ation",
        commonality: 10,
    },
    CharacterPiece {
        piece: "ous",
        commonality: 10,
    },
    CharacterPiece {
        piece: "ent",
        commonality: 10,
    },
    CharacterPiece {
        piece: "ate",
        commonality: 10,
    },
    CharacterPiece {
        piece: "ible",
        commonality: 10,
    },
    CharacterPiece {
        piece: "age",
        commonality: 10,
    },
    CharacterPiece {
        piece: "ity",
        commonality: 10,
    },
    CharacterPiece {
        piece: "ist",
        commonality: 10,
    },
    CharacterPiece {
        piece: "ism",
        commonality: 10,
    },
    CharacterPiece {
        piece: "ime",
        commonality: 10,
    },
    CharacterPiece {
        piece: "ic",
        commonality: 10,
    },
    CharacterPiece {
        piece: "ant",
        commonality: 10,
    },
    CharacterPiece {
        piece: "etry",
        commonality: 10,
    },
    CharacterPiece {
        piece: "ious",
        commonality: 10,
    },
    CharacterPiece {
        piece: "ative",
        commonality: 10,
    },
    CharacterPiece {
        piece: "er",
        commonality: 10,
    },
    CharacterPiece {
        piece: "ize",
        commonality: 10,
    },
    CharacterPiece {
        piece: "able",
        commonality: 10,
    },
    CharacterPiece {
        piece: "itude",
        commonality: 10,
    },
];

/// Raven `FindPiece` — draws one weighted-random piece from `type`'s table.
///
/// Raven takes `char *&pos` and both writes the piece text at `pos` and
/// advances it past the copy; per porting-rules §C7 (out-params -> return
/// values) this returns the picked piece and lets the caller append + advance
/// its own buffer, rather than mutating a caller-owned cursor in place.
///
/// Source: `oracle/codemp/qcommon/cm_randomterrain.cpp:960-1005`
fn find_piece(host: &mut impl EngineHost, kind: EcpType) -> &'static str {
    // Raven's `switch(type) { case CP_CONSONANT: default: start = Consonants; ... }`
    // (`:966-968`) — the default arm falls through to Consonants alongside the
    // named `CP_CONSONANT` case, so `EcpType::None`/`NumPieces` (never actually
    // passed by `rmg_create_seed`) resolve there too.
    let table: &[CharacterPiece] = match kind {
        EcpType::ComplexConsonant => COMPLEX_CONSONANTS,
        EcpType::Vowel => VOWELS,
        EcpType::ComplexVowel => COMPLEX_VOWELS,
        EcpType::Ending => ENDINGS,
        EcpType::Consonant | EcpType::None | EcpType::NumPieces => CONSONANTS,
    };

    // First while-loop (`:987-991`): sum the table's commonality weights.
    let total: i32 = table.iter().map(|piece| piece.commonality).sum();

    // `count = irand(0, count-1)` (`:993`).
    let mut count = host.irand(0, total - 1);

    // Second while-loop (`:994-999`): walk down, subtracting each piece's
    // weight, until `count <= search->mCommonality`.
    let mut chosen = table[table.len() - 1].piece;
    for piece in table {
        if count <= piece.commonality {
            chosen = piece.piece;
            break;
        }
        count -= piece.commonality;
    }
    chosen
}

/// Raven `RMG_CreateSeed`.
///
/// Builds a pronounceable "text seed" (a `char TextSeed[]` in the oracle,
/// caller-allocated `MAX_QPATH`-ish sized) by chaining weighted syllable
/// pieces via [`find_piece`], then folds that text into a 32-bit hash. Per
/// porting-rules §C7 the out-param `char *TextSeed` becomes an owned `String`
/// return; the `unsigned` return becomes the pinned `u32` (Seam §C,
/// `docs/subsystems/rmg-terrain.md`). Every random draw
/// (`irand(4,9)`/`irand(0,100)`/piece selection) threads through
/// `EngineHost::irand`, i.e. `Engine.common.rng` (`mp_qshared::QRand`,
/// RMG-D4f) — this is the harness's golden #1, so no live caller reaches it
/// (RMG-D1).
///
/// Source: `oracle/codemp/qcommon/cm_randomterrain.cpp:1008-1084`
pub fn rmg_create_seed(host: &mut impl EngineHost) -> (String, u32) {
    // `Length = irand(4, 9);` (`:1016`).
    let mut length = host.irand(4, 9);

    // `if (irand(0, 100) < 20) LookingFor = CP_VOWEL; else LookingFor = CP_CONSONANT;`
    // (`:1018-1025`).
    let mut looking_for = if host.irand(0, 100) < 20 {
        EcpType::Vowel
    } else {
        EcpType::Consonant
    };

    // `Ending[0] = 0;` then the 55%-chance ending draw (`:1027-1033`). Raven
    // writes into a caller `Ending[256]` buffer via the `pos` out-cursor and
    // shrinks `Length` by the copied piece's length; ported as an owned
    // `String` per porting-rules §C7.
    let mut ending = String::new();
    if host.irand(0, 100) < 55 {
        let piece = find_piece(host, EcpType::Ending);
        ending.push_str(piece);
        length -= piece.len() as i32;
    }

    // `pos = TextSeed; *pos = 0;` (`:1035-1036`).
    let mut text_seed = String::new();

    // `ComplexVowelChance = -1; ComplexConsonantChance = -1;` (`:1038-1039`).
    let mut complex_vowel_chance: i32 = -1;
    let mut complex_consonant_chance: i32 = -1;

    // `while((pos - TextSeed) < Length || LookingFor == CP_CONSONANT)` (`:1041`).
    while (text_seed.len() as i32) < length || looking_for == EcpType::Consonant {
        if looking_for == EcpType::Vowel {
            // `:1043-1051`.
            if host.irand(0, 100) < complex_vowel_chance {
                complex_vowel_chance = -1;
                looking_for = EcpType::ComplexVowel;
            } else {
                complex_vowel_chance += 10;
            }
            text_seed.push_str(find_piece(host, looking_for));
            looking_for = EcpType::Consonant;
        } else {
            // `:1053-1063`.
            if host.irand(0, 100) < complex_consonant_chance {
                complex_consonant_chance = -1;
                looking_for = EcpType::ComplexConsonant;
            } else {
                complex_consonant_chance += 45;
            }
            text_seed.push_str(find_piece(host, looking_for));
            looking_for = EcpType::Vowel;
        }
    }

    // `if (Ending[0]) strcpy(pos, Ending);` (`:1067-1070`).
    if !ending.is_empty() {
        text_seed.push_str(&ending);
    }

    // The hash fold (`:1072-1080`): `SeedValue ^= (SeedValue << 4) + ((*pos)-'a');
    // SeedValue ^= high;` per byte, `high` captured before the xor-add. Raven's
    // `unsigned` arithmetic wraps; `wrapping_add` mirrors that (the shift itself
    // cannot overflow-panic in Rust).
    let mut seed_value: u32 = 0;
    for byte in text_seed.bytes() {
        let high = seed_value >> 28;
        seed_value ^= (seed_value << 4).wrapping_add((byte as u32).wrapping_sub(b'a' as u32));
        seed_value ^= high;
    }

    (text_seed, seed_value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mp_host_interface::mock::MockHost;

    /// `FindPiece` never returns a piece text longer than any table's longest
    /// entry, and always returns *some* piece from the requested table — a
    /// cheap way to pin the "weighted draw picks a real table entry" shape
    /// without hard-coding the LCG's exact sequence (that belongs to the
    /// harness golden, RMG-D4f).
    #[test]
    fn find_piece_returns_a_table_member() {
        let mut host = MockHost::new();
        for _ in 0..64 {
            let piece = find_piece(&mut host, EcpType::Consonant);
            assert!(CONSONANTS.iter().any(|p| p.piece == piece));
            let piece = find_piece(&mut host, EcpType::ComplexConsonant);
            assert!(COMPLEX_CONSONANTS.iter().any(|p| p.piece == piece));
            let piece = find_piece(&mut host, EcpType::Vowel);
            assert!(VOWELS.iter().any(|p| p.piece == piece));
            let piece = find_piece(&mut host, EcpType::ComplexVowel);
            assert!(COMPLEX_VOWELS.iter().any(|p| p.piece == piece));
            let piece = find_piece(&mut host, EcpType::Ending);
            assert!(ENDINGS.iter().any(|p| p.piece == piece));
        }
    }

    /// Raven's default-arm fallthrough (`case CP_CONSONANT: default:`,
    /// `cm_randomterrain.cpp:966-968`) — an out-of-band `EcpType` still draws
    /// from `Consonants`, matching the switch's fallthrough rather than
    /// diverging into a panic/empty result.
    #[test]
    fn find_piece_defaults_unmapped_kinds_to_consonants() {
        let mut host = MockHost::new();
        let piece = find_piece(&mut host, EcpType::None);
        assert!(CONSONANTS.iter().any(|p| p.piece == piece));
        let piece = find_piece(&mut host, EcpType::NumPieces);
        assert!(CONSONANTS.iter().any(|p| p.piece == piece));
    }

    /// Two draws off freshly-seeded (`0x89abcdef`) `MockHost` instances are
    /// bit-identical — pins that `rmg_create_seed` is a pure function of the
    /// threaded `EngineHost` RNG, per the golden-#1 role (RMG-D4f).
    #[test]
    fn rmg_create_seed_is_deterministic_over_the_same_rng_seed() {
        let mut host_a = MockHost::new();
        let mut host_b = MockHost::new();
        let (text_a, seed_a) = rmg_create_seed(&mut host_a);
        let (text_b, seed_b) = rmg_create_seed(&mut host_b);
        assert_eq!(text_a, text_b);
        assert_eq!(seed_a, seed_b);
    }

    /// The returned text seed is exactly ASCII lowercase-letter pieces (the
    /// hash fold assumes `byte - b'a'` never underflows).
    #[test]
    fn rmg_create_seed_text_is_ascii_lowercase() {
        let mut host = MockHost::new();
        let (text, _) = rmg_create_seed(&mut host);
        assert!(!text.is_empty());
        assert!(text.bytes().all(|b| b.is_ascii_lowercase()));
    }
}
