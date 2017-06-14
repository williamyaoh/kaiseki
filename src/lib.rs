//! kaiseki -- literate programming preprocessing

#[macro_use] extern crate error_chain;
extern crate regex;

pub mod input;
pub mod list;

mod parsing;

pub mod processing_errors {
  error_chain! {
    errors {
      NotUTF8(file: String, lineno: usize) {
        description("line is not valid UTF-8")
        display("error: '{}', line {}: not valid UTF-8", file, lineno)
      }

      MalformedAnchor(file: String, lineno: usize, anchor: String) {
        description("could not parse anchor tag")
        display("warn: '{}', line {}: ignoring malformed anchor: '{}'", file, lineno, anchor)
      }

      DuplicateAnchor(file: String, lineno: usize, tag: String) {
        description("found a duplicate anchor tag")
        display("warn: '{}', line {}: ignoring duplicate anchor tag: '{}'", file, lineno, tag)
      }

      MissingTag(file: String, lineno: usize, tag: String) {
        description("nonexistent tag name")
        display("warn: '{}', line {}: nonexistent tag name: '{}'", file, lineno, tag)
      }
    }
  } 
}

use std::rc::Rc;
use std::io;
use std::result;
use std::default::Default;

use std::collections::BTreeMap;

use input::File;
use list::List;

pub struct OutputOptions {
  pub comment: Option<String>
}

impl Default for OutputOptions {
  fn default() -> Self {
    OutputOptions {
      comment: None
    }
  }
}

struct Block {
  lines: Vec<String>,
  file: Rc<String>,
  lineno: usize
}

impl Block {
  fn new(file: Rc<String>, lineno: usize) -> Self {
    Block {
      lines: Vec::new(),
      file: file,
      lineno: lineno
    }
  }
}

struct Anchor {
  indentation: usize,  // The *absolute* level of indentation.
  tangled: Tangled
}

impl Anchor {
  fn new(indentation: usize) -> Self {
    Anchor {
      indentation: indentation,
      tangled: List::new()
    }
  }
}

/// An `Either` represents the situation when *either* arm is a valid
/// value, as opposed to a `Result`, where one arm designates an error.
enum Either<T, U> {
  Left(T),
  Right(U)
}

struct AnchorRef(String);

type Tangled = List<Either<Block, AnchorRef>>;

enum OutputTarget {
  Insert,
  Before(AnchorRef),
  After(AnchorRef)
}

/// Process all the literate programming directives in the contents of the
/// given files, return a Vec of output lines (suitable for immediate
/// printing to, say, `stdout`)
pub fn tangle_output(inputs: Vec<File>, options: OutputOptions) -> (Vec<String>, Vec<processing_errors::Error>) {
  use std::io::{BufReader, BufRead};

  use parsing::Anchor;
  use processing_errors::ErrorKind;

  let mut tangled = List::new();
  let mut anchors = BTreeMap::new();
  let mut errors = Vec::new();  // Errors that we accrue during processing.

  for input in inputs {
    let filename = Rc::new(input.name);

    let mut lines = BufReader::new(input.contents)
      .lines()
      .enumerate()
      .map(|(lineno, line)| (lineno + 1, line));
    let mut state = OutputTarget::Insert;
    let mut tangled_section = List::new();
    let mut block = Block::new(filename.clone(), 1);

    macro_rules! emplace_section {
      () => {
        match state {
          OutputTarget::Insert => tangled.append_back(&mut tangled_section),
          OutputTarget::Before(AnchorRef(anchor_name)) => {
            let anchor: &mut ::Anchor = anchors.get_mut(&anchor_name)
              .expect("invariant violated: anchor name does not exist");
            anchor.tangled.append_front(&mut tangled_section);
          },
          OutputTarget::After(AnchorRef(anchor_name)) => {
            let anchor: &mut ::Anchor = anchors.get_mut(&anchor_name)
              .expect("invariant violated: anchor name does not exist");
            anchor.tangled.append_back(&mut tangled_section);
          }
        }
      }
    }

    loop {
      let next_anchor = process_block_lines(&mut lines, &mut block, &mut errors);

      if !block.lines.is_empty() {
        tangled_section.push_back(Either::Left(block));
      }

      match next_anchor {
        Some((lineno, indentation, anchor)) => {
          macro_rules! has_anchor {
            ($anchor_name:expr) => {{
              if anchors.contains_key($anchor_name) {
                true
              } else {
                let filename: &String = &filename;
                let error = ErrorKind::MissingTag(filename.clone(), lineno, $anchor_name.clone()).into();
                errors.push(error);
                false
              }
            }}
          }

          block = Block::new(filename.clone(), lineno);
          match anchor {
            Anchor::Insert => {
              emplace_section!();
              tangled_section = List::new();
              state = OutputTarget::Insert;
            },
            Anchor::Before(anchor_name) => {
              emplace_section!();
              tangled_section = List::new();
              if has_anchor!(&anchor_name) {
                state = OutputTarget::Before(AnchorRef(anchor_name));
              } else {
                state = OutputTarget::Insert;
              }
            },
            Anchor::After(anchor_name) => {
              emplace_section!();
              tangled_section = List::new();
              if has_anchor!(&anchor_name) {
                state = OutputTarget::After(AnchorRef(anchor_name));
              } else {
                state = OutputTarget::Insert;
              }
            },
            Anchor::Label(anchor_name) => {
              let anchor = ::Anchor::new(indentation);
              anchors.insert(anchor_name.clone(), anchor);
              tangled_section.push_back(Either::Right(AnchorRef(anchor_name)));
            }
          };
        },
        None => {
          emplace_section!();
          break;
        }
      };
    }
  }
  
  (collect_tangled_output(tangled, anchors, options), errors)
}

