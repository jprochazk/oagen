#![allow(clippy::upper_case_acronyms)]

use {
  oagen::{ast::AsAst, emit::emit},
  oapi3::OpenAPI,
  openapiv3 as oapi3,
  std::{
    ffi::OsStr,
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
  },
  structopt::StructOpt,
};

#[derive(Debug, StructOpt)]
#[structopt(
  name = "oagen",
  about = "Generate API client code from OpenAPI specifications"
)]
struct Options {
  #[structopt(parse(from_os_str))]
  input: PathBuf,
  #[structopt(parse(from_os_str))]
  output: PathBuf,
}

fn main() {
  let Options { input, output } = Options::from_args();

  if input.extension() != Some(OsStr::new("json")) {
    panic!("OpenAPI spec must be provided as .json");
  }
  let file = File::open(input).expect("Failed to open input file");
  let json = serde_json::from_reader::<_, OpenAPI>(BufReader::new(file))
    .expect("Failed to read input file");
  let ast = match json.as_ast() {
    Ok(ast) => ast,
    Err((_, errors)) => {
      for error in errors {
        eprintln!("{error}");
      }
      return;
    }
  };
  fs::write(output, emit(ast)).expect("Failed to write to output file");
}
