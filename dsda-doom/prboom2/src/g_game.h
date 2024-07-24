/* Emacs style mode select   -*- C -*-
 *-----------------------------------------------------------------------------
 *
 *  PrBoom: a Doom port merged with LxDoom and LSDLDoom
 *  based on BOOM, a modified and improved DOOM engine
 *  Copyright (C) 1999 by
 *  id Software, Chi Hoang, Lee Killough, Jim Flynn, Rand Phares, Ty Halderman
 *  Copyright (C) 1999-2000 by
 *  Jess Haas, Nicolas Kalkhof, Colin Phipps, Florian Schulze
 *  Copyright 2005, 2006 by
 *  Florian Schulze, Colin Phipps, Neil Stevens, Andrey Budko
 *
 *  This program is free software; you can redistribute it and/or
 *  modify it under the terms of the GNU General Public License
 *  as published by the Free Software Foundation; either version 2
 *  of the License, or (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program; if not, write to the Free Software
 *  Foundation, Inc., 59 Temple Place - Suite 330, Boston, MA
 *  02111-1307, USA.
 *
 * DESCRIPTION: Main game control interface.
 *-----------------------------------------------------------------------------*/

#ifndef __G_GAME__
#define __G_GAME__

#include "doomdef.h"
#include "d_event.h"
#include "d_ticcmd.h"
#include "tables.h"

#include "viletech.zig.h"

#define DEMOMARKER 0x80

dboolean G_Responder(CCore*, event_t *ev);
dboolean G_CheckDemoStatus(CCore*);
void G_DeathMatchSpawnPlayer(CCore*, int playernum);
void G_InitNew(CCore*, int skill, int episode, int map, dboolean prepare);
void G_DeferedInitNew(CCore*, int skill, int episode, int map);
void G_DeferedPlayDemo(CCore*, const char *demo); // CPhipps - const
void G_LoadGame(int slot); // killough 5/15/98
void G_ForcedLoadGame(void);  // killough 5/15/98: forced loadgames
void G_DoLoadGame(CCore*);
void G_SaveGame(CCore*, int slot, const char *description); // Called by M_Responder.
void G_BeginRecording(CCore*);
void G_ExitLevel(int position);
void G_SecretExitLevel(int position);
void G_WorldDone(CCore*);
void G_EndGame(void); /* cph - make m_menu.c call a G_* function for this */

/// @fn G_Ticker
/// Make ticcmd_ts for the players.
void G_Ticker(CCore*);

void G_ReloadDefaults(void); // killough 3/1/98: loads game defaults
void G_RefreshFastMonsters(void); // killough 4/10/98: sets -fast parameters
void G_DoNewGame(CCore*);
void G_DoReborn(CCore*, int playernum);
void G_StartDemoPlayback(CCore*, const byte *buffer, int length, int behaviour);
void G_DoPlayDemo(CCore*);
void G_DoCompleted(CCore*);
void G_WriteDemoTiccmd(ticcmd_t *cmd);
void G_DoWorldDone(CCore*);
void G_Compatibility(void);
const byte *G_ReadOptions(const byte *demo_p);   /* killough 3/1/98 - cph: const byte* */
byte *G_WriteOptions(byte *demo_p);        // killough 3/1/98
void G_PlayerReborn(int player);
void G_DoVictory(void);
void G_BuildTiccmd(CCore*, ticcmd_t* cmd); // CPhipps - move decl to header
void G_ReadOneTick(ticcmd_t* cmd, const byte **data_p);
void G_ChangedPlayerColour(int pn, int cl); // CPhipps - On-the-fly player colour changing
void G_MakeSpecialEvent(buttoncode_t bc, ...); /* cph - new event stuff */
int G_ValidateMapName(const char *mapname, int *pEpi, int *pMap);

//e6y
void G_ContinueDemo(CCore*, const char *playback_name);
void G_SetSpeed(dboolean force);

//e6y
#define RDH_SAFE 0x00000001
#define RDH_SKIP_HEADER 0x00000002
const byte* G_ReadDemoHeaderEx(CCore* cx, const byte* demo_p, size_t size, unsigned int params);
void G_CalculateDemoParams(const byte *demo_p);

// killough 1/18/98: Doom-style printf;   killough 4/25/98: add gcc attributes
// CPhipps - renames to doom_printf to avoid name collision with glibc
void doom_printf(CCore*, const char *, ...) __attribute__((format(printf,2,3)));

// killough 5/2/98: moved from m_misc.c:

extern int  key_forward;
extern int  key_backward;

extern dboolean haswolflevels;  //jff 4/18/98 wolf levels present
extern dboolean secretexit;

// killough 5/2/98: moved from d_deh.c:
// Par times (new item with BOOM) - from g_game.c
extern int pars[5][10];  // hardcoded array size
extern int cpars[];      // hardcoded array size
// CPhipps - Make savedesciption visible in wider scope
#define SAVEDESCLEN 32
extern char savedescription[SAVEDESCLEN];  // Description to save in savegame

/* cph - compatibility level strings */
extern const char * comp_lev_str[];

// e6y
// There is a new command-line switch "-shorttics".
// This makes it possible to practice routes and tricks
// (e.g. glides, where this makes a significant difference)
// with the same mouse behaviour as when recording,
// but without having to be recording every time.
extern int shorttics;
extern int longtics;

typedef enum PistolStart {
    pistolstart_off,
    pistolstart_on,
    pistolstart_held,
} PistolStart;

// automatic pistol start when advancing from one level to the next
extern PistolStart pistolstart;

// hexen

void G_Completed(CCore*, int map, int position, int flags, angle_t angle);

#endif
