//! Combinators applicable to multiple other parts of the syntax.

use crate::{
	parser::Parser,
	zdoom::{zscript::Syntax, Token},
};

use super::expr;

// Identifiers /////////////////////////////////////////////////////////////////

/// Allows the following to be considered identifiers:
/// - [`Token::KwLoop`]
/// - [`Token::KwFail`]
/// - [`Token::KwWait`]
/// - [`Token::KwOffset`]
/// - [`Token::KwSlow`]
pub(super) const ID_SFKW: u8 = 1 << 0;

/// Allows the following to be considered identifiers:
/// - [`Token::KwBright`]
/// - [`Token::KwCanRaise`]
/// - [`Token::KwFast`]
/// - [`Token::KwLight`]
/// - [`Token::KwOffset`]
/// - [`Token::KwSlow`]
pub(super) const ID_SQKW: u8 = 1 << 1;

/// Allows [`Token::KwProperty`] and builtin type names.
pub(super) const ID_TYPES: u8 = 1 << 2;

/// Allows [`Token::KwDefault`].
pub(super) const ID_DEFAULT: u8 = 1 << 3;

const STATEFLOW_KWS: &[Token] = &[
	Token::KwLoop,
	Token::KwFail,
	Token::KwWait,
	Token::KwOffset,
	Token::KwSlow,
];

const STATEQUAL_KWS: &[Token] = &[
	Token::KwBright,
	Token::KwCanRaise,
	Token::KwFast,
	Token::KwLight,
	Token::KwOffset,
	Token::KwSlow,
];

const PRIMTYPE_KWS: &[Token] = &[
	Token::KwInt16,
	Token::KwSByte,
	Token::KwByte,
	Token::KwInt8,
	Token::KwUInt8,
	Token::KwShort,
	Token::KwUShort,
	Token::KwInt16,
	Token::KwUInt16,
	Token::KwInt,
	Token::KwUInt,
	Token::KwFloat,
	Token::KwDouble,
	Token::KwString,
	Token::KwVector2,
	Token::KwVector3,
	// Curiously, ZScript's Lemon grammar prescribes a `vector4` keyword as
	// being an option here, but there's no RE2C lexer rule for it.
	Token::KwName,
	Token::KwMap,
	Token::KwMapIterator,
	Token::KwArray,
	Token::KwVoid,
	Token::KwState,
	Token::KwColor,
	Token::KwSound,
	Token::KwProperty,
];

/// Combine [`ID_SFKW`], [`ID_SQKW`], and [`ID_TYPES`] via bitwise or to form `CFG`.
/// If `0` is given, only [`Token::Ident`] will match.
pub(super) fn ident<const CFG: u8>(p: &mut Parser<Syntax>) {
	let token = p.nth(0);

	if is_ident::<CFG>(token) {
		p.advance(Syntax::Ident);
	} else {
		p.advance_with_error(Syntax::from(token), &[&["an identifier"]])
	}
}

/// Combine [`ID_SFKW`], [`ID_SQKW`], and [`ID_TYPES`] via bitwise or to form `CFG`.
/// If `0` is given, only [`Token::Ident`] will match.
pub(super) fn is_ident<const CFG: u8>(token: Token) -> bool {
	if token == Token::Ident {
		return true;
	}

	if (CFG & ID_SFKW) != 0 && STATEFLOW_KWS.contains(&token) {
		return true;
	}

	if (CFG & ID_SQKW) != 0 && STATEQUAL_KWS.contains(&token) {
		return true;
	}

	if (CFG & ID_TYPES) != 0 && PRIMTYPE_KWS.contains(&token) {
		return true;
	}

	if (CFG & ID_DEFAULT) != 0 && token == Token::KwDefault {
		return true;
	}

	false
}

/// Shorthand for `ident::<{ ID_SFKW | ID_SQKW | ID_TYPES }>(p);`.
pub(super) fn ident_lax(p: &mut Parser<Syntax>) {
	ident::<{ ID_SFKW | ID_SQKW | ID_TYPES }>(p);
}

/// Shorthand for `is_ident::<{ ID_SFKW | ID_SQKW | ID_TYPES }>(token);`.
#[must_use]
pub(super) fn is_ident_lax(token: Token) -> bool {
	is_ident::<{ ID_SFKW | ID_SQKW | ID_TYPES }>(token)
}

/// Builds a [`Syntax::IdentChain`] node.
/// Backed by [`is_ident`]; see that function's documentation for details on `CFG`.
pub(super) fn ident_chain<const CFG: u8>(p: &mut Parser<Syntax>) {
	p.debug_assert_at_if(|token| is_ident::<CFG>(token) || token == Token::Dot);

	let chain = p.open();

	if p.eat(Token::Dot, Syntax::Dot) {
		trivia_0plus(p);
	}

	p.advance(Syntax::Ident);

	while p.find(0, |token| !token.is_trivia()) == Token::Dot {
		trivia_0plus(p);
		p.advance(Syntax::Dot);
		trivia_0plus(p);
		ident::<CFG>(p);
	}

	p.close(chain, Syntax::IdentChain);
}

/// Builds a series of [`Syntax::Ident`] tokens, separated by trivia and commas.
/// Returns `true` if more than one identifier was parsed.
/// Backed by [`is_ident`]; see that function's documentation for details on `CFG`.
pub(super) fn ident_list<const CFG: u8>(p: &mut Parser<Syntax>) -> bool {
	let mut ret = false;
	ident::<CFG>(p);

	while p.find(0, |token| !token.is_trivia()) == Token::Comma {
		trivia_0plus(p);
		p.advance(Syntax::Comma);
		trivia_0plus(p);
		ident::<CFG>(p);
		ret = true;
	}

	ret
}

