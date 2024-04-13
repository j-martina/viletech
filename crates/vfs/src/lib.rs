//! # VileTechFS
//!
//! VileTech's virtual file system; an abstraction over the operating system's
//! "physical" FS. Real files, directories, and various archives are all merged
//! into one tree so that reading from them is more convenient at all other levels
//! of the engine, without exposing any details of the user's underlying machine.

mod detail;
mod mount;
mod path;
mod refs;

#[cfg(test)]
mod test;

use std::{
	borrow::Cow,
	ops::Range,
	path::{Path, PathBuf},
	string::FromUtf8Error,
	sync::Arc,
};

use indexmap::IndexSet;
use parking_lot::Mutex;
use rayon::prelude::*;
use slotmap::{new_key_type, HopSlotMap};
use util::SmallString;
use zip_structs::zip_error::ZipReadError;

use self::detail::{Compression, Reader};

pub use self::{path::*, refs::*};

#[derive(Debug)]
pub struct VirtualFs {
	pub(crate) root: FolderSlot,
	pub(crate) mounts: Vec<MountInfo>,
	pub(crate) files: HopSlotMap<FileSlot, VFile>,
	pub(crate) folders: HopSlotMap<FolderSlot, VFolder>,
}

impl VirtualFs {
	#[must_use]
	pub fn root(&self) -> FolderRef {
		FolderRef {
			vfs: self,
			slot: self.root,
			vfolder: &self.folders[self.root],
		}
	}

	pub fn mount(&mut self, real_path: &Path, mount_point: &VPath) -> Result<(), Error> {
		if mount_point.byte_len() == 0 {
			return Err(Error::MountPointEmpty);
		}

		if mount_point.as_str().contains(['/', '\\', '*']) {
			return Err(Error::MountPointInvalidChars);
		}

		if self.mounts.iter().any(|mntinfo| {
			mntinfo
				.mount_point
				.as_str()
				.eq_ignore_ascii_case(mount_point.as_str())
		}) {
			return Err(Error::MountPointDuplicate);
		}

		let canon = real_path.canonicalize().map_err(Error::Canonicalize)?;

		if canon.is_symlink() {
			return Err(Error::MountSymlink);
		}

		match mount::mount(self, &canon, mount_point.as_str()) {
			Ok(mntinfo) => {
				self.mounts.push(mntinfo);
				Ok(())
			}
			Err(err) => {
				let to_clean = match self.lookup(mount_point) {
					Some(Ref::File(iref)) => Some(Slot::File(iref.slot)),
					Some(Ref::Folder(oref)) => Some(Slot::Folder(oref.slot)),
					None => None,
				};

				match to_clean {
					Some(Slot::File(islot)) => {
						self.remove_file_by_slot(islot);
					}
					Some(Slot::Folder(oslot)) => {
						self.remove_folder_by_slot(oslot);
					}
					None => {}
				}

				Err(err)
			}
		}
	}

	#[must_use]
	pub fn exists(&self, vpath: &VPath) -> bool {
		self.lookup(vpath).is_some()
	}

	#[must_use]
	pub fn file_exists(&self, slot: FileSlot) -> bool {
		self.files.contains_key(slot)
	}

	#[must_use]
	pub fn folder_exists(&self, slot: FolderSlot) -> bool {
		self.folders.contains_key(slot)
	}

	/// Returns `true` if a file was removed.
	pub fn remove_file_by_slot(&mut self, slot: FileSlot) -> bool {
		let ret = self.files.remove(slot).is_some();

		if let Some(p) = self.mounts.iter().position(|mntinfo| mntinfo.root == slot) {
			self.mounts.remove(p);
		}

		ret
	}

	pub fn remove_folder_by_slot(&mut self, slot: FolderSlot) {
		assert_ne!(slot, self.root, "root folder cannot be removed");
		self.remove_folder_recur(slot);

		if let Some(p) = self.mounts.iter().position(|mntinfo| mntinfo.root == slot) {
			self.mounts.remove(p);
		}
	}

	fn remove_folder_recur(&mut self, oslot: FolderSlot) {
		let parent_slot = self.folders[oslot].parent.unwrap();
		let parent = &mut self.folders[parent_slot];
		let did_remove = parent.subfolders.remove(&oslot);
		debug_assert!(did_remove);

		let subfolders = std::mem::take(&mut self.folders[oslot].subfolders);

		for slot in subfolders {
			self.remove_folder_recur(slot);
			let removed = self.folders.remove(slot);
			debug_assert!(removed.is_some());
		}

		for islot in self.folders[oslot].files.iter().copied() {
			let removed = self.files.remove(islot);
			debug_assert!(removed.is_some());
		}
	}

