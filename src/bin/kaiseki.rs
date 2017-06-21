//! Preprocess and rearrange lines of input.
//!
//! Used for literate programming.

#[macro_use] extern crate error_chain;
extern crate structopt;
#[macro_use] extern crate structopt_derive;
extern crate kaiseki;

mod errors {
  error_chain! {
    errors {
      Processing {
        description("encountered errors while tangling output")
        display("encountered errors while tangling output")
      }
    }
    links {
      Input(::kaiseki::input::errors::Error, ::kaiseki::input::errors::ErrorKind);
    }
  }
}

use structopt::StructOpt;

use std::process;
use std::io::stderr;
use std::io::Write;

use errors::*;
use kaiseki::input;

#[derive(StructOpt, Debug)]
#[structopt(name = "kaiseki", about = "literate programming preprocessor")]
struct CLIArgs {
  #[structopt(help = "Files to tangle")]
  files: Vec<String>,

  #[structopt(short = "c", long = "comment", help = "Show where source lines came from with comments")]
  comment_leader: Option<String>,

  #[structopt(short = "i", long = "ignore-errors", help = "Exit normally, ignore errors")]
  ignore_errors: bool
}

fn main() {
  let cli_args = CLIArgs::from_args();

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
  let files = input::open_files(args.files)?;

  let output_options = kaiseki::OutputOptions {
    comment: args.comment_leader
  };

  let (output, errors) = kaiseki::tangle_output(files, output_options);
  
  for line in output {
    println!("{}", line);
  }

  if !args.ignore_errors && !errors.is_empty() {
    for error in errors {
      writeln!(stderr(), "kaiseki: {}", error)
        .unwrap();
    }
    Err(ErrorKind::Processing.into())
  } else {
    Ok(())
  }
}
