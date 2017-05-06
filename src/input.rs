//! For opening the files passed as arguments on the command line.

use std::io::Read;

pub mod errors {
  error_chain! {
    errors {
      CouldNotOpenFile(filename: String) {
        description("could not open file")
        display("could not open file '{}'", filename)
      }
    }
  }
}

use self::errors::*;

pub struct File {
  pub name: String,
  pub contents: Box<Read>
}

/// Attempt to open all the files passed in on the command line.
/// If no files were passed, open `stdin`.
pub fn open_files(mut files: Vec<String>) -> Result<Vec<File>> {
  use std::convert::From;

  let mut output = Vec::new();

  if files.is_empty() {
    files.push(From::from("-"));
  }

  for file in files {
    let file = open_file(file)?;
    output.push(file);
  }

  Ok(output)
}

/// The "file"'s name might be '-', in which case it refers to
/// `stdin()`.
fn open_file(file: String) -> Result<File> {
  use std::io;
  use std::fs;
  use std::path;
  use std::convert::From;

  Ok(
    if &file == "-" {
      File {
        name: From::from("<stdin>"),
        contents: Box::new(io::stdin())
      }
    } else if { let path: &path::Path = file.as_ref(); path.is_dir() } {
      File {
        name: file,
        contents: Box::new(io::empty())
      }
    } else {
      let contents = fs::File::open(&file);

      match contents {
        Ok(contents) => File {
          name: file,
          contents: Box::new(contents)
        },
        Err(err) => return {
          let err = Err(err);
          err.chain_err(|| ErrorKind::CouldNotOpenFile(file))
        }
      }
    }
  )
}
