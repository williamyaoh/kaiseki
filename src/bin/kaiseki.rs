//! Preprocess and rearrange lines of input.
//!
//! Used for literate programming.

#[macro_use] extern crate error_chain;
extern crate docopt;
extern crate rustc_serialize;
extern crate kaiseki;

mod errors {
  error_chain! {
    links {
      Input(::kaiseki::input::errors::Error, ::kaiseki::input::errors::ErrorKind);
    }
  }
}

use docopt::Docopt;

use std::process;
use std::io::stderr;
use std::io::Write;

use errors::*;
use kaiseki::input;

static VERSION: &'static str = "0.2.1";
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

fn go(args: CLIArgs) -> Result<()> {
  let files = input::open_files(args.arg_files)?;

  let output_options = kaiseki::OutputOptions {
    comment: args.flag_comment
  };

  let (output, errors) = kaiseki::tangle_output(files, output_options);
  
  for line in output {
    println!("{}", line);
  }

  for error in errors {
    writeln!(stderr(), "kaiseki: {}", error)
      .unwrap();
  }

  Ok(())
}
