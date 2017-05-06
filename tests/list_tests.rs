extern crate kaiseki;

use kaiseki::list::List;

static FILE_HEADER: &'static str = include_str!("text/file_header");
static BODY: &'static str = include_str!("text/body");

#[test]
fn test_lines() {
  let mut lines: List<String> = List::new();

  for line in FILE_HEADER.lines() {
    lines.push_back(line.to_string());
  }
  
  for line in BODY.lines() {
    lines.push_back(line.to_string());
  }
  
  for (line1, line2) in lines.into_iter()
    .zip(FILE_HEADER.lines().chain(BODY.lines()))
  {
    assert_eq!(&line1 as &str, line2);
  }
}
