//! MP game imports enum vocabulary.
//!
//! Transcribed from Raven `oracle/oracle/codemp/game/g_public.h`.
//! These discriminants are ABI wire values; do not renumber them.

#![allow(non_camel_case_types)] // C enumerator names kept for 1:1 traceability

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MpGameImport {
    /// ============== general Quake services ==================
    /// ( const char *string );
    /// print message on the local console
    /// Source: `oracle/oracle/codemp/game/g_public.h:105`
    G_PRINT,

    /// ( const char *string );
    /// abort the game
    /// Source: `oracle/oracle/codemp/game/g_public.h:108`
    G_ERROR,

    /// ( void );
    /// get current time for profiling reasons
    /// this should NOT be used for any game related tasks,
    /// because it is not journaled
    /// Also for profiling.. do not use for game related tasks.
    /// Source: `oracle/oracle/codemp/game/g_public.h:111`
    G_MILLISECONDS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:117`
    G_PRECISIONTIMER_START,

    /// console variable interaction
    /// Source: `oracle/oracle/codemp/game/g_public.h:118`
    G_PRECISIONTIMER_END,

    /// ( vmCvar_t *vmCvar, const char *varName, const char *defaultValue, int flags );
    /// Source: `oracle/oracle/codemp/game/g_public.h:121`
    G_CVAR_REGISTER,

    /// ( vmCvar_t *vmCvar );
    /// Source: `oracle/oracle/codemp/game/g_public.h:122`
    G_CVAR_UPDATE,

    /// ( const char *var_name, const char *value );
    /// Source: `oracle/oracle/codemp/game/g_public.h:123`
    G_CVAR_SET,

    /// ( const char *var_name );
    /// Source: `oracle/oracle/codemp/game/g_public.h:124`
    G_CVAR_VARIABLE_INTEGER_VALUE,

    /// ( const char *var_name, char *buffer, int bufsize );
    /// Source: `oracle/oracle/codemp/game/g_public.h:126`
    G_CVAR_VARIABLE_STRING_BUFFER,

    /// ( void );
    /// ClientCommand and ServerCommand parameter access
    /// Source: `oracle/oracle/codemp/game/g_public.h:128`
    G_ARGC,

    /// ( int n, char *buffer, int bufferLength );
    /// Source: `oracle/oracle/codemp/game/g_public.h:131`
    G_ARGV,

    /// ( const char *qpath, fileHandle_t *file, fsMode_t mode );
    /// Source: `oracle/oracle/codemp/game/g_public.h:133`
    G_FS_FOPEN_FILE,

    /// ( void *buffer, int len, fileHandle_t f );
    /// Source: `oracle/oracle/codemp/game/g_public.h:134`
    G_FS_READ,

    /// ( const void *buffer, int len, fileHandle_t f );
    /// Source: `oracle/oracle/codemp/game/g_public.h:135`
    G_FS_WRITE,

    /// ( fileHandle_t f );
    /// Source: `oracle/oracle/codemp/game/g_public.h:136`
    G_FS_FCLOSE_FILE,

    /// ( const char *text );
    /// add commands to the console as if they were typed in
    /// for map changing, etc
    /// =========== server specific functionality =============
    /// Source: `oracle/oracle/codemp/game/g_public.h:138`
    G_SEND_CONSOLE_COMMAND,

    /// ( gentity_t *gEnts, int numGEntities, int sizeofGEntity_t,
    /// playerState_t *clients, int sizeofGameClient );
    /// the game needs to let the server system know where and how big the gentities
    /// are, so it can look at them directly without going through an interface
    /// Source: `oracle/oracle/codemp/game/g_public.h:145`
    G_LOCATE_GAME_DATA,

    /// ( int clientNum, const char *reason );
    /// kick a client off the server with a message
    /// Source: `oracle/oracle/codemp/game/g_public.h:150`
    G_DROP_CLIENT,

    /// ( int clientNum, const char *fmt, ... );
    /// reliably sends a command string to be interpreted by the given
    /// client.  If clientNum is -1, it will be sent to all clients
    /// Source: `oracle/oracle/codemp/game/g_public.h:153`
    G_SEND_SERVER_COMMAND,

    /// ( int num, const char *string );
    /// config strings hold all the index strings, and various other information
    /// that is reliably communicated to all clients
    /// All of the current configstrings are sent to clients when
    /// they connect, and changes are sent to all connected clients.
    /// All confgstrings are cleared at each level start.
    /// Source: `oracle/oracle/codemp/game/g_public.h:157`
    G_SET_CONFIGSTRING,

    /// ( int num, char *buffer, int bufferSize );
    /// Source: `oracle/oracle/codemp/game/g_public.h:164`
    G_GET_CONFIGSTRING,

    /// ( int num, char *buffer, int bufferSize );
    /// userinfo strings are maintained by the server system, so they
    /// are persistant across level loads, while all other game visible
    /// data is completely reset
    /// Source: `oracle/oracle/codemp/game/g_public.h:166`
    G_GET_USERINFO,

    /// ( int num, const char *buffer );
    /// Source: `oracle/oracle/codemp/game/g_public.h:171`
    G_SET_USERINFO,

    /// ( char *buffer, int bufferSize );
    /// the serverinfo info string has all the cvars visible to server browsers
    /// Source: `oracle/oracle/codemp/game/g_public.h:173`
    G_GET_SERVERINFO,

    /// server culling to reduce traffic on open maps -rww
    /// Source: `oracle/oracle/codemp/game/g_public.h:176`
    G_SET_SERVER_CULL,

    /// ( gentity_t *ent, const char *name );
    /// sets mins and maxs based on the brushmodel name
    /// Source: `oracle/oracle/codemp/game/g_public.h:179`
    G_SET_BRUSH_MODEL,

    /// ( trace_t *results, const vec3_t start, const vec3_t mins, const vec3_t maxs, const vec3_t end, int passEntityNum, int contentmask );
    /// collision detection against all linked entities
    /// Source: `oracle/oracle/codemp/game/g_public.h:182`
    G_TRACE,

    /// ( trace_t *results, const vec3_t start, const vec3_t mins, const vec3_t maxs, const vec3_t end, int passEntityNum, int contentmask );
    /// collision detection against all linked entities with ghoul2 check
    /// Source: `oracle/oracle/codemp/game/g_public.h:185`
    G_G2TRACE,

    /// ( const vec3_t point, int passEntityNum );
    /// point contents against all linked entities
    /// Source: `oracle/oracle/codemp/game/g_public.h:188`
    G_POINT_CONTENTS,

    /// ( const vec3_t p1, const vec3_t p2 );
    /// Source: `oracle/oracle/codemp/game/g_public.h:191`
    G_IN_PVS,

    /// ( const vec3_t p1, const vec3_t p2 );
    /// Source: `oracle/oracle/codemp/game/g_public.h:193`
    G_IN_PVS_IGNORE_PORTALS,

    /// ( gentity_t *ent, qboolean open );
    /// Source: `oracle/oracle/codemp/game/g_public.h:195`
    G_ADJUST_AREA_PORTAL_STATE,

    /// ( int area1, int area2 );
    /// Source: `oracle/oracle/codemp/game/g_public.h:197`
    G_AREAS_CONNECTED,

    /// ( gentity_t *ent );
    /// an entity will never be sent to a client or used for collision
    /// if it is not passed to linkentity.  If the size, position, or
    /// solidity changes, it must be relinked.
    /// Source: `oracle/oracle/codemp/game/g_public.h:199`
    G_LINKENTITY,

    /// ( gentity_t *ent );
    /// call before removing an interactive entity
    /// Source: `oracle/oracle/codemp/game/g_public.h:204`
    G_UNLINKENTITY,

    /// ( const vec3_t mins, const vec3_t maxs, gentity_t **list, int maxcount );
    /// EntitiesInBox will return brush models based on their bounding box,
    /// so exact determination must still be done with EntityContact
    /// Source: `oracle/oracle/codemp/game/g_public.h:207`
    G_ENTITIES_IN_BOX,

    /// ( const vec3_t mins, const vec3_t maxs, const gentity_t *ent );
    /// perform an exact check against inline brush models of non-square shape
    /// access for bots to get and free a server client (FIXME?)
    /// Source: `oracle/oracle/codemp/game/g_public.h:211`
    G_ENTITY_CONTACT,

    /// ( void );
    /// Source: `oracle/oracle/codemp/game/g_public.h:215`
    G_BOT_ALLOCATE_CLIENT,

    /// ( int clientNum );
    /// Source: `oracle/oracle/codemp/game/g_public.h:217`
    G_BOT_FREE_CLIENT,

    /// ( int clientNum, usercmd_t *cmd )
    /// Source: `oracle/oracle/codemp/game/g_public.h:219`
    G_GET_USERCMD,

    /// qboolean ( char *buffer, int bufferSize )
    /// Retrieves the next string token from the entity spawn text, returning
    /// false when all tokens have been parsed.
    /// This should only be done at GAME_INIT time.
    /// Source: `oracle/oracle/codemp/game/g_public.h:221`
    G_GET_ENTITY_TOKEN,

    /// Source: `oracle/oracle/codemp/game/g_public.h:226`
    G_SIEGEPERSSET,

    /// Source: `oracle/oracle/codemp/game/g_public.h:227`
    G_SIEGEPERSGET,

    /// Source: `oracle/oracle/codemp/game/g_public.h:229`
    G_FS_GETFILELIST,

    /// Source: `oracle/oracle/codemp/game/g_public.h:230`
    G_DEBUG_POLYGON_CREATE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:231`
    G_DEBUG_POLYGON_DELETE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:232`
    G_REAL_TIME,

    /// Source: `oracle/oracle/codemp/game/g_public.h:233`
    G_SNAPVECTOR,

    /// ( trace_t *results, const vec3_t start, const vec3_t mins, const vec3_t maxs, const vec3_t end, int passEntityNum, int contentmask );
    /// Source: `oracle/oracle/codemp/game/g_public.h:235`
    G_TRACECAPSULE,

    /// ( const vec3_t mins, const vec3_t maxs, const gentity_t *ent );
    /// SP_REGISTER_SERVER_CMD,
    /// Source: `oracle/oracle/codemp/game/g_public.h:236`
    G_ENTITY_CONTACTCAPSULE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:239`
    SP_GETSTRINGTEXTSTRING,

    /// qboolean	ROFF_Clean(void);
    /// Source: `oracle/oracle/codemp/game/g_public.h:241`
    G_ROFF_CLEAN,

    /// void		ROFF_UpdateEntities(void);
    /// Source: `oracle/oracle/codemp/game/g_public.h:242`
    G_ROFF_UPDATE_ENTITIES,

    /// int		ROFF_Cache(char *file);
    /// Source: `oracle/oracle/codemp/game/g_public.h:243`
    G_ROFF_CACHE,

    /// qboolean	ROFF_Play(int entID, int roffID, qboolean doTranslation);
    /// Source: `oracle/oracle/codemp/game/g_public.h:244`
    G_ROFF_PLAY,

    /// qboolean ROFF_PurgeEnt( int entID )
    /// rww - dynamic vm memory allocation!
    /// Source: `oracle/oracle/codemp/game/g_public.h:245`
    G_ROFF_PURGE_ENT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:248`
    G_TRUEMALLOC,

    /// rww - icarus traps
    /// Source: `oracle/oracle/codemp/game/g_public.h:249`
    G_TRUEFREE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:252`
    G_ICARUS_RUNSCRIPT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:253`
    G_ICARUS_REGISTERSCRIPT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:255`
    G_ICARUS_INIT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:256`
    G_ICARUS_VALIDENT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:257`
    G_ICARUS_ISINITIALIZED,

    /// Source: `oracle/oracle/codemp/game/g_public.h:258`
    G_ICARUS_MAINTAINTASKMANAGER,

    /// Source: `oracle/oracle/codemp/game/g_public.h:259`
    G_ICARUS_ISRUNNING,

    /// Source: `oracle/oracle/codemp/game/g_public.h:260`
    G_ICARUS_TASKIDPENDING,

    /// Source: `oracle/oracle/codemp/game/g_public.h:261`
    G_ICARUS_INITENT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:262`
    G_ICARUS_FREEENT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:263`
    G_ICARUS_ASSOCIATEENT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:264`
    G_ICARUS_SHUTDOWN,

    /// Source: `oracle/oracle/codemp/game/g_public.h:265`
    G_ICARUS_TASKIDSET,

    /// Source: `oracle/oracle/codemp/game/g_public.h:266`
    G_ICARUS_TASKIDCOMPLETE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:267`
    G_ICARUS_SETVAR,

    /// Source: `oracle/oracle/codemp/game/g_public.h:268`
    G_ICARUS_VARIABLEDECLARED,

    /// Source: `oracle/oracle/codemp/game/g_public.h:269`
    G_ICARUS_GETFLOATVARIABLE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:270`
    G_ICARUS_GETSTRINGVARIABLE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:271`
    G_ICARUS_GETVECTORVARIABLE,

    /// BEGIN VM STUFF
    /// Source: `oracle/oracle/codemp/game/g_public.h:273`
    G_SET_SHARED_BUFFER,

    /// Source: `oracle/oracle/codemp/game/g_public.h:276`
    G_MEMSET = 100,

    /// Source: `oracle/oracle/codemp/game/g_public.h:277`
    G_MEMCPY,

    /// Source: `oracle/oracle/codemp/game/g_public.h:278`
    G_STRNCPY,

    /// Source: `oracle/oracle/codemp/game/g_public.h:279`
    G_SIN,

    /// Source: `oracle/oracle/codemp/game/g_public.h:280`
    G_COS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:281`
    G_ATAN2,

    /// Source: `oracle/oracle/codemp/game/g_public.h:282`
    G_SQRT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:283`
    G_MATRIXMULTIPLY,

    /// Source: `oracle/oracle/codemp/game/g_public.h:284`
    G_ANGLEVECTORS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:285`
    G_PERPENDICULARVECTOR,

    /// Source: `oracle/oracle/codemp/game/g_public.h:286`
    G_FLOOR,

    /// Source: `oracle/oracle/codemp/game/g_public.h:287`
    G_CEIL,

    /// Source: `oracle/oracle/codemp/game/g_public.h:289`
    G_TESTPRINTINT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:290`
    G_TESTPRINTFLOAT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:292`
    G_ACOS,

    /// END VM STUFF
    /// rww - BEGIN NPC NAV TRAPS
    /// Source: `oracle/oracle/codemp/game/g_public.h:293`
    G_ASIN,

    /// Source: `oracle/oracle/codemp/game/g_public.h:298`
    G_NAV_INIT = 200,

    /// Source: `oracle/oracle/codemp/game/g_public.h:299`
    G_NAV_FREE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:300`
    G_NAV_LOAD,

    /// Source: `oracle/oracle/codemp/game/g_public.h:301`
    G_NAV_SAVE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:302`
    G_NAV_ADDRAWPOINT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:303`
    G_NAV_CALCULATEPATHS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:304`
    G_NAV_HARDCONNECT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:305`
    G_NAV_SHOWNODES,

    /// Source: `oracle/oracle/codemp/game/g_public.h:306`
    G_NAV_SHOWEDGES,

    /// Source: `oracle/oracle/codemp/game/g_public.h:307`
    G_NAV_SHOWPATH,

    /// Source: `oracle/oracle/codemp/game/g_public.h:308`
    G_NAV_GETNEARESTNODE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:309`
    G_NAV_GETBESTNODE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:310`
    G_NAV_GETNODEPOSITION,

    /// Source: `oracle/oracle/codemp/game/g_public.h:311`
    G_NAV_GETNODENUMEDGES,

    /// Source: `oracle/oracle/codemp/game/g_public.h:312`
    G_NAV_GETNODEEDGE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:313`
    G_NAV_GETNUMNODES,

    /// Source: `oracle/oracle/codemp/game/g_public.h:314`
    G_NAV_CONNECTED,

    /// Source: `oracle/oracle/codemp/game/g_public.h:315`
    G_NAV_GETPATHCOST,

    /// Source: `oracle/oracle/codemp/game/g_public.h:316`
    G_NAV_GETEDGECOST,

    /// Source: `oracle/oracle/codemp/game/g_public.h:317`
    G_NAV_GETPROJECTEDNODE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:318`
    G_NAV_CHECKFAILEDNODES,

    /// Source: `oracle/oracle/codemp/game/g_public.h:319`
    G_NAV_ADDFAILEDNODE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:320`
    G_NAV_NODEFAILED,

    /// Source: `oracle/oracle/codemp/game/g_public.h:321`
    G_NAV_NODESARENEIGHBORS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:322`
    G_NAV_CLEARFAILEDEDGE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:323`
    G_NAV_CLEARALLFAILEDEDGES,

    /// Source: `oracle/oracle/codemp/game/g_public.h:324`
    G_NAV_EDGEFAILED,

    /// Source: `oracle/oracle/codemp/game/g_public.h:325`
    G_NAV_ADDFAILEDEDGE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:326`
    G_NAV_CHECKFAILEDEDGE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:327`
    G_NAV_CHECKALLFAILEDEDGES,

    /// Source: `oracle/oracle/codemp/game/g_public.h:328`
    G_NAV_ROUTEBLOCKED,

    /// Source: `oracle/oracle/codemp/game/g_public.h:329`
    G_NAV_GETBESTNODEALTROUTE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:330`
    G_NAV_GETBESTNODEALT2,

    /// Source: `oracle/oracle/codemp/game/g_public.h:331`
    G_NAV_GETBESTPATHBETWEENENTS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:332`
    G_NAV_GETNODERADIUS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:333`
    G_NAV_CHECKBLOCKEDEDGES,

    /// Source: `oracle/oracle/codemp/game/g_public.h:334`
    G_NAV_CLEARCHECKEDNODES,

    /// Source: `oracle/oracle/codemp/game/g_public.h:335`
    G_NAV_CHECKEDNODE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:336`
    G_NAV_SETCHECKEDNODE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:337`
    G_NAV_FLAGALLNODES,

    /// Source: `oracle/oracle/codemp/game/g_public.h:338`
    G_NAV_GETPATHSCALCULATED,

    /// rww - END NPC NAV TRAPS
    /// Source: `oracle/oracle/codemp/game/g_public.h:339`
    G_NAV_SETPATHSCALCULATED,

    /// ( void );
    /// Source: `oracle/oracle/codemp/game/g_public.h:342`
    BOTLIB_SETUP = 250,

    /// ( void );
    /// Source: `oracle/oracle/codemp/game/g_public.h:343`
    BOTLIB_SHUTDOWN,

    /// Source: `oracle/oracle/codemp/game/g_public.h:344`
    BOTLIB_LIBVAR_SET,

    /// Source: `oracle/oracle/codemp/game/g_public.h:345`
    BOTLIB_LIBVAR_GET,

    /// Source: `oracle/oracle/codemp/game/g_public.h:346`
    BOTLIB_PC_ADD_GLOBAL_DEFINE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:347`
    BOTLIB_START_FRAME,

    /// Source: `oracle/oracle/codemp/game/g_public.h:348`
    BOTLIB_LOAD_MAP,

    /// Source: `oracle/oracle/codemp/game/g_public.h:349`
    BOTLIB_UPDATENTITY,

    /// Source: `oracle/oracle/codemp/game/g_public.h:350`
    BOTLIB_TEST,

    /// ( int client, int ent );
    /// Source: `oracle/oracle/codemp/game/g_public.h:352`
    BOTLIB_GET_SNAPSHOT_ENTITY,

    /// ( int client, char *message, int size );
    /// Source: `oracle/oracle/codemp/game/g_public.h:353`
    BOTLIB_GET_CONSOLE_MESSAGE,

    /// ( int client, usercmd_t *ucmd );
    /// Source: `oracle/oracle/codemp/game/g_public.h:354`
    BOTLIB_USER_COMMAND,

    /// Source: `oracle/oracle/codemp/game/g_public.h:356`
    BOTLIB_AAS_ENABLE_ROUTING_AREA = 300,

    /// Source: `oracle/oracle/codemp/game/g_public.h:357`
    BOTLIB_AAS_BBOX_AREAS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:358`
    BOTLIB_AAS_AREA_INFO,

    /// Source: `oracle/oracle/codemp/game/g_public.h:359`
    BOTLIB_AAS_ENTITY_INFO,

    /// Source: `oracle/oracle/codemp/game/g_public.h:361`
    BOTLIB_AAS_INITIALIZED,

    /// Source: `oracle/oracle/codemp/game/g_public.h:362`
    BOTLIB_AAS_PRESENCE_TYPE_BOUNDING_BOX,

    /// Source: `oracle/oracle/codemp/game/g_public.h:363`
    BOTLIB_AAS_TIME,

    /// Source: `oracle/oracle/codemp/game/g_public.h:365`
    BOTLIB_AAS_POINT_AREA_NUM,

    /// Source: `oracle/oracle/codemp/game/g_public.h:366`
    BOTLIB_AAS_TRACE_AREAS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:368`
    BOTLIB_AAS_POINT_CONTENTS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:369`
    BOTLIB_AAS_NEXT_BSP_ENTITY,

    /// Source: `oracle/oracle/codemp/game/g_public.h:370`
    BOTLIB_AAS_VALUE_FOR_BSP_EPAIR_KEY,

    /// Source: `oracle/oracle/codemp/game/g_public.h:371`
    BOTLIB_AAS_VECTOR_FOR_BSP_EPAIR_KEY,

    /// Source: `oracle/oracle/codemp/game/g_public.h:372`
    BOTLIB_AAS_FLOAT_FOR_BSP_EPAIR_KEY,

    /// Source: `oracle/oracle/codemp/game/g_public.h:373`
    BOTLIB_AAS_INT_FOR_BSP_EPAIR_KEY,

    /// Source: `oracle/oracle/codemp/game/g_public.h:375`
    BOTLIB_AAS_AREA_REACHABILITY,

    /// Source: `oracle/oracle/codemp/game/g_public.h:377`
    BOTLIB_AAS_AREA_TRAVEL_TIME_TO_GOAL_AREA,

    /// Source: `oracle/oracle/codemp/game/g_public.h:379`
    BOTLIB_AAS_SWIMMING,

    /// Source: `oracle/oracle/codemp/game/g_public.h:380`
    BOTLIB_AAS_PREDICT_CLIENT_MOVEMENT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:382`
    BOTLIB_EA_SAY = 400,

    /// Source: `oracle/oracle/codemp/game/g_public.h:383`
    BOTLIB_EA_SAY_TEAM,

    /// Source: `oracle/oracle/codemp/game/g_public.h:384`
    BOTLIB_EA_COMMAND,

    /// Source: `oracle/oracle/codemp/game/g_public.h:386`
    BOTLIB_EA_ACTION,

    /// Source: `oracle/oracle/codemp/game/g_public.h:387`
    BOTLIB_EA_GESTURE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:388`
    BOTLIB_EA_TALK,

    /// Source: `oracle/oracle/codemp/game/g_public.h:389`
    BOTLIB_EA_ATTACK,

    /// Source: `oracle/oracle/codemp/game/g_public.h:390`
    BOTLIB_EA_ALT_ATTACK,

    /// Source: `oracle/oracle/codemp/game/g_public.h:391`
    BOTLIB_EA_FORCEPOWER,

    /// Source: `oracle/oracle/codemp/game/g_public.h:392`
    BOTLIB_EA_USE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:393`
    BOTLIB_EA_RESPAWN,

    /// Source: `oracle/oracle/codemp/game/g_public.h:394`
    BOTLIB_EA_CROUCH,

    /// Source: `oracle/oracle/codemp/game/g_public.h:395`
    BOTLIB_EA_MOVE_UP,

    /// Source: `oracle/oracle/codemp/game/g_public.h:396`
    BOTLIB_EA_MOVE_DOWN,

    /// Source: `oracle/oracle/codemp/game/g_public.h:397`
    BOTLIB_EA_MOVE_FORWARD,

    /// Source: `oracle/oracle/codemp/game/g_public.h:398`
    BOTLIB_EA_MOVE_BACK,

    /// Source: `oracle/oracle/codemp/game/g_public.h:399`
    BOTLIB_EA_MOVE_LEFT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:400`
    BOTLIB_EA_MOVE_RIGHT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:402`
    BOTLIB_EA_SELECT_WEAPON,

    /// Source: `oracle/oracle/codemp/game/g_public.h:403`
    BOTLIB_EA_JUMP,

    /// Source: `oracle/oracle/codemp/game/g_public.h:404`
    BOTLIB_EA_DELAYED_JUMP,

    /// Source: `oracle/oracle/codemp/game/g_public.h:405`
    BOTLIB_EA_MOVE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:406`
    BOTLIB_EA_VIEW,

    /// Source: `oracle/oracle/codemp/game/g_public.h:408`
    BOTLIB_EA_END_REGULAR,

    /// Source: `oracle/oracle/codemp/game/g_public.h:409`
    BOTLIB_EA_GET_INPUT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:410`
    BOTLIB_EA_RESET_INPUT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:413`
    BOTLIB_AI_LOAD_CHARACTER = 500,

    /// Source: `oracle/oracle/codemp/game/g_public.h:414`
    BOTLIB_AI_FREE_CHARACTER,

    /// Source: `oracle/oracle/codemp/game/g_public.h:415`
    BOTLIB_AI_CHARACTERISTIC_FLOAT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:416`
    BOTLIB_AI_CHARACTERISTIC_BFLOAT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:417`
    BOTLIB_AI_CHARACTERISTIC_INTEGER,

    /// Source: `oracle/oracle/codemp/game/g_public.h:418`
    BOTLIB_AI_CHARACTERISTIC_BINTEGER,

    /// Source: `oracle/oracle/codemp/game/g_public.h:419`
    BOTLIB_AI_CHARACTERISTIC_STRING,

    /// Source: `oracle/oracle/codemp/game/g_public.h:421`
    BOTLIB_AI_ALLOC_CHAT_STATE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:422`
    BOTLIB_AI_FREE_CHAT_STATE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:423`
    BOTLIB_AI_QUEUE_CONSOLE_MESSAGE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:424`
    BOTLIB_AI_REMOVE_CONSOLE_MESSAGE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:425`
    BOTLIB_AI_NEXT_CONSOLE_MESSAGE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:426`
    BOTLIB_AI_NUM_CONSOLE_MESSAGE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:427`
    BOTLIB_AI_INITIAL_CHAT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:428`
    BOTLIB_AI_REPLY_CHAT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:429`
    BOTLIB_AI_CHAT_LENGTH,

    /// Source: `oracle/oracle/codemp/game/g_public.h:430`
    BOTLIB_AI_ENTER_CHAT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:431`
    BOTLIB_AI_STRING_CONTAINS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:432`
    BOTLIB_AI_FIND_MATCH,

    /// Source: `oracle/oracle/codemp/game/g_public.h:433`
    BOTLIB_AI_MATCH_VARIABLE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:434`
    BOTLIB_AI_UNIFY_WHITE_SPACES,

    /// Source: `oracle/oracle/codemp/game/g_public.h:435`
    BOTLIB_AI_REPLACE_SYNONYMS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:436`
    BOTLIB_AI_LOAD_CHAT_FILE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:437`
    BOTLIB_AI_SET_CHAT_GENDER,

    /// Source: `oracle/oracle/codemp/game/g_public.h:438`
    BOTLIB_AI_SET_CHAT_NAME,

    /// Source: `oracle/oracle/codemp/game/g_public.h:440`
    BOTLIB_AI_RESET_GOAL_STATE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:441`
    BOTLIB_AI_RESET_AVOID_GOALS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:442`
    BOTLIB_AI_PUSH_GOAL,

    /// Source: `oracle/oracle/codemp/game/g_public.h:443`
    BOTLIB_AI_POP_GOAL,

    /// Source: `oracle/oracle/codemp/game/g_public.h:444`
    BOTLIB_AI_EMPTY_GOAL_STACK,

    /// Source: `oracle/oracle/codemp/game/g_public.h:445`
    BOTLIB_AI_DUMP_AVOID_GOALS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:446`
    BOTLIB_AI_DUMP_GOAL_STACK,

    /// Source: `oracle/oracle/codemp/game/g_public.h:447`
    BOTLIB_AI_GOAL_NAME,

    /// Source: `oracle/oracle/codemp/game/g_public.h:448`
    BOTLIB_AI_GET_TOP_GOAL,

    /// Source: `oracle/oracle/codemp/game/g_public.h:449`
    BOTLIB_AI_GET_SECOND_GOAL,

    /// Source: `oracle/oracle/codemp/game/g_public.h:450`
    BOTLIB_AI_CHOOSE_LTG_ITEM,

    /// Source: `oracle/oracle/codemp/game/g_public.h:451`
    BOTLIB_AI_CHOOSE_NBG_ITEM,

    /// Source: `oracle/oracle/codemp/game/g_public.h:452`
    BOTLIB_AI_TOUCHING_GOAL,

    /// Source: `oracle/oracle/codemp/game/g_public.h:453`
    BOTLIB_AI_ITEM_GOAL_IN_VIS_BUT_NOT_VISIBLE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:454`
    BOTLIB_AI_GET_LEVEL_ITEM_GOAL,

    /// Source: `oracle/oracle/codemp/game/g_public.h:455`
    BOTLIB_AI_AVOID_GOAL_TIME,

    /// Source: `oracle/oracle/codemp/game/g_public.h:456`
    BOTLIB_AI_INIT_LEVEL_ITEMS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:457`
    BOTLIB_AI_UPDATE_ENTITY_ITEMS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:458`
    BOTLIB_AI_LOAD_ITEM_WEIGHTS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:459`
    BOTLIB_AI_FREE_ITEM_WEIGHTS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:460`
    BOTLIB_AI_SAVE_GOAL_FUZZY_LOGIC,

    /// Source: `oracle/oracle/codemp/game/g_public.h:461`
    BOTLIB_AI_ALLOC_GOAL_STATE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:462`
    BOTLIB_AI_FREE_GOAL_STATE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:464`
    BOTLIB_AI_RESET_MOVE_STATE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:465`
    BOTLIB_AI_MOVE_TO_GOAL,

    /// Source: `oracle/oracle/codemp/game/g_public.h:466`
    BOTLIB_AI_MOVE_IN_DIRECTION,

    /// Source: `oracle/oracle/codemp/game/g_public.h:467`
    BOTLIB_AI_RESET_AVOID_REACH,

    /// Source: `oracle/oracle/codemp/game/g_public.h:468`
    BOTLIB_AI_RESET_LAST_AVOID_REACH,

    /// Source: `oracle/oracle/codemp/game/g_public.h:469`
    BOTLIB_AI_REACHABILITY_AREA,

    /// Source: `oracle/oracle/codemp/game/g_public.h:470`
    BOTLIB_AI_MOVEMENT_VIEW_TARGET,

    /// Source: `oracle/oracle/codemp/game/g_public.h:471`
    BOTLIB_AI_ALLOC_MOVE_STATE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:472`
    BOTLIB_AI_FREE_MOVE_STATE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:473`
    BOTLIB_AI_INIT_MOVE_STATE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:475`
    BOTLIB_AI_CHOOSE_BEST_FIGHT_WEAPON,

    /// Source: `oracle/oracle/codemp/game/g_public.h:476`
    BOTLIB_AI_GET_WEAPON_INFO,

    /// Source: `oracle/oracle/codemp/game/g_public.h:477`
    BOTLIB_AI_LOAD_WEAPON_WEIGHTS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:478`
    BOTLIB_AI_ALLOC_WEAPON_STATE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:479`
    BOTLIB_AI_FREE_WEAPON_STATE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:480`
    BOTLIB_AI_RESET_WEAPON_STATE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:482`
    BOTLIB_AI_GENETIC_PARENTS_AND_CHILD_SELECTION,

    /// Source: `oracle/oracle/codemp/game/g_public.h:483`
    BOTLIB_AI_INTERBREED_GOAL_FUZZY_LOGIC,

    /// Source: `oracle/oracle/codemp/game/g_public.h:484`
    BOTLIB_AI_MUTATE_GOAL_FUZZY_LOGIC,

    /// Source: `oracle/oracle/codemp/game/g_public.h:485`
    BOTLIB_AI_GET_NEXT_CAMP_SPOT_GOAL,

    /// Source: `oracle/oracle/codemp/game/g_public.h:486`
    BOTLIB_AI_GET_MAP_LOCATION_GOAL,

    /// Source: `oracle/oracle/codemp/game/g_public.h:487`
    BOTLIB_AI_NUM_INITIAL_CHATS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:488`
    BOTLIB_AI_GET_CHAT_MESSAGE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:489`
    BOTLIB_AI_REMOVE_FROM_AVOID_GOALS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:490`
    BOTLIB_AI_PREDICT_VISIBLE_POSITION,

    /// Source: `oracle/oracle/codemp/game/g_public.h:492`
    BOTLIB_AI_SET_AVOID_GOAL_TIME,

    /// Source: `oracle/oracle/codemp/game/g_public.h:493`
    BOTLIB_AI_ADD_AVOID_SPOT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:494`
    BOTLIB_AAS_ALTERNATIVE_ROUTE_GOAL,

    /// Source: `oracle/oracle/codemp/game/g_public.h:495`
    BOTLIB_AAS_PREDICT_ROUTE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:496`
    BOTLIB_AAS_POINT_REACHABILITY_AREA_INDEX,

    /// Source: `oracle/oracle/codemp/game/g_public.h:498`
    BOTLIB_PC_LOAD_SOURCE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:499`
    BOTLIB_PC_FREE_SOURCE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:500`
    BOTLIB_PC_READ_TOKEN,

    /// Ghoul2 Insert Start
    /// Source: `oracle/oracle/codemp/game/g_public.h:501`
    BOTLIB_PC_SOURCE_FILE_AND_LINE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:506`
    G_R_REGISTERSKIN,

    /// Source: `oracle/oracle/codemp/game/g_public.h:507`
    G_G2_LISTBONES,

    /// Source: `oracle/oracle/codemp/game/g_public.h:508`
    G_G2_LISTSURFACES,

    /// Source: `oracle/oracle/codemp/game/g_public.h:509`
    G_G2_HAVEWEGHOULMODELS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:510`
    G_G2_SETMODELS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:511`
    G_G2_GETBOLT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:512`
    G_G2_GETBOLT_NOREC,

    /// Source: `oracle/oracle/codemp/game/g_public.h:513`
    G_G2_GETBOLT_NOREC_NOROT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:514`
    G_G2_INITGHOUL2MODEL,

    /// Source: `oracle/oracle/codemp/game/g_public.h:515`
    G_G2_SETSKIN,

    /// Source: `oracle/oracle/codemp/game/g_public.h:516`
    G_G2_SIZE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:517`
    G_G2_ADDBOLT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:518`
    G_G2_SETBOLTINFO,

    /// Source: `oracle/oracle/codemp/game/g_public.h:519`
    G_G2_ANGLEOVERRIDE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:520`
    G_G2_PLAYANIM,

    /// Source: `oracle/oracle/codemp/game/g_public.h:521`
    G_G2_GETBONEANIM,

    /// Source: `oracle/oracle/codemp/game/g_public.h:522`
    G_G2_GETGLANAME,

    /// Source: `oracle/oracle/codemp/game/g_public.h:523`
    G_G2_COPYGHOUL2INSTANCE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:524`
    G_G2_COPYSPECIFICGHOUL2MODEL,

    /// Source: `oracle/oracle/codemp/game/g_public.h:525`
    G_G2_DUPLICATEGHOUL2INSTANCE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:526`
    G_G2_HASGHOUL2MODELONINDEX,

    /// Source: `oracle/oracle/codemp/game/g_public.h:527`
    G_G2_REMOVEGHOUL2MODEL,

    /// Source: `oracle/oracle/codemp/game/g_public.h:528`
    G_G2_REMOVEGHOUL2MODELS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:529`
    G_G2_CLEANMODELS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:530`
    G_G2_COLLISIONDETECT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:531`
    G_G2_COLLISIONDETECTCACHE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:533`
    G_G2_SETROOTSURFACE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:534`
    G_G2_SETSURFACEONOFF,

    /// Source: `oracle/oracle/codemp/game/g_public.h:535`
    G_G2_SETNEWORIGIN,

    /// Source: `oracle/oracle/codemp/game/g_public.h:536`
    G_G2_DOESBONEEXIST,

    /// Source: `oracle/oracle/codemp/game/g_public.h:537`
    G_G2_GETSURFACERENDERSTATUS,

    /// rww - RAGDOLL_BEGIN
    /// Source: `oracle/oracle/codemp/game/g_public.h:539`
    G_G2_ABSURDSMOOTHING,

    /// Source: `oracle/oracle/codemp/game/g_public.h:544`
    G_G2_SETRAGDOLL,

    /// rww - RAGDOLL_END
    /// additional ragdoll options -rww
    /// Source: `oracle/oracle/codemp/game/g_public.h:545`
    G_G2_ANIMATEG2MODELS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:550`
    G_G2_RAGPCJCONSTRAINT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:551`
    G_G2_RAGPCJGRADIENTSPEED,

    /// Source: `oracle/oracle/codemp/game/g_public.h:552`
    G_G2_RAGEFFECTORGOAL,

    /// Source: `oracle/oracle/codemp/game/g_public.h:553`
    G_G2_GETRAGBONEPOS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:554`
    G_G2_RAGEFFECTORKICK,

    /// rww - ik move method, allows you to specify a bone and move it to a world point (within joint constraints)
    /// by using the majority of gil's existing bone angling stuff from the ragdoll code.
    /// Source: `oracle/oracle/codemp/game/g_public.h:555`
    G_G2_RAGFORCESOLVE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:559`
    G_G2_SETBONEIKSTATE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:560`
    G_G2_IKMOVE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:562`
    G_G2_REMOVEBONE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:564`
    G_G2_ATTACHINSTANCETOENTNUM,

    /// Source: `oracle/oracle/codemp/game/g_public.h:565`
    G_G2_CLEARATTACHEDINSTANCE,

    /// Source: `oracle/oracle/codemp/game/g_public.h:566`
    G_G2_CLEANENTATTACHMENTS,

    /// Source: `oracle/oracle/codemp/game/g_public.h:567`
    G_G2_OVERRIDESERVER,

    /// Source: `oracle/oracle/codemp/game/g_public.h:569`
    G_G2_GETSURFACENAME,

    /// Source: `oracle/oracle/codemp/game/g_public.h:571`
    G_SET_ACTIVE_SUBBSP,

    /// Source: `oracle/oracle/codemp/game/g_public.h:572`
    G_CM_REGISTER_TERRAIN,

    /// Source: `oracle/oracle/codemp/game/g_public.h:573`
    G_RMG_INIT,

    /// Source: `oracle/oracle/codemp/game/g_public.h:575`
    G_BOT_UPDATEWAYPOINTS,

    /// Ghoul2 Insert End
    /// Source: `oracle/oracle/codemp/game/g_public.h:576`
    G_BOT_CALCULATEPATHS,
}