// Miscellaneous ///////////////////////////////////////////////////////////////

/// Builds a [`Syntax::ArrayLen`] node.
pub(super) fn array_len(p: &mut Parser<Syntax>) {
	p.debug_assert_at(Token::BracketL);
	let l = p.open();
	p.advance(Syntax::BracketL);
	trivia_0plus(p);

	if p.at_if(expr::in_first_set) {
		expr(p);
	}

	trivia_0plus(p);
	p.expect(Token::BracketR, Syntax::BracketR, &[&["`]`"]]);
	p.close(l, Syntax::ArrayLen);
}

/// Builds a [`Syntax::DeprecationQual`] node.
pub(super) fn deprecation_qual(p: &mut Parser<Syntax>) {
	p.debug_assert_at(Token::KwDeprecated);
	let qual = p.open();
	p.advance(Syntax::KwDeprecated);
	trivia_0plus(p);
	p.expect(Token::ParenL, Syntax::ParenL, &[&["`(`"]]);
	trivia_0plus(p);
	p.expect(
		Token::StringLit,
		Syntax::StringLit,
		&[&["a version string"]],
	);
	trivia_0plus(p);

	if p.eat(Token::Comma, Syntax::Comma) {
		trivia_0plus(p);
		p.expect(Token::StringLit, Syntax::StringLit, &[&["a reason string"]]);
	}

	trivia_0plus(p);
	p.expect(Token::ParenR, Syntax::ParenR, &[&["`)`"]]);
	p.close(qual, Syntax::DeprecationQual);
}

/// Parse 0 or more [`Token::DocComment`]s, additionally consuming trailing trivia.
pub(super) fn doc_comments(p: &mut Parser<Syntax>) {
	while p.eat(Token::DocComment, Syntax::DocComment) {
		trivia_no_doc_0plus(p);
	}
}

/// May or may not build a token tagged with one of the following:
/// - [`Syntax::Whitespace`]
/// - [`Syntax::Comment`]
/// - [`Syntax::RegionStart`]
/// - [`Syntax::RegionEnd`]
/// Note that [`Token::DocComment`] becomes [`Token::Comment`]. For positions
/// where doc comments are valid, handle them with [`trivia_no_doc`].
pub(super) fn trivia(p: &mut Parser<Syntax>) -> bool {
	p.eat_any(&[
		(Token::Whitespace, Syntax::Whitespace),
		(Token::Comment, Syntax::Comment),
		(Token::DocComment, Syntax::Comment),
		(Token::RegionStart, Syntax::RegionStart),
		(Token::RegionEnd, Syntax::RegionEnd),
	])
}

/// Shorthand for `while trivia(p) {}`.
pub(super) fn trivia_0plus(p: &mut Parser<Syntax>) {
	while trivia(p) {}
}

/// Expects one [`trivia`] and then calls [`trivia_0plus`].
pub(super) fn trivia_1plus(p: &mut Parser<Syntax>) {
	p.expect_any(
		&[
			(Token::Whitespace, Syntax::Whitespace),
			(Token::Comment, Syntax::Comment),
			(Token::DocComment, Syntax::Comment),
			(Token::RegionStart, Syntax::RegionStart),
			(Token::RegionEnd, Syntax::RegionEnd),
		],
		&[&["whitespace or a comment (one or more)"]],
	);

	trivia_0plus(p);
}

pub(super) fn trivia_no_doc(p: &mut Parser<Syntax>) -> bool {
	p.eat_any(&[
		(Token::Whitespace, Syntax::Whitespace),
		(Token::Comment, Syntax::Comment),
		(Token::RegionStart, Syntax::RegionStart),
		(Token::RegionEnd, Syntax::RegionEnd),
	])
}

/// Shorthand for `while trivia_no_doc(p) {}`.
pub(super) fn trivia_no_doc_0plus(p: &mut Parser<Syntax>) {
	while trivia_no_doc(p) {}
}

/// Builds a [`Syntax::VarName`] node.
pub(super) fn var_name(p: &mut Parser<Syntax>) {
	p.debug_assert_at_if(is_ident::<{ ID_SFKW | ID_SQKW | ID_TYPES }>);
	let name = p.open();
	p.advance(Syntax::Ident);

	loop {
		if p.find(0, |token| !token.is_trivia()) == Token::BracketL {
			trivia_0plus(p);
			array_len(p);
		} else {
			break;
		}
	}

	p.close(name, Syntax::VarName);
}

/// Builds a [`Syntax::VersionQual`] node.
pub(super) fn version_qual(p: &mut Parser<Syntax>) {
	p.debug_assert_at(Token::KwVersion);
	let qual = p.open();
	p.advance(Syntax::KwVersion);
	trivia_0plus(p);
	p.expect(Token::ParenL, Syntax::ParenL, &[&["`(`"]]);
	trivia_0plus(p);
	p.expect(
		Token::StringLit,
		Syntax::StringLit,
		&[&["a version string"]],
	);
	trivia_0plus(p);
	p.expect(Token::ParenR, Syntax::ParenR, &[&["`)`"]]);
	p.close(qual, Syntax::VersionQual);
}
