//! Combinators applicable to multiple other parts of the syntax.

use chumsky::{primitive, IterParser, Parser};
use rowan::{GreenNode, GreenToken};

use crate::{
	comb, parser_t,
	parsing::*,
	zdoom::{zscript::Syn, Token},
	GreenElement,
};

use super::ParserBuilder;

impl ParserBuilder {
	pub(super) fn array_len<'i>(&self) -> parser_t!(Vec<GreenNode>) {
		primitive::group((
			comb::just_ts(Token::BracketL, Syn::BracketL),
			self.trivia_0plus(),
			self.expr().or_not(),
			self.trivia_0plus(),
			comb::just_ts(Token::BracketR, Syn::BracketR),
		))
		.map(|group| coalesce_node(group, Syn::ArrayLen))
		.repeated()
		.at_least(1)
		.collect()
	}

	pub(super) fn ident<'i>(&self) -> parser_t!(GreenToken) {
		primitive::any()
			.filter(|token: &Token| {
				matches!(
					token,
					Token::Ident
						| Token::KwBright | Token::KwFast
						| Token::KwSlow | Token::KwNoDelay
						| Token::KwCanRaise | Token::KwOffset
						| Token::KwLight
				)
			})
			.map_with_state(comb::green_token(Syn::Ident))
	}

	pub fn ident_chain<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			self.ident(),
			primitive::group((
				self.trivia_0plus(),
				comb::just_ts(Token::Dot, Syn::Dot),
				self.ident(),
			))
			.repeated()
			.collect::<Vec<_>>(),
		))
		.map(|group| coalesce_node(group, Syn::IdentChain))
	}

	pub(super) fn trivia<'i>(&self) -> parser_t!(GreenElement) {
		primitive::choice((
			comb::just_ts(Token::Whitespace, Syn::Whitespace),
			comb::just_ts(Token::Comment, Syn::Comment),
		))
		.map(|token| token.into())
	}

	pub(super) fn trivia_0plus<'i>(&self) -> parser_t!(Vec<GreenElement>) {
		self.trivia().repeated().collect()
	}

	pub(super) fn _trivia_1plus<'i>(&self) -> parser_t!(Vec<GreenElement>) {
		self.trivia().repeated().at_least(1).collect()
	}

	pub fn type_ref<'i>(&self) -> parser_t!(GreenNode) {
		chumsky::recursive::recursive(|tref| {
			let at_ident = primitive::group((comb::just_ts(Token::At, Syn::At), self.ident()))
				.map(|group| coalesce_node(group, Syn::TypeRef));

			let ident = self
				.ident()
				.map(|gtok| GreenNode::new(Syn::IdentChain.into(), [gtok.into()]));

			let readonly = primitive::group((
				comb::just_ts(Token::KwReadonly, Syn::KwReadonly),
				self.trivia_0plus(),
				comb::just_ts(Token::AngleL, Syn::AngleL),
				self.trivia_0plus(),
				primitive::choice((ident, at_ident.clone())),
				self.trivia_0plus(),
				comb::just_ts(Token::AngleR, Syn::AngleR),
			))
			.map(|group| coalesce_node(group, Syn::TypeRef));

			let tref_identchain = self
				.ident_chain()
				.map(|gnode| GreenNode::new(Syn::TypeRef.into(), [gnode.into()]));

			let tref_let = comb::just_ts(Token::KwLet, Syn::KwLet)
				.map(|gtok| GreenNode::new(Syn::TypeRef.into(), [gtok.into()]));

			let simple = primitive::choice((readonly, at_ident, tref_identchain, tref_let));

			let tref_or_fixedlen_array =
				primitive::group((tref.clone(), self.array_len().or_not())).map(coalesce_vec);

			let class_restrictor = primitive::group((
				self.trivia_0plus(),
				comb::just_ts(Token::AngleL, Syn::AngleL),
				self.trivia_0plus(),
				self.ident_chain(),
				self.trivia_0plus(),
				comb::just_ts(Token::AngleR, Syn::AngleR),
			))
			.map(coalesce_vec);

			let tref_class = primitive::group((
				comb::just_ts(Token::KwClass, Syn::KwClass),
				class_restrictor.or_not(),
			))
			.map(|group| coalesce_node(group, Syn::TypeRef));

			let tref_array_dyn = primitive::group((
				comb::just_ts(Token::KwArray, Syn::KwArray),
				self.trivia_0plus(),
				comb::just_ts(Token::AngleL, Syn::AngleL),
				self.trivia_0plus(),
				tref_or_fixedlen_array.clone(),
				self.trivia_0plus(),
				comb::just_ts(Token::AngleR, Syn::AngleR),
			))
			.map(|group| coalesce_node(group, Syn::TypeRef));

			let tref_map = primitive::group((
				comb::just_ts(Token::KwMap, Syn::KwMap),
				self.trivia_0plus(),
				comb::just_ts(Token::AngleL, Syn::AngleL),
				self.trivia_0plus(),
				tref_or_fixedlen_array.clone(),
				self.trivia_0plus(),
				comb::just_ts(Token::Comma, Syn::Comma),
				self.trivia_0plus(),
				tref_or_fixedlen_array.clone(),
				self.trivia_0plus(),
				comb::just_ts(Token::AngleR, Syn::AngleR),
			))
			.map(|group| coalesce_node(group, Syn::TypeRef));

			let tref_mapiter = primitive::group((
				comb::just_ts(Token::KwMapIterator, Syn::KwMapIterator),
				self.trivia_0plus(),
				comb::just_ts(Token::AngleL, Syn::AngleL),
				self.trivia_0plus(),
				tref_or_fixedlen_array.clone(),
				self.trivia_0plus(),
				comb::just_ts(Token::Comma, Syn::Comma),
				self.trivia_0plus(),
				tref_or_fixedlen_array.clone(),
				self.trivia_0plus(),
				comb::just_ts(Token::AngleR, Syn::AngleR),
			))
			.map(|group| coalesce_node(group, Syn::TypeRef));

			primitive::choice((tref_class, tref_array_dyn, tref_map, tref_mapiter, simple)).boxed()
		})
	}
}

#[cfg(test)]
mod test {
	use crate::{
		testing::*,
		zdoom::{zscript::ParseTree, Version},
	};

	use super::*;

	#[test]
	fn smoke_typeref() {
		const SOURCES: &[&str] = &[
			"TeenyLittleBase",
			"Dead.On.Arrival",
			"readonly<Corruption2Factory>",
			"class",
			"class<Forge>",
			"array<Unwelcome>",
			"array<class<TheOssuary> >",
			"map<Corruption[1], Mortem[2][3]>",
			"mapiterator<FishInABarrel, Neoplasm>",
		];

		for source in SOURCES {
			let tbuf = crate::scan(source);
			let parser = ParserBuilder::new(Version::default()).type_ref();
			let ptree: ParseTree = crate::parse(parser, source, &tbuf);
			assert_no_errors(&ptree);
		}
	}
}
