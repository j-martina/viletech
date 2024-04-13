//! Utilities for unit testing and benchmarking.

use std::path::PathBuf;

use rowan::{SyntaxElement, SyntaxNode, WalkEvent};

use crate::{LangExt, ParseError, ParseTree};

/// A wrapper for `ptree.errors.is_empty()` which also formats and prints out
/// each error, if any are detected.
pub fn assert_no_errors<L>(ptree: &ParseTree<L>)
where
	L: LangExt,
	L::Token: std::fmt::Debug,
{
	assert!(
		ptree.errors.is_empty(),
		"encountered errors: {}\r\n",
		format_errors(&ptree.errors)
	);
}

/// Unit testing helper; checks that `elem` is a node with the given syntax tag.
pub fn assert_node<L: LangExt>(elem: SyntaxElement<L>, kind: L::Kind) {
	let node = elem.as_node();

	assert!(
		node.is_some(),
		"element {elem:?} is unexpectedly not a node",
	);

	let node = node.unwrap();

	assert_eq!(
		node.kind(),
		kind,
		"expected token kind {kind:?}, found {:?}",
		node.kind()
	);
}

/// Unit testing helper; checks that `elem` is a token with the given syntax tag and text.
pub fn assert_token<L>(
	elem: rowan::NodeOrToken<SyntaxNode<L>, rowan::SyntaxToken<L>>,
	kind: L::Kind,
	text: &'static str,
) where
	L: rowan::Language,
{
	let token = elem.as_token();

	assert!(
		token.is_some(),
		"element {elem:?} is unexpectedly not a token",
	);

	let token = token.unwrap();

	assert_eq!(
		token.kind(),
		kind,
		"expected token kind {kind:?}, found {:?}",
		token.kind()
	);

	assert_eq!(
		token.text(),
		text,
		"expected token text {text}, found {}",
		token.text()
	);
}

/// Unit testing helper; checks that [`rowan::WalkEvent::Enter`] events match
/// the node or token data provided in `seq`.
pub fn assert_sequence<L>(seq: &'static [(L::Kind, Option<&'static str>)], cursor: SyntaxNode<L>)
where
	L: rowan::Language,
{
	let seq_count = seq.iter().clone().count();
	let elem_count = cursor.preorder_with_tokens().count();

	assert_eq!(
		seq_count,
		(elem_count / 2),
		"expected a parsed sequence of {seq_count} elements, but found {elem_count}",
	);

	let iter_s = seq.iter().copied();
	let iter_c = cursor
		.preorder_with_tokens()
		.filter_map(|event| match event {
			WalkEvent::Enter(enter) => Some(enter),
			WalkEvent::Leave(_) => None,
		});

	let iter_z = iter_s.zip(iter_c);

	for (i, ((kind, text), elem)) in iter_z.enumerate() {
		assert_eq!(
			elem.kind(),
			kind,
			"expected element {i} to have kind {kind:?} but found {:?}",
			elem.kind()
		);

		if let Some(text) = text {
			let token = elem.as_token();

			assert!(token.is_some());

			assert!(token.is_some(), "element {i} is unexpectedly not a token",);

			let token = token.unwrap();

			assert_eq!(
				token.text(),
				text,
				"expected element {i} to have text {text} but found {}",
				token.text()
			);
		} else {
			assert!(
				elem.as_node().is_some(),
				"element {i} is unexpectedly not a node"
			);
		}
	}
}

/// For diagnosing parsers (or tests). Walks the node tree in preorder,
/// printing each node and token's display representation with indentation
/// according to the depth in the tree.
pub fn prettyprint<L: LangExt>(cursor: SyntaxNode<L>) {
	let mut depth = 0;

	for event in cursor.preorder_with_tokens() {
		match event {
			WalkEvent::Enter(elem) => {
				let mut print = String::new();

				for _ in 0..depth {
					print.push_str("    ");
				}

				print.push_str(&format!("{elem:?}"));
				println!("{print}");

				depth += 1;
			}
			WalkEvent::Leave(_) => {
				depth -= 1;
			}
		}
	}
}

/// If the environment variable `DOOMFRONT_TEST_PRETTYPRINT` has been set to "1",
/// `cursor`'s syntax tree will be recursively printed to stdout.
pub fn prettyprint_maybe<L: LangExt>(cursor: SyntaxNode<L>) -> bool {
	if std::env::var("DOOMFRONT_TEST_PRETTYPRINT").is_ok_and(|v| v == "1") {
		prettyprint(cursor);
		true
	} else {
		false
	}
}

/// `Err` variants contain the reason the read failed. This can happen because:
/// - the environment variable behind `env_var_name` could not be retrieved
/// - the path at the environment variable is to a non-existent file
/// - filesystem IO fails
pub fn read_sample_data(env_var_name: &'static str) -> Result<(PathBuf, String), String> {
	let path = match std::env::var(env_var_name) {
		Ok(p) => PathBuf::from(p),
		Err(err) => {
			return Err(format!(
				"failed to get environment variable `{env_var_name}` ({err})"
			))
		}
	};

	if !path.exists() {
		return Err(format!("file `{}` does not exist", path.display()));
	}

	let bytes = match std::fs::read(&path) {
		Ok(b) => b,
		Err(err) => return Err(format!("{err}")),
	};

	let sample = String::from_utf8_lossy(&bytes).to_string();

	Ok((path, sample))
}

/// `Err` variants contain the reason the read failed. This can happen because:
/// - the environment variable behind `env_var_name` could not be retrieved
/// - the path at the environment variable is to a non-existent directory
/// - the path at the environment variable is to a non-directory entity
pub fn check_sample_dir(env_var_name: &'static str) -> Result<PathBuf, String> {
	let path = match std::env::var(env_var_name) {
		Ok(p) => PathBuf::from(p),
		Err(err) => {
			return Err(format!(
				"failed to get environment variable `{env_var_name}` ({err})"
			))
		}
	};

	if !path.exists() {
		return Err(format!("directory `{}` does not exist", path.display()));
	}

	if !path.is_dir() {
		return Err(format!("`{}` is not a directory", path.display()));
	}

	Ok(path)
}

// Details /////////////////////////////////////////////////////////////////////

#[must_use]
fn format_errors<L>(errors: &Vec<ParseError<L>>) -> String
where
	L: LangExt,
	L::Token: std::fmt::Debug,
{
	let mut output = String::new();

	for err in errors {
		output.push_str(&format!("\r\n{err:#?}"));
	}

	output
}
