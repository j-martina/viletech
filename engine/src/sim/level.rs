//! Level state for the playsim and renderer.
//!
//! While not strictly necessarily, making this a part of the ECS allows use of
//! Bevy's ECS hierarchies to easily clean up an entire level recursively with
//! one call.

mod init;
pub mod line;
pub mod sector;

use std::{collections::HashMap, hash::Hash, sync::Arc};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
	data::asset,
	sparse::{SparseSet, SparseSetIndex},
};

pub use self::init::*;
use self::sector::Sector;

/// Strongly-typed [`Entity`] wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Level(Entity);

/// The principal component in a level entity.
#[derive(Component, Debug)]
pub struct Core {
	pub base: Option<asset::Handle<asset::Level>>,
	pub flags: Flags,
	/// Time spent in this level thus far.
	pub ticks_elapsed: u64,
	pub geom: Geometry,
}

/// Sub-structure for composing [`Core`].
///
/// The vertex array, trigger map, and some counters.
#[derive(Debug)]
pub struct Geometry {
	pub mesh: Handle<Mesh>,
	pub verts: SparseSet<VertIndex, Vertex>,
	pub sides: SparseSet<SideIndex, Side>,
	/// Each stored entity ID points to a sector.
	///
	/// When a line is triggered (walked over, interacted-with, shot), all sectors
	/// in the corresponding array have all "activatable" components get activated.
	pub triggers: HashMap<line::Trigger, Vec<Sector>>,
	/// Updated as map geometry changes.
	pub num_sectors: usize,
}

bitflags::bitflags! {
	#[derive(Default)]
	pub struct Flags: u8 {
		// From GZ. Purpose unclear.
		const FROZEN_LOCAL = 1 << 0;
		// From GZ. Purpose unclear.
		const FROZEN_GLOBAL = 1 << 1;
		/// Monsters which teleport so as to have bounding box intersection with
		/// a player actor kill that actor. Primarily for use in Doom 2's MAP30.
		const MONSTERS_TELEFRAG = 1 << 2;
	}
}

// Vertex information //////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vertex(pub Vec3);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct VertIndex(usize);

impl From<VertIndex> for usize {
	fn from(value: VertIndex) -> Self {
		value.0
	}
}

impl SparseSetIndex for VertIndex {}

// Line sides //////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Side {
	pub offset: IVec2,
	pub sector: Sector,
	pub udmf: Udmf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SideIndex(usize);

impl From<SideIndex> for usize {
	fn from(value: SideIndex) -> Self {
		value.0
	}
}

impl SparseSetIndex for SideIndex {}

// UDMF ////////////////////////////////////////////////////////////////////////

/// A map of arbitrary string-keyed values defined in a UDMF TEXTMAP file.
///
/// Can be attached to a line, side, or sector.
#[derive(Component, Debug, Default)]
pub struct Udmf(HashMap<Arc<str>, UdmfValue>);

#[derive(Debug)]
pub enum UdmfValue {
	Int(i32),
	Float(f64),
	String(Arc<str>),
}
