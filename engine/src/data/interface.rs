//! Assorted parts of the public API, in a separate file for cleanliness.

use std::{
	any::TypeId,
	collections::HashMap,
	marker::PhantomData,
	path::Path,
	sync::{
		atomic::{self, AtomicU32, AtomicU64},
		Arc, Weak,
	},
};

use parking_lot::{RwLock, RwLockReadGuard};

use crate::{lith, EditorNum, ShortId, SpawnNum, VPath};

use super::{
	detail::{AssetKey, VfsKey},
	Asset, Catalog, Record, VirtFileKind, VirtualFile,
};

/// "Interned string". Wraps an [`Arc`].
/// At the moment, an [`RwLock`] is used to protect the string within.
#[derive(Debug, Clone)]
pub struct InString(Arc<RwLock<Box<str>>>);

impl InString {
	pub fn get(&self) -> RwLockReadGuard<'_, Box<str>> {
		self.0.read()
	}
}

impl PartialEq for InString {
	/// Check if these are two pointers to the same interned string.
	fn eq(&self, other: &Self) -> bool {
		Arc::ptr_eq(&self.0, &other.0)
	}
}

impl Eq for InString {}

impl<T: AsRef<str>> From<T> for InString {
	fn from(value: T) -> Self {
		Self(Arc::new(RwLock::new(
			value.as_ref().to_string().into_boxed_str(),
		)))
	}
}

// FileRef /////////////////////////////////////////////////////////////////////

/// The primary interface for quick introspection into the virtual file system;
/// provides read access to one entry and the catalog itself. Prefer these over
/// working directly with references to [`VirtualFile`]s, since this can trace
/// inter-file relationships.
#[derive(Debug, Clone)]
pub struct FileRef<'cat> {
	pub(super) catalog: &'cat Catalog,
	pub(super) file: &'cat VirtualFile,
}

impl<'cat> FileRef<'cat> {
	/// The catalog this reference came from.
	#[must_use]
	pub fn catalog(&self) -> &Catalog {
		self.catalog
	}

	/// This only returns `None` if this file is the root directory.
	#[must_use]
	pub fn parent(&self) -> Option<&VirtualFile> {
		if let Some(parent) = self.file.parent_path() {
			Some(
				self.catalog
					.files
					.get(&VfsKey::new(parent))
					.expect("A VFS entry has a dangling parent path."),
			)
		} else {
			None
		}
	}

	/// This only returns `None` if this file is the root directory.
	#[must_use]
	pub fn parent_ref(&'cat self) -> Option<Self> {
		self.parent().map(|file| Self {
			catalog: self.catalog,
			file,
		})
	}

	/// If this file is not a directory, or is an empty directory, the returned
	/// iterator will yield no items.
	pub fn children(&self) -> impl Iterator<Item = &VirtualFile> {
		let closure = |key: &VfsKey| {
			self.catalog
				.files
				.get(key)
				.expect("A VFS directory has a dangling child key.")
		};

		match &self.file.kind {
			VirtFileKind::Directory(children) => children.iter().map(closure),
			_ => [].iter().map(closure),
		}
	}

	/// If this file is not a directory, or is an empty directory, the returned
	/// iterator will yield no items.
	pub fn child_refs(&'cat self) -> impl Iterator<Item = FileRef<'cat>> + '_ {
		self.children().map(|file| Self {
			catalog: self.catalog,
			file,
		})
	}

	/// Returns 0 if this is a leaf node or an empty directory.
	#[must_use]
	pub fn child_count(&self) -> usize {
		match &self.kind {
			VirtFileKind::Directory(children) => children.len(),
			_ => 0,
		}
	}
}

impl std::ops::Deref for FileRef<'_> {
	type Target = VirtualFile;

	fn deref(&self) -> &Self::Target {
		self.file
	}
}

impl PartialEq for FileRef<'_> {
	fn eq(&self, other: &Self) -> bool {
		std::ptr::eq(self.catalog, other.catalog) && std::ptr::eq(self.file, other.file)
	}
}

impl Eq for FileRef<'_> {}

