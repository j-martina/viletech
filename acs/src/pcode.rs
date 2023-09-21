//! The opcodes making up GZDoom's ACS instruction set.
//!
//! Assume all code within originates from GZDoom-original source.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PCode {
	NoOp,
	Terminate,
	Suspend,
	PushNumber,
	LSpec1,
	LSpec2,
	LSpec3,
	LSpec4,
	LSpec5,
	LSpec1Direct,
	LSpec2Direct,
	LSpec3Direct,
	LSpec4Direct,
	LSpec5Direct,
	Add,
	Subtract,
	Multiply,
	Divide,
	Modulus,
	Eq,
	Ne,
	Lt,
	Gt,
	Le,
	Ge,
	AssignScriptVar,
	AssignMapVar,
	AssignWorldVar,
	PushScriptVar,
	PushMapVar,
	PushWorldVar,
	AddScriptVar,
	AddMapVar,
	AddWorldVar,
	SubScriptVar,
	SubMapVar,
	SubWorldVar,
	MulScriptVar,
	MulMapVar,
	MulWorldVar,
	DivScriptVar,
	DivMapVar,
	DivWorldVar,
	ModScriptVar,
	ModMapVar,
	ModWorldVar,
	IncScriptVar,
	IncMapVar,
	IncWorldVar,
	DecScriptVar,
	DecMapVar,
	DecWorldVar,
	Goto,
	IfGoto,
	Drop,
	Delay,
	DelayDirect,
	Random,
	RandomDirect,
	ThingCount,
	ThingCountDirect,
	TagWait,
	TagWaitDirect,
	PolyWait,
	PolyWaitDirect,
	ChangeFloor,
	ChangeFloorDirect,
	ChangeCeiling,
	ChangeCeilingDirect,
	Restart,
	AndLogical,
	OrLogical,
	AndBitwise,
	OrBitwise,
	EOrBitwise,
	NegateLogical,
	LShift,
	RShift,
	UnaryMnus,
	IfNotGoto,
	LineSide,
	ScriptWait,
	ScriptWaitDirect,
	ClearLineSpecial,
	CaseGoto,
	BeginPrint,
	EndPrint,
	PrintString,
	PrintNumber,
	PrintCharacter,
	PlayerCount,
	GameType,
	GameSkill,
	Timer,
	SectorSound,
	AmbientSound,
	SoundSequence,
	SetLineTexture,
	SetLineBlocking,
	SetLineSpecial,
	ThingSound,
	EndPrintBold,
	ActivatorSound,
	LocalAmbientSound,
	SetLineMonsterBlocking,
	PlayerBlueSkull,
	PlayerRedSkull,
	PlayerYellowSkull,
	PlayerMasterSkull,
	PlayerBlueCard,
	PlayerRedCard,
	PlayerYellowCard,
	PlayerMasterCard,
	PlayerBlackSkull,
	PlayerSilverSkull,
	PlayerGoldSkull,
	PlayerBlackCard,
	PlayerSilverCard,
	IsNetworkGame,
	PlayerTeam,
	PlayerHealth,
	PlayerArmorPoints,
	PlayerFrags,
	PlayerExpert,
	BlueTeamCount,
	RedTeamCount,
	BlueTeamScore,
	RedTeamScore,
	IsOneFlagCtf,
	LSpec6,
	LSpec6Direct,
	PrintName,
	MusicChange,
	ConsoleCommandDirect,
	ConsoleCommand,
	SinglePlayer,
	FixedMul,
	FixedDiv,
	SetGravity,
	SetGravityDirect,
	SetAirControl,
	SetAirControlDirect,
	ClearInventory,
	GiveInventory,
	GiveInventoryDirect,
	TakeInventory,
	TakeInventoryDirect,
	CheckInventory,
	CheckInventoryDirect,
	Spawn,
	SpawnDirect,
	SpawnSpot,
	SpawnSpotDirect,
	SetMusic,
	SetMusicDirect,
	LocalSetMusic,
	LocalSetMusicDirect,
	PrintFixed,
	PrintLocalized,
	MoreHudMessage,
	OptHudMessage,
	EndHudMessage,
	EndHudMessageBold,
	SetStyle,
	SetStyleDirect,
	SetFont,
	SetFontDirect,
	PushByte,
	LSpec1DirectB,
	LSpec2DirectB,
	LSpec3DirectB,
	LSpec4DirectB,
	LSpec5DirectB,
	DelayDirectB,
	RandomDirectB,
	PushBytes,
	Push2Bytes,
	Push3Bytes,
	Push4Bytes,
	Push5Bytes,
	SetThingSpecial,
	AssignGlobalVar,
	PushGlobalVar,
	AddGlobalVar,
	SubGlobalVar,
	MulGlobalVar,
	DivGlobalVar,
	ModGlobalVar,
	IncGlobalVar,
	DecGlobalVar,
	FadeTo,
	FadeRange,
	CancelFade,
	PlayMovie,
	SetFloorTrigger,
	SetCeilingTrigger,
	GetActorX,
	GetActorY,
	GetActorZ,
	StartTranslation,
	TranslationRange1,
	TranslationRange2,
	EndTranslation,
	Call,
	CallDiscard,
	ReturnVoid,
	ReturnVal,
	PushMapArray,
	AssignMapArray,
	AddMapArray,
	SubMapArray,
	MulMapArray,
	DivMapArray,
	ModMapArray,
	IncMapArray,
	DecMapArray,
	Dup,
	Swap,
	WriteToIni,
	GetFromIni,
	Sin,
	Cos,
	VectorAngle,
	CheckWeapon,
	SetWeapon,
	TagString,
	PushWorldArray,
	AssignWorldArray,
	AddWorldArray,
	SubWorldArray,
	MulWorldArray,
	DivWorldArray,
	ModWorldArray,
	IncWorldArray,
	DecWorldArray,
	PushGlobalArray,
	AssignGlobalArray,
	AddGlobalArray,
	SubGlobalArray,
	MulGlobalArray,
	DivGlobalArray,
	ModGlobalArray,
	IncGlobalArray,
	DecGlobalArray,
	SetMarineWeapon,
	SetActorProperty,
	GetActorProperty,
	PlayerNumber,
	ActivatorTid,
	SetMarineSprite,
	GetScreenWidth,
	GetScreenHeight,
	ThingProjectile2,
	StrLen,
	SetHudSize,
	GetCVar,
	CaseGotoSorted,
	SetResultValue,
	GetLineRowOffset,
	GetActorFloorZ,
	GetActorAngle,
	GetSectorFloorZ,
	GetSectorCeilingZ,
	LSpec5Result,
	GetSigilPieces,
	GetLevelInfo,
	ChangeSky,
	PlayerInGame,
	PlayerIsBot,
	SetCameraToTexture,
	EndLog,
	GetAmmoCapacity,
	SetAmmoCapacity,
	PrintMapCharArray,
	PrintWorldCharArray,
	PrintGlobalCharArray,
	SetActorAngle,
	GrabInput,
	SetMousePointer,
	MoveMousePointer,
	SpawnProjectile,
	GetSectorLightLevel,
	GetActorCeilingZ,
	SetActorPosition,
	ClearActorInventory,
	GiveActorInventory,
	TakeActorInventory,
	CheckActorInventory,
	ThingCountName,
	SpawnSpotFacing,
	PlayerClass,
	AndScriptVar,
	AndMapVar,
	AndWorldVar,
	AndGlobalVar,
	AndMapArray,
	AndWorldArray,
	AndGlobalArray,
	EOrScriptVar,
	EOrMapVar,
	EOrWorldVar,
	EOrGlobalVar,
	EOrMapArray,
	EOrWorldArray,
	EOrGlobalArray,
	OrScriptVar,
	OrMapVar,
	OrWorldVar,
	OrGlobalVar,
	OrMapArray,
	OrWorldArray,
	OrGlobalArray,
	LSScriptVar,
	LSMapVar,
	LSWorldVar,
	LSGlobalVar,
	LSMapArray,
	LSWorldArray,
	LSGlobalArray,
	RSScriptVar,
	RSMapar,
	RSWorldVar,
	RSGlobalVar,
	RSMapArray,
	RSWorldArray,
	RSGlobalArray,
	GetPlayerInfo,
	ChangeLevel,
	SectorDamage,
	ReplaceTextures,
	NegateBinary,
	GetActorPitch,
	SetActorPitch,
	PrintBind,
	SetActorState,
	ThingDamage2,
	UseInventory,
	UseActorInventory,
	CheckActorCeilingTexture,
	CheckActorFloorTexture,
	GetActorLightLevel,
	SetMugShotState,
	ThingCountSector,
	ThingCountNameSector,
	CheckPlayerCamera,
	MorphActor,
	UnmorphActor,
	GetPlayerInput,
	ClassifyActor,
	PrintBinary,
	PrintHex,
	CallFunc,
	SaveString,
	PrintMapCHRange,
	PrintWorldCHRange,
	PrintGlobalCHRange,
	StrCpyToMapCHRange,
	StrCpyToWorldCHRange,
	StrCpyToGlobalCHRange,
	PushFunction,
	CallStack,
	ScriptWaitNamed,
	TranslationRange3,
	GotoStack,
	AssignScriptArray,
	PushScriptArray,
	AddScriptArray,
	SubScriptArray,
	MulScriptArray,
	DivScriptArray,
	ModScriptArray,
	IncScriptArray,
	DecScriptArray,
	AndScriptArray,
	EorScriptArray,
	OrScriptArray,
	LsScriptArray,
	RsScriptArray,
	PrintScriptCharArray,
	PrintScriptCHRange,
	StrCpyToScriptCHRange,
	LSpec5Ex,
	LSpec5ExResult,
	TranslationRange4,
	TranslationRange5,
}
