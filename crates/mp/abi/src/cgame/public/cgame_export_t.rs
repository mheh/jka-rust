#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

/// Raven `cgameExport_t` — cgame syscall export indices.
///
/// Type definition source: `oracle/codemp/cgame/cg_public.h:352-440`
#[repr(i32)]
pub enum cgameExport_t {
    CG_INIT,
    //	void CG_Init( int serverMessageNum, int serverCommandSequence, int clientNum )
    // called when the level loads or when the renderer is restarted
    // all media should be registered at this time
    // cgame will display loading status by calling SCR_Update, which
    // will call CG_DrawInformation during the loading process
    // reliableCommandSequence will be 0 on fresh loads, but higher for
    // demos, tourney restarts, or vid_restarts
    CG_SHUTDOWN,
    //	void (*CG_Shutdown)( void );
    // oportunity to flush and close any open files
    CG_CONSOLE_COMMAND,
    //	qboolean (*CG_ConsoleCommand)( void );
    // a console command has been issued locally that is not recognized by the
    // main game system.
    // use Cmd_Argc() / Cmd_Argv() to read the command, return qfalse if the
    // command is not known to the game
    CG_DRAW_ACTIVE_FRAME,
    //	void (*CG_DrawActiveFrame)( int serverTime, stereoFrame_t stereoView, qboolean demoPlayback );
    // Generates and draws a game scene and status information at the given time.
    // If demoPlayback is set, local movement prediction will not be enabled
    CG_CROSSHAIR_PLAYER,
    //	int (*CG_CrosshairPlayer)( void );
    CG_LAST_ATTACKER,
    //	int (*CG_LastAttacker)( void );
    CG_KEY_EVENT,
    //	void	(*CG_KeyEvent)( int key, qboolean down );
    CG_MOUSE_EVENT,
    //	void	(*CG_MouseEvent)( int dx, int dy );
    CG_EVENT_HANDLING,
    //	void (*CG_EventHandling)(int type);
    CG_POINT_CONTENTS,
    //	int	CG_PointContents( const vec3_t point, int passEntityNum );
    CG_GET_LERP_ORIGIN,
    //	void CG_LerpOrigin(int num, vec3_t result);
    CG_GET_LERP_DATA,
    CG_GET_GHOUL2,
    CG_GET_MODEL_LIST,

    CG_CALC_LERP_POSITIONS,
    //	void CG_CalcEntityLerpPositions(int num);
    CG_TRACE,
    CG_G2TRACE,
    //void CG_Trace( trace_t *result, const vec3_t start, const vec3_t mins, const vec3_t maxs, const vec3_t end,
    //	 int skipNumber, int mask );
    CG_G2MARK,

    CG_RAG_CALLBACK,

    CG_INCOMING_CONSOLE_COMMAND,

    CG_GET_USEABLE_FORCE,

    CG_GET_ORIGIN, // int entnum, vec3_t origin
    CG_GET_ANGLES, // int entnum, vec3_t angle

    CG_GET_ORIGIN_TRAJECTORY, // int entnum
    CG_GET_ANGLE_TRAJECTORY,  // int entnum

    CG_ROFF_NOTETRACK_CALLBACK, // int entnum, char *notetrack

    CG_IMPACT_MARK,
    //void CG_ImpactMark( qhandle_t markShader, const vec3_t origin, const vec3_t dir,
    //	float orientation, float red, float green, float blue, float alpha,
    //	qboolean alphaFade, float radius, qboolean temporary )
    CG_MAP_CHANGE,

    CG_AUTOMAP_INPUT,

    CG_MISC_ENT, //rwwRMG - added

    CG_GET_SORTED_FORCE_POWER,

    CG_FX_CAMERASHAKE, //mcg post-gold added
}