	pub fn retain<F>(&mut self, mut predicate: F) -> Result<(), Error>
	where
		F: FnMut(&MountInfo) -> bool,
	{
		let mut to_unmount = vec![];

		self.mounts.retain(|mntinfo| {
			if predicate(mntinfo) {
				true
			} else {
				to_unmount.push(mntinfo.root);
				false
			}
		});

		for root in to_unmount {
			match root {
				Slot::File(islot) => {
					let removed = self.files.remove(islot);
					debug_assert!(removed.is_some());
				}
				Slot::Folder(oslot) => {
					self.remove_folder_recur(oslot);
				}
			}
		}

		Ok(())
	}

	pub fn lookup<'vfs: 'p, 'p>(&'vfs self, vpath: &'p VPath) -> Option<Ref<'vfs>> {
		self.lookup_recur(self.root, &self.folders[self.root], vpath.components())
	}

	#[must_use]
	pub fn get_file(&self, slot: FileSlot) -> Option<FileRef> {
		self.files.get(slot).map(|vfile| FileRef {
			vfs: self,
			slot,
			vfile,
		})
	}

	#[must_use]
	pub fn get_folder(&self, slot: FolderSlot) -> Option<FolderRef> {
		self.folders.get(slot).map(|vfolder| FolderRef {
			vfs: self,
			slot,
			vfolder,
		})
	}

	fn lookup_recur<'vfs: 'p, 'p>(
		&'vfs self,
		slot: FolderSlot,
		folder: &'vfs VFolder,
		mut components: impl Iterator<Item = &'p VPath>,
	) -> Option<Ref<'vfs>> {
		let Some(pcomp) = components.next() else {
			return Some(Ref::Folder(FolderRef {
				vfs: self,
				slot,
				vfolder: folder,
			}));
		};

		if let Some((sfslot, subfold)) = folder.subfolders.iter().copied().find_map(|s| {
			let fold = &self.folders[s];

			fold.name
				.eq_ignore_ascii_case(pcomp.as_str())
				.then_some((s, fold))
		}) {
			return self.lookup_recur(sfslot, subfold, components);
		}

		let option = match folder.files.len() {
			// TODO: tweak the parallel search threshold to determine an optima.
			0..=4096 => folder.files.iter().copied().find_map(|slot| {
				let file = &self.files[slot];

				file.name
					.eq_ignore_ascii_case(pcomp.as_str())
					.then_some((slot, file))
			}),
			_ => folder.files.par_iter().copied().find_map_any(|slot| {
				let file = &self.files[slot];

				file.name
					.eq_ignore_ascii_case(pcomp.as_str())
					.then_some((slot, file))
			}),
		};

		let Some((slot, file)) = option else {
			return None;
		};

		Some(Ref::File(FileRef {
			vfs: self,
			slot,
			vfile: file,
		}))
	}

	/// Each virtual file backed by a physical file reads its slice into a buffer
	/// belonging exclusively to that virtual file.
	pub fn ingest_all(&mut self) {
		#[must_use]
		fn ingest(
			reader: &mut Reader,
			orig_span: Range<usize>,
			compression: Compression,
		) -> Option<Vec<u8>> {
			let result = match reader {
				Reader::File(fh) => Reader::read_from_file(fh, orig_span),
				Reader::Memory(_) => return None,
				Reader::_Super(_) => unimplemented!(),
			};

			result
				.and_then(|b| detail::decompress(Cow::Owned(b), compression))
				.ok()
				.map(|cow| cow.into_owned())
		}

		let mut vfiles = self.files.values_mut();

		let Some(vfile0) = vfiles.next() else {
			return;
		};

		let mut guard = vfile0.reader.lock_arc();
		let mut prev_arc = Arc::as_ptr(&vfile0.reader);

		if let Some(bytes) = ingest(&mut guard, vfile0.span(), vfile0.compression) {
			vfile0.span = 0..(bytes.len() as u32);
			vfile0.reader = Arc::new(Mutex::new(Reader::Memory(bytes)));
			vfile0.compression = Compression::None;
		}

		for vfile in vfiles {
			// If the new lock is the same as the previous lock,
			// don't waste time on another re-open.
			let arc_ptr = Arc::as_ptr(&vfile.reader);

			if !std::ptr::eq(arc_ptr, prev_arc) {
				guard = vfile.reader.lock_arc();
			};

			prev_arc = arc_ptr;

			if let Some(bytes) = ingest(&mut guard, vfile.span(), vfile.compression) {
				vfile.span = 0..(bytes.len() as u32);
				vfile.reader = Arc::new(Mutex::new(Reader::Memory(bytes)));
				vfile.compression = Compression::None;
			}
		}
	}

	/// Changes the names of all files and folders to be ASCII lowercase.
	pub fn normalize_names(&mut self) {
		self.files.values_mut().par_bridge().for_each(|vfile| {
			vfile.name.make_ascii_lowercase();
		});

		self.folders.values_mut().for_each(|vfolder| {
			vfolder.name.make_ascii_lowercase();
		});
	}

	#[must_use]
	pub fn mounts(&self) -> &[MountInfo] {
		&self.mounts
	}

	/// Computes in `O(1)` time.
	#[must_use]
	pub fn file_count(&self) -> usize {
		self.files.len()
	}

	/// Computes in `O(1)` time.
	#[must_use]
	pub fn folder_count(&self) -> usize {
		self.folders.len()
	}

	/// Shorthand for adding [`Self::file_count`] to [`Self::folder_count`].
	#[must_use]
	pub fn total_count(&self) -> usize {
		self.file_count() + self.folder_count()
	}

	pub fn files(&self) -> impl Iterator<Item = FileRef> {
		self.files.iter().map(|(k, v)| FileRef {
			vfs: self,
			slot: k,
			vfile: v,
		})
	}

	pub fn folders(&self) -> impl Iterator<Item = FolderRef> {
		self.folders.iter().map(|(k, v)| FolderRef {
			vfs: self,
			slot: k,
			vfolder: v,
		})
	}

	#[must_use]
	pub fn file_is_mount(&self, slot: FileSlot) -> bool {
		self.mounts.iter().any(|mntinfo| mntinfo.root == slot)
	}

	#[must_use]
	pub fn folder_is_mount(&self, slot: FolderSlot) -> bool {
		self.mounts.iter().any(|mntinfo| mntinfo.root == slot)
	}

	pub fn clear(&mut self) {
		let root = self.folders.remove(self.root).unwrap();
		self.folders.clear();
		self.files.clear();
		self.root = self.folders.insert(root);
	}
}

