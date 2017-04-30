use regex::Regex;
use regex::Match;

use std::collections::VecDeque;

pub mod errors {
  error_chain! {
    errors {
      LexError
      ParseError
    }
  }
}

use self::errors::*;

#[derive(Debug, Eq, PartialEq)]
enum Token {
  /// Only used for initialization of token gathering.
  Null,
  AnchorStart,
  AnchorEnd,
  AnchorOp(Op),
  AnchorOpArg(String)
}

#[derive(Debug, Eq, PartialEq)]
enum Op {
  Insert,
  Before,
  After,
  Label
}

#[derive(Debug, Eq, PartialEq)]
pub enum Anchor {
  Insert,
  Before(String),
  After(String),
  Label(String)
}

/// Attempt to parse the given string as a Kaiseki anchor.
pub fn parse(text: &str) -> Result<Anchor> {
  let lex_result = lex_tokens(text)?;
  parse_anchor(lex_result)
}

macro_rules! check_next {
  ($tokens:ident { $($token:pat => $result:block),+ }) => {{
    let next_token = $tokens.pop_front();

    let next_token = match next_token {
      Some(token) => token,
      None => bail!(ErrorKind::ParseError)
    };

    match next_token {
      $(
        $token => $result
      ),+
      _ => bail!(ErrorKind::ParseError)
    }
  }}
}

/// Check if the line *might* contain an anchor. Returns the matching
/// string, so that it can then be formally checked with a parser.
pub fn might_be_anchor(line: &str) -> Option<Match> {
  let anchor = Regex::new(r"##\[[^]]+\]").unwrap();

  anchor.find(line)
}

fn parse_anchor(mut tokens: VecDeque<Token>) -> Result<Anchor> {
  check_next!(tokens {
    Token::AnchorStart => { parse_op(&mut tokens) }
  })
}

fn parse_op(tokens: &mut VecDeque<Token>) -> Result<Anchor> {
  check_next!(tokens {
    Token::AnchorOp(Op::Insert) => {
      parse_end(tokens)?;

      Ok(Anchor::Insert)
    },
    Token::AnchorOp(Op::Before) => {
      let arg = parse_arg(tokens)?;
      parse_end(tokens)?;

      Ok(Anchor::Before(arg))
    },
    Token::AnchorOp(Op::After) => {
      let arg = parse_arg(tokens)?;
      parse_end(tokens)?;

      Ok(Anchor::After(arg))
    },
    Token::AnchorOp(Op::Label) => {
      let arg = parse_arg(tokens)?;
      parse_end(tokens)?;

      Ok(Anchor::Label(arg))
    }
  })
}

fn maybe_parse_arg(tokens: &mut VecDeque<Token>) -> Option<String> {
  match parse_arg(tokens) {
    Ok(arg) => Some(arg),
    Err(_) => None
  }
}

fn parse_arg(tokens: &mut VecDeque<Token>) -> Result<String> {
  check_next!(tokens {
    Token::AnchorOpArg(str) => {
      Ok(str)
    }
  })
}

fn parse_end(tokens: &mut VecDeque<Token>) -> Result<()> {
  check_next!(tokens {
    Token::AnchorEnd => { }
  });

  Ok(())
}

/// For now, we assume that every regular expression passed in has
/// a '^' anchor at the beginning. Otherwise, bad things will happen.
macro_rules! lexer {
  ($($regex:expr => $out:expr),+) => {
    |lexing: &str| {
      let mut chars = &lexing[..];
      let lexers: Vec<(Regex, Box<Fn(&str) -> Token>)> = vec![
        $({
          let regex = Regex::new($regex).unwrap();
          (regex, Box::new($out))
        }),+
      ];
      let mut tokens = VecDeque::new();

      while !chars.is_empty() {
        let mut max_match = 0;
        let mut max_token = Token::Null;

        for i in 0..lexers.len() {
          let &(ref regex, ref out) = &lexers[i];

          if let Some(matched) = regex.find(chars) {
            if matched.end() > max_match {
              max_match = matched.end();
              max_token = out(matched.as_str());
            }
          }
        }

        if max_match == 0 { bail!(ErrorKind::LexError); }

        chars = &chars[max_match..];
        tokens.push_back(max_token);
      }

      Ok(tokens)
    }
  }
}

