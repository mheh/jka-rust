use abi_transport::generic::FunctionTableImport;

/// Opaque Raven `game_import_t` SP game import function table.
///
/// Raven: functions provided by the main engine
/// Source: `oracle/oracle/code/game/g_public.h:159`
/// Source (table): `oracle/oracle/code/game/g_public.h:164-471`
/// Source (GetGameAPI arg): `oracle/oracle/code/game/g_main.cpp:875`
///
/// Field layout is intentionally deferred until the full table is ported.
#[repr(C)]
pub struct SpGameImportTable {
    _private: [u8; 0],
}

/// `game_import_t` SP game `GetGameAPI` import-table ABI token.
///
/// Raven: `game_export_t *GetGameAPI( game_import_t *import )`
/// Raven: `gi = *import;`
/// Source (GetGameAPI): `oracle/oracle/code/game/g_main.cpp:875-878`
/// Source (table): `oracle/oracle/code/game/g_public.h:164-471`
pub struct SpGameImport;

impl FunctionTableImport for SpGameImport {
    type Table = SpGameImportTable;
}
