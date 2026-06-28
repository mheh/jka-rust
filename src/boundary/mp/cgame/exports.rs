//! MP cgame exports enum vocabulary.
//!
//! Transcribed from Raven `oracle/oracle/codemp/cgame/cg_public.h`.
//! These discriminants are ABI wire values; do not renumber them.

#![allow(non_camel_case_types)]

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MpCgameExport {
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:353`
    CG_INIT,

    /// void CG_Init( int serverMessageNum, int serverCommandSequence, int clientNum )
    /// called when the level loads or when the renderer is restarted
    /// all media should be registered at this time
    /// cgame will display loading status by calling SCR_Update, which
    /// will call CG_DrawInformation during the loading process
    /// reliableCommandSequence will be 0 on fresh loads, but higher for
    /// demos, tourney restarts, or vid_restarts
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:362`
    CG_SHUTDOWN,

    /// void (*CG_Shutdown)( void );
    /// oportunity to flush and close any open files
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:366`
    CG_CONSOLE_COMMAND,

    /// qboolean (*CG_ConsoleCommand)( void );
    /// a console command has been issued locally that is not recognized by the
    /// main game system.
    /// use Cmd_Argc() / Cmd_Argv() to read the command, return qfalse if the
    /// command is not known to the game
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:373`
    CG_DRAW_ACTIVE_FRAME,

    /// void (*CG_DrawActiveFrame)( int serverTime, stereoFrame_t stereoView, qboolean demoPlayback );
    /// Generates and draws a game scene and status information at the given time.
    /// If demoPlayback is set, local movement prediction will not be enabled
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:378`
    CG_CROSSHAIR_PLAYER,

    /// int (*CG_CrosshairPlayer)( void );
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:381`
    CG_LAST_ATTACKER,

    /// int (*CG_LastAttacker)( void );
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:384`
    CG_KEY_EVENT,

    /// void	(*CG_KeyEvent)( int key, qboolean down );
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:387`
    CG_MOUSE_EVENT,

    /// void	(*CG_MouseEvent)( int dx, int dy );
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:389`
    CG_EVENT_HANDLING,

    /// void (*CG_EventHandling)(int type);
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:392`
    CG_POINT_CONTENTS,

    /// int	CG_PointContents( const vec3_t point, int passEntityNum );
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:395`
    CG_GET_LERP_ORIGIN,

    /// void CG_LerpOrigin(int num, vec3_t result);
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:398`
    CG_GET_LERP_DATA,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:399`
    CG_GET_GHOUL2,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:400`
    CG_GET_MODEL_LIST,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:402`
    CG_CALC_LERP_POSITIONS,

    /// void CG_CalcEntityLerpPositions(int num);
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:405`
    CG_TRACE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:406`
    CG_G2TRACE,

    /// void CG_Trace( trace_t *result, const vec3_t start, const vec3_t mins, const vec3_t maxs, const vec3_t end,
    /// int skipNumber, int mask );
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:410`
    CG_G2MARK,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:412`
    CG_RAG_CALLBACK,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:414`
    CG_INCOMING_CONSOLE_COMMAND,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:416`
    CG_GET_USEABLE_FORCE,

    /// int entnum, vec3_t origin
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:418`
    CG_GET_ORIGIN,

    /// int entnum, vec3_t angle
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:419`
    CG_GET_ANGLES,

    /// int entnum
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:421`
    CG_GET_ORIGIN_TRAJECTORY,

    /// int entnum
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:422`
    CG_GET_ANGLE_TRAJECTORY,

    /// int entnum, char *notetrack
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:424`
    CG_ROFF_NOTETRACK_CALLBACK,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:426`
    CG_IMPACT_MARK,

    /// void CG_ImpactMark( qhandle_t markShader, const vec3_t origin, const vec3_t dir,
    /// float orientation, float red, float green, float blue, float alpha,
    /// qboolean alphaFade, float radius, qboolean temporary )
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:431`
    CG_MAP_CHANGE,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:433`
    CG_AUTOMAP_INPUT,

    /// rwwRMG - added
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:435`
    CG_MISC_ENT,

    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:437`
    CG_GET_SORTED_FORCE_POWER,

    /// mcg post-gold added
    /// Source: `oracle/oracle/codemp/cgame/cg_public.h:439`
    CG_FX_CAMERASHAKE,

}
