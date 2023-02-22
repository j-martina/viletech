//! Sound- and music-related code.

mod gui;
mod midi;

use std::{
	io::{Cursor, Read, Seek},
	ops::{Deref, DerefMut},
	path::{Path, PathBuf},
	sync::Arc,
};

use kira::{
	manager::{
		backend::{cpal::CpalBackend, Backend},
		error::PlaySoundError,
		AudioManager, AudioManagerSettings,
	},
	sound::{
		static_sound::{PlaybackState, StaticSoundData, StaticSoundHandle, StaticSoundSettings},
		SoundData,
	},
	tween::Tween,
};
use log::{info, warn};
use nodi::midly;
use once_cell::sync::Lazy;
use parking_lot::RwLock;

use crate::{
	data::{Catalog, FileRef},
	sim::ActorId,
	utils,
};

pub use midi::{
	render as render_midi, Data as MidiData, Handle as MidiHandle, Settings as MidiSettings,
};

use self::gui::DeveloperGui;

pub struct AudioCore {
	/// The centre of waveform sound synthesis and playback.
	pub manager: AudioManager,
	pub soundfonts: Vec<SoundFont>,
	/// General-purpose music slot.
	pub music1: Option<Handle>,
	/// Secondary music slot. Allows scripts to set a song to pause the level's
	/// main song, briefly play another piece, and then carry on with `music1`
	/// wherever it left off.
	pub music2: Option<Handle>,
	/// Sounds currently being played.
	pub sounds: Vec<Sound>,
	catalog: Arc<RwLock<Catalog>>,
	gui: DeveloperGui,
}

impl AudioCore {
	/// If `None` is given, the defaults will be used.
	pub fn new(
		catalog: Arc<RwLock<Catalog>>,
		manager_settings: Option<AudioManagerSettings<CpalBackend>>,
	) -> Result<Self, Error> {
		let manager_settings = manager_settings.unwrap_or_default();
		let sound_cap = manager_settings.capacities.sound_capacity;

		fluidlite::Log::set(
			fluidlite::LogLevel::DEBUG,
			fluidlite::FnLogger::new(|level, msg| match level {
				fluidlite::LogLevel::Panic => log::error!(target: "fluidlite", "(FATAL): {msg}"),
				fluidlite::LogLevel::Error => log::error!(target: "fluidlite", "msg"),
				fluidlite::LogLevel::Warning => log::warn!(target: "fluidlite", "{msg}"),
				fluidlite::LogLevel::Info => log::info!(target: "fluidlite", "{msg}"),
				fluidlite::LogLevel::Debug => log::debug!(target: "fluidlite", "{msg}"),
			}),
		);

		let mut ret = Self {
			catalog,
			manager: AudioManager::new(manager_settings).map_err(Error::KiraBackend)?,
			soundfonts: Vec::with_capacity(1),
			music1: None,
			music2: None,
			sounds: Vec::with_capacity(sound_cap),
			gui: DeveloperGui::default(),
		};

		ret.collect_soundfonts()?;

		if !ret.soundfonts.is_empty() {
			let cow = ret.soundfonts[0].path.to_string_lossy();
			ret.gui.soundfont_buf = cow.to_string();
		}

		Ok(ret)
	}

	/// Sound handles which have finished playing get swap-removed.
	/// Music handles which have finished playing get assigned `None`.
	pub fn update(&mut self) {
		let mut i = 0;

		while i < self.sounds.len() {
			if self.sounds[i].state() == PlaybackState::Stopped {
				self.sounds.swap_remove(i);
			} else {
				i += 1;
			}
		}

		if let Some(mus) = &mut self.music1 {
			if mus.state() == PlaybackState::Stopped {
				let _ = self.music1.take();
			}
		}
	}

	/// This assumes that `data` has already been completely configured.
	pub fn start_music_wave<const SLOT2: bool>(
		&mut self,
		data: StaticSoundData,
	) -> Result<(), Error> {
		let handle = self.manager.play(data).map_err(Error::PlayWave)?;

		if !SLOT2 {
			self.music1 = Some(Handle::Wave(handle));
		} else {
			self.music2 = Some(Handle::Wave(handle));
		}

		Ok(())
	}

	/// Returns an error if:
	/// - The given song fails to start playback.
	/// - The given music slot fails to stop and be cleared.
	pub fn start_music_midi<const SLOT2: bool>(&mut self, data: MidiData) -> Result<(), Error> {
		let handle = self.manager.play(data).map_err(Error::PlayMidi)?;
		self.stop_music::<SLOT2>()?;

		if !SLOT2 {
			self.music1 = Some(Handle::Midi(handle));
		} else {
			self.music2 = Some(Handle::Midi(handle));
		}

		Ok(())
	}

