//! Sound- and music-related code.

/*

Copyright (C) 2022 ***REMOVED***

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.

*/

use std::{
	io::{self, Read, Seek},
	ops::{Deref, DerefMut},
	path::{Path, PathBuf},
	time::Duration,
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
use once_cell::sync::Lazy;
use zmusic::{config::SoundFontKindMask, soundfont};

use crate::{ecs::EntityId, utils, vfs::FileRef};

pub struct SourcedHandle {
	inner: StaticSoundHandle,
	#[allow(unused)]
	source: Option<EntityId>,
}

impl Deref for SourcedHandle {
	type Target = StaticSoundHandle;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl DerefMut for SourcedHandle {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}

pub enum MusicHandle {
	Raw(StaticSoundHandle),
	Midi(zmusic::Song),
}

#[non_exhaustive]
pub struct AudioCore {
	pub soundfonts: Vec<SoundFont>,
	pub manager: AudioManager,
	/// General-purpose music slot.
	pub music1: Option<MusicHandle>,
	/// Secondary music slot. Allows scripts to set a song to pause the level's
	/// main song, briefly play another piece, and then carry on with `music1`
	/// wherever it left off.
	pub music2: Option<MusicHandle>,
	/// Sounds currently being played.
	pub sounds: Vec<SourcedHandle>,
}

pub type PlayError = PlaySoundError<<StaticSoundData as SoundData>::Error>;

const TWEEN_INSTANT: Tween = Tween {
	start_time: kira::StartTime::Immediate,
	duration: Duration::ZERO,
	easing: kira::tween::Easing::Linear,
};

impl AudioCore {
	/// If `None` is given, the defaults will be used.
	pub fn new(manager_settings: Option<AudioManagerSettings<CpalBackend>>) -> Result<Self, Error> {
		let manager_settings = manager_settings.unwrap_or_default();
		let sound_cap = manager_settings.capacities.sound_capacity;

		let mut ret = Self {
			soundfonts: Vec::with_capacity(1),
			manager: AudioManager::new(manager_settings).map_err(Error::Backend)?,
			music1: None,
			music2: None,
			sounds: Vec::with_capacity(sound_cap),
		};

		ret.collect_soundfonts()?;

		Ok(ret)
	}

	/// Clear handles for sounds which have finished playing.
	pub fn update(&mut self) {
		let mut i = 0;

		while i < self.sounds.len() {
			if self.sounds[i].state() == PlaybackState::Stopped {
				self.sounds.swap_remove(i);
			} else {
				i += 1;
			}
		}

		if let Some(MusicHandle::Midi(midi)) = &mut self.music1 {
			midi.update();

			if !midi.is_playing() {
				let _ = self.music1.take();
			}
		}

		if let Some(MusicHandle::Midi(midi)) = &mut self.music2 {
			midi.update();

			if !midi.is_playing() {
				let _ = self.music2.take();
			}
		}
	}

	pub fn start_music_raw<const SLOT2: bool>(
		&mut self,
		data: StaticSoundData,
	) -> Result<(), PlayError> {
		let handle = self.manager.play(data)?;

		if !SLOT2 {
			self.music1 = Some(MusicHandle::Raw(handle));
		} else {
			self.music2 = Some(MusicHandle::Raw(handle));
		}

		Ok(())
	}

	/// `song` should not have started playing yet; a debug assertion guards
	/// against this, so make sure of it.
	pub fn start_music_midi<const SLOT2: bool>(
		&mut self,
		mut song: zmusic::Song,
		looping: bool,
	) -> Result<(), Error> {
		debug_assert!(!song.is_playing());

		song.start(looping).map_err(Error::Midi)?;

		if !SLOT2 {
			self.music1 = Some(MusicHandle::Midi(song));
		} else {
			self.music2 = Some(MusicHandle::Midi(song));
		}

		Ok(())
	}

	/// Instantly stops the music track in the requested slot and then empties it.
	pub fn stop_music<const SLOT2: bool>(&mut self) -> Result<(), kira::CommandError> {
		let slot = if !SLOT2 {
			&mut self.music1
		} else {
			&mut self.music2
		};

		let res = match slot {
			Some(MusicHandle::Midi(midi)) => {
				midi.stop();
				Ok(())
			}
			Some(MusicHandle::Raw(raw)) => raw.stop(TWEEN_INSTANT),
			None => Ok(()),
		};

		*slot = None;
		res
	}

	/// Play a sound without an in-world source.
	/// Always audible to all clients, not subject to panning or attenuation.
	pub fn start_sound_global(&mut self, data: StaticSoundData) -> Result<(), PlayError> {
		self.sounds.push(SourcedHandle {
			inner: self.manager.play(data)?,
			source: None,
		});

		Ok(())
	}

	pub fn start_sound_sourced(
		&mut self,
		data: StaticSoundData,
		entity: EntityId,
	) -> Result<(), PlayError> {
		self.sounds.push(SourcedHandle {
			inner: self.manager.play(data)?,
			source: Some(entity),
		});

		Ok(())
	}

	/// Instantly pauses every sound and music handle.
	pub fn pause_all(&mut self) {
		for handle in &mut self.sounds {
			let res = handle.pause(TWEEN_INSTANT);
			debug_assert!(res.is_ok(), "Failed to pause a sound: {}", res.unwrap_err());
		}

		match &mut self.music1 {
			Some(MusicHandle::Raw(raw)) => {
				let res = raw.pause(TWEEN_INSTANT);
				debug_assert!(res.is_ok(), "Failed to pause music 1: {}", res.unwrap_err());
			}
			Some(MusicHandle::Midi(midi)) => midi.pause(),
			None => {}
		}

		match &mut self.music2 {
			Some(MusicHandle::Raw(raw)) => {
				let res = raw.pause(TWEEN_INSTANT);
				debug_assert!(res.is_ok(), "Failed to pause music 2: {}", res.unwrap_err());
			}
			Some(MusicHandle::Midi(midi)) => midi.pause(),
			None => {}
		}
	}

	/// Instantly resumes every sound and music handle.
	pub fn resume_all(&mut self) {
		for handle in &mut self.sounds {
			let res = handle.resume(TWEEN_INSTANT);

			debug_assert!(
				res.is_ok(),
				"Failed to resume a sound: {}",
				res.unwrap_err()
			);
		}

		match &mut self.music1 {
			Some(MusicHandle::Raw(raw)) => {
				let res = raw.resume(TWEEN_INSTANT);
				debug_assert!(
					res.is_ok(),
					"Failed to resume music 1: {}",
					res.unwrap_err()
				);
			}
			Some(MusicHandle::Midi(midi)) => midi.resume(),
			None => {}
		}

		match &mut self.music2 {
			Some(MusicHandle::Raw(raw)) => {
				let res = raw.resume(TWEEN_INSTANT);
				debug_assert!(
					res.is_ok(),
					"Failed to resume music 2: {}",
					res.unwrap_err()
				);
			}
			Some(MusicHandle::Midi(midi)) => midi.resume(),
			None => {}
		}
	}

	/// Instantly stops every ongoing sound and music track. The sound array
	/// gets cleared along with both music slots.
	pub fn stop_all(&mut self) -> Result<(), Error> {
		for sound in &mut self.sounds {
			sound.stop(TWEEN_INSTANT).map_err(Error::Command)?;
		}

		self.sounds.clear();

		self.stop_music::<false>().map_err(Error::Command)?;
		self.stop_music::<true>().map_err(Error::Command)?;

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
						"Failed to retrieve metadata for file: {}\r\nError: {}",
						path.display(),
						err
					);
					continue;
				}
			};

			if metadata.is_dir() || metadata.is_symlink() || metadata.len() == 0 {
				continue;
			}

			// Check if another SoundFont by this name has already been collected
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
				soundfont::Kind::Sf2
			} else if &header[..11] == b"WOPL3-BANK\0" {
				soundfont::Kind::Wopl
			} else if &header[..11] == b"WOPN2-BANK\0" {
				soundfont::Kind::Wopn
			} else if utils::io::is_zip(&header) {
				soundfont::Kind::Gus
			} else {
				info!(
					"Failed to determine SoundFont type of file: {}\r\nSkipping it.",
					path.display()
				);
				continue;
			};

			if sf_kind == soundfont::Kind::Gus {
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

				// [GZ] A SoundFont archive with only one file can't be a packed GUS patch.
				// Just skip this entirely
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

				// This GUS SoundFont has been validated. Now it can be pushed
			}

			self.soundfonts
				.push(SoundFont::new(path.to_path_buf(), sf_kind));
		}

		if self.soundfonts.is_empty() {
			Err(Error::NoSoundFonts)
		} else {
			Ok(())
		}
	}

	#[must_use]
	pub fn find_soundfont(&self, name: &str, mask: SoundFontKindMask) -> Option<&SoundFont> {
		for sf in &self.soundfonts {
			if !mask.is_allowed(sf.kind()) {
				continue;
			}

			if sf
				.name()
				.to_string_lossy()
				.as_ref()
				.eq_ignore_ascii_case(name)
			{
				return Some(sf);
			}

			if sf
				.name_ext()
				.to_string_lossy()
				.as_ref()
				.eq_ignore_ascii_case(name)
			{
				return Some(sf);
			}

			continue;
		}

		return self.soundfonts.iter().find(|sf| mask.is_allowed(sf.kind()));
	}
}

