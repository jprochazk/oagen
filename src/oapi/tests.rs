use anyhow::Result;
use openapiv3::OpenAPI;
use pretty_assertions::assert_eq;
use std::collections::HashMap;

use crate::ast::{self, AsAst, Method, Route};

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

#[test]
fn parse_scheduler_api() -> Result<()> {
  match serde_oapi!("./odinuv-scheduler-1.0.0-swagger.json")?.as_ast() {
    Ok(v) => println!("{:#?}", v),
    Err((v, e)) => {
      println!("{:#?}", v);
      eprintln!("{:#?}", e);
      panic!("Errors were not empty");
    }
  };
  Ok(())
}

#[test]
fn parse_simple_api() -> Result<()> {
  match serde_oapi!("./odinuv-sync-actions-1.0.0-swagger.json")?.as_ast() {
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
            parameters: HashMap::new(),
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
            parameters: HashMap::new(),
            request: None,
            response: vec![],
          },
        ],
      );

      use ast::Type::*;
      assert!(map_match(
        &ast.types,
        &[
          ("Success".into(), Any),
          (
            "Error".into(),
            Object(
              [
                ("error".into(), String),
                ("context".into(), Any),
                ("status".into(), String),
                ("exceptionId".into(), Optional(box String)),
                ("code".into(), Number),
              ]
              .into_iter()
              .collect(),
            )
          ),
        ]
        .into_iter()
        .collect()
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