	/// Instantly stops the music track in the requested slot and then empties it.
	pub fn stop_music<const SLOT2: bool>(&mut self) -> Result<(), Error> {
		let slot = if !SLOT2 {
			&mut self.music1
		} else {
			&mut self.music2
		};

		let res = if let Some(mus) = slot {
			mus.stop(Tween::default())
		} else {
			return Ok(());
		};

		*slot = None;
		res
	}

	/// If no `source` is given, the sound will always audible to all clients
	/// and not be subjected to any panning or attenuation.
	pub fn start_sound_wave(
		&mut self,
		data: StaticSoundData,
		source: Option<ActorId>,
	) -> Result<(), Error> {
		self.sounds.push(Sound {
			handle: Handle::Wave(self.manager.play(data).map_err(Error::PlayWave)?),
			_source: source,
		});

		Ok(())
	}

	/// If no `source` is given, the sound will always audible to all clients
	/// and not be subjected to any panning or attenuation.
	pub fn start_sound_midi(
		&mut self,
		data: MidiData,
		source: Option<ActorId>,
	) -> Result<(), Error> {
		self.sounds.push(Sound {
			handle: Handle::Midi(self.manager.play(data).map_err(Error::PlayMidi)?),
			_source: source,
		});

		Ok(())
	}

	/// Instantly pauses every sound and music handle.
	pub fn pause_all(&mut self) {
		for handle in &mut self.sounds {
			let res = handle.pause(Tween::default());
			debug_assert!(res.is_ok(), "Failed to pause a sound: {}", res.unwrap_err());
		}

		if let Some(mus) = &mut self.music1 {
			let res = mus.pause(Tween::default());
			debug_assert!(res.is_ok(), "Failed to pause music 1: {}", res.unwrap_err());
		}

		if let Some(mus) = &mut self.music2 {
			let res = mus.pause(Tween::default());
			debug_assert!(res.is_ok(), "Failed to pause music 2: {}", res.unwrap_err());
		}
	}

	/// Instantly resumes every sound and music handle.
	pub fn resume_all(&mut self) {
		for handle in &mut self.sounds {
			let res = handle.resume(Tween::default());

			debug_assert!(
				res.is_ok(),
				"Failed to resume a sound: {}",
				res.unwrap_err()
			);
		}

		if let Some(mus) = &mut self.music1 {
			let res = mus.resume(Tween::default());

			debug_assert!(
				res.is_ok(),
				"Failed to resume music 1: {}",
				res.unwrap_err()
			);
		}

		if let Some(mus) = &mut self.music2 {
			let res = mus.resume(Tween::default());

			debug_assert!(
				res.is_ok(),
				"Failed to resume music 2: {}",
				res.unwrap_err()
			);
		}
	}

	/// Instantly stops every ongoing sound and music track. The sound array
	/// gets cleared along with both music slots.
	pub fn stop_all(&mut self) -> Result<(), Error> {
		for sound in &mut self.sounds {
			sound.stop(Tween::default())?;
		}

		self.sounds.clear();

		self.stop_music::<false>()?;
		self.stop_music::<true>()?;

		Ok(())
	}

