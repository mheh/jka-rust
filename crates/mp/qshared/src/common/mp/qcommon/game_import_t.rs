#![allow(non_camel_case_types, non_snake_case)]

/// Raven `gameImport_t` — enumeration of engine syscalls available to the game module.
///
/// Type definition source: `oracle/codemp/game/g_public.h:102-581`
#[repr(i32)]
pub enum gameImport_t {
    // ============== general Quake services ==================
    G_PRINT = 0, // ( const char *string );
    // print message on the local console
    G_ERROR, // ( const char *string );
    // abort the game
    G_MILLISECONDS, // ( void );
    // get current time for profiling reasons
    // this should NOT be used for any game related tasks,
    // because it is not journaled

    // Also for profiling.. do not use for game related tasks.
    G_PRECISIONTIMER_START,
    G_PRECISIONTIMER_END,

    // console variable interaction
    G_CVAR_REGISTER, // ( vmCvar_t *vmCvar, const char *varName, const char *defaultValue, int flags );
    G_CVAR_UPDATE,   // ( vmCvar_t *vmCvar );
    G_CVAR_SET,      // ( const char *var_name, const char *value );
    G_CVAR_VARIABLE_INTEGER_VALUE, // ( const char *var_name );

    G_CVAR_VARIABLE_STRING_BUFFER, // ( const char *var_name, char *buffer, int bufsize );

    G_ARGC, // ( void );
    // ClientCommand and ServerCommand parameter access
    G_ARGV, // ( int n, char *buffer, int bufferLength );

    G_FS_FOPEN_FILE,  // ( const char *qpath, fileHandle_t *file, fsMode_t mode );
    G_FS_READ,        // ( void *buffer, int len, fileHandle_t f );
    G_FS_WRITE,       // ( const void *buffer, int len, fileHandle_t f );
    G_FS_FCLOSE_FILE, // ( fileHandle_t f );

    G_SEND_CONSOLE_COMMAND, // ( const char *text );
    // add commands to the console as if they were typed in
    // for map changing, etc

    // =========== server specific functionality =============
    G_LOCATE_GAME_DATA, // ( gentity_t *gEnts, int numGEntities, int sizeofGEntity_t,
    // playersState_t *clients, int sizeofGameClient );
    // the game needs to let the server system know where and how big the gentities
    // are, so it can look at them directly without going through an interface
    G_DROP_CLIENT, // ( int clientNum, const char *reason );
    // kick a client off the server with a message
    G_SEND_SERVER_COMMAND, // ( int clientNum, const char *fmt, ... );
    // reliably sends a command string to be interpreted by the given
    // client.  If clientNum is -1, it will be sent to all clients
    G_SET_CONFIGSTRING, // ( int num, const char *string );
    // config strings hold all the index strings, and various other information
    // that is reliably communicated to all clients
    // All of the current configstrings are sent to clients when
    // they connect, and changes are sent to all connected clients.
    // All confgstrings are cleared at each level start.
    G_GET_CONFIGSTRING, // ( int num, char *buffer, int bufferSize );

    G_GET_USERINFO, // ( int num, char *buffer, int bufferSize );
    // userinfo strings are maintained by the server system, so they
    // are persistant across level loads, while all other game visible
    // data is completely reset
    G_SET_USERINFO, // ( int num, const char *buffer );

    G_GET_SERVERINFO, // ( char *buffer, int bufferSize );
    // the serverinfo info string has all the cvars visible to server browsers
    G_SET_SERVER_CULL, // server culling to reduce traffic on open maps -rww

    G_SET_BRUSH_MODEL, // ( gentity_t *ent, const char *name );
    // sets mins and maxs based on the brushmodel name
    G_TRACE, // ( trace_t *results, const vec3_t start, const vec3_t mins, const vec3_t maxs, const vec3_t end, int passEntityNum, int contentmask );
    // collision detection against all linked entities
    G_G2TRACE, // ( trace_t *results, const vec3_t start, const vec3_t mins, const vec3_t maxs, const vec3_t end, int passEntityNum, int contentmask );
    // collision detection against all linked entities with ghoul2 check
    G_POINT_CONTENTS, // ( const vec3_t point, int passEntityNum );
    // point contents against all linked entities
    G_IN_PVS, // ( const vec3_t p1, const vec3_t p2 );

    G_IN_PVS_IGNORE_PORTALS, // ( const vec3_t p1, const vec3_t p2 );

    G_ADJUST_AREA_PORTAL_STATE, // ( gentity_t *ent, qboolean open );

    G_AREAS_CONNECTED, // ( int area1, int area2 );

