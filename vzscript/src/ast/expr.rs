//! AST nodes for representing expressions.

use doomfront::{
	rowan::{ast::AstNode, Language},
	simple_astnode, AstError, AstResult,
};

use crate::{Syn, SyntaxNode, SyntaxToken};

use super::LitToken;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
	Array(ArrayExpr),
	Binary(BinExpr),
	Block(BlockExpr),
	Call(CallExpr),
	Class(ClassExpr),
	Construct(ConstructExpr),
	Enum(EnumExpr),
	Field(FieldExpr),
	For(ForExpr),
	Group(GroupExpr),
	Function(FunctionExpr),
	Ident(IdentExpr),
	Index(IndexExpr),
	Literal(Literal),
	Prefix(PrefixExpr),
	Struct(StructExpr),
	Switch(SwitchExpr),
	Type(TypeExpr),
	Union(UnionExpr),
	Variant(VariantExpr),
	While(WhileExpr),
}

impl AstNode for Expr {
	type Language = Syn;

	fn can_cast(kind: <Self::Language as Language>::Kind) -> bool
	where
		Self: Sized,
	{
		matches!(
			kind,
			Syn::ArrayExpr
				| Syn::BinExpr | Syn::BlockExpr
				| Syn::CallExpr | Syn::ClassExpr
				| Syn::ConstructExpr
				| Syn::EnumExpr | Syn::FieldExpr
				| Syn::GroupExpr | Syn::FunctionExpr
				| Syn::IdentExpr | Syn::IndexExpr
				| Syn::Literal | Syn::PrefixExpr
				| Syn::StructExpr
				| Syn::SwitchExpr
				| Syn::TypeExpr | Syn::UnionExpr
				| Syn::VariantExpr
		)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::ArrayExpr => Some(Self::Array(ArrayExpr(node))),
			Syn::BinExpr => Some(Self::Binary(BinExpr(node))),
			Syn::BlockExpr => Some(Self::Block(BlockExpr(node))),
			Syn::CallExpr => Some(Self::Call(CallExpr(node))),
			Syn::ClassExpr => Some(Self::Class(ClassExpr(node))),
			Syn::ConstructExpr => Some(Self::Construct(ConstructExpr(node))),
			Syn::EnumExpr => Some(Self::Enum(EnumExpr(node))),
			Syn::FieldExpr => Some(Self::Field(FieldExpr(node))),
			Syn::ForExpr => Some(Self::For(ForExpr(node))),
			Syn::GroupExpr => Some(Self::Group(GroupExpr(node))),
			Syn::FunctionExpr => Some(Self::Function(FunctionExpr(node))),
			Syn::IdentExpr => Some(Self::Ident(IdentExpr(node))),
			Syn::IndexExpr => Some(Self::Index(IndexExpr(node))),
			Syn::Literal => Some(Self::Literal(Literal(node))),
			Syn::PrefixExpr => Some(Self::Prefix(PrefixExpr(node))),
			Syn::StructExpr => Some(Self::Struct(StructExpr(node))),
			Syn::SwitchExpr => Some(Self::Switch(SwitchExpr(node))),
			Syn::TypeExpr => Some(Self::Type(TypeExpr::cast(node).unwrap())),
			Syn::UnionExpr => Some(Self::Union(UnionExpr(node))),
			Syn::VariantExpr => Some(Self::Variant(VariantExpr(node))),
			Syn::WhileExpr => Some(Self::While(WhileExpr(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::Array(inner) => inner.syntax(),
			Self::Binary(inner) => inner.syntax(),
			Self::Block(inner) => inner.syntax(),
			Self::Call(inner) => inner.syntax(),
			Self::Class(inner) => inner.syntax(),
			Self::Construct(inner) => inner.syntax(),
			Self::Enum(inner) => inner.syntax(),
			Self::Field(inner) => inner.syntax(),
			Self::Group(inner) => inner.syntax(),
			Self::For(inner) => inner.syntax(),
			Self::Function(inner) => inner.syntax(),
			Self::Ident(inner) => inner.syntax(),
			Self::Index(inner) => inner.syntax(),
			Self::Literal(inner) => inner.syntax(),
			Self::Prefix(inner) => inner.syntax(),
			Self::Struct(inner) => inner.syntax(),
			Self::Switch(inner) => inner.syntax(),
			Self::Type(inner) => inner.syntax(),
			Self::Union(inner) => inner.syntax(),
			Self::Variant(inner) => inner.syntax(),
			Self::While(inner) => inner.syntax(),
		}
	}
}

// Array ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ArrayExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrayExpr(SyntaxNode);

simple_astnode!(Syn, ArrayExpr, Syn::ArrayExpr);

// Binary //////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::BinExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BinExpr(SyntaxNode);

simple_astnode!(Syn, BinExpr, Syn::BinExpr);

impl BinExpr {
	#[must_use]
	pub fn left(&self) -> Expr {
		Expr::cast(self.0.first_child().unwrap()).unwrap()
	}

	pub fn right(&self) -> AstResult<Expr> {
		Expr::cast(self.0.children().nth(1).ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}
}

// Block ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::BlockExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockExpr(SyntaxNode);

simple_astnode!(Syn, BlockExpr, Syn::BlockExpr);

// Call ////////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::CallExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CallExpr(SyntaxNode);

simple_astnode!(Syn, CallExpr, Syn::CallExpr);

impl CallExpr {}

// Class ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ClassExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClassExpr(SyntaxNode);

simple_astnode!(Syn, ClassExpr, Syn::ClassExpr);

// Construct ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ConstructExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConstructExpr(SyntaxNode);

simple_astnode!(Syn, ConstructExpr, Syn::ConstructExpr);

// Enum ////////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::EnumExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumExpr(SyntaxNode);

simple_astnode!(Syn, EnumExpr, Syn::EnumExpr);

// Field ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::FieldExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldExpr(SyntaxNode);

simple_astnode!(Syn, FieldExpr, Syn::FieldExpr);

impl FieldExpr {
	#[must_use]
	pub fn lhs(&self) -> Expr {
		Expr::cast(self.0.first_child().unwrap()).unwrap()
	}

	pub fn field_name(&self) -> AstResult<SyntaxToken> {
		self.0.last_token().ok_or(AstError::Missing)
	}
}

// For /////////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ForExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ForExpr(SyntaxNode);

simple_astnode!(Syn, ForExpr, Syn::ForExpr);

// Function ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::FunctionExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionExpr(SyntaxNode);

simple_astnode!(Syn, FunctionExpr, Syn::FunctionExpr);

// Group ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::GroupExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GroupExpr(SyntaxNode);

simple_astnode!(Syn, GroupExpr, Syn::GroupExpr);

impl GroupExpr {
	pub fn inner(&self) -> AstResult<Expr> {
		Expr::cast(self.0.first_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}
}

// Ident. //////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::IdentExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdentExpr(SyntaxNode);

simple_astnode!(Syn, IdentExpr, Syn::IdentExpr);

impl IdentExpr {
	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn token(&self) -> SyntaxToken {
		let ret = self.0.last_token().unwrap();
		debug_assert_eq!(ret.kind(), Syn::Ident);
		ret
	}

	/// The returned token is always tagged [`Syn::Dot`].
	#[must_use]
	pub fn leading_dot(&self) -> Option<SyntaxToken> {
		self.0
			.first_token()
			.filter(|token| token.kind() == Syn::Dot)
	}
}

// Index ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::IndexExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndexExpr(SyntaxNode);

simple_astnode!(Syn, IndexExpr, Syn::IndexExpr);

impl IndexExpr {
	#[must_use]
	pub fn indexed(&self) -> Expr {
		Expr::cast(self.0.first_child().unwrap()).unwrap()
	}

	pub fn index(&self) -> AstResult<Expr> {
		match self.0.children().nth(1) {
			Some(node) => Expr::cast(node).ok_or(AstError::Incorrect),
			None => Err(AstError::Missing),
		}
	}
}

// Literal /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::Literal`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Literal(SyntaxNode);

simple_astnode!(Syn, Literal, Syn::Literal);

impl Literal {
	#[must_use]
	pub fn token(&self) -> LitToken {
		LitToken(self.0.first_token().unwrap())
	}
}

// Prefix //////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::PrefixExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PrefixExpr(SyntaxNode);

simple_astnode!(Syn, PrefixExpr, Syn::PrefixExpr);

impl PrefixExpr {
	pub fn operand(&self) -> AstResult<Expr> {
		Expr::cast(self.0.last_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}

	#[must_use]
	pub fn operator(&self) -> (SyntaxToken, PrefixOp) {
		let ret0 = self.0.first_token().unwrap();

		let ret1 = match ret0.kind() {
			Syn::Bang => PrefixOp::Bang,
			Syn::Minus => PrefixOp::Minus,
			Syn::Tilde => PrefixOp::Tilde,
			_ => unreachable!(),
		};

		(ret0, ret1)
	}
}

/// See [`PrefixExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrefixOp {
	Bang,
	Minus,
	Tilde,
}

// Struct //////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::StructExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructExpr(SyntaxNode);

simple_astnode!(Syn, StructExpr, Syn::StructExpr);

// Switch //////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::SwitchExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SwitchExpr(SyntaxNode);

simple_astnode!(Syn, SwitchExpr, Syn::SwitchExpr);

// Type ////////////////////////////////////////////////////////////////////////

/// Each variant wraps a node tagged [`Syn::TypeExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeExpr {
	Auto(AutoTypeExpr),
	KwType(KwTypeExpr),
	Prefixed(PrefixedTypeExpr),
}

impl AstNode for TypeExpr {
	type Language = Syn;

	fn can_cast(kind: Syn) -> bool
	where
		Self: Sized,
	{
		kind == Syn::TypeExpr
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		if node.kind() != Syn::TypeExpr {
			return None;
		}

		match node.first_token().unwrap().kind() {
			Syn::KwAuto => Some(Self::Auto(AutoTypeExpr(node))),
			Syn::KwType => Some(Self::KwType(KwTypeExpr(node))),
			_ => Some(Self::Prefixed(PrefixedTypeExpr(node))),
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::Auto(inner) => inner.syntax(),
			Self::KwType(inner) => inner.syntax(),
			Self::Prefixed(inner) => inner.syntax(),
		}
	}
}

/// Wraps a node tagged [`Syn::TypeExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AutoTypeExpr(SyntaxNode);

simple_astnode!(Syn, AutoTypeExpr, Syn::TypeExpr);

/// Wraps a node tagged [`Syn::TypeExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KwTypeExpr(SyntaxNode);

simple_astnode!(Syn, KwTypeExpr, Syn::TypeExpr);

/// Wraps a node tagged [`Syn::TypeExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PrefixedTypeExpr(SyntaxNode);

simple_astnode!(Syn, PrefixedTypeExpr, Syn::TypeExpr);

impl PrefixedTypeExpr {
	pub fn prefixes(&self) -> impl Iterator<Item = TypeExprPrefix> {
		self.0.children().filter_map(TypeExprPrefix::cast)
	}

	pub fn expr(&self) -> AstResult<Expr> {
		Expr::cast(self.0.last_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeExprPrefix {
	Array(ArrayPrefix),
	Option(OptionPrefix),
	Ref(RefPrefix),
}

impl AstNode for TypeExprPrefix {
	type Language = Syn;

	fn can_cast(kind: Syn) -> bool
	where
		Self: Sized,
	{
		matches!(kind, Syn::ArrayPrefix | Syn::OptionPrefix | Syn::RefPrefix)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::ArrayPrefix => Some(Self::Array(ArrayPrefix(node))),
			Syn::OptionPrefix => Some(Self::Option(OptionPrefix(node))),
			Syn::RefPrefix => Some(Self::Ref(RefPrefix(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::Array(inner) => inner.syntax(),
			Self::Option(inner) => inner.syntax(),
			Self::Ref(inner) => inner.syntax(),
		}
	}
}

/// Wraps a node tagged [`Syn::ArrayPrefix`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrayPrefix(SyntaxNode);

simple_astnode!(Syn, ArrayPrefix, Syn::ArrayPrefix);

impl ArrayPrefix {
	pub fn len_expr(&self) -> AstResult<Expr> {
		Expr::cast(self.0.first_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}
}

/// Wraps a node tagged [`Syn::OptionPrefix`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OptionPrefix(SyntaxNode);

simple_astnode!(Syn, OptionPrefix, Syn::OptionPrefix);

impl OptionPrefix {
	/// The returned token is always tagged [`Syn::Question`].
	#[must_use]
	pub fn token(&self) -> SyntaxToken {
		let ret = self.0.last_token().unwrap();
		debug_assert_eq!(ret.kind(), Syn::Question);
		ret
	}
}

/// Wraps a node tagged [`Syn::RefPrefix`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct RefPrefix(SyntaxNode);

simple_astnode!(Syn, RefPrefix, Syn::RefPrefix);

impl RefPrefix {
	/// The returned token is always tagged [`Syn::Ampersand`].
	#[must_use]
	pub fn token(&self) -> SyntaxToken {
		let ret = self.0.last_token().unwrap();
		debug_assert_eq!(ret.kind(), Syn::Ampersand);
		ret
	}
}

// Union ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::UnionExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnionExpr(SyntaxNode);

simple_astnode!(Syn, UnionExpr, Syn::UnionExpr);

// Variant /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::VariantExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VariantExpr(SyntaxNode);

simple_astnode!(Syn, VariantExpr, Syn::VariantExpr);

// While ///////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::WhileExpr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WhileExpr(SyntaxNode);

simple_astnode!(Syn, WhileExpr, Syn::WhileExpr);
