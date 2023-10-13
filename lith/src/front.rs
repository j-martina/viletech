//! The parts of Lithica's compiler frontend not concerned with [lexing] or [parsing].
//!
//! [lexing]: crate::syn
//! [parsing]: crate::parse

pub(crate) mod decl;

use doomfront::rowan::{ast::AstNode, TextRange};

use crate::{
	ast,
	compile::Scope,
	data::{DefPtr, Location, SymPtr, Symbol},
	filetree::{self, FileIx},
	Compiler, ParseTree, Syn, SyntaxNode, SyntaxToken,
};

struct FrontendContext<'c> {
	compiler: &'c Compiler,
	arena: &'c bumpalo::Bump,
	lib_ix: u16,
	file_ix: FileIx,
	path: &'c str,
	ptree: &'c ParseTree,
}

impl FrontendContext<'_> {
	fn declare(
		&self,
		scope: &mut Scope,
		name: &SyntaxToken,
		node: &SyntaxNode,
	) -> Result<SymPtr, SymPtr> {
		let location = Location {
			lib_ix: self.lib_ix,
			file_ix: self.file_ix,
			span: node.text_range(),
		};

		let name = self.names.intern(name);

		let sym_ptr = match scope.entry(name) {
			im::hashmap::Entry::Vacant(vac) => {
				let sym = Symbol {
					location,
					def: DefPtr::null(),
				};

				let sym_ptr = SymPtr::alloc(self.arena, sym);
				self.symbols.insert(location, sym_ptr.clone());
				vac.insert(sym_ptr.clone());
				sym_ptr
			}
			im::hashmap::Entry::Occupied(occ) => {
				return Err(occ.get().clone());
			}
		};

		Ok(sym_ptr)
	}

	#[must_use]
	fn resolve_file(&self, sym: &Symbol) -> (&String, &ParseTree) {
		let prev_lib = &self.sources[sym.location.lib_ix as usize];
		let prev_ftn_ix = petgraph::graph::NodeIndex::new(sym.location.file_ix as usize);
		let prev_ftn = &prev_lib.filetree.files[prev_ftn_ix];

		let filetree::Node::File { path, ptree } = prev_ftn else {
			unreachable!()
		};

		(path, ptree)
	}
}

impl std::ops::Deref for FrontendContext<'_> {
	type Target = Compiler;

	fn deref(&self) -> &Self::Target {
		self.compiler
	}
}

/// A symbol's "critical span" is the part that is used to present diagnostics.
///
/// For example, a function definition's critical span starts at its
/// first qualifier keyword or return type token and ends at the last token
/// of its parameter list (or return type, if there is one).
#[must_use]
pub(crate) fn crit_span(node: &SyntaxNode) -> TextRange {
	if let Some(fndecl) = ast::FunctionDecl::cast(node.clone()) {
		let start = fndecl
			.syntax()
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|t| t.kind() != Syn::DocComment))
			.unwrap()
			.text_range()
			.start();

		let end = if let Some(ret_t) = fndecl.return_type() {
			ret_t.syntax().text_range().end()
		} else if let Ok(param_list) = fndecl.params() {
			param_list.syntax().text_range().end()
		} else if let Ok(name) = fndecl.name() {
			name.text_range().end()
		} else {
			fndecl.syntax().text_range().end()
		};

		TextRange::new(start, end)
	} else if let Some(symconst) = ast::SymConst::cast(node.clone()) {
		let start = symconst
			.syntax()
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|t| t.kind() != Syn::DocComment))
			.unwrap()
			.text_range()
			.start();

		let end = symconst.syntax().last_token().unwrap().text_range().end();

		TextRange::new(start, end)
	} else {
		unreachable!()
	}
}