    G_LINKENTITY, // ( gentity_t *ent );
    // an entity will never be sent to a client or used for collision
    // if it is not passed to linkentity.  If the size, position, or
    // solidity changes, it must be relinked.
    G_UNLINKENTITY, // ( gentity_t *ent );
    // call before removing an interactive entity
    G_ENTITIES_IN_BOX, // ( const vec3_t mins, const vec3_t maxs, gentity_t **list, int maxcount );
    // EntitiesInBox will return brush models based on their bounding box,
    // so exact determination must still be done with EntityContact
    G_ENTITY_CONTACT, // ( const vec3_t mins, const vec3_t maxs, const gentity_t *ent );
    // perform an exact check against inline brush models of non-square shape

    // access for bots to get and free a server client (FIXME?)
    G_BOT_ALLOCATE_CLIENT, // ( void );

    G_BOT_FREE_CLIENT, // ( int clientNum );

    G_GET_USERCMD, // ( int clientNum, usercmd_t *cmd )

    G_GET_ENTITY_TOKEN, // qboolean ( char *buffer, int bufferSize )
    // Retrieves the next string token from the entity spawn text, returning
    // false when all tokens have been parsed.
    // This should only be done at GAME_INIT time.
    G_SIEGEPERSSET,
    G_SIEGEPERSGET,

    G_FS_GETFILELIST,
    G_DEBUG_POLYGON_CREATE,
    G_DEBUG_POLYGON_DELETE,
    G_REAL_TIME,
    G_SNAPVECTOR,

    G_TRACECAPSULE, // ( trace_t *results, const vec3_t start, const vec3_t mins, const vec3_t maxs, const vec3_t end, int passEntityNum, int contentmask );
    G_ENTITY_CONTACTCAPSULE, // ( const vec3_t mins, const vec3_t maxs, const gentity_t *ent );

    // SP_REGISTER_SERVER_CMD,
    SP_GETSTRINGTEXTSTRING,

    G_ROFF_CLEAN,           // qboolean ROFF_Clean(void);
    G_ROFF_UPDATE_ENTITIES, // void ROFF_UpdateEntities(void);
    G_ROFF_CACHE,           // int ROFF_Cache(char *file);
    G_ROFF_PLAY,            // qboolean ROFF_Play(int entID, int roffID, qboolean doTranslation);
    G_ROFF_PURGE_ENT,       // qboolean ROFF_PurgeEnt( int entID )

    // rww - dynamic vm memory allocation!
    G_TRUEMALLOC,
    G_TRUEFREE,

    // rww - icarus traps
    G_ICARUS_RUNSCRIPT,
    G_ICARUS_REGISTERSCRIPT,

    G_ICARUS_INIT,
    G_ICARUS_VALIDENT,
    G_ICARUS_ISINITIALIZED,
    G_ICARUS_MAINTAINTASKMANAGER,
    G_ICARUS_ISRUNNING,
    G_ICARUS_TASKIDPENDING,
    G_ICARUS_INITENT,
    G_ICARUS_FREEENT,
    G_ICARUS_ASSOCIATEENT,
    G_ICARUS_SHUTDOWN,
    G_ICARUS_TASKIDSET,
    G_ICARUS_TASKIDCOMPLETE,
    G_ICARUS_SETVAR,
    G_ICARUS_VARIABLEDECLARED,
    G_ICARUS_GETFLOATVARIABLE,
    G_ICARUS_GETSTRINGVARIABLE,
    G_ICARUS_GETVECTORVARIABLE,

    G_SET_SHARED_BUFFER,

    // BEGIN VM STUFF
    G_MEMSET = 100,
    G_MEMCPY,
    G_STRNCPY,
    G_SIN,
    G_COS,
    G_ATAN2,
    G_SQRT,
    G_MATRIXMULTIPLY,
    G_ANGLEVECTORS,
    G_PERPENDICULARVECTOR,
    G_FLOOR,
    G_CEIL,

    G_TESTPRINTINT,
    G_TESTPRINTFLOAT,

    G_ACOS,
    G_ASIN,

    // END VM STUFF

