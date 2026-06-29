# Trap definitions (game→engine syscalls)

Generated from `oracle/codemp/game/g_syscalls.c` (Raven JKA). Each row is a
`trap_*` wrapper — its C return type and parameter list, the authoritative
signature for that call’s typed ABI definition. `shape` is a heuristic
typing bucket: scalar / string / ptr / opaque.

| # | trap | return | parameters | shape |
|---|---|---|---|---|
| 1 | `trap_Printf` | `void` | `const char *fmt` | string |
| 2 | `trap_Error` | `void` | `const char *fmt` | string |
| 3 | `trap_Milliseconds` | `int` | `void` | scalar |
| 4 | `trap_PrecisionTimer_Start` | `void` | `void **theNewTimer` | opaque |
| 5 | `trap_PrecisionTimer_End` | `int` | `void *theTimer` | opaque |
| 6 | `trap_Cvar_Register` | `void` | `vmCvar_t *cvar, const char *var_name, const char *value, int flags` | string |
| 7 | `trap_Cvar_Update` | `void` | `vmCvar_t *cvar` | ptr |
| 8 | `trap_Cvar_Set` | `void` | `const char *var_name, const char *value` | string |
| 9 | `trap_Cvar_VariableIntegerValue` | `int` | `const char *var_name` | string |
| 10 | `trap_Cvar_VariableStringBuffer` | `void` | `const char *var_name, char *buffer, int bufsize` | string |
| 11 | `trap_Argc` | `int` | `void` | scalar |
| 12 | `trap_Argv` | `void` | `int n, char *buffer, int bufferLength` | string |
| 13 | `trap_FS_FOpenFile` | `int` | `const char *qpath, fileHandle_t *f, fsMode_t mode` | string |
| 14 | `trap_FS_Read` | `void` | `void *buffer, int len, fileHandle_t f` | opaque |
| 15 | `trap_FS_Write` | `void` | `const void *buffer, int len, fileHandle_t f` | opaque |
| 16 | `trap_FS_FCloseFile` | `void` | `fileHandle_t f` | scalar |
| 17 | `trap_SendConsoleCommand` | `void` | `int exec_when, const char *text` | string |
| 18 | `trap_LocateGameData` | `void` | `gentity_t *gEnts, int numGEntities, int sizeofGEntity_t, playerState_t *clients, int sizeofGClient` | ptr |
| 19 | `trap_DropClient` | `void` | `int clientNum, const char *reason` | string |
| 20 | `trap_SendServerCommand` | `void` | `int clientNum, const char *text` | string |
| 21 | `trap_SetConfigstring` | `void` | `int num, const char *string` | string |
| 22 | `trap_GetConfigstring` | `void` | `int num, char *buffer, int bufferSize` | string |
| 23 | `trap_GetUserinfo` | `void` | `int num, char *buffer, int bufferSize` | string |
| 24 | `trap_SetUserinfo` | `void` | `int num, const char *buffer` | string |
| 25 | `trap_GetServerinfo` | `void` | `char *buffer, int bufferSize` | string |
| 26 | `trap_SetServerCull` | `void` | `float cullDistance` | scalar |
| 27 | `trap_SetBrushModel` | `void` | `gentity_t *ent, const char *name` | string |
| 28 | `trap_Trace` | `void` | `trace_t *results, const vec3_t start, const vec3_t mins, const vec3_t maxs, const vec3_t end, int passEntityNum, int contentmask` | ptr |
| 29 | `trap_G2Trace` | `void` | `trace_t *results, const vec3_t start, const vec3_t mins, const vec3_t maxs, const vec3_t end, int passEntityNum, int contentmask, int g2TraceType, int traceLod` | ptr |
| 30 | `trap_PointContents` | `int` | `const vec3_t point, int passEntityNum` | scalar |
| 31 | `trap_InPVS` | `qboolean` | `const vec3_t p1, const vec3_t p2` | scalar |
| 32 | `trap_InPVSIgnorePortals` | `qboolean` | `const vec3_t p1, const vec3_t p2` | scalar |
| 33 | `trap_AdjustAreaPortalState` | `void` | `gentity_t *ent, qboolean open` | ptr |
| 34 | `trap_AreasConnected` | `qboolean` | `int area1, int area2` | scalar |
| 35 | `trap_LinkEntity` | `void` | `gentity_t *ent` | ptr |
| 36 | `trap_UnlinkEntity` | `void` | `gentity_t *ent` | ptr |
| 37 | `trap_EntitiesInBox` | `int` | `const vec3_t mins, const vec3_t maxs, int *list, int maxcount` | ptr |
| 38 | `trap_EntityContact` | `qboolean` | `const vec3_t mins, const vec3_t maxs, const gentity_t *ent` | ptr |
| 39 | `trap_BotAllocateClient` | `int` | `void` | scalar |
| 40 | `trap_BotFreeClient` | `void` | `int clientNum` | scalar |
| 41 | `trap_GetUsercmd` | `void` | `int clientNum, usercmd_t *cmd` | ptr |
| 42 | `trap_GetEntityToken` | `qboolean` | `char *buffer, int bufferSize` | string |
| 43 | `trap_SiegePersSet` | `void` | `siegePers_t *pers` | ptr |
| 44 | `trap_SiegePersGet` | `void` | `siegePers_t *pers` | ptr |
| 45 | `trap_FS_GetFileList` | `int` | `const char *path, const char *extension, char *listbuf, int bufsize` | string |
| 46 | `trap_DebugPolygonCreate` | `int` | `int color, int numPoints, vec3_t *points` | ptr |
| 47 | `trap_DebugPolygonDelete` | `void` | `int id` | scalar |
| 48 | `trap_RealTime` | `int` | `qtime_t *qtime` | ptr |
| 49 | `trap_SnapVector` | `void` | `float *v` | ptr |
| 50 | `trap_TraceCapsule` | `void` | `trace_t *results, const vec3_t start, const vec3_t mins, const vec3_t maxs, const vec3_t end, int passEntityNum, int contentmask` | ptr |
| 51 | `trap_EntityContactCapsule` | `qboolean` | `const vec3_t mins, const vec3_t maxs, const gentity_t *ent` | ptr |
| 52 | `trap_SP_GetStringTextString` | `int` | `const char *text, char *buffer, int bufferLength` | string |
| 53 | `trap_ROFF_Clean` | `qboolean` | `void` | scalar |
| 54 | `trap_ROFF_UpdateEntities` | `void` | `void` | scalar |
| 55 | `trap_ROFF_Cache` | `int` | `char *file` | string |
| 56 | `trap_ROFF_Play` | `qboolean` | `int entID, int roffID, qboolean doTranslation` | scalar |
| 57 | `trap_ROFF_Purge_Ent` | `qboolean` | `int entID` | scalar |
| 58 | `trap_TrueMalloc` | `void` | `void **ptr, int size` | opaque |
| 59 | `trap_TrueFree` | `void` | `void **ptr` | opaque |
| 60 | `trap_ICARUS_RunScript` | `int` | `gentity_t *ent, const char *name` | string |
| 61 | `trap_ICARUS_RegisterScript` | `qboolean` | `const char *name, qboolean bCalledDuringInterrogate` | string |
| 62 | `trap_ICARUS_Init` | `void` | `void` | scalar |
| 63 | `trap_ICARUS_ValidEnt` | `qboolean` | `gentity_t *ent` | ptr |
| 64 | `trap_ICARUS_IsInitialized` | `qboolean` | `int entID` | scalar |
| 65 | `trap_ICARUS_MaintainTaskManager` | `qboolean` | `int entID` | scalar |
| 66 | `trap_ICARUS_IsRunning` | `qboolean` | `int entID` | scalar |
| 67 | `trap_ICARUS_TaskIDPending` | `qboolean` | `gentity_t *ent, int taskID` | ptr |
| 68 | `trap_ICARUS_InitEnt` | `void` | `gentity_t *ent` | ptr |
| 69 | `trap_ICARUS_FreeEnt` | `void` | `gentity_t *ent` | ptr |
| 70 | `trap_ICARUS_AssociateEnt` | `void` | `gentity_t *ent` | ptr |
| 71 | `trap_ICARUS_Shutdown` | `void` | `void` | scalar |
| 72 | `trap_ICARUS_TaskIDSet` | `void` | `gentity_t *ent, int taskType, int taskID` | ptr |
| 73 | `trap_ICARUS_TaskIDComplete` | `void` | `gentity_t *ent, int taskType` | ptr |
| 74 | `trap_ICARUS_SetVar` | `void` | `int taskID, int entID, const char *type_name, const char *data` | string |
| 75 | `trap_ICARUS_VariableDeclared` | `int` | `const char *type_name` | string |
| 76 | `trap_ICARUS_GetFloatVariable` | `int` | `const char *name, float *value` | string |
| 77 | `trap_ICARUS_GetStringVariable` | `int` | `const char *name, const char *value` | string |
| 78 | `trap_ICARUS_GetVectorVariable` | `int` | `const char *name, const vec3_t value` | string |
| 79 | `trap_Nav_Init` | `void` | `void` | scalar |
| 80 | `trap_Nav_Free` | `void` | `void` | scalar |
| 81 | `trap_Nav_Load` | `qboolean` | `const char *filename, int checksum` | string |
| 82 | `trap_Nav_Save` | `qboolean` | `const char *filename, int checksum` | string |
| 83 | `trap_Nav_AddRawPoint` | `int` | `vec3_t point, int flags, int radius` | scalar |
| 84 | `trap_Nav_CalculatePaths` | `void` | `qboolean recalc` | scalar |
| 85 | `trap_Nav_HardConnect` | `void` | `int first, int second` | scalar |
| 86 | `trap_Nav_ShowNodes` | `void` | `void` | scalar |
| 87 | `trap_Nav_ShowEdges` | `void` | `void` | scalar |
| 88 | `trap_Nav_ShowPath` | `void` | `int start, int end` | scalar |
| 89 | `trap_Nav_GetNearestNode` | `int` | `gentity_t *ent, int lastID, int flags, int targetID` | ptr |
| 90 | `trap_Nav_GetBestNode` | `int` | `int startID, int endID, int rejectID` | scalar |
| 91 | `trap_Nav_GetNodePosition` | `int` | `int nodeID, vec3_t out` | scalar |
| 92 | `trap_Nav_GetNodeNumEdges` | `int` | `int nodeID` | scalar |
| 93 | `trap_Nav_GetNodeEdge` | `int` | `int nodeID, int edge` | scalar |
| 94 | `trap_Nav_GetNumNodes` | `int` | `void` | scalar |
| 95 | `trap_Nav_Connected` | `qboolean` | `int startID, int endID` | scalar |
| 96 | `trap_Nav_GetPathCost` | `int` | `int startID, int endID` | scalar |
| 97 | `trap_Nav_GetEdgeCost` | `int` | `int startID, int endID` | scalar |
| 98 | `trap_Nav_GetProjectedNode` | `int` | `vec3_t origin, int nodeID` | scalar |
| 99 | `trap_Nav_CheckFailedNodes` | `void` | `gentity_t *ent` | ptr |
| 100 | `trap_Nav_AddFailedNode` | `void` | `gentity_t *ent, int nodeID` | ptr |
| 101 | `trap_Nav_NodeFailed` | `qboolean` | `gentity_t *ent, int nodeID` | ptr |
| 102 | `trap_Nav_NodesAreNeighbors` | `qboolean` | `int startID, int endID` | scalar |
| 103 | `trap_Nav_ClearFailedEdge` | `void` | `failedEdge_t *failedEdge` | ptr |
| 104 | `trap_Nav_ClearAllFailedEdges` | `void` | `void` | scalar |
| 105 | `trap_Nav_EdgeFailed` | `int` | `int startID, int endID` | scalar |
| 106 | `trap_Nav_AddFailedEdge` | `void` | `int entID, int startID, int endID` | scalar |
| 107 | `trap_Nav_CheckFailedEdge` | `qboolean` | `failedEdge_t *failedEdge` | ptr |
| 108 | `trap_Nav_CheckAllFailedEdges` | `void` | `void` | scalar |
| 109 | `trap_Nav_RouteBlocked` | `qboolean` | `int startID, int testEdgeID, int endID, int rejectRank` | scalar |
| 110 | `trap_Nav_GetBestNodeAltRoute` | `int` | `int startID, int endID, int *pathCost, int rejectID` | ptr |
| 111 | `trap_Nav_GetBestNodeAltRoute2` | `int` | `int startID, int endID, int rejectID` | scalar |
| 112 | `trap_Nav_GetBestPathBetweenEnts` | `int` | `gentity_t *ent, gentity_t *goal, int flags` | ptr |
| 113 | `trap_Nav_GetNodeRadius` | `int` | `int nodeID` | scalar |
| 114 | `trap_Nav_CheckBlockedEdges` | `void` | `void` | scalar |
| 115 | `trap_Nav_ClearCheckedNodes` | `void` | `void` | scalar |
| 116 | `trap_Nav_CheckedNode` | `int` | `int wayPoint, int ent` | scalar |
| 117 | `trap_Nav_SetCheckedNode` | `void` | `int wayPoint, int ent, int value` | scalar |
| 118 | `trap_Nav_FlagAllNodes` | `void` | `int newFlag` | scalar |
| 119 | `trap_Nav_GetPathsCalculated` | `qboolean` | `void` | scalar |
| 120 | `trap_Nav_SetPathsCalculated` | `void` | `qboolean newVal` | scalar |
| 121 | `trap_SV_RegisterSharedMemory` | `void` | `char *memory` | string |
| 122 | `trap_BotLibSetup` | `int` | `void` | scalar |
| 123 | `trap_BotLibShutdown` | `int` | `void` | scalar |
| 124 | `trap_BotLibVarSet` | `int` | `char *var_name, char *value` | string |
| 125 | `trap_BotLibVarGet` | `int` | `char *var_name, char *value, int size` | string |
| 126 | `trap_BotLibDefine` | `int` | `char *string` | string |
| 127 | `trap_BotLibStartFrame` | `int` | `float time` | scalar |
| 128 | `trap_BotLibLoadMap` | `int` | `const char *mapname` | string |
| 129 | `trap_BotLibUpdateEntity` | `int` | `int ent, void /* struct bot_updateentity_s */ *bue` | opaque |
| 130 | `trap_BotLibTest` | `int` | `int parm0, char *parm1, vec3_t parm2, vec3_t parm3` | string |
| 131 | `trap_BotGetSnapshotEntity` | `int` | `int clientNum, int sequence` | scalar |
| 132 | `trap_BotGetServerCommand` | `int` | `int clientNum, char *message, int size` | string |
| 133 | `trap_BotUserCommand` | `void` | `int clientNum, usercmd_t *ucmd` | ptr |
| 134 | `trap_AAS_EntityInfo` | `void` | `int entnum, void /* struct aas_entityinfo_s */ *info` | opaque |
| 135 | `trap_AAS_Initialized` | `int` | `void` | scalar |
| 136 | `trap_AAS_PresenceTypeBoundingBox` | `void` | `int presencetype, vec3_t mins, vec3_t maxs` | scalar |
| 137 | `trap_AAS_Time` | `float` | `void` | scalar |
| 138 | `trap_AAS_PointAreaNum` | `int` | `vec3_t point` | scalar |
| 139 | `trap_AAS_PointReachabilityAreaIndex` | `int` | `vec3_t point` | scalar |
| 140 | `trap_AAS_TraceAreas` | `int` | `vec3_t start, vec3_t end, int *areas, vec3_t *points, int maxareas` | ptr |
| 141 | `trap_AAS_BBoxAreas` | `int` | `vec3_t absmins, vec3_t absmaxs, int *areas, int maxareas` | ptr |
| 142 | `trap_AAS_AreaInfo` | `int` | `int areanum, void /* struct aas_areainfo_s */ *info` | opaque |
| 143 | `trap_AAS_PointContents` | `int` | `vec3_t point` | scalar |
| 144 | `trap_AAS_NextBSPEntity` | `int` | `int ent` | scalar |
| 145 | `trap_AAS_ValueForBSPEpairKey` | `int` | `int ent, char *key, char *value, int size` | string |
| 146 | `trap_AAS_VectorForBSPEpairKey` | `int` | `int ent, char *key, vec3_t v` | string |
| 147 | `trap_AAS_FloatForBSPEpairKey` | `int` | `int ent, char *key, float *value` | string |
| 148 | `trap_AAS_IntForBSPEpairKey` | `int` | `int ent, char *key, int *value` | string |
| 149 | `trap_AAS_AreaReachability` | `int` | `int areanum` | scalar |
| 150 | `trap_AAS_AreaTravelTimeToGoalArea` | `int` | `int areanum, vec3_t origin, int goalareanum, int travelflags` | scalar |
| 151 | `trap_AAS_EnableRoutingArea` | `int` | `int areanum, int enable` | scalar |
| 152 | `trap_AAS_PredictRoute` | `int` | `void /*struct aas_predictroute_s*/ *route, int areanum, vec3_t origin, int goalareanum, int travelflags, int maxareas, int maxtime, int stopevent, int stopcontents, int stoptfl, int stopareanum` | opaque |
| 153 | `trap_AAS_AlternativeRouteGoals` | `int` | `vec3_t start, int startareanum, vec3_t goal, int goalareanum, int travelflags, void /*struct aas_altroutegoal_s*/ *altroutegoals, int maxaltroutegoals, int type` | opaque |
| 154 | `trap_AAS_Swimming` | `int` | `vec3_t origin` | scalar |
| 155 | `trap_AAS_PredictClientMovement` | `int` | `void /* struct aas_clientmove_s */ *move, int entnum, vec3_t origin, int presencetype, int onground, vec3_t velocity, vec3_t cmdmove, int cmdframes, int maxframes, float frametime, int stopevent, int stopareanum, int visualize` | opaque |
| 156 | `trap_EA_Say` | `void` | `int client, char *str` | string |
| 157 | `trap_EA_SayTeam` | `void` | `int client, char *str` | string |
| 158 | `trap_EA_Command` | `void` | `int client, char *command` | string |
| 159 | `trap_EA_Action` | `void` | `int client, int action` | scalar |
| 160 | `trap_EA_Gesture` | `void` | `int client` | scalar |
| 161 | `trap_EA_Talk` | `void` | `int client` | scalar |
| 162 | `trap_EA_Attack` | `void` | `int client` | scalar |
| 163 | `trap_EA_Alt_Attack` | `void` | `int client` | scalar |
| 164 | `trap_EA_ForcePower` | `void` | `int client` | scalar |
| 165 | `trap_EA_Use` | `void` | `int client` | scalar |
| 166 | `trap_EA_Respawn` | `void` | `int client` | scalar |
| 167 | `trap_EA_Crouch` | `void` | `int client` | scalar |
| 168 | `trap_EA_MoveUp` | `void` | `int client` | scalar |
| 169 | `trap_EA_MoveDown` | `void` | `int client` | scalar |
| 170 | `trap_EA_MoveForward` | `void` | `int client` | scalar |
| 171 | `trap_EA_MoveBack` | `void` | `int client` | scalar |
| 172 | `trap_EA_MoveLeft` | `void` | `int client` | scalar |
| 173 | `trap_EA_MoveRight` | `void` | `int client` | scalar |
| 174 | `trap_EA_SelectWeapon` | `void` | `int client, int weapon` | scalar |
| 175 | `trap_EA_Jump` | `void` | `int client` | scalar |
| 176 | `trap_EA_DelayedJump` | `void` | `int client` | scalar |
| 177 | `trap_EA_Move` | `void` | `int client, vec3_t dir, float speed` | scalar |
| 178 | `trap_EA_View` | `void` | `int client, vec3_t viewangles` | scalar |
| 179 | `trap_EA_EndRegular` | `void` | `int client, float thinktime` | scalar |
| 180 | `trap_EA_GetInput` | `void` | `int client, float thinktime, void /* struct bot_input_s */ *input` | opaque |
| 181 | `trap_EA_ResetInput` | `void` | `int client` | scalar |
| 182 | `trap_BotLoadCharacter` | `int` | `char *charfile, float skill` | string |
| 183 | `trap_BotFreeCharacter` | `void` | `int character` | scalar |
| 184 | `trap_Characteristic_Float` | `float` | `int character, int index` | scalar |
| 185 | `trap_Characteristic_BFloat` | `float` | `int character, int index, float min, float max` | scalar |
| 186 | `trap_Characteristic_Integer` | `int` | `int character, int index` | scalar |
| 187 | `trap_Characteristic_BInteger` | `int` | `int character, int index, int min, int max` | scalar |
| 188 | `trap_Characteristic_String` | `void` | `int character, int index, char *buf, int size` | string |
| 189 | `trap_BotAllocChatState` | `int` | `void` | scalar |
| 190 | `trap_BotFreeChatState` | `void` | `int handle` | scalar |
| 191 | `trap_BotQueueConsoleMessage` | `void` | `int chatstate, int type, char *message` | string |
| 192 | `trap_BotRemoveConsoleMessage` | `void` | `int chatstate, int handle` | scalar |
| 193 | `trap_BotNextConsoleMessage` | `int` | `int chatstate, void /* struct bot_consolemessage_s */ *cm` | opaque |
| 194 | `trap_BotNumConsoleMessages` | `int` | `int chatstate` | scalar |
| 195 | `trap_BotInitialChat` | `void` | `int chatstate, char *type, int mcontext, char *var0, char *var1, char *var2, char *var3, char *var4, char *var5, char *var6, char *var7` | string |
| 196 | `trap_BotNumInitialChats` | `int` | `int chatstate, char *type` | string |
| 197 | `trap_BotReplyChat` | `int` | `int chatstate, char *message, int mcontext, int vcontext, char *var0, char *var1, char *var2, char *var3, char *var4, char *var5, char *var6, char *var7` | string |
| 198 | `trap_BotChatLength` | `int` | `int chatstate` | scalar |
| 199 | `trap_BotEnterChat` | `void` | `int chatstate, int client, int sendto` | scalar |
| 200 | `trap_BotGetChatMessage` | `void` | `int chatstate, char *buf, int size` | string |
| 201 | `trap_StringContains` | `int` | `char *str1, char *str2, int casesensitive` | string |
| 202 | `trap_BotFindMatch` | `int` | `char *str, void /* struct bot_match_s */ *match, unsigned long int context` | opaque |
| 203 | `trap_BotMatchVariable` | `void` | `void /* struct bot_match_s */ *match, int variable, char *buf, int size` | opaque |
| 204 | `trap_UnifyWhiteSpaces` | `void` | `char *string` | string |
| 205 | `trap_BotReplaceSynonyms` | `void` | `char *string, unsigned long int context` | string |
| 206 | `trap_BotLoadChatFile` | `int` | `int chatstate, char *chatfile, char *chatname` | string |
| 207 | `trap_BotSetChatGender` | `void` | `int chatstate, int gender` | scalar |
| 208 | `trap_BotSetChatName` | `void` | `int chatstate, char *name, int client` | string |
| 209 | `trap_BotResetGoalState` | `void` | `int goalstate` | scalar |
| 210 | `trap_BotResetAvoidGoals` | `void` | `int goalstate` | scalar |
| 211 | `trap_BotRemoveFromAvoidGoals` | `void` | `int goalstate, int number` | scalar |
| 212 | `trap_BotPushGoal` | `void` | `int goalstate, void /* struct bot_goal_s */ *goal` | opaque |
| 213 | `trap_BotPopGoal` | `void` | `int goalstate` | scalar |
| 214 | `trap_BotEmptyGoalStack` | `void` | `int goalstate` | scalar |
| 215 | `trap_BotDumpAvoidGoals` | `void` | `int goalstate` | scalar |
| 216 | `trap_BotDumpGoalStack` | `void` | `int goalstate` | scalar |
| 217 | `trap_BotGoalName` | `void` | `int number, char *name, int size` | string |
| 218 | `trap_BotGetTopGoal` | `int` | `int goalstate, void /* struct bot_goal_s */ *goal` | opaque |
| 219 | `trap_BotGetSecondGoal` | `int` | `int goalstate, void /* struct bot_goal_s */ *goal` | opaque |
| 220 | `trap_BotChooseLTGItem` | `int` | `int goalstate, vec3_t origin, int *inventory, int travelflags` | ptr |
| 221 | `trap_BotChooseNBGItem` | `int` | `int goalstate, vec3_t origin, int *inventory, int travelflags, void /* struct bot_goal_s */ *ltg, float maxtime` | opaque |
| 222 | `trap_BotTouchingGoal` | `int` | `vec3_t origin, void /* struct bot_goal_s */ *goal` | opaque |
| 223 | `trap_BotItemGoalInVisButNotVisible` | `int` | `int viewer, vec3_t eye, vec3_t viewangles, void /* struct bot_goal_s */ *goal` | opaque |
| 224 | `trap_BotGetLevelItemGoal` | `int` | `int index, char *classname, void /* struct bot_goal_s */ *goal` | opaque |
| 225 | `trap_BotGetNextCampSpotGoal` | `int` | `int num, void /* struct bot_goal_s */ *goal` | opaque |
| 226 | `trap_BotGetMapLocationGoal` | `int` | `char *name, void /* struct bot_goal_s */ *goal` | opaque |
| 227 | `trap_BotAvoidGoalTime` | `float` | `int goalstate, int number` | scalar |
| 228 | `trap_BotSetAvoidGoalTime` | `void` | `int goalstate, int number, float avoidtime` | scalar |
| 229 | `trap_BotInitLevelItems` | `void` | `void` | scalar |
| 230 | `trap_BotUpdateEntityItems` | `void` | `void` | scalar |
| 231 | `trap_BotLoadItemWeights` | `int` | `int goalstate, char *filename` | string |
| 232 | `trap_BotFreeItemWeights` | `void` | `int goalstate` | scalar |
| 233 | `trap_BotInterbreedGoalFuzzyLogic` | `void` | `int parent1, int parent2, int child` | scalar |
| 234 | `trap_BotSaveGoalFuzzyLogic` | `void` | `int goalstate, char *filename` | string |
| 235 | `trap_BotMutateGoalFuzzyLogic` | `void` | `int goalstate, float range` | scalar |
| 236 | `trap_BotAllocGoalState` | `int` | `int state` | scalar |
| 237 | `trap_BotFreeGoalState` | `void` | `int handle` | scalar |
| 238 | `trap_BotResetMoveState` | `void` | `int movestate` | scalar |
| 239 | `trap_BotAddAvoidSpot` | `void` | `int movestate, vec3_t origin, float radius, int type` | scalar |
| 240 | `trap_BotMoveToGoal` | `void` | `void /* struct bot_moveresult_s */ *result, int movestate, void /* struct bot_goal_s */ *goal, int travelflags` | opaque |
| 241 | `trap_BotMoveInDirection` | `int` | `int movestate, vec3_t dir, float speed, int type` | scalar |
| 242 | `trap_BotResetAvoidReach` | `void` | `int movestate` | scalar |
| 243 | `trap_BotResetLastAvoidReach` | `void` | `int movestate` | scalar |
| 244 | `trap_BotReachabilityArea` | `int` | `vec3_t origin, int testground` | scalar |
| 245 | `trap_BotMovementViewTarget` | `int` | `int movestate, void /* struct bot_goal_s */ *goal, int travelflags, float lookahead, vec3_t target` | opaque |
| 246 | `trap_BotPredictVisiblePosition` | `int` | `vec3_t origin, int areanum, void /* struct bot_goal_s */ *goal, int travelflags, vec3_t target` | opaque |
| 247 | `trap_BotAllocMoveState` | `int` | `void` | scalar |
| 248 | `trap_BotFreeMoveState` | `void` | `int handle` | scalar |
| 249 | `trap_BotInitMoveState` | `void` | `int handle, void /* struct bot_initmove_s */ *initmove` | opaque |
| 250 | `trap_BotChooseBestFightWeapon` | `int` | `int weaponstate, int *inventory` | ptr |
| 251 | `trap_BotGetWeaponInfo` | `void` | `int weaponstate, int weapon, void /* struct weaponinfo_s */ *weaponinfo` | opaque |
| 252 | `trap_BotLoadWeaponWeights` | `int` | `int weaponstate, char *filename` | string |
| 253 | `trap_BotAllocWeaponState` | `int` | `void` | scalar |
| 254 | `trap_BotFreeWeaponState` | `void` | `int weaponstate` | scalar |
| 255 | `trap_BotResetWeaponState` | `void` | `int weaponstate` | scalar |
| 256 | `trap_GeneticParentsAndChildSelection` | `int` | `int numranks, float *ranks, int *parent1, int *parent2, int *child` | ptr |
| 257 | `trap_PC_LoadSource` | `int` | `const char *filename` | string |
| 258 | `trap_PC_FreeSource` | `int` | `int handle` | scalar |
| 259 | `trap_PC_ReadToken` | `int` | `int handle, pc_token_t *pc_token` | ptr |
| 260 | `trap_PC_SourceFileAndLine` | `int` | `int handle, char *filename, int *line` | string |
| 261 | `trap_R_RegisterSkin` | `qhandle_t` | `const char *name` | string |
| 262 | `trap_G2_ListModelBones` | `void` | `void *ghlInfo, int frame` | opaque |
| 263 | `trap_G2_ListModelSurfaces` | `void` | `void *ghlInfo` | opaque |
| 264 | `trap_G2_HaveWeGhoul2Models` | `qboolean` | `void *ghoul2` | opaque |
| 265 | `trap_G2_SetGhoul2ModelIndexes` | `void` | `void *ghoul2, qhandle_t *modelList, qhandle_t *skinList` | opaque |
| 266 | `trap_G2API_GetBoltMatrix` | `qboolean` | `void *ghoul2, const int modelIndex, const int boltIndex, mdxaBone_t *matrix, const vec3_t angles, const vec3_t position, const int frameNum, qhandle_t *modelList, vec3_t scale` | opaque |
| 267 | `trap_G2API_GetBoltMatrix_NoReconstruct` | `qboolean` | `void *ghoul2, const int modelIndex, const int boltIndex, mdxaBone_t *matrix, const vec3_t angles, const vec3_t position, const int frameNum, qhandle_t *modelList, vec3_t scale` | opaque |
| 268 | `trap_G2API_GetBoltMatrix_NoRecNoRot` | `qboolean` | `void *ghoul2, const int modelIndex, const int boltIndex, mdxaBone_t *matrix, const vec3_t angles, const vec3_t position, const int frameNum, qhandle_t *modelList, vec3_t scale` | opaque |
| 269 | `trap_G2API_InitGhoul2Model` | `int` | `void **ghoul2Ptr, const char *fileName, int modelIndex, qhandle_t customSkin, qhandle_t customShader, int modelFlags, int lodBias` | opaque |
| 270 | `trap_G2API_SetSkin` | `qboolean` | `void *ghoul2, int modelIndex, qhandle_t customSkin, qhandle_t renderSkin` | opaque |
| 271 | `trap_G2API_Ghoul2Size` | `int` | `void* ghlInfo` | opaque |
| 272 | `trap_G2API_AddBolt` | `int` | `void *ghoul2, int modelIndex, const char *boneName` | opaque |
| 273 | `trap_G2API_SetBoltInfo` | `void` | `void *ghoul2, int modelIndex, int boltInfo` | opaque |
| 274 | `trap_G2API_SetBoneAngles` | `qboolean` | `void *ghoul2, int modelIndex, const char *boneName, const vec3_t angles, const int flags, const int up, const int right, const int forward, qhandle_t *modelList, int blendTime , int currentTime` | opaque |
| 275 | `trap_G2API_SetBoneAnim` | `qboolean` | `void *ghoul2, const int modelIndex, const char *boneName, const int startFrame, const int endFrame, const int flags, const float animSpeed, const int currentTime, const float setFrame , const int blendTime` | opaque |
| 276 | `trap_G2API_GetBoneAnim` | `qboolean` | `void *ghoul2, const char *boneName, const int currentTime, float *currentFrame, int *startFrame, int *endFrame, int *flags, float *animSpeed, int *modelList, const int modelIndex` | opaque |
| 277 | `trap_G2API_GetGLAName` | `void` | `void *ghoul2, int modelIndex, char *fillBuf` | opaque |
| 278 | `trap_G2API_CopyGhoul2Instance` | `int` | `void *g2From, void *g2To, int modelIndex` | opaque |
| 279 | `trap_G2API_CopySpecificGhoul2Model` | `void` | `void *g2From, int modelFrom, void *g2To, int modelTo` | opaque |
| 280 | `trap_G2API_DuplicateGhoul2Instance` | `void` | `void *g2From, void **g2To` | opaque |
| 281 | `trap_G2API_HasGhoul2ModelOnIndex` | `qboolean` | `void *ghlInfo, int modelIndex` | opaque |
| 282 | `trap_G2API_RemoveGhoul2Model` | `qboolean` | `void *ghlInfo, int modelIndex` | opaque |
| 283 | `trap_G2API_RemoveGhoul2Models` | `qboolean` | `void *ghlInfo` | opaque |
| 284 | `trap_G2API_CleanGhoul2Models` | `void` | `void **ghoul2Ptr` | opaque |
| 285 | `trap_G2API_CollisionDetect` | `void` | ` CollisionRecord_t *collRecMap,  void* ghoul2,  const vec3_t angles,  const vec3_t position, int frameNumber,  int entNum,  vec3_t rayStart,  vec3_t rayEnd,  vec3_t scale,  int traceFlags,  int useLod, float fRadius` | opaque |
| 286 | `trap_G2API_CollisionDetectCache` | `void` | ` CollisionRecord_t *collRecMap,  void* ghoul2,  const vec3_t angles,  const vec3_t position, int frameNumber,  int entNum,  vec3_t rayStart,  vec3_t rayEnd,  vec3_t scale,  int traceFlags,  int useLod, float fRadius` | opaque |
| 287 | `trap_G2API_GetSurfaceName` | `void` | `void *ghoul2, int surfNumber, int modelIndex, char *fillBuf` | opaque |
| 288 | `trap_G2API_SetRootSurface` | `qboolean` | `void *ghoul2, const int modelIndex, const char *surfaceName` | opaque |
| 289 | `trap_G2API_SetSurfaceOnOff` | `qboolean` | `void *ghoul2, const char *surfaceName, const int flags` | opaque |
| 290 | `trap_G2API_SetNewOrigin` | `qboolean` | `void *ghoul2, const int boltIndex` | opaque |
| 291 | `trap_G2API_DoesBoneExist` | `qboolean` | `void *ghoul2, int modelIndex, const char *boneName` | opaque |
| 292 | `trap_G2API_GetSurfaceRenderStatus` | `int` | `void *ghoul2, const int modelIndex, const char *surfaceName` | opaque |
| 293 | `trap_G2API_AbsurdSmoothing` | `void` | `void *ghoul2, qboolean status` | opaque |
| 294 | `trap_G2API_SetRagDoll` | `void` | `void *ghoul2, sharedRagDollParams_t *params` | opaque |
| 295 | `trap_G2API_AnimateG2Models` | `void` | `void *ghoul2, int time, sharedRagDollUpdateParams_t *params` | opaque |
| 296 | `trap_G2API_RagPCJConstraint` | `qboolean` | `void *ghoul2, const char *boneName, vec3_t min, vec3_t max` | opaque |
| 297 | `trap_G2API_RagPCJGradientSpeed` | `qboolean` | `void *ghoul2, const char *boneName, const float speed` | opaque |
| 298 | `trap_G2API_RagEffectorGoal` | `qboolean` | `void *ghoul2, const char *boneName, vec3_t pos) //override an effector bone's goal position (world coordinates` | opaque |
| 299 | `trap_G2API_GetRagBonePos` | `qboolean` | `void *ghoul2, const char *boneName, vec3_t pos, vec3_t entAngles, vec3_t entPos, vec3_t entScale) //current position of said bone is put into pos (world coordinates` | opaque |
| 300 | `trap_G2API_RagEffectorKick` | `qboolean` | `void *ghoul2, const char *boneName, vec3_t velocity` | opaque |
| 301 | `trap_G2API_RagForceSolve` | `qboolean` | `void *ghoul2, qboolean force` | opaque |
| 302 | `trap_G2API_SetBoneIKState` | `qboolean` | `void *ghoul2, int time, const char *boneName, int ikState, sharedSetBoneIKStateParams_t *params` | opaque |
| 303 | `trap_G2API_IKMove` | `qboolean` | `void *ghoul2, int time, sharedIKMoveParams_t *params` | opaque |
| 304 | `trap_G2API_RemoveBone` | `qboolean` | `void *ghoul2, const char *boneName, int modelIndex` | opaque |
| 305 | `trap_G2API_AttachInstanceToEntNum` | `void` | `void *ghoul2, int entityNum, qboolean server` | opaque |
| 306 | `trap_G2API_ClearAttachedInstance` | `void` | `int entityNum` | scalar |
| 307 | `trap_G2API_CleanEntAttachments` | `void` | `void` | scalar |
| 308 | `trap_G2API_OverrideServer` | `qboolean` | `void *serverInstance` | opaque |
| 309 | `trap_SetActiveSubBSP` | `void` | `int index` | scalar |
| 310 | `trap_CM_RegisterTerrain` | `int` | `const char *config` | string |
| 311 | `trap_RMG_Init` | `void` | `int terrainID` | scalar |
| 312 | `trap_Bot_UpdateWaypoints` | `void` | `int wpnum, wpobject_t **wps` | ptr |
| 313 | `trap_Bot_CalculatePaths` | `void` | `int rmg` | scalar |
