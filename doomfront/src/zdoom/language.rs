//! [LANGUAGE](https://zdoom.org/wiki/LANGUAGE) is a language for defining
//! localized strings.

pub mod parse;
mod syn;

pub use syn::Syn;

use rowan::{GreenNode, GreenToken};

use crate::{parsing::Gtb8, GreenElement};

peg::parser! {
	pub grammar parser() for str {
		pub rule file() -> GreenNode
			= elems:(trivia() / key_val_pair() / locale_tag())* ![_]
		{
			GreenNode::new(Syn::Root.into(), elems)
		}

		pub rule key_val_pair() -> GreenElement
			= 	id:ident()
				t0:trivia()*
				eq:$("=")?
				strings:string()*
				term:$(";")?
		{
			let mut gtb = Gtb8::new(Syn::KeyValuePair, Syn::Error);
			gtb.start(id);
			gtb.append(t0);
			gtb.just_s(Syn::Eq, eq);
			if strings.is_empty() {
				gtb.fail();
			} else {
				for s in strings {
					gtb.append(s);
				}
			}
			gtb.maybe_s(Syn::Semicolon, term);
			gtb.finish().into()
		}

		pub rule string() -> Vec<GreenElement>
			= t:trivia()* s:$("\"" (("\\" "\"") / ([^ '"']))* "\"")
		{
			let mut t = t;
			t.push(GreenToken::new(Syn::StringLit.into(), s).into());
			t
		}

		pub rule locale_tag() -> GreenElement
			= 	lb:$("[")
				t0:trivia()*
				id:ident()?
				t1:trivia()+
				kw_def:$("default")?
				t2:trivia()*
				rb:$("]")?
		{
			let mut gtb = Gtb8::new(Syn::LocaleTag, Syn::Error);
			gtb.start_s(Syn::BracketL, lb);
			gtb.append(t0);
			gtb.just(id);
			gtb.append(t1);
			gtb.just_s(Syn::KwDefault, kw_def);
			gtb.append(t2);
			gtb.just_s(Syn::BracketR, rb);
			gtb.finish().into()
		}

		rule ident() -> GreenToken
			= string:$(
				['a'..='z' | 'A'..='Z' | '_']
				['a'..='z' | 'A'..='Z' | '0'..='9' | '_']*
			)
		{
			GreenToken::new(Syn::Ident.into(), string)
		}

		pub rule trivia() -> GreenElement = t:(wsp() / comment()) { t.into() }

		pub rule wsp() -> GreenToken = string:$(['\0'..=' ']+) {
			GreenToken::new(Syn::Whitespace.into(), string)
		}

		pub rule comment() -> GreenToken
			= string:(
				$(
					"//" [^ '\n']* "\n"*
				) /
				$(
					"/*" ([^ '*'] / ("*" [^ '/']))* "*"+ "/"
				)
			)
		{
			GreenToken::new(Syn::Comment.into(), string)
		}
	}
}