impl Default for VirtualFs {
	fn default() -> Self {
		let mut folders = HopSlotMap::default();

		let root = folders.insert(VFolder {
			name: SmallString::from("/"),
			parent: None,
			files: indexmap::indexset![],
			subfolders: indexmap::indexset![],
			kind: FolderKind::Root,
		});

		Self {
			root,
			mounts: vec![],
			files: HopSlotMap::default(),
			folders,
		}
	}
}

/// Metadata for a file subtree registered using [`VirtualFs::mount`].
#[derive(Debug)]
pub struct MountInfo {
	pub real_path: PathBuf,
	pub mount_point: VPathBuf,
	pub root: Slot,
	pub format: MountFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountFormat {
	Uncompressed,
	Directory,
	Wad,
	Zip,
}

/// Short for "virtual file".
/// May represent a real file or an entry in an archive.
#[derive(Debug)]
pub struct VFile {
	pub(crate) name: SmallString,
	pub(crate) parent: FolderSlot,
	pub(crate) reader: Arc<Mutex<Reader>>,
	pub(crate) span: Range<u32>,
	pub(crate) compression: Compression,
}

impl VFile {
	#[must_use]
	pub fn name(&self) -> &VPath {
		VPath::new(self.name.as_str())
	}

	/// How many bytes are represented by this virtual file?
	/// Beware that this is pre-compression, if any.
	#[must_use]
	pub fn size(&self) -> usize {
		self.span().len()
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.span.is_empty()
	}

	#[must_use]
	fn span(&self) -> Range<usize> {
		(self.span.start as usize)..(self.span.end as usize)
	}
}

/// Short for "virtual folder".
/// May represent a real directory or a logical directory in a (non-WAD) archive.
#[derive(Debug)]
pub struct VFolder {
	pub(crate) name: SmallString,
	/// Only `None` for the root.
	pub(crate) parent: Option<FolderSlot>,
	pub(crate) files: IndexSet<FileSlot>,
	pub(crate) subfolders: IndexSet<FolderSlot>,
	pub(crate) kind: FolderKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderKind {
	Directory,
	Root,
	Wad,
	Zip,
	ZipDir,
}

impl VFolder {
	#[must_use]
	pub fn name(&self) -> &VPath {
		VPath::new(self.name.as_str())
	}