// Mount, MountInfo ////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Mount {
	/// Metadata.
	pub(super) info: MountInfo,
	pub(super) lith: Option<lith::Module>,
	/// The "source of truth" for record pointers.
	pub(super) assets: HashMap<AssetKey, Arc<Record>>,
	/// See the key type's documentation for background details.
	pub(super) shortid_map: HashMap<ShortId, Weak<Record>>,
	/// See the key type's documentation for background details.
	/// Stored records always wrap a [`Blueprint`].
	///
	/// [`Blueprint`]: super::asset::Blueprint
	pub(super) editor_numbers: HashMap<EditorNum, Weak<Record>>,
	/// See the key type's documentation for background details.
	/// Stored records always wrap a [`Blueprint`].
	///
	/// [`Blueprint`]: super::asset::Blueprint
	pub(super) spawn_numbers: HashMap<SpawnNum, Weak<Record>>,
	/// Keys take the form `$ID` as in (G)ZDoom.
	pub(super) strings: HashMap<String, InString>,
	// Q: FNV hashing for int-keyed, short ID-keyed maps?
}

impl Mount {
	#[must_use]
	pub(super) fn new(info: MountInfo) -> Self {
		Self {
			info,
			lith: None,
			assets: HashMap::default(),
			shortid_map: HashMap::default(),
			editor_numbers: HashMap::default(),
			spawn_numbers: HashMap::default(),
			strings: HashMap::default(),
		}
	}

	/// Metadata about this mount.
	#[must_use]
	pub fn info(&self) -> &MountInfo {
		&self.info
	}

	#[must_use]
	pub fn lith_module(&self) -> &Option<lith::Module> {
		&self.lith
	}
}

/// Metadata about a mounted file/directory. For VileTech packages, this comes
/// from a `meta.toml` file. Otherwise it is left largely unpopulated.
#[derive(Debug)]
pub struct MountInfo {
	/// Specified by `meta.toml` if one exists.
	/// Otherwise, this comes from the file stem of the mount point.
	pub(super) id: String,
	pub(super) format: MountFormat,
	pub(super) kind: MountKind,
	pub(super) version: Option<String>,
	/// Specified by `meta.toml` if one exists.
	/// Human-readable, presented to users in the frontend.
	pub(super) name: Option<String>,
	/// Specified by `meta.toml` if one exists.
	/// Human-readable, presented to users in the frontend.
	pub(super) description: Option<String>,
	/// Specified by `meta.toml` if one exists.
	/// Human-readable, presented to users in the frontend.
	pub(super) authors: Vec<String>,
	/// Specified by `meta.toml` if one exists.
	/// Human-readable, presented to users in the frontend.
	pub(super) copyright: Option<String>,
	/// Specified by `meta.toml` if one exists.
	/// Allow a package to link to its forum post/homepage/Discord server/etc.
	pub(super) links: Vec<String>,
	/// Always canonicalized, but may not necessarily be valid UTF-8.
	pub(super) real_path: Box<Path>,
	pub(super) virtual_path: Box<VPath>,
	/// The base of the package's LithScript include tree.
	///
	/// - For VileTech packages, this is specified by a `meta.toml` file.
	/// - For ZDoom and Eternity packages, the script root is the first
	/// "lump" with the file stem `VTECHLITH` in the package's global namespace.
	/// - For WADs, the script root is the first `VTECHLITH` "lump" found.
	///
	/// Normally, the scripts can define manifest items used to direct post-processing,
	/// but if there is no script root or manifests, ZDoom loading rules are used.
	///
	/// A package can only specify a file owned by it as a script root, so this
	/// is always relative. `viletech.vpk3`'s script root, for example, is `main.lith`.
	pub(super) script_root: Option<Box<VPath>>,
	// Q:
	// - Dependency specification?
	// - Incompatibility specification?
	// - Ordering specification?
	// - Forced specifications, or just strongly-worded warnings? Multiple levels?
}

