//! AST nodes for parts of definitions for classes inheriting from `Actor`.

use rowan::ast::AstNode;

use crate::{simple_astnode, zdoom::ast::LitToken, AstError, AstResult};

use super::{ArgList, CompoundStat, DocComment, Expr, IdentChain, Syntax, SyntaxNode, SyntaxToken};

// FlagDef /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::FlagDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FlagDef(pub(super) SyntaxNode);

simple_astnode!(Syntax, FlagDef, Syntax::FlagDef);

impl FlagDef {
	/// The returned token is always tagged [`Syntax::Ident`].
	pub fn name(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::Ident)
			})
			.ok_or(AstError::Missing)
	}

	/// The returned token is always tagged [`Syntax::Ident`].
	pub fn backing_field(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.skip_while(|elem| elem.kind() != Syntax::Colon)
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::Ident)
			})
			.ok_or(AstError::Missing)
	}

	/// The returned token is always tagged [`Syntax::IntLit`].
	pub fn bit(&self) -> AstResult<LitToken<Syntax>> {
		self.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::IntLit)
					.map(LitToken::new)
			})
			.ok_or(AstError::Missing)
	}

	pub fn docs(&self) -> impl Iterator<Item = DocComment> {
		super::doc_comments(&self.0)
	}
}

// PropertyDef /////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::PropertyDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PropertyDef(pub(super) SyntaxNode);

simple_astnode!(Syntax, PropertyDef, Syntax::PropertyDef);

impl PropertyDef {
	/// The returned token is always tagged [`Syntax::Ident`].
	pub fn name(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::Ident)
			})
			.ok_or(AstError::Missing)
	}

	/// Yielded tokens are always tagged [`Syntax::Ident`].
	pub fn backing_fields(&self) -> impl Iterator<Item = SyntaxToken> {
		self.0
			.children_with_tokens()
			.skip_while(|elem| elem.kind() != Syntax::Colon)
			.filter_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::Ident)
			})
	}

	pub fn docs(&self) -> impl Iterator<Item = DocComment> {
		super::doc_comments(&self.0)
	}
}

// DefaultBlock ////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::DefaultBlock`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DefaultBlock(pub(super) SyntaxNode);

simple_astnode!(Syntax, DefaultBlock, Syntax::DefaultBlock);

impl DefaultBlock {
	pub fn innards(&self) -> impl Iterator<Item = DefaultInnard> {
		self.0.children().filter_map(|node| match node.kind() {
			Syntax::FlagSetting => Some(DefaultInnard::FlagSetting(FlagSetting(node))),
			Syntax::PropertySetting => Some(DefaultInnard::PropertySetting(PropertySetting(node))),
			_ => None,
		})
	}
}

// DefaultInnard ///////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum DefaultInnard {
	FlagSetting(FlagSetting),
	PropertySetting(PropertySetting),
}

impl AstNode for DefaultInnard {
	type Language = Syntax;

