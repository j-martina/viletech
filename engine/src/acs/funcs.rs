//! Function indices.
//!
//! Assume all code within originates from GZDoom-original source.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Function {
	GetLineUdmfInt = 1,
	GetLineUdmfFixed,
	GetThingUdmfInt,
	GetThingUdmfFixed,
	GetSectorUdmfInt,
	GetSectorUdmfFixed,
	GetSideUdmfInt,
	GetSideUdmfFixed,
	GetEntityVelX,
	GetEntityVelY,
	GetEntityVelZ,
	SetActivator,
	SetActivatorToTarget,
	GetEntityViewHeight,
	GetChar,
	GetAirSupply,
	SetAirSupply,
	SetSkyScrollSpeed,
	GetArmorType,
	SpawnSpotForced,
	SpawnSpotFacingForced,
	CheckEntityProperty,
	SetEntityVelocity,
	SetUserVariable,
	GetUserVariable,
	RadiusQuake2,
	CheckEntityClass,
	SetUserArray,
	GetUserArray,
	SoundSequenceOnEntity,
	SoundSequenceOnSector,
	SoundSequenceOnPolyobj,
	GetPolyobjX,
	GetPolyobjY,
	CheckSight,
	SpawnForced,
	AnnouncerSound,
	SetPointer,
	NamedExecute,
	NamedSuspend,
	NamedTerminate,
	NamedLockedExecute,
	NamedLockedExecuteDoor,
	NamedExecuteWithResult,
	NamedExecuteAlways,
	UniqueTid,
	IsTidUsed,
	Sqrt,
	FixedSqrt,
	VectorLength,
	SetHudClipRect,
	SetHudWrapWidth,
	SetCVar,
	GetUserCVar,
	SetUserCVar,
	GetCVarString,
	SetCVarString,
	GetUserCVarString,
	SetUserCVarString,
	LineAttack,
	PlaySound,
	StopSound,
	StrCmp,
	StriCmp,
	StrLeft,
	StrRight,
	StrMid,
	GetEntityClass,
	GetWeapon,
	SoundVolume,
	PlayEntitySound,
	SpawnDecal,
	CheckFont,
	DropItem,
	CheckFlag,
	SetLineActivation,
	GetLineActivation,
	GetEntityPowerupTics,
	ChangeEntityAngle,
	ChangeEntityPitch,
	GetArmorInfo,
	DropInventory,
	PickEntity,
	IsPointerEqual,
	CanRaiseEntity,
	SetEntityTeleFog,
	SwapEntityTeleFog,
	SetEntityRoll,
	ChangeEntityRoll,
	GetEntityRoll,
	QuakeEx,
	Warp,
	GetMaxInventory,
	SetSectorDamage,
	SetSectorTerrain,
	SpawnParticle,
	SetMusicVolume,
	CheckProximity,
	CheckEntityState,

	// Zandronum
	// 100 : ResetMap(0),
	// 101 : PlayerIsSpectator(1),
	// 102 : ConsolePlayerNumber(0),
	// 103 : GetTeamProperty(2),
	// 104 : GetPlayerLivesLeft(1),
	// 105 : SetPlayerLivesLeft(2),
	// 106 : KickFromGame(2),
	CheckClass = 200,
	DamageEntity,
	SetEntityFlag,
	SetTranslation,
	GetEntityFloorTexture,
	GetEntityFloorTerrain,
	StrArg,
	Floor,
	Round,
	Ceil,
	ScriptCall,
	StartSlideshow,
	GetSectorHealth,
	GetLineHealth,
	SetSubtitleNumber,
	// Eternity Engne
	GetLineX = 300,
	GetLineY,
	// Hardware renderer
	SetSectorGlow = 400,
	SetFogDensity,
	// ZDaemon
	GetTeamScore = 19620, // (int team)
	SetTeamScore,         // (int team, int value
}