/// Informs the rules used for post-processing assets from a mount.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountKind {
	/// If the mount's own root has an immediate child text file named `meta.toml`
	/// (ASCII-case-ignored), that indicates that the mount is a VileTech package.
	VileTech,
	/// If mounting an archive with:
	/// - no immediate text file child named `meta.toml`
	/// - the extension `.pk3`, `.ipk3`, `.pk7`, or `.ipk7`
	/// then this is what gets resolved. If it's a directory instead of an archive,
	/// the heuristic used is if there's an immediate child text file with a file
	/// stem belonging to a ZDoom-exclusive lump.
	ZDoom,
	/// If mounting an archive with:
	/// - no immediate text file child named `meta.toml`
	/// - the extension `.pke`
	/// then this is what gets resolved. If it's a directory instead of an archive,
	/// the heuristic used is if there's an immediate child text file with the
	/// file stem `edfroot` or `emapinfo` (ASCII-case-ignored).
	Eternity,
	/// Deduced from [`MountFormat`], which is itself deduced from the file header.
	Wad,
	/// Fallback if the mount resolved to none of the other kinds.
	/// Usually used if mounting a single non-archive file.
	Misc,
}

/// Primarily serves to specify the type of compression used, if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountFormat {
	PlainFile,
	Directory,
	Wad,
	Zip,
	// TODO: Support LZMA, XZ, GRP, PAK, RFF, SSI
}

impl MountInfo {
	#[must_use]
	pub fn id(&self) -> &str {
		&self.id
	}

	#[must_use]
	pub fn format(&self) -> MountFormat {
		self.format
	}

	/// The real file/directory this mount represents.
	/// Always canonicalized, but may not necessarily be valid UTF-8.
	#[must_use]
	pub fn real_path(&self) -> &Path {
		&self.real_path
	}

	/// Also known as the "mount point". Corresponds to a VFS node.
	#[must_use]
	pub fn virtual_path(&self) -> &VPath {
		&self.virtual_path
	}

	#[must_use]
	pub fn name(&self) -> Option<&str> {
		match &self.name {
			Some(s) => Some(s),
			None => None,
		}
	}

	#[must_use]
	pub fn version(&self) -> Option<&str> {
		match &self.version {
			Some(s) => Some(s),
			None => None,
		}
	}

	#[must_use]
	pub fn description(&self) -> Option<&str> {
		match &self.description {
			Some(s) => Some(s),
			None => None,
		}
	}

	#[must_use]
	pub fn authors(&self) -> &[String] {
		self.authors.as_ref()
	}

	#[must_use]
	pub fn copyright_info(&self) -> Option<&str> {
		match &self.copyright {
			Some(s) => Some(s),
			None => None,
		}
	}

	/// User-specified URLS to a forum post/homepage/Discord server/et cetera.
	#[must_use]
	pub fn public_links(&self) -> &[String] {
		&self.links
	}
}

// Handle //////////////////////////////////////////////////////////////////////

/// Thin wrapper around an [`Arc`] pointing to a [`Record`]. Attaching a generic
/// asset type allows the asset pointer to be safely downcast without any checks,
/// enabling safe, instant access to an asset's data from anywhere in the engine.
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Handle<A: Asset>(Arc<Record>, PhantomData<A>);

impl<A: Asset> Handle<A> {
	/// For use in inter-asset relationships.
	#[must_use]
	pub fn downgrade(&self) -> InHandle<A> {
		InHandle(Arc::downgrade(&self.0), PhantomData)
	}
}

impl<A: Asset> From<Arc<Record>> for Handle<A> {
	/// This conversion panics if the asset type of the given record is not `A`.
	fn from(value: Arc<Record>) -> Self {
		let expected = TypeId::of::<A>();
		let typeid = value.data.type_id();

		assert_eq!(
			expected, typeid,
			"Expected asset type: {expected:#?}, but got: {typeid:#?}",
		);

		Self(value, PhantomData)
	}
}

impl<A: Asset> From<&Arc<Record>> for Handle<A> {
	/// This conversion panics if the asset type of the given record is not `A`.
	fn from(value: &Arc<Record>) -> Self {
		let expected = TypeId::of::<A>();
		let typeid = value.data.type_id();

		assert_eq!(
			expected, typeid,
			"Expected asset type: {expected:#?}, but got: {typeid:#?}",
		);

		Self(value.clone(), PhantomData)
	}
}

