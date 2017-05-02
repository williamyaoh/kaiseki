//! Preprocess and rearrange lines of input.
//!
//! Used for literate programming.

#[macro_use] extern crate error_chain;
extern crate regex;
extern crate docopt;
extern crate rustc_serialize;

mod parsing;
mod input;

mod errors {
  error_chain! {
    links {
      Parse(::parsing::errors::Error, ::parsing::errors::ErrorKind);
      Input(::input::errors::Error, ::input::errors::ErrorKind);
    }
  }
}

use errors::*;

use std::process;
use std::io::{stdout, stderr};
use std::io::Write;
use std::io;

use std::collections::BTreeMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::iter::IntoIterator;

use docopt::Docopt;

static VERSION: &'static str = "0.1.2";
macro_rules! VERSION_INFO {
  () => { "\
kaiseki {}
copyright (c) 2017 William Yao <williamyaoh@gmail.com>
license BSD 3-Clause
no warranty, whether implied or not
" }
}
static USAGE: &'static str = "
kaiseki -- literate programming preprocessing

Usage:
    kaiseki [options] [(<files> | [-])...]
    kaiseki (--help | --version)

Options:
    -h, --help              Display this help message
    --version               Display version information
    --comment, -c COMMENT   Add comments to output showing where lines of code
                            came from. Prefix them with the comment syntax COMMENT.

Tangles together all lines of code into a single file, which then
gets output to `stdout'. See kaiseki(1) for a description of the
literate programming syntax.
";

#[derive(RustcDecodable)]
struct CLIArgs {
  arg_files: Vec<String>,
  flag_comment: Option<String>
}

fn main() {
  let cli_parser = Docopt::new(USAGE).unwrap()
    .version(Some(VERSION.to_string()))
    .help(true);

  let cli_args: CLIArgs = cli_parser.decode().map_err(|err| match err {
    ::docopt::Error::Version(version) => {
      print!(VERSION_INFO!(), version);
      process::exit(0);
    },
    other => other.exit()
  }).unwrap();

  if let Err(ref e) = go(cli_args) {
    writeln!(stderr(), "kaiseki: {}", e)
      .unwrap();

    for e in e.iter().skip(1) {
      writeln!(stderr(), "  caused by: {}", e)
        .unwrap();
    }

    process::exit(1);
  }
}

/// Each Block is a part of an input file, delineated by anchor tags.
struct Block {
  contents: Vec<String>,
  from: Rc<String>,
  line: usize
}

impl Block {
  fn new(from: Rc<String>, line: usize) -> Self {
    Block {
      contents: Vec::new(),
      from: from,
      line: line
    }
  }
}

enum Section {
  Normal(Block),
  Anchored {
    /// Stored in reverse order of appearance in output file
    before: Vec<Block>,
    /// Stored in normal order of appearance in output file
    after: Vec<Block>
  }
}

fn go(args: CLIArgs) -> Result<()> {
  let files = input::open_files(args.arg_files)?;

  // Errors that we accumulate during processing. We output them at the end.
  let mut errors = Vec::new();

  let mut sections = Vec::new();

  {
    let mut lookup: BTreeMap<String, Rc<RefCell<Section>>> = BTreeMap::new();

    enum State {
      Insert,
      Before(String),
      After(String)
    }

    for file in files {
      use std::io::{BufReader, BufRead};

      let filename = Rc::new(file.name);
      let lines = BufReader::new(file.contents).lines();
      let mut lines = lines
        .enumerate()
        .map(|(lineno, line)| (lineno+1, line))
        .peekable();

      let mut state = State::Insert;
      let mut last_block = Block::new(filename.clone(), 1);

      loop {
        let next_anchor = process_block_lines(&mut lines, &mut last_block, &mut errors);

        // Stick the last block where it needs to go.
        if last_block.contents.len() > 0 {
          match state {
            State::Insert => {
              let section = Section::Normal(last_block);
              let section = Rc::new(RefCell::new(section));

              sections.push(section);
            },
            State::After(label) => {
              match lookup.get_mut(&label) {
                None => {
                  errors.push(format!("error: {}: anchor {} doesn't exist", filename, label));
                  let section = Section::Normal(last_block);
                  let section = Rc::new(RefCell::new(section));

                  sections.push(section);
                },
                Some(section) => {
                  let mut section_ref = section.borrow_mut();
                  let section: &mut Section = &mut section_ref;

                  if let &mut Section::Anchored { ref mut after, .. } = section {
                    after.push(last_block);
                  }

                  // No reason to worry if the section is a Normal. We don't
                  // ever put that kind of section in `lookup'.
                }
              };
            },
            State::Before(label) => {
              match lookup.get_mut(&label) {
                None => {
                  errors.push(format!("error: {}: anchor {} doesn't exist", filename, label));
                  let section = Section::Normal(last_block);
                  let section = Rc::new(RefCell::new(section));

                  sections.push(section);
                }
                Some(section) => {
                  let mut section_ref = section.borrow_mut();
                  let section: &mut Section = &mut section_ref;
                  
                  if let &mut Section::Anchored { ref mut before, .. } = section {
                    before.push(last_block);
                  }

                  // No reason to worry if the section is a Normal. We don't
                  // every put that kind of section in `lookup'.
                }
              };
            }
          };
        }

        // Process the next anchor we've found and create sections accordingly.
        match next_anchor {
          None => break,
          Some(anchor) => {
            use parsing::Anchor;

            match anchor {
              Anchor::Insert => state = State::Insert,
              Anchor::Before(label) => state = State::Before(label),
              Anchor::After(label) => state = State::After(label),
              Anchor::Label(label) => {
                let section = Section::Anchored {
                  before: Vec::new(),
                  after: Vec::new()
                };
                let section = Rc::new(RefCell::new(section));

                sections.push(section.clone());
                lookup.insert(label, section.clone());

                state = State::Insert;
              }
            };

            let lineno = match lines.peek() {
              None => 9999999,
              Some(&(lineno, ref _line)) => lineno
            };

            last_block = Block::new(filename.clone(), lineno);
          }
        };
      }
    }
  }

  let sections = unpack_sections(sections);
  let out: Box<Write> = Box::new(stdout());
  print_processed_output(sections, args.flag_comment, out)?;

  for error in errors {
    writeln!(stderr(), "{}", error)
      .unwrap();
  }

  Ok(())
}