	fn can_cast(kind: <Self::Language as rowan::Language>::Kind) -> bool
	where
		Self: Sized,
	{
		matches!(kind, Syntax::FlagSetting | Syntax::PropertySetting)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syntax::FlagSetting => Some(Self::FlagSetting(FlagSetting(node))),
			Syntax::PropertySetting => Some(Self::PropertySetting(PropertySetting(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			DefaultInnard::FlagSetting(inner) => inner.syntax(),
			DefaultInnard::PropertySetting(inner) => inner.syntax(),
		}
	}
}

// FlagSetting /////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::FlagSetting`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FlagSetting(pub(super) SyntaxNode);

simple_astnode!(Syntax, FlagSetting, Syntax::FlagSetting);

impl FlagSetting {
	#[must_use]
	pub fn is_adding(&self) -> bool {
		self.0.first_token().unwrap().kind() == Syntax::Plus
	}

	#[must_use]
	pub fn is_removing(&self) -> bool {
		self.0.first_token().unwrap().kind() == Syntax::Minus
	}

	pub fn name(&self) -> AstResult<IdentChain> {
		match self.syntax().last_child() {
			Some(node) => IdentChain::cast(node).ok_or(AstError::Incorrect),
			None => Err(AstError::Missing),
		}
	}
}

// PropertySetting /////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::PropertySetting`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PropertySetting(pub(super) SyntaxNode);

simple_astnode!(Syntax, PropertySetting, Syntax::PropertySetting);

impl PropertySetting {
	#[must_use]
	pub fn name(&self) -> IdentChain {
		IdentChain::cast(self.0.first_child().unwrap()).unwrap()
	}

	/// Mind that the returned iterator may yield no items.
	pub fn exprs(&self) -> impl Iterator<Item = Expr> {
		self.0.children().skip(1).filter_map(Expr::cast)
	}
}

// StatesBlock /////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::StatesBlock`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StatesBlock(pub(super) SyntaxNode);

simple_astnode!(Syntax, StatesBlock, Syntax::StatesBlock);

impl StatesBlock {
	#[must_use]
	pub fn usage_quals(&self) -> Option<impl Iterator<Item = StateUsage>> {
		self.0
			.first_child()
			.filter(|node| node.kind() == Syntax::StatesUsage)
			.map(|node| {
				node.children_with_tokens().filter_map(|elem| {
					let Some(token) = elem.into_token() else {
						return None;
					};

					if token.text().eq_ignore_ascii_case("actor") {
						Some(StateUsage::Actor(token))
					} else if token.text().eq_ignore_ascii_case("item") {
						Some(StateUsage::Item(token))
					} else if token.text().eq_ignore_ascii_case("overlay") {
						Some(StateUsage::Overlay(token))
					} else if token.text().eq_ignore_ascii_case("weapon") {
						Some(StateUsage::Weapon(token))
					} else {
						None
					}
				})
			})
	}

	pub fn innards(&self) -> impl Iterator<Item = StatesInnard> {
		self.0.children().filter_map(|node| match node.kind() {
			Syntax::StateDef => Some(StatesInnard::State(StateDef(node))),
			Syntax::StateFlow => Some(StatesInnard::Flow(StateFlow(node))),
			Syntax::StateLabel => Some(StatesInnard::Label(StateLabel(node))),
			_ => None,
		})
	}
}

/// Wraps a node tagged [`Syntax::ActionQual`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ActionQual(pub(super) SyntaxNode);

simple_astnode!(Syntax, ActionQual, Syntax::ActionQual);

impl ActionQual {
	/// All yielded tokens are tagged [`Syntax::Ident`].
	pub fn usages(&self) -> impl Iterator<Item = StateUsage> {
		self.0.children_with_tokens().filter_map(|elem| {
			let Some(token) = elem.into_token() else {
				return None;
			};

			if token.text().eq_ignore_ascii_case("actor") {
				Some(StateUsage::Actor(token))
			} else if token.text().eq_ignore_ascii_case("item") {
				Some(StateUsage::Item(token))
			} else if token.text().eq_ignore_ascii_case("overlay") {
				Some(StateUsage::Overlay(token))
			} else if token.text().eq_ignore_ascii_case("weapon") {
				Some(StateUsage::Weapon(token))
			} else {
				None
			}
		})
	}
}

/// All wrapped tokens are always tagged [`Syntax::Ident`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum StateUsage {
	Actor(SyntaxToken),
	Item(SyntaxToken),
	Overlay(SyntaxToken),
	Weapon(SyntaxToken),
}

impl StateUsage {
	#[must_use]
	pub fn syntax(&self) -> &SyntaxToken {
		match self {
			Self::Actor(inner) | Self::Item(inner) | Self::Overlay(inner) | Self::Weapon(inner) => {
				inner
			}
		}
	}
}

// StatesInnard ////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum StatesInnard {
	Flow(StateFlow),
	Label(StateLabel),
	State(StateDef),
}

// StateLabel //////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::StateLabel`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StateLabel(SyntaxNode);

simple_astnode!(Syntax, StateLabel, Syntax::StateLabel);

impl StateLabel {
	/// The returned token is always tagged [`Syntax::NonWhitespace`].
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		let ret = self.0.first_token().unwrap();
		debug_assert_eq!(ret.kind(), Syntax::NonWhitespace);
		ret
	}
}

// StateFlow ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::StateFlow`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StateFlow(SyntaxNode);

simple_astnode!(Syntax, StateFlow, Syntax::StateFlow);

impl StateFlow {
	#[must_use]
	pub fn kind(&self) -> StateFlowKind {
		let token0 = self.0.first_token().unwrap();

		match self.0.first_token().unwrap().kind() {
			Syntax::KwGoto => {}
			Syntax::KwFail => return StateFlowKind::Fail(token0),
			Syntax::KwLoop => return StateFlowKind::Loop(token0),
			Syntax::KwStop => return StateFlowKind::Stop(token0),
			Syntax::KwWait => return StateFlowKind::Wait(token0),
			_ => unreachable!(),
		}

		let name = self.0.children().find_map(IdentChain::cast).unwrap();

		let mut prev = name
			.syntax()
			.siblings_with_tokens(rowan::Direction::Prev)
			.skip_while(|elem| elem.kind() != Syntax::Colon2);
		let mut next = name
			.syntax()
			.siblings_with_tokens(rowan::Direction::Next)
			.skip_while(|elem| elem.kind() != Syntax::Plus);

		let scope = prev.find_map(|elem| {
			elem.into_token()
				.filter(|token| matches!(token.kind(), Syntax::Ident | Syntax::KwSuper))
		});

		let offset = next.find_map(|elem| {
			elem.into_token()
				.filter(|token| token.kind() == Syntax::IntLit)
		});

		StateFlowKind::Goto {
			scope,
			name,
			offset: offset.map(LitToken::new),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum StateFlowKind {
	Fail(SyntaxToken),
	Goto {
		/// Always tagged either [`Syntax::Ident`] or [`Syntax::KwSuper`].
		scope: Option<SyntaxToken>,
		name: IdentChain,
		/// Always tagged [`Syntax::IntLit`] if present.
		offset: Option<LitToken<Syntax>>,
	},
	Loop(SyntaxToken),
	Stop(SyntaxToken),
	Wait(SyntaxToken),
}

// StateDef ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::StateDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StateDef(SyntaxNode);

simple_astnode!(Syntax, StateDef, Syntax::StateDef);

impl StateDef {
	/// The returned token is always tagged [`Syntax::NonWhitespace`].
	#[must_use]
	pub fn sprite(&self) -> SyntaxToken {
		let ret = self.0.first_token().unwrap();
		debug_assert_eq!(ret.kind(), Syntax::NonWhitespace);
		ret
	}

	pub fn frames(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.skip(1)
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::NonWhitespace)
			})
			.ok_or(AstError::Missing)
	}