fn lex_tokens(chars: &str) -> Result<VecDeque<Token>> {
  let lexer = lexer! {
    r"^##\[" => |_| Token::AnchorStart,
    r"^\]" => |_| Token::AnchorEnd,
    r"^before" => |_| Token::AnchorOp(Op::Before),
    r"^after" => |_| Token::AnchorOp(Op::After),
    r"^insert" => |_| Token::AnchorOp(Op::Insert),
    r"^label" => |_| Token::AnchorOp(Op::Label),
    r"^\([\w\d\s\-]+\)" => |str| Token::AnchorOpArg(str.to_string())
  };

  lexer(chars)
}

#[cfg(test)]
mod parsing_tests {
  use super::Anchor;
  use super::might_be_anchor;
  use super::{lex_tokens, parse_anchor};

  #[test]
  fn test_might_be_anchor_1() {
    let str = "// ##[label(Processing)]  where we put all the imports";
    let result = might_be_anchor(str);
    
    assert!(result.is_some());
    let result = result.unwrap();
    assert_eq!(result.as_str(), "##[label(Processing)]");
  }

  #[test]
  fn test_might_be_anchor_2() {
    let str = ";;; ##[insert]";
    let result = might_be_anchor(str);

    assert!(result.is_some());
    let result = result.unwrap();
    assert_eq!(result.as_str(), "##[insert]");
  }

  #[test]
  fn test_might_be_anchor_failure_1() {
    let str = "#[macro_use]";
    assert!(might_be_anchor(str).is_none());
  }

  #[test]
  fn test_might_be_anchor_failure_2() {
    let str = "// ##[]";
    assert!(might_be_anchor(str).is_none());
  }

  #[test]
  fn test_might_be_anchor_failure_3() {
    let str = "extern crate docopt;";
    assert!(might_be_anchor(str).is_none());
  }

  #[test]
  fn test_parse_anchor_1() {
    let str = "##[insert]";
    let lex_result = lex_tokens(str).unwrap();
    let parse_result = parse_anchor(lex_result).unwrap();

    assert_eq!(parse_result, Anchor::Insert);
  }

  #[test]
  fn test_parse_anchor_2() {
    let str = "##[before(Something Else)]";
    let lex_result = lex_tokens(str).unwrap();
    let parse_result = parse_anchor(lex_result).unwrap();

    assert_eq!(parse_result, Anchor::Before("(Something Else)".to_string()));
  }

  #[test]
  fn test_parse_anchor_3() {
    let str = "##[after(kebab-case)]";
    let lex_result = lex_tokens(str).unwrap();
    let parse_result = parse_anchor(lex_result).unwrap();

    assert_eq!(parse_result, Anchor::After("(kebab-case)".to_string()));
  }

  #[test]
  fn test_parse_anchor_4() {
    let str = "##[label(label)]";
    let lex_result = lex_tokens(str).unwrap();
    let parse_result = parse_anchor(lex_result).unwrap();

    assert_eq!(parse_result, Anchor::Label("(label)".to_string()));
  }

  #[test]
  fn test_parse_anchor_fail_1() {
    let str = "##[label]";
    let lex_result = lex_tokens(str).unwrap();
    let parse_result = parse_anchor(lex_result);

    assert!(parse_result.is_err());
  }

  #[test]
  fn test_parse_anchor_fail_2() {
    let str = "##[]";
    let lex_result = lex_tokens(str).unwrap();
    let parse_result = parse_anchor(lex_result);

    assert!(parse_result.is_err());
  }
}

#[cfg(test)]
mod lexing_tests {
  use ::std::iter::FromIterator;

  use super::lex_tokens;
  use super::{Token, Op};

