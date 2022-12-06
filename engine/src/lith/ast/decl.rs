//! Declarations of members, functions...

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

use serde::Serialize;

use crate::utils::lang::{FileSpan, Identifier};

use super::Resolver;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FieldDeclaration<'inp> {
	pub span: FileSpan<'inp>,
	pub name: Identifier<'inp>,
	pub type_spec: Resolver<'inp>,
	pub quals: Vec<DeclQualifier<'inp>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DeclQualifier<'inp> {
	pub span: FileSpan<'inp>,
	#[serde(flatten)]
	pub kind: DeclQualifierKind,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum DeclQualifierKind {
	Static,
	Private,
	Protected,
	Final,
	Virtual,
	Override,
	Abstract,
	Action,
}