	/// A fundamental part of engine initialization. Recursively read the contents of
	/// `<executable_directory>/soundfonts`, determine their types, and store their
	/// paths. Note that in the debug build, `<working_directory>/data/soundfonts`
	/// will be walked instead. Returns [`Error::NoSoundFonts`] if no SoundFont
	/// files whatsoever could be found. This should never be considered fatal;
	/// it just means the engine won't be able to render MIDIs.
	pub fn collect_soundfonts(&mut self) -> Result<(), Error> {
		self.soundfonts.clear();

		let walker = walkdir::WalkDir::new::<&Path>(SOUNDFONT_DIR.as_ref())
			.follow_links(false)
			.max_depth(8)
			.same_file_system(true)
			.sort_by_file_name()
			.into_iter()
			.filter_map(|res| res.ok());

		for dir_entry in walker {
			let path = dir_entry.path();

			let metadata = match dir_entry.metadata() {
				Ok(m) => m,
				Err(err) => {
					warn!(
						"Failed to retrieve metadata for file: {}\r\n\tError: {err}",
						path.display(),
					);
					continue;
				}
			};

			if metadata.is_dir() || metadata.is_symlink() || metadata.len() == 0 {
				continue;
			}

			// Check if another SoundFont by this name has already been collected.
			if self
				.soundfonts
				.iter()
				.any(|sf| sf.name().as_os_str().eq_ignore_ascii_case(path.as_os_str()))
			{
				continue;
			}

			let mut file = match std::fs::File::open(path) {
				Ok(f) => f,
				Err(err) => {
					warn!("Failed to open file: {}\r\nError: {}", path.display(), err);
					continue;
				}
			};

			let mut header = [0_u8; 16];

			match file.read_exact(&mut header) {
				Ok(()) => {}
				Err(err) => {
					warn!("Failed to read file: {}\r\nError: {}", path.display(), err);
				}
			};

			let sf_kind = if &header[0..4] == b"RIFF" && &header[8..16] == b"sfbkLIST" {
				SoundFontKind::Sf2
			} else if &header[..11] == b"WOPL3-BANK\0" {
				SoundFontKind::Wopl
			} else if &header[..11] == b"WOPN2-BANK\0" {
				SoundFontKind::Wopn
			} else if utils::io::is_zip(&header) {
				SoundFontKind::Gus
			} else {
				info!(
					"Failed to determine SoundFont type of file: {}\r\nSkipping it.",
					path.display()
				);
				continue;
			};

			if sf_kind == SoundFontKind::Gus {
				match file.rewind() {
					Ok(()) => {}
					Err(err) => {
						warn!(
							"Failed to rewind file stream for zip read: {}\r\nError: {}",
							path.display(),
							err
						);
						continue;
					}
				};

				let mut archive = match zip::ZipArchive::new(&mut file) {
					Ok(zf) => zf,
					Err(err) => {
						warn!("Failed to unzip file: {}\r\nError: {}", path.display(), err);
						continue;
					}
				};

				// (GZ)
				// A SoundFont archive with only one file can't be a packed GUS patch.
				// Just skip this entirely.
				if archive.len() <= 1 {
					continue;
				}

				let timidity = match archive.by_name("timidity.cfg") {
					Ok(timid) => timid,
					Err(err) => {
						warn!(
							"Failed to find `timidity.cfg` file in: {}\r\nError: {}",
							path.display(),
							err
						);
						continue;
					}
				};

				if !timidity.is_file() || timidity.size() < 1 {
					warn!(
						"Found `timidity.cfg` in a zip SoundFont but it's malformed. ({})",
						path.display()
					);
					continue;
				}

				// This GUS SoundFont has been validated. Now it can be pushed.
			}

			self.soundfonts.push(SoundFont {
				path: path.to_owned(),
				kind: sf_kind,
			});
		}

		if self.soundfonts.is_empty() {
			Err(Error::NoSoundFonts)
		} else {
			Ok(())
		}
	}

	/// Draw the egui-based developer/debug/diagnosis menu, and perform any
	/// state mutations requested through it by the user.
	pub fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
		self.ui_impl(ctx, ui);
	}
}

impl std::fmt::Debug for AudioCore {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("AudioCore")
			.field("soundfonts", &self.soundfonts)
			.field("music1", &self.music1)
			.field("music2", &self.music2)
			.field("sounds", &self.sounds)
			.field("catalog", &self.catalog)
			.field("gui", &self.gui)
			.finish()
	}
}

/// Enables inspection and control of a currently-playing sound or musical track,
/// whether it's waveform or MIDI.
pub enum Handle {
	Wave(StaticSoundHandle),
	Midi(MidiHandle),
}

impl Handle {
	#[must_use]
	pub fn state(&self) -> PlaybackState {
		match self {
			Self::Wave(wave) => wave.state(),
			Self::Midi(midi) => midi.state(),
		}
	}

	pub fn pause(&mut self, tween: Tween) -> Result<(), Error> {
		match self {
			Handle::Wave(wave) => wave.pause(tween).map_err(Error::CommandWave),
			Handle::Midi(midi) => midi.pause(tween),
		}
	}

	pub fn resume(&mut self, tween: Tween) -> Result<(), Error> {
		match self {
			Handle::Wave(wave) => wave.resume(tween).map_err(Error::CommandWave),
			Handle::Midi(midi) => midi.resume(tween),
		}
	}

	pub fn stop(&mut self, tween: Tween) -> Result<(), Error> {
		match self {
			Handle::Wave(wave) => wave.stop(tween).map_err(Error::CommandWave),
			Handle::Midi(midi) => midi.stop(tween),
		}
	}

	#[must_use]
	pub fn is_playing(&self) -> bool {
		match self {
			Handle::Wave(wave) => wave.state() == PlaybackState::Playing,
			Handle::Midi(midi) => midi.is_playing(),
		}
	}
}