	pub fn quals(&self) -> impl Iterator<Item = StateQual> {
		let quals = self
			.0
			.children()
			.find(|node| node.kind() == Syntax::StateQuals)
			.unwrap();

		quals
			.children_with_tokens()
			.filter_map(|elem| match elem.kind() {
				Syntax::KwBright => Some(StateQual::Bright(elem.into_token().unwrap())),
				Syntax::KwCanRaise => Some(StateQual::CanRaise(elem.into_token().unwrap())),
				Syntax::KwFast => Some(StateQual::Fast(elem.into_token().unwrap())),
				Syntax::StateLight => Some(StateQual::Light(StateLight(elem.into_node().unwrap()))),
				Syntax::KwNoDelay => Some(StateQual::NoDelay(elem.into_token().unwrap())),
				Syntax::StateOffset => {
					Some(StateQual::Offset(StateOffset(elem.into_node().unwrap())))
				}
				Syntax::KwSlow => Some(StateQual::Slow(elem.into_token().unwrap())),
				_ => None,
			})
	}

	pub fn duration(&self) -> AstResult<Expr> {
		self.0
			.children()
			.find_map(Expr::cast)
			.ok_or(AstError::Missing)
	}

	#[must_use]
	pub fn action(&self) -> Option<ActionFunction> {
		let Some(node) = self.0.last_child() else {
			return None;
		};
		ActionFunction::cast(node)
	}
}

// StateQual ///////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum StateQual {
	Bright(SyntaxToken),
	CanRaise(SyntaxToken),
	Fast(SyntaxToken),
	Light(StateLight),
	NoDelay(SyntaxToken),
	Offset(StateOffset),
	Slow(SyntaxToken),
}

/// Wraps a node tagged [`Syntax::StateLight`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StateLight(SyntaxNode);

simple_astnode!(Syntax, StateLight, Syntax::StateLight);

impl StateLight {
	/// Each yielded token is tagged [`Syntax::StringLit`] or [`Syntax::NameLit`].
	pub fn lights(&self) -> impl Iterator<Item = LitToken<Syntax>> {
		self.0.children_with_tokens().filter_map(|elem| {
			elem.into_token()
				.filter(|token| matches!(token.kind(), Syntax::NameLit | Syntax::StringLit))
				.map(LitToken::new)
		})
	}
}

/// Wraps a node tagged [`Syntax::StateOffset`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StateOffset(SyntaxNode);

simple_astnode!(Syntax, StateOffset, Syntax::StateOffset);

impl StateOffset {
	pub fn x(&self) -> AstResult<Expr> {
		let Some(node) = self.0.first_child() else {
			return Err(AstError::Missing);
		};
		Expr::cast(node).ok_or(AstError::Incorrect)
	}

	pub fn y(&self) -> AstResult<Expr> {
		let Some(node) = self.0.last_child() else {
			return Err(AstError::Missing);
		};
		Expr::cast(node).ok_or(AstError::Incorrect)
	}
}

// ActionFunction //////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::ActionFunction`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ActionFunction(SyntaxNode);

simple_astnode!(Syntax, ActionFunction, Syntax::ActionFunction);

impl ActionFunction {
	/// Token `0` is always tagged [`Syntax::Ident`].
	#[must_use]
	pub fn into_call(&self) -> Option<(SyntaxToken, Option<ArgList>)> {
		let ident = match self
			.0
			.first_token()
			.filter(|token| token.kind() == Syntax::Ident)
		{
			Some(t) => t,
			None => return None,
		};

		let args = match self.0.last_child() {
			Some(n) => ArgList::cast(n),
			None => None,
		};

		Some((ident, args))
	}

	#[must_use]
	pub fn into_anon(&self) -> Option<CompoundStat> {
		match self.0.first_child() {
			Some(node) => CompoundStat::cast(node),
			None => None,
		}
	}
}