fn collect_tangled_output(tangled: Tangled, 
                          mut anchors: BTreeMap<String, Anchor>,
                          options: OutputOptions) -> Vec<String> 
{
  let mut lines = Vec::new();
  collect_anchor_lines(tangled, &mut anchors, &mut lines, 0, &options);
  lines
}

fn maybe_block_header(block: &Block, options: &OutputOptions) -> Option<String> {
  match &options.comment {
    &Some(ref comment_prefix) => {
      let header = format!(
        "{} '{}', line {}",
        comment_prefix,
        &block.file,
        block.lineno
      );

      Some(header)
    }
    &None => None
  }
}

fn collect_anchor_lines(tangled: Tangled,
                        anchors: &mut BTreeMap<String, Anchor>,
                        lines: &mut Vec<String>,
                        indentation: usize,
                        options: &OutputOptions)
{
  use std::iter;

  let indent_prefix = iter::repeat(' ').take(indentation).collect::<String>();
   
  for knot in tangled {
    match knot {
      Either::Left(block) => {
        if let Some(comment) = maybe_block_header(&block, options) {
          lines.push(indent_prefix.clone() + &comment);
        }

        for line in block.lines {
          lines.push(indent_prefix.clone() + &line);
        }
      },
      Either::Right(AnchorRef(ref anchor_name)) => {
        let anchor = anchors.remove(anchor_name)
          .expect("invariant violated: anchor name does not exist");

        collect_anchor_lines(
          anchor.tangled,
          anchors,
          lines,
          indentation + anchor.indentation,
          options
        );
      }
    };
  }
}

/// We scan through each file block by block.
/// Each block will end in either an anchor tag, or the end of the file.
fn process_block_lines<I>(lines: &mut I, block: &mut Block, errors: &mut Vec<processing_errors::Error>) -> Option<(usize, usize, parsing::Anchor)> where
  I: Iterator<Item=(usize, result::Result<String, io::Error>)>
{
  use processing_errors::ErrorKind;
  use std::ops::Deref;

  let filename = block.file.deref();

  for (lineno, line) in lines {
    match line {
      Ok(line) => {
        let result = parsing::might_be_anchor(&line)
          .ok_or(None)
          .and_then(|found| {
            parsing::parse(found.as_str())
              .map_err(|_| Some(ErrorKind::MalformedAnchor(
                filename.clone(),
                lineno,
                found.as_str().to_string()
              ).into()))
          });

        match result {
          Ok(anchor) => return Some((lineno, indentation_level(&line), anchor)),
          Err(Some(error)) => {
            errors.push(error);
            block.lines.push(line);
          },
          Err(None) => {
            block.lines.push(line);
          }
        };
      },
      Err(_) => errors.push(ErrorKind::NotUTF8(filename.clone(), lineno).into())
    };
  }

  None
}

/// Index of first non-whitespace character.
fn indentation_level(line: &str) -> usize {
  use regex::Regex;

  let nonwhitespace = Regex::new(r"[^\s]").unwrap();
  match nonwhitespace.find(line) {
    Some(found) => found.start(),
    None => 0
  }
}
