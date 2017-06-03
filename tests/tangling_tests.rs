extern crate kaiseki;

use kaiseki::input;

#[test]
fn test_test1() {
  static OUTPUT: &'static str = include_str!("tangling/test1/output");

  let files = ["tests/tangling/test1/000-file1", "tests/tangling/test1/001-file2"];
  let files: Vec<String> = files.iter().map(|str| str.to_string()).collect();
  let files = input::open_files(files).unwrap();

  let output_options = kaiseki::OutputOptions {
    comment: None
  };

  let (output, errors) = kaiseki::tangle_output(files, output_options);

  assert_eq!(errors.len(), 0);
  for (line1, line2) in OUTPUT.lines().zip(output) {
    assert_eq!(line1, &line2 as &str);
  }
}

#[test]
fn test_test2() {
  static OUTPUT: &'static str = include_str!("tangling/test2/output");

  let files = [
    "000-file1",
    "001-file2",
    "002-file3"
  ];

  let files: Vec<String> = files.iter().map(|str| {
    let mut filepath = String::new();
    filepath.push_str("tests/tangling/test2/");
    filepath.push_str(str);
    filepath
  })
  .collect();

  let files = input::open_files(files).unwrap();

  let output_options = kaiseki::OutputOptions {
    comment: None
  };

  let (output, errors) = kaiseki::tangle_output(files, output_options);

  assert_eq!(errors.len(), 0);
  for (line1, line2) in OUTPUT.lines().zip(output) {
    assert_eq!(line1, &line2 as &str);
  }
}
