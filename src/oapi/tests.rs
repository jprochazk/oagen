use crate::ast::AsAst;

macro_rules! serde_oapi {
  ($p:literal) => {
    serde_json::from_str::<openapiv3::OpenAPI>(include_str!($p))
  };
}

macro_rules! try_parse_api {
  ($test_name:ident, $name:literal) => {
    #[test]
    fn $test_name() {
      match serde_oapi!($name).unwrap().as_ast() {
        Ok(v) => {
          println!("{v:#?}");
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

try_parse_api!(job_queue, "./job-queue.json");
try_parse_api!(sandboxes, "./sandboxes.json");
try_parse_api!(sapi_importer, "./sapi-importer.json");
try_parse_api!(scheduler, "./scheduler.json");
try_parse_api!(sync_actions, "./sync-actions.json");
