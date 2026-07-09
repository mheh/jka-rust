#![allow(non_camel_case_types, non_snake_case)]

/// Raven `gameExport_t` — enumeration of game export functions called by the engine.
///
/// Type definition source: `oracle/codemp/game/g_public.h:734-799`
#[repr(i32)]
pub enum gameExport_t {
	GAME_INIT,                          // ( int levelTime, int randomSeed, int restart );
	                                     // init and shutdown will be called every single level
	                                     // The game should call G_GET_ENTITY_TOKEN to parse through all the
	                                     // entity configuration text and spawn gentities.

	GAME_SHUTDOWN,                      // (void);

	GAME_CLIENT_CONNECT,                // ( int clientNum, qboolean firstTime, qboolean isBot );
	                                     // return NULL if the client is allowed to connect, otherwise return
	                                     // a text string with the reason for denial

	GAME_CLIENT_BEGIN,                  // ( int clientNum );

	GAME_CLIENT_USERINFO_CHANGED,       // ( int clientNum );

	GAME_CLIENT_DISCONNECT,             // ( int clientNum );

	GAME_CLIENT_COMMAND,                // ( int clientNum );

	GAME_CLIENT_THINK,                  // ( int clientNum );

	GAME_RUN_FRAME,                     // ( int levelTime );

	GAME_CONSOLE_COMMAND,               // ( void );
	                                     // ConsoleCommand will be called when a command has been issued
	                                     // that is not recognized as a builtin function.
	                                     // The game can issue trap_argc() / trap_argv() commands to get the command
	                                     // and parameters.  Return qfalse if the game doesn't recognize it as a command.

	BOTAI_START_FRAME,                  // ( int time );

	GAME_ROFF_NOTETRACK_CALLBACK,       // int entnum, char *notetrack

	GAME_SPAWN_RMG_ENTITY,              // rwwRMG - added

	// rww - icarus callbacks
	GAME_ICARUS_PLAYSOUND,
	GAME_ICARUS_SET,
	GAME_ICARUS_LERP2POS,
	GAME_ICARUS_LERP2ORIGIN,
	GAME_ICARUS_LERP2ANGLES,
	GAME_ICARUS_GETTAG,
	GAME_ICARUS_LERP2START,
	GAME_ICARUS_LERP2END,
	GAME_ICARUS_USE,
	GAME_ICARUS_KILL,
	GAME_ICARUS_REMOVE,
	GAME_ICARUS_PLAY,
	GAME_ICARUS_GETFLOAT,
	GAME_ICARUS_GETVECTOR,
	GAME_ICARUS_GETSTRING,
	GAME_ICARUS_SOUNDINDEX,
	GAME_ICARUS_GETSETIDFORSTRING,
	GAME_NAV_CLEARPATHTOPOINT,
	GAME_NAV_CLEARLOS,
	GAME_NAV_CLEARPATHBETWEENPOINTS,
	GAME_NAV_CHECKNODEFAILEDFORENT,
	GAME_NAV_ENTISUNLOCKEDDOOR,
	GAME_NAV_ENTISDOOR,
	GAME_NAV_ENTISBREAKABLE,
	GAME_NAV_ENTISREMOVABLEUSABLE,
	GAME_NAV_FINDCOMBATPOINTWAYPOINTS,

	GAME_GETITEMINDEXBYTAG,
}
