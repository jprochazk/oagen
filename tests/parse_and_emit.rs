macro_rules! try_parse_and_emit {
  ($test_name:ident, $name:literal) => {
    #[test]
    fn $test_name() {
      use gen::ast::AsAst;
      let oapi =
        serde_json::from_str::<openapiv3::OpenAPI>(include_str!($name))
          .unwrap();
      match oapi.as_ast() {
        Ok(v) => {
          println!("{}", gen::emit::emit(v));
        }
        Err((v, e)) => {
          println!("{v:#?}");
          eprintln!("{e:#?}");
          panic!("Errors were not empty");
        }
      }
    }
  };
}

try_parse_and_emit!(job_queue, "./data/job-queue.json");
try_parse_and_emit!(sandboxes, "./data/sandboxes.json");
