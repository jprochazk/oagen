use super::*;

use anyhow::Result;
use pretty_assertions::assert_eq;
use std::collections::HashMap;

use crate::ast::{self, AsAst, Index, Method, Parameter, ParameterKind, Route, Type};

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
