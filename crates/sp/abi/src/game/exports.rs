use abi_transport::generic::FunctionTableExport;

/// Opaque Raven `game_export_t` SP game export function table.
///
/// Raven: functions exported by the game subsystem
/// Source: `oracle/oracle/code/game/g_public.h:474`
/// Source (table): `oracle/oracle/code/game/g_public.h:477-527`
/// Source (GetGameAPI output): `oracle/oracle/code/game/g_main.cpp:875`
///
/// Field layout is intentionally deferred until the full table is ported.
#[repr(C)]
pub struct SpGameExportTable {
    _private: [u8; 0],
}

/// `game_export_t` SP game `GetGameAPI` export-table ABI token.
///
/// Raven: `game_export_t *GetGameAPI( game_import_t *import )`
/// Raven: `return &globals;`
/// Source (GetGameAPI): `oracle/oracle/code/game/g_main.cpp:875-912`
/// Source (table): `oracle/oracle/code/game/g_public.h:477-527`
pub struct SpGameExport;

impl FunctionTableExport for SpGameExport {
    type Table = SpGameExportTable;
}
