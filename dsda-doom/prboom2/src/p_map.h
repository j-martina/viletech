/* Emacs style mode select   -*- C -*-
 *-----------------------------------------------------------------------------
 *
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
 * DESCRIPTION:
 *      Map functions
 *
 *-----------------------------------------------------------------------------*/

#ifndef __P_MAP__
#define __P_MAP__

#include "r_defs.h"
#include "d_player.h"

#include "viletech.zig.h"

#define USERANGE        (64*FRACUNIT)
#define MELEERANGE      (64*FRACUNIT)
#define MISSILERANGE    (32*64*FRACUNIT)

// a couple of explicit constants for non-melee things that used to use MELEERANGE
#define WAKEUPRANGE     (64*FRACUNIT)
#define SNEAKRANGE      (128*FRACUNIT)

// MAXRADIUS is for precalculated sector block boxes the spider demon
// is larger, but we do not have any moving sectors nearby
#define MAXRADIUS       (32*FRACUNIT)

//e6y
#define STAIRS_UNINITIALIZED_CRUSH_FIELD_VALUE -2

enum {
    laf_none = 0,
    laf_painless = 1 << 0,
};
typedef int LineAttackFlags;

typedef struct LineAttackParams {
    mobj_t* t1;
    angle_t angle;
    fixed_t distance, slope;
    int damage;
    LineAttackFlags flags;
} LineAttackParams;

typedef struct
{
  msecnode_t *node;
  sector_t *sector;
} mobj_in_sector_t;

// killough 3/15/98: add fourth argument to P_TryMove
dboolean P_TryMove(CCore*, mobj_t*, fixed_t x, fixed_t y, dboolean dropoff);

// killough 8/9/98: extra argument for telefragging
dboolean P_TeleportMove(CCore*, mobj_t*, fixed_t x, fixed_t y,dboolean boss);
void    P_UnqualifiedMove(CCore*, mobj_t*, fixed_t x, fixed_t y);
void    P_SlideMove(CCore*, mobj_t*);
dboolean P_CheckSight(mobj_t *t1, mobj_t *t2);
dboolean P_CheckFov(mobj_t *t1, mobj_t *t2, angle_t fov);
void    P_UseLines(CCore*, player_t*);

typedef dboolean (*CrossSubsectorFunc)(int num);
extern CrossSubsectorFunc P_CrossSubsector;
dboolean P_CrossSubsector_Doom(int num);
dboolean P_CrossSubsector_Boom(int num);
dboolean P_CrossSubsector_PrBoom(int num);

// killough 8/2/98: add 'mask' argument to prevent friends autoaiming at others
fixed_t P_AimLineAttack(CCore*, mobj_t *t1,angle_t angle,fixed_t distance, uint64_t mask);

void P_LineAttack(
	CCore*,
	mobj_t* t1,
	angle_t angle,
	fixed_t distance,
	fixed_t slope,
	int damage
);
void P_LineAttack2(CCore* cx, LineAttackParams args);

void P_RadiusAttack(
	CCore* cx,
	mobj_t* spot,
	mobj_t* source,
	int damage,
	int distance,
	dboolean damageSource
);

dboolean P_CheckPosition(CCore*, mobj_t*, fixed_t x, fixed_t y);

void P_InitSectorSearch(mobj_in_sector_t*, sector_t*);
mobj_t *P_FindMobjInSector(mobj_in_sector_t*);

//jff 3/19/98 P_CheckSector(): new routine to replace P_ChangeSector()
dboolean P_ChangeSector(CCore*, sector_t*, int crunch);
dboolean P_CheckSector(CCore*, sector_t*, int crunch);
void    P_DelSeclist(msecnode_t*); // phares 3/16/98
void    P_FreeSecNodeList(void); // sf
void    P_CreateSecNodeList(CCore*, mobj_t*, fixed_t, fixed_t); // phares 3/14/98
dboolean Check_Sides(CCore*, mobj_t *, int, int); // phares

int     P_GetMoveFactor(mobj_t *mo, int *friction); // killough 8/28/98
int     P_GetFriction(const mobj_t *mo, int *factor); // killough 8/28/98
void    P_ApplyTorque(CCore*, mobj_t *mo); // killough 9/12/98

/* cphipps 2004/08/30 */
void	P_MapStart(void);
void	P_MapEnd(void);

// If "floatok" true, move would be ok if within "tmfloorz - tmceilingz".
extern dboolean floatok;
extern dboolean felldown; // killough 11/98: indicates object pushed off ledge
extern fixed_t tmfloorz;
extern fixed_t tmceilingz;
extern line_t *ceilingline;
extern line_t *floorline; // killough 8/23/98
extern mobj_t *linetarget; // who got hit (or NULL)
extern mobj_t *crosshair_target;
extern msecnode_t *sector_list; // phares 3/16/98
extern fixed_t tmbbox[4]; // phares 3/20/98
extern line_t *blockline; // killough 8/11/98

// heretic

dboolean P_TestMobjLocation(CCore*, mobj_t*);
mobj_t *P_CheckOnmobj(CCore*, mobj_t*);
void P_FakeZMovement(mobj_t*);

void P_AppendSpecHit(line_t*);

// hexen

extern int tmfloorpic;
extern mobj_t *BlockingMobj;

void P_BounceWall(CCore*, mobj_t*);
dboolean P_UsePuzzleItem(CCore*, player_t*, int itemType);
void PIT_ThrustSpike(CCore*, mobj_t*);

// zdoom

dboolean P_MoveThing(CCore*, mobj_t*, fixed_t x, fixed_t y, fixed_t z, dboolean fog);
int P_SplashDamage(fixed_t dist);
void P_AdjustZLimits(mobj_t *thing);

void P_CheckCompatibleImpact(CCore*, mobj_t *);
void P_CheckHereticImpact(CCore*, mobj_t *);
void P_CheckZDoomImpact(CCore*, mobj_t *);

#endif // __P_MAP__