    // rww - BEGIN NPC NAV TRAPS
    G_NAV_INIT = 200,
    G_NAV_FREE,
    G_NAV_LOAD,
    G_NAV_SAVE,
    G_NAV_ADDRAWPOINT,
    G_NAV_CALCULATEPATHS,
    G_NAV_HARDCONNECT,
    G_NAV_SHOWNODES,
    G_NAV_SHOWEDGES,
    G_NAV_SHOWPATH,
    G_NAV_GETNEARESTNODE,
    G_NAV_GETBESTNODE,
    G_NAV_GETNODEPOSITION,
    G_NAV_GETNODENUMEDGES,
    G_NAV_GETNODEEDGE,
    G_NAV_GETNUMNODES,
    G_NAV_CONNECTED,
    G_NAV_GETPATHCOST,
    G_NAV_GETEDGECOST,
    G_NAV_GETPROJECTEDNODE,
    G_NAV_CHECKFAILEDNODES,
    G_NAV_ADDFAILEDNODE,
    G_NAV_NODEFAILED,
    G_NAV_NODESARENEIGHBORS,
    G_NAV_CLEARFAILEDEDGE,
    G_NAV_CLEARALLFAILEDEDGES,
    G_NAV_EDGEFAILED,
    G_NAV_ADDFAILEDEDGE,
    G_NAV_CHECKFAILEDEDGE,
    G_NAV_CHECKALLFAILEDEDGES,
    G_NAV_ROUTEBLOCKED,
    G_NAV_GETBESTNODEALTROUTE,
    G_NAV_GETBESTNODEALT2,
    G_NAV_GETBESTPATHBETWEENENTS,
    G_NAV_GETNODERADIUS,
    G_NAV_CHECKBLOCKEDEDGES,
    G_NAV_CLEARCHECKEDNODES,
    G_NAV_CHECKEDNODE,
    G_NAV_SETCHECKEDNODE,
    G_NAV_FLAGALLNODES,
    G_NAV_GETPATHSCALCULATED,
    G_NAV_SETPATHSCALCULATED,
    // rww - END NPC NAV TRAPS
    BOTLIB_SETUP = 250, // ( void );
    BOTLIB_SHUTDOWN,    // ( void );
    BOTLIB_LIBVAR_SET,
    BOTLIB_LIBVAR_GET,
    BOTLIB_PC_ADD_GLOBAL_DEFINE,
    BOTLIB_START_FRAME,
    BOTLIB_LOAD_MAP,
    BOTLIB_UPDATENTITY,
    BOTLIB_TEST,

    BOTLIB_GET_SNAPSHOT_ENTITY, // ( int client, int ent );
    BOTLIB_GET_CONSOLE_MESSAGE, // ( int client, char *message, int size );
    BOTLIB_USER_COMMAND,        // ( int client, usercmd_t *ucmd );

    BOTLIB_AAS_ENABLE_ROUTING_AREA = 300,
    BOTLIB_AAS_BBOX_AREAS,
    BOTLIB_AAS_AREA_INFO,
    BOTLIB_AAS_ENTITY_INFO,

    BOTLIB_AAS_INITIALIZED,
    BOTLIB_AAS_PRESENCE_TYPE_BOUNDING_BOX,
    BOTLIB_AAS_TIME,

    BOTLIB_AAS_POINT_AREA_NUM,
    BOTLIB_AAS_TRACE_AREAS,

    BOTLIB_AAS_POINT_CONTENTS,
    BOTLIB_AAS_NEXT_BSP_ENTITY,
    BOTLIB_AAS_VALUE_FOR_BSP_EPAIR_KEY,
    BOTLIB_AAS_VECTOR_FOR_BSP_EPAIR_KEY,
    BOTLIB_AAS_FLOAT_FOR_BSP_EPAIR_KEY,
    BOTLIB_AAS_INT_FOR_BSP_EPAIR_KEY,

    BOTLIB_AAS_AREA_REACHABILITY,

    BOTLIB_AAS_AREA_TRAVEL_TIME_TO_GOAL_AREA,

    BOTLIB_AAS_SWIMMING,
    BOTLIB_AAS_PREDICT_CLIENT_MOVEMENT,

    BOTLIB_EA_SAY = 400,
    BOTLIB_EA_SAY_TEAM,
    BOTLIB_EA_COMMAND,

    BOTLIB_EA_ACTION,
    BOTLIB_EA_GESTURE,
    BOTLIB_EA_TALK,
    BOTLIB_EA_ATTACK,
    BOTLIB_EA_ALT_ATTACK,
    BOTLIB_EA_FORCEPOWER,
    BOTLIB_EA_USE,
    BOTLIB_EA_RESPAWN,
    BOTLIB_EA_CROUCH,
    BOTLIB_EA_MOVE_UP,
    BOTLIB_EA_MOVE_DOWN,
    BOTLIB_EA_MOVE_FORWARD,
    BOTLIB_EA_MOVE_BACK,
    BOTLIB_EA_MOVE_LEFT,
    BOTLIB_EA_MOVE_RIGHT,

