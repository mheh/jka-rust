// roff-oracle — deterministic host control surface for the dumpers. Lets the
// dumper drive svs.time, the mock gentity array, and read back the note-track
// VM_Call log the unmodified RoffSystem.cpp emits. oracle/ untouched.
#ifndef ROFF_ORACLE_HOST_H
#define ROFF_ORACLE_HOST_H

// --- clock ---------------------------------------------------------------
void host_set_time(int t);
int  host_get_time(void);

// --- mock gentity array (SV_GentityNum returns &host_ent[num]) -----------
// Reset all entities to zero and set entity `num`'s apos.trBase (the angle Play
// copies into SROFFEntity.mStartAngles).
void host_reset_entities(void);
void host_set_ent_angles(int num, float pitch, float yaw, float roll);

// --- note-track VM_Call log ----------------------------------------------
// Each server-side ProcessNote emit is recorded as (callNum, entnum, note).
int         host_note_count(void);
int         host_note_callnum(int i);
int         host_note_entnum(int i);
const char *host_note_text(int i);
void        host_note_clear(void);

#endif // ROFF_ORACLE_HOST_H
