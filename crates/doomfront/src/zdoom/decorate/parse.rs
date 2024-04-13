mod actor;
mod common;
mod expr;
mod top;

#[cfg(test)]
mod test;

use crate::{parser::Parser, zdoom::Token};

use super::Syntax;

use self::{actor::*, common::*, expr::*, top::*};

/// Builds a [`Syntax::Root`] node.
pub fn file(p: &mut Parser<Syntax>) {
	let root = p.open();

	while !p.eof() {
		if trivia(p) {
			continue;
		}

		if p.at_str_nc(Token::Ident, "actor") {
			actor_def(p);
			continue;
		} else if p.at_str_nc(Token::Ident, "damagetype") {
			damage_type(p);
			continue;
		}

		match p.nth(0) {
			Token::KwConst => {
				const_def(p);
				continue;
			}
			Token::KwEnum => {
				enum_def(p);
				continue;
			}
			Token::KwInclude => {
				include_directive(p);
				continue;
			}
			t => {
				p.advance_with_error(
					Syntax::from(t),
					&[&["`actor`", "`const`", "`damagetype`", "`enum`", "`#include`"]],
				);
			}
		}
	}

	p.close(root, Syntax::Root);
}