    BOTLIB_EA_SELECT_WEAPON,
    BOTLIB_EA_JUMP,
    BOTLIB_EA_DELAYED_JUMP,
    BOTLIB_EA_MOVE,
    BOTLIB_EA_VIEW,

    BOTLIB_EA_END_REGULAR,
    BOTLIB_EA_GET_INPUT,
    BOTLIB_EA_RESET_INPUT,

    BOTLIB_AI_LOAD_CHARACTER = 500,
    BOTLIB_AI_FREE_CHARACTER,
    BOTLIB_AI_CHARACTERISTIC_FLOAT,
    BOTLIB_AI_CHARACTERISTIC_BFLOAT,
    BOTLIB_AI_CHARACTERISTIC_INTEGER,
    BOTLIB_AI_CHARACTERISTIC_BINTEGER,
    BOTLIB_AI_CHARACTERISTIC_STRING,

    BOTLIB_AI_ALLOC_CHAT_STATE,
    BOTLIB_AI_FREE_CHAT_STATE,
    BOTLIB_AI_QUEUE_CONSOLE_MESSAGE,
    BOTLIB_AI_REMOVE_CONSOLE_MESSAGE,
    BOTLIB_AI_NEXT_CONSOLE_MESSAGE,
    BOTLIB_AI_NUM_CONSOLE_MESSAGE,
    BOTLIB_AI_INITIAL_CHAT,
    BOTLIB_AI_REPLY_CHAT,
    BOTLIB_AI_CHAT_LENGTH,
    BOTLIB_AI_ENTER_CHAT,
    BOTLIB_AI_STRING_CONTAINS,
    BOTLIB_AI_FIND_MATCH,
    BOTLIB_AI_MATCH_VARIABLE,
    BOTLIB_AI_UNIFY_WHITE_SPACES,
    BOTLIB_AI_REPLACE_SYNONYMS,
    BOTLIB_AI_LOAD_CHAT_FILE,
    BOTLIB_AI_SET_CHAT_GENDER,
    BOTLIB_AI_SET_CHAT_NAME,

    BOTLIB_AI_RESET_GOAL_STATE,
    BOTLIB_AI_RESET_AVOID_GOALS,
    BOTLIB_AI_PUSH_GOAL,
    BOTLIB_AI_POP_GOAL,
    BOTLIB_AI_EMPTY_GOAL_STACK,
    BOTLIB_AI_DUMP_AVOID_GOALS,
    BOTLIB_AI_DUMP_GOAL_STACK,
    BOTLIB_AI_GOAL_NAME,
    BOTLIB_AI_GET_TOP_GOAL,
    BOTLIB_AI_GET_SECOND_GOAL,
    BOTLIB_AI_CHOOSE_LTG_ITEM,
    BOTLIB_AI_CHOOSE_NBG_ITEM,
    BOTLIB_AI_TOUCHING_GOAL,
    BOTLIB_AI_ITEM_GOAL_IN_VIS_BUT_NOT_VISIBLE,
    BOTLIB_AI_GET_LEVEL_ITEM_GOAL,
    BOTLIB_AI_AVOID_GOAL_TIME,
    BOTLIB_AI_INIT_LEVEL_ITEMS,
    BOTLIB_AI_UPDATE_ENTITY_ITEMS,
    BOTLIB_AI_LOAD_ITEM_WEIGHTS,
    BOTLIB_AI_FREE_ITEM_WEIGHTS,
    BOTLIB_AI_SAVE_GOAL_FUZZY_LOGIC,
    BOTLIB_AI_ALLOC_GOAL_STATE,
    BOTLIB_AI_FREE_GOAL_STATE,

    BOTLIB_AI_RESET_MOVE_STATE,
    BOTLIB_AI_MOVE_TO_GOAL,
    BOTLIB_AI_MOVE_IN_DIRECTION,
    BOTLIB_AI_RESET_AVOID_REACH,
    BOTLIB_AI_RESET_LAST_AVOID_REACH,
    BOTLIB_AI_REACHABILITY_AREA,
    BOTLIB_AI_MOVEMENT_VIEW_TARGET,
    BOTLIB_AI_ALLOC_MOVE_STATE,
    BOTLIB_AI_FREE_MOVE_STATE,
    BOTLIB_AI_INIT_MOVE_STATE,