impl std::fmt::Debug for Handle {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Wave(_) => f.debug_tuple("Wave").finish(),
			Self::Midi(_) => f.debug_tuple("Midi").finish(),
		}
	}
}

#[derive(Debug)]
pub struct Sound {
	handle: Handle,
	_source: Option<ActorId>,
}

impl Deref for Sound {
	type Target = Handle;

	fn deref(&self) -> &Self::Target {
		&self.handle
	}
}

impl DerefMut for Sound {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.handle
	}
}

pub fn sound_from_file(
	file: FileRef,
	settings: StaticSoundSettings,
) -> Result<StaticSoundData, Box<dyn std::error::Error>> {
	let bytes = file.try_read_bytes()?.to_owned();
	let cursor = Cursor::new(bytes);

	match StaticSoundData::from_cursor(cursor, settings) {
		Ok(ssd) => Ok(ssd),
		Err(err) => Err(Box::new(err)),
	}
}

pub fn sound_from_bytes(
	bytes: impl Into<Vec<u8>>,
	settings: StaticSoundSettings,
) -> Result<StaticSoundData, kira::sound::FromFileError> {
	let cursor = Cursor::new(bytes.into());
	StaticSoundData::from_cursor(cursor, settings)
}

#[derive(Debug, Clone, PartialEq)]
pub struct SoundFont {
	/// The canonicalized path to this SoundFont's file.
	/// Needed by the FluidSynth backend.
	path: PathBuf,
	kind: SoundFontKind,
}

impl SoundFont {
	#[must_use]
	pub fn new(path: PathBuf, kind: SoundFontKind) -> Self {
		Self { path, kind }
	}

	/// The name of the SoundFont file, without the extension (i.e. the file stem).
	#[must_use]
	pub fn name(&self) -> &Path {
		Path::new(self.path.file_stem().unwrap_or_default())
	}

	/// The name of the SoundFont file, along with the extension.
	#[must_use]
	pub fn name_ext(&self) -> &Path {
		Path::new(self.path.file_name().unwrap_or_default())
	}

	/// The canonicalized path to this SoundFont's file.
	#[must_use]
	pub fn full_path(&self) -> &Path {
		&self.path
	}

	#[must_use]
	pub fn kind(&self) -> SoundFontKind {
		self.kind
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundFontKind {
	Sf2,
	Gus,
	Wopl,
	Wopn,
}

static SOUNDFONT_DIR: Lazy<PathBuf> = Lazy::new(|| {
	#[cfg(not(debug_assertions))]
	{
		let ret = utils::path::exe_dir().join("soundfonts");

		if !ret.exists() {
			let res = std::fs::create_dir(&ret);

			if let Err(err) = res {
				panic!(
					"Failed to create directory: {}\r\n\tError: {}",
					ret.display(),
					err
				)
			}
		}

		ret
	}

	#[cfg(debug_assertions)]
	{
		[
			std::env::current_dir().expect("Failed to get working directory."),
			PathBuf::from("data/soundfonts"),
		]
		.iter()
		.collect()
	}
});

#[must_use]
pub fn soundfont_dir() -> &'static Path {
	&SOUNDFONT_DIR
}

#[derive(Debug)]
pub enum Error {
	NoSoundFonts,
	SoundFontRead(PathBuf, fluidlite::Error),
	KiraBackend(<CpalBackend as Backend>::Error),
	ParseMidi(midly::Error),
	MidiSynth(fluidlite::Error),
	PlayMidi(PlayMidiError),
	PlayWave(PlayWaveError),
	CommandMidi,
	CommandWave(kira::CommandError),
}

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		None
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::NoSoundFonts => write!(
				f,
				"No SoundFont files found under path: {}",
				SOUNDFONT_DIR.to_string_lossy().as_ref()
			),
			Self::SoundFontRead(path, err) => write!(
				f,
				"Failed to read SoundFont at path: {p}\r\n\t\
				Details: {err}",
				p = path.display()
			),
			Self::KiraBackend(err) => err.fmt(f),
			Self::ParseMidi(err) => err.fmt(f),
			Self::MidiSynth(err) => err.fmt(f),
			Self::PlayMidi(err) => err.fmt(f),
			Self::PlayWave(err) => err.fmt(f),
			Self::CommandMidi => write!(f, "Failed to send a command to a MIDI sound."),
			Self::CommandWave(err) => err.fmt(f),
		}
	}
}

pub type PlayWaveError = PlaySoundError<<StaticSoundData as SoundData>::Error>;
pub type PlayMidiError = PlaySoundError<<MidiData as SoundData>::Error>;