  #[test]
  fn test_lex_1() {
    let stream = "";
    let lexed = lex_tokens(stream);

    assert!(lexed.is_ok());

    let lexed = lexed.unwrap();

    assert_eq!(lexed.len(), 0);
  }

  #[test]
  fn test_lex_2() {
    let stream = "]]]";
    let lexed = lex_tokens(stream);

    assert!(lexed.is_ok());

    let lexed = Vec::from_iter(lexed.unwrap());

    assert_eq!(lexed.len(), 3);
    assert_eq!(&lexed as &[Token], [
      Token::AnchorEnd,
      Token::AnchorEnd,
      Token::AnchorEnd
    ]);
  }

  #[test]
  fn test_lex_3() {
    let stream = "##[label(Processing)]";
    let lexed = lex_tokens(stream);

    assert!(lexed.is_ok());

    let lexed = Vec::from_iter(lexed.unwrap());

    assert_eq!(lexed.len(), 4);
    assert_eq!(&lexed as &[Token], [
      Token::AnchorStart,
      Token::AnchorOp(Op::Label),
      Token::AnchorOpArg("(Processing)".to_string()),
      Token::AnchorEnd
    ]);
  }

  #[test]
  fn test_lex_4() {
    let stream = "##[after(Processing)]";
    let lexed = lex_tokens(stream);

    assert!(lexed.is_ok());

    let lexed = Vec::from_iter(lexed.unwrap());

    assert_eq!(lexed.len(), 4);
    assert_eq!(&lexed as &[Token], [
      Token::AnchorStart,
      Token::AnchorOp(Op::After),
      Token::AnchorOpArg("(Processing)".to_string()),
      Token::AnchorEnd
    ]);
  }

  #[test]
  fn test_lex_5() {
    let stream = "##[before(Processing)]";
    let lexed = lex_tokens(stream);

    assert!(lexed.is_ok());

    let lexed = Vec::from_iter(lexed.unwrap());

    assert_eq!(lexed.len(), 4);
    assert_eq!(&lexed as &[Token], [
      Token::AnchorStart,
      Token::AnchorOp(Op::Before),
      Token::AnchorOpArg("(Processing)".to_string()),
      Token::AnchorEnd
    ]);
  }

  #[test]
  fn test_lex_6() {
    let stream = "##[insert]";
    let lexed = lex_tokens(stream);

    assert!(lexed.is_ok());

    let lexed = Vec::from_iter(lexed.unwrap());

    assert_eq!(lexed.len(), 3);
    assert_eq!(&lexed as &[Token], [
      Token::AnchorStart,
      Token::AnchorOp(Op::Insert),
      Token::AnchorEnd
    ]);
  }

  #[test]
  fn test_lex_7() {
    let stream = "##[label(kebab-case)]";
    let lexed = lex_tokens(stream);

    assert!(lexed.is_ok());

    let lexed = Vec::from_iter(lexed.unwrap());

    assert_eq!(lexed.len(), 4);
    assert_eq!(&lexed as &[Token], [
      Token::AnchorStart,
      Token::AnchorOp(Op::Label),
      Token::AnchorOpArg("(kebab-case)".to_string()),
      Token::AnchorEnd
    ]);
  }

  #[test]
  fn test_lex_8() {
    let stream = "##[label(Has Spaces)]";
    let lexed = lex_tokens(stream);

    assert!(lexed.is_ok());

    let lexed = Vec::from_iter(lexed.unwrap());

    assert_eq!(lexed.len(), 4);
    assert_eq!(&lexed as &[Token], [
      Token::AnchorStart,
      Token::AnchorOp(Op::Label),
      Token::AnchorOpArg("(Has Spaces)".to_string()),
      Token::AnchorEnd
    ]);
  }

  #[test]
  fn test_lex_failure_1() {
    let stream = "[[[";
    let lexed = lex_tokens(stream);

    assert!(lexed.is_err());
  }

  #[test]
  fn test_lex_failure_2() {
    let stream = "// 101";
    let lexed = lex_tokens(stream);

    assert!(lexed.is_err());
  }
}