static SOUNDFONT_DIR: Lazy<PathBuf> = Lazy::new(|| {
	#[cfg(not(debug_assertions))]
	{
		let ret = utils::path::exe_dir().join("soundfonts");

		if !ret.exists() {
			std::fs::create_dir(ret).or_else(|err| {
				panic!(
					"Failed to create directory: {}\r\nError: {}",
					ret.display(),
					err
				)
			})
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

pub fn sound_from_file(
	file: FileRef,
	settings: StaticSoundSettings,
) -> Result<StaticSoundData, Box<dyn std::error::Error>> {
	let bytes = file.read()?.to_owned();
	let cursor = io::Cursor::new(bytes);

	match StaticSoundData::from_cursor(cursor, settings) {
		Ok(ssd) => Ok(ssd),
		Err(err) => Err(Box::new(err)),
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct SoundFont {
	/// The canonicalized path to this SoundFont's file.
	/// Needed by the FluidSynth backend of the ZMusic library.
	path: PathBuf,
	kind: soundfont::Kind,
}

impl SoundFont {
	#[must_use]
	pub fn new(path: PathBuf, kind: soundfont::Kind) -> Self {
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
	/// Needed by the FluidSynth backend of the ZMusic library.
	#[must_use]
	pub fn full_path(&self) -> &Path {
		&self.path
	}

	#[must_use]
	pub fn kind(&self) -> soundfont::Kind {
		self.kind
	}
}

#[derive(Debug)]
pub enum Error {
	Backend(<CpalBackend as Backend>::Error),
	Command(kira::CommandError),
	Midi(zmusic::Error),
	NoSoundFonts,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Backend(err) => err.fmt(f),
			Self::Command(err) => err.fmt(f),
			Self::Midi(err) => err.fmt(f),
			Self::NoSoundFonts => write!(
				f,
				"No SoundFont files found under path: {}",
				SOUNDFONT_DIR.to_string_lossy().as_ref()
			),
		}
	}
}
