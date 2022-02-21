#![allow(clippy::upper_case_acronyms)]

use {
  gen::{ast::AsAst, emit::emit},
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
  name = "ts_api_gen",
  about = "Generate TS API clients from Swagger/Apiary specs"
)]
enum CLI {
  OAPI {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
    #[structopt(parse(from_os_str))]
    output: PathBuf,
  },
  APIB {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
    #[structopt(parse(from_os_str))]
    output: PathBuf,
  },
}

fn main() {
  match CLI::from_args() {
    CLI::OAPI { input, output } => {
      if input.extension() != Some(OsStr::new("json")) {
        panic!("Swagger spec must be provided as .json");
      }
      match serde_json::from_reader::<BufReader<File>, OpenAPI>(BufReader::new(
        File::open(input).unwrap(),
      ))
      .unwrap()
      .as_ast()
      {
        Ok(ast) => {
          fs::write(output, emit(ast)).unwrap();
        }
        Err(e) => {
          for e in e.1 {
            eprintln!("{e}");
          }
          panic!("Encountered some errors");
        }
      }
    }
    CLI::APIB { input, output } => panic!(".apib is unsupported"),
  }
}