	/// Only returns `None` if this is the root folder.
	#[must_use]
	pub fn parent(&self) -> Option<FolderSlot> {
		self.parent
	}

	#[must_use]
	pub fn kind(&self) -> FolderKind {
		self.kind
	}

	/// Computes in `O(1)` time.
	#[must_use]
	pub fn file_count(&self) -> usize {
		self.files.len()
	}

	/// Computes in `O(1)` time.
	#[must_use]
	pub fn subfolder_count(&self) -> usize {
		self.subfolders.len()
	}

	/// Shorthand for `self.file_count() + self.subfolder_count()`.
	#[must_use]
	pub fn child_count(&self) -> usize {
		self.files.len() + self.subfolders.len()
	}
}

new_key_type! {
	/// A unique identifier for a virtual file. This is always valid for the
	/// VFS which emitted it, regardless of what mounts/unmounts/insertions/removals
	/// take place.
	///
	/// Using this in a VFS other than the one that emitted it will yield
	/// unexpected results but is safe.
	///
	/// Also see [`Slot`].
	pub struct FileSlot;
	/// A unique identifier for a virtual folder. This is always valid for the
	/// VFS which emitted it, regardless of what mounts/unmounts/insertions/removals
	/// take place.
	///
	/// Using this in a VFS other than the one that emitted it will yield
	/// unexpected results but is safe.
	///
	/// Also see [`Slot`].
	pub struct FolderSlot;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Slot {
	File(FileSlot),
	Folder(FolderSlot),
}

impl PartialEq<FileSlot> for Slot {
	fn eq(&self, other: &FileSlot) -> bool {
		match self {
			Self::File(islot) => *islot == *other,
			Self::Folder(_) => false,
		}
	}
}

impl PartialEq<FolderSlot> for Slot {
	fn eq(&self, other: &FolderSlot) -> bool {
		match self {
			Self::Folder(oslot) => *oslot == *other,
			Self::File(_) => false,
		}
	}
}

impl From<FileSlot> for Slot {
	fn from(value: FileSlot) -> Self {
		Self::File(value)
	}
}

impl From<FolderSlot> for Slot {
	fn from(value: FolderSlot) -> Self {
		Self::Folder(value)
	}
}

#[derive(Debug)]
pub enum Error {
	Canonicalize(std::io::Error),
	Decompress(std::io::Error),
	DirRead(std::io::Error),
	EmptyRead,
	FileHandleClone(std::io::Error),
	FileOpen(std::io::Error),
	FileRead(std::io::Error),
	Metadata(std::io::Error),
	MountPointDuplicate,
	MountPointEmpty,
	MountPointInvalidChars,
	MountSymlink,
	NotFound,
	Seek(std::io::Error),
	Utf8(FromUtf8Error),
	VFolderRead,
	Wad(wadload::Error),
	Zip(ZipReadError),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Canonicalize(err) => write!(f, "failed to canonicalize a mount path: {err}"),
			Self::Decompress(err) => write!(f, "failed to decompress an archive entry: {err}"),
			Self::DirRead(err) => write!(
				f,
				"failed to get the contents of a physical directory: {err}"
			),
			Self::EmptyRead => write!(f, "attempted to read the byte content of an empty entry"),
			Self::FileHandleClone(err) => {
				write!(f, "failed to clone a physical file handle: {err}")
			}
			Self::FileOpen(err) => write!(f, "failed to open a physical file handle: {err}"),
			Self::FileRead(err) => write!(f, "failed to read a physical file: {err}"),
			Self::Metadata(err) => write!(f, "failed to retrieve physical file metadata: {err}"),
			Self::MountPointDuplicate => {
				write!(f, "attempt a mount using an already-present mount point")
			}
			Self::MountPointEmpty => write!(f, "given mount point is empty"),
			Self::MountPointInvalidChars => write!(f, "given mount point has invalid characters"),
			Self::NotFound => write!(f, "no entry found by the given path"),
			Self::Seek(err) => write!(f, "failed to seek a physical file handle: {err}"),
			Self::MountSymlink => write!(f, "attempted to mount a symbolic link"),
			Self::Utf8(err) => write!(f, "failed to read UTF-8 text from a virtual file: {err}"),
			Self::VFolderRead => write!(f, "attempted to read byte content of a virtual folder"),
			Self::Wad(err) => write!(f, "WAD read error: {err}"),
			Self::Zip(err) => write!(f, "zip archive read error: {err}"),
		}
	}
}
