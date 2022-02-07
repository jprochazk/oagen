use anyhow::Result;
use pretty_assertions::assert_eq;
use std::collections::HashMap;

use crate::ast::{self, AsAst, Index, Method, Parameter, ParameterKind, Route};

macro_rules! serde_oapi {
  ($p:literal) => {
    serde_json::from_str::<openapiv3::OpenAPI>(include_str!($p))
  };
}

fn map_match<K, V, S>(a: &HashMap<K, V, S>, b: &HashMap<K, V, S>) -> bool
where
  K: std::cmp::Eq,
  K: std::hash::Hash,
  V: std::cmp::PartialEq<V>,
  S: std::hash::BuildHasher,
{
  if a.len() != b.len() {
    return false;
  }
  for (k, v_a) in a {
    match b.get(k) {
      Some(v_b) => {
        if v_a != v_b {
          return false;
        }
      }
      None => return false,
    }
  }
  true
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

#[test]
fn parse_and_compare() -> Result<()> {
  match serde_oapi!("./sync-actions.json")?.as_ast() {
    Ok(ast) => {
      println!("{:#?}", ast);
      assert_eq!(
        ast.routes,
        vec![
          Route {
            name: "listAvailableActions".into(),
            endpoint: "/actions".into(),
            method: Method::Get,
            description: Some("Lists defined actions of a given component.".into()),
            parameters: map! {
              "componentId" => Parameter {
                name: "componentId".into(),
                description: Some(
                  "List actions for a given component.".into(),
                ),
                kind: ParameterKind::Query(
                  Index::Array,
                ),
                ty: String,
              }
            },
            request: None,
            response: vec![],
          },
          Route {
            name: "processAction".into(),
            endpoint: "/actions".into(),
            method: Method::Post,
            description: Some(
              "Runs the specified synchronous actions of the specified component.".into()
            ),
            parameters: map! {},
            request: None,
            response: vec![],
          },
        ],
      );

      use ast::Type::*;
      assert!(map_match(
        &ast.types,
        &map! {
          "Success" => Any,
          "Error" => Object(map! {
            "error" => String,
            "context" => Any,
            "status" => String,
            "exceptionId" => Optional(box String),
            "code" => Number
          })
        }
      ));
    }
    Err((v, e)) => {
      println!("{:#?}", v);
      eprintln!("{:#?}", e);
      panic!("Errors were not empty");
    }
  };

  Ok(())
}