impl<A: 'static + Asset> std::ops::Deref for Handle<A> {
	type Target = A;

	#[inline]
	fn deref(&self) -> &Self::Target {
		// SAFETY: Type correctness was validated during handle acquisition.
		// Note that `unwrap_unchecked` contains a debug assertion.
		// Q: `downcast_ref_unchecked` when it stabilizes?
		unsafe { self.0.data.as_any().downcast_ref::<A>().unwrap_unchecked() }
	}
}

impl<A: Asset> PartialEq for Handle<A> {
	/// Check that these are two handles to the same [`Record`].
	fn eq(&self, other: &Self) -> bool {
		Arc::ptr_eq(&self.0, &other.0)
	}
}

impl<A: Asset> Eq for Handle<A> {}

/// Internal handle. Like [`Handle`] but [`Weak`], allowing inter-asset
/// relationships (without preventing in-place mutation or removal) in a way
/// that can't leak.
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct InHandle<A: Asset>(Weak<Record>, PhantomData<A>);

impl<A: Asset> InHandle<A> {
	#[must_use]
	pub fn upgrade(&self) -> Option<Handle<A>> {
		self.0.upgrade().map(Handle::from)
	}
}

impl<A: Asset> PartialEq for InHandle<A> {
	/// Check that these are two handles to the same [`Record`].
	fn eq(&self, other: &Self) -> bool {
		Weak::ptr_eq(&self.0, &other.0)
	}
}

impl<A: Asset> Eq for InHandle<A> {}

// Configuration ///////////////////////////////////////////////////////////////

/// Configuration methods are kept in a wrapper around a [`Catalog`] reference
/// to prevent bloat in the interface of the catalog itself.
#[derive(Debug)]
#[repr(transparent)]
pub struct ConfigGet<'cat>(pub(super) &'cat Catalog);

impl ConfigGet<'_> {
	/// The limit on the size of a virtual binary file. Irrelevant to asset management.
	/// A mount can be any size the physical filesystem can handle, but any children
	/// over this size are rejected with a logged warning.
	///
	/// The returned value is in bytes, and defaults to [`limits::DEFAULT_BIN_FILE_SIZE`].
	#[must_use]
	pub fn bin_size_limit(&self) -> usize {
		self.0.config.bin_size_limit
	}

	/// The limit on the size of a virtual text file. Irrelevant to asset management.
	/// A mount can be any size the physical filesystem can handle, but any children
	/// over this size are rejected with a logged warning.
	///
	/// The returned value is in bytes, and defaults to [`limits::DEFAULT_TEXT_FILE_SIZE`].
	#[must_use]
	pub fn text_size_limit(&self) -> usize {
		self.0.config.text_size_limit
	}
}

/// Configuration methods are kept in a wrapper around a [`Catalog`] reference
/// to prevent bloat in the interface of the catalog itself.
#[derive(Debug)]
#[repr(transparent)]
pub struct ConfigSet<'cat>(pub(super) &'cat mut Catalog);

impl ConfigSet<'_> {
	/// The limit on the size of a virtual binary file. Irrelevant to asset management.
	/// A mount can be any size the physical filesystem can handle, but any children
	/// over this size are rejected with a logged warning.
	///
	/// The parameter is in bytes, and gets clamped between 0 and
	/// [`limits::MAX_BIN_FILE_SIZE`]. The default is [`limits::DEFAULT_BIN_FILE_SIZE`].
	pub fn bin_size_limit(self, limit: usize) -> Self {
		self.0.config.bin_size_limit = limit.clamp(0, limits::MAX_BIN_FILE_SIZE);
		self
	}

	/// The limit on the size of a virtual text file. Irrelevant to asset management.
	/// A mount can be any size the physical filesystem can handle, but any children
	/// over this size are rejected with a logged warning.
	///
	/// The parameter is in bytes, and gets clamped between 0 and
	/// [`limits::MAX_TEXT_FILE_SIZE`]. The default is [`limits::DEFAULT_TEXT_FILE_SIZE`].
	pub fn text_size_limit(self, limit: usize) -> Self {
		self.0.config.text_size_limit = limit.clamp(0, limits::MAX_TEXT_FILE_SIZE);
		self
	}
}