/// Each block will end in either an anchor tag, or the end of the file.
/// Since our caller needs to handle the anchor tag, we pass it back.
fn process_block_lines<I>(lines: &mut I, block: &mut Block, errors: &mut Vec<String>) -> Option<parsing::Anchor> where
  I: Iterator<Item=(usize, std::result::Result<String, io::Error>)>
{
  for (lineno, line) in lines {
    match line {
      Err(err) => errors.push(format!("error: {}, line {}: {}", block.from, lineno, err)),
      Ok(line) => {
        let is_normal_line;

        match parsing::might_be_anchor(&line) {
          None => is_normal_line = true,
          Some(anchor) => {
            let text = anchor.as_str();
            let anchor = parsing::parse(text);

            match anchor {
              Ok(anchor) => {
                return Some(anchor);
              },
              Err(_) => {
                errors.push(format!("warn: {}, line {}: ignoring something that looks like an anchor: {}", block.from, lineno, text));
                is_normal_line = true;
              }
            };
          }
        };

        if is_normal_line {
          block.contents.push(line);
        }
      }
    }
  }

  None
}

/// Get rid of all that indirection around our sections.
fn unpack_sections(sections: Vec<Rc<RefCell<Section>>>) -> Vec<Section> {
  let mut result = Vec::new();

  for section in sections {
    // this unwrap should never fail
    let section = Rc::try_unwrap(section)
      .map_err(|_| "invariant failed: section had more than one strong reference")
      .unwrap();
    let section = section.into_inner();

    result.push(section);
  }

  result
}

/// Print all the lines in the order that they should be.
fn print_processed_output(sections: Vec<Section>, comment: Option<String>, mut out: Box<Write>) -> Result<()> {
  macro_rules! try_output {
    ($format:expr, $($arg:expr),*) => {
      writeln!(out, $format, $($arg),*).chain_err(|| "failed to write output")?
    }
  }
  macro_rules! block_header {
    ($block:expr) => {
      if let &Some(ref leader) = &comment {
        writeln!(out, "{} '{}', line {}", leader, $block.from, $block.line)
          .chain_err(|| "failed to write output")?
      }
    }
  }

  // Typically, we only have confusion over where *inserted* blocks are
  // coming from. So we don't bother outputting headers for normal sections.

  for section in sections {
    match section {
      Section::Normal(block) => {
        for line in block.contents {
          try_output!("{}", line);
        }
      },
      Section::Anchored { before, after } => {
        for block in before.into_iter().rev() {
          block_header!(block);
          for line in block.contents {
            try_output!("{}", line);
          }
        }
        for block in after.into_iter() {
          block_header!(block);
          for line in block.contents {
            try_output!("{}", line);
          }
        }
      }
    };
  }

  Ok(())
}