    BOTLIB_AI_CHOOSE_BEST_FIGHT_WEAPON,
    BOTLIB_AI_GET_WEAPON_INFO,
    BOTLIB_AI_LOAD_WEAPON_WEIGHTS,
    BOTLIB_AI_ALLOC_WEAPON_STATE,
    BOTLIB_AI_FREE_WEAPON_STATE,
    BOTLIB_AI_RESET_WEAPON_STATE,

    BOTLIB_AI_GENETIC_PARENTS_AND_CHILD_SELECTION,
    BOTLIB_AI_INTERBREED_GOAL_FUZZY_LOGIC,
    BOTLIB_AI_MUTATE_GOAL_FUZZY_LOGIC,
    BOTLIB_AI_GET_NEXT_CAMP_SPOT_GOAL,
    BOTLIB_AI_GET_MAP_LOCATION_GOAL,
    BOTLIB_AI_NUM_INITIAL_CHATS,
    BOTLIB_AI_GET_CHAT_MESSAGE,
    BOTLIB_AI_REMOVE_FROM_AVOID_GOALS,
    BOTLIB_AI_PREDICT_VISIBLE_POSITION,

    BOTLIB_AI_SET_AVOID_GOAL_TIME,
    BOTLIB_AI_ADD_AVOID_SPOT,
    BOTLIB_AAS_ALTERNATIVE_ROUTE_GOAL,
    BOTLIB_AAS_PREDICT_ROUTE,
    BOTLIB_AAS_POINT_REACHABILITY_AREA_INDEX,

    BOTLIB_PC_LOAD_SOURCE,
    BOTLIB_PC_FREE_SOURCE,
    BOTLIB_PC_READ_TOKEN,
    BOTLIB_PC_SOURCE_FILE_AND_LINE,

    /*
    Ghoul2 Insert Start
    */
    G_R_REGISTERSKIN,
    G_G2_LISTBONES,
    G_G2_LISTSURFACES,
    G_G2_HAVEWEGHOULMODELS,
    G_G2_SETMODELS,
    G_G2_GETBOLT,
    G_G2_GETBOLT_NOREC,
    G_G2_GETBOLT_NOREC_NOROT,
    G_G2_INITGHOUL2MODEL,
    G_G2_SETSKIN,
    G_G2_SIZE,
    G_G2_ADDBOLT,
    G_G2_SETBOLTINFO,
    G_G2_ANGLEOVERRIDE,
    G_G2_PLAYANIM,
    G_G2_GETBONEANIM,
    G_G2_GETGLANAME,
    G_G2_COPYGHOUL2INSTANCE,
    G_G2_COPYSPECIFICGHOUL2MODEL,
    G_G2_DUPLICATEGHOUL2INSTANCE,
    G_G2_HASGHOUL2MODELONINDEX,
    G_G2_REMOVEGHOUL2MODEL,
    G_G2_REMOVEGHOUL2MODELS,
    G_G2_CLEANMODELS,
    G_G2_COLLISIONDETECT,
    G_G2_COLLISIONDETECTCACHE,

    G_G2_SETROOTSURFACE,
    G_G2_SETSURFACEONOFF,
    G_G2_SETNEWORIGIN,
    G_G2_DOESBONEEXIST,
    G_G2_GETSURFACERENDERSTATUS,

    G_G2_ABSURDSMOOTHING,

    /*
    //rww - RAGDOLL_BEGIN
    	*/
    G_G2_SETRAGDOLL,
    G_G2_ANIMATEG2MODELS,
    /*
    //rww - RAGDOLL_END
    	*/
    // additional ragdoll options -rww
    G_G2_RAGPCJCONSTRAINT,
    G_G2_RAGPCJGRADIENTSPEED,
    G_G2_RAGEFFECTORGOAL,
    G_G2_GETRAGBONEPOS,
    G_G2_RAGEFFECTORKICK,
    G_G2_RAGFORCESOLVE,

    // rww - ik move method, allows you to specify a bone and move it to a world point (within joint constraints)
    // by using the majority of gil's existing bone angling stuff from the ragdoll code.
    G_G2_SETBONEIKSTATE,
    G_G2_IKMOVE,

    G_G2_REMOVEBONE,

    G_G2_ATTACHINSTANCETOENTNUM,
    G_G2_CLEARATTACHEDINSTANCE,
    G_G2_CLEANENTATTACHMENTS,
    G_G2_OVERRIDESERVER,

    G_G2_GETSURFACENAME,

    G_SET_ACTIVE_SUBBSP,
    G_CM_REGISTER_TERRAIN,
    G_RMG_INIT,

    G_BOT_UPDATEWAYPOINTS,
    G_BOT_CALCULATEPATHS,
    /*
    Ghoul2 Insert End
    */
}