pub mod limits {
	/// 1024 B * 1024 kB * 512 MB = 536870912 bytes.
	pub const DEFAULT_BIN_FILE_SIZE: usize = 1024 * 1024 * 512;
	/// 1024 B * 1024 kB * 64 MB = 67108864 bytes.
	pub const DEFAULT_TEXT_FILE_SIZE: usize = 1024 * 1024 * 64;
	/// 1024 B * 1024 kB * 1024 MB * 2 GB = 2147483648 bytes.
	pub const MAX_BIN_FILE_SIZE: usize = 1024 * 1024 * 1024 * 2;
	/// 1024 B * 1024 kB * 128 MB = 134217728 bytes.
	pub const MAX_TEXT_FILE_SIZE: usize = 1024 * 1024 * 128;

	// (RAT) If you guessed that the default text file size limit could
	// be much lower if not for the UDMF TEXTMAP format, then you're correct.
	// Ar Luminae's MAP01 TEXTMAP is 43.69 MB.
}

// Loading /////////////////////////////////////////////////////////////////////

/// Also make sure to read [`Catalog::load`].
#[derive(Debug)]
pub struct LoadRequest<RP, MP>
where
	RP: AsRef<Path>,
	MP: AsRef<VPath>,
{
	/// In any given tuple, element `::0` should be a real path and `::1` should
	/// be the mount point. `mymount` and `/mymount` both put the mount on the root.
	/// An empty path and `/` are both invalid mount points, but one can mount
	/// `/mymount` and then `mymount/myothermount`.
	///
	/// If this is an empty slice, any mount operation becomes a no-op, and
	/// an empty array of results is returned.
	pub paths: Vec<(RP, MP)>,
	/// Only pass a `Some` if you need to, for instance, display a loading screen,
	/// or otherwise report to the end user on the progress of a mount operation.
	pub tracker: Option<Arc<LoadTracker>>,
}

/// Wrap in an [`Arc`] and use to check how far along a load operation is.
#[derive(Debug, Default)]
pub struct LoadTracker {
	/// The total number of bytes the user requested to mount.
	pub(super) mount_progress: AtomicU64,
	/// The total number of bytes the VFS has mounted thus far.
	pub(super) mount_target: AtomicU64,
	pub(super) pproc_progress: AtomicU32,
	pub(super) pproc_target: AtomicU32,
}

impl LoadTracker {
	/// 0.0 means just started; 1.0 means done.
	#[must_use]
	pub fn mount_progress_percent(&self) -> f64 {
		let prog = self.mount_progress.load(atomic::Ordering::SeqCst);
		let tgt = self.mount_target.load(atomic::Ordering::SeqCst);

		if tgt == 0 {
			return 0.0;
		}

		(prog / tgt) as f64
	}

	/// 0.0 means just started; 1.0 means done.
	#[must_use]
	pub fn pproc_progress_percent(&self) -> f64 {
		let prog = self.pproc_progress.load(atomic::Ordering::SeqCst);
		let tgt = self.pproc_target.load(atomic::Ordering::SeqCst);

		if tgt == 0 {
			return 0.0;
		}

		(prog / tgt) as f64
	}

	#[must_use]
	pub fn mount_done(&self) -> bool {
		self.mount_progress.load(atomic::Ordering::SeqCst)
			>= self.mount_target.load(atomic::Ordering::SeqCst)
	}

	#[must_use]
	pub fn pproc_done(&self) -> bool {
		self.pproc_progress.load(atomic::Ordering::SeqCst)
			>= self.pproc_target.load(atomic::Ordering::SeqCst)
	}

	pub(super) fn add_mount_progress(&self, bytes: u64) {
		self.mount_progress
			.fetch_add(bytes, atomic::Ordering::SeqCst);
	}
}
