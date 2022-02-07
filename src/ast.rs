use std::borrow::Cow;
use std::collections::HashMap;

pub trait AsAst {
  type Error;
  fn as_ast(&self) -> Result<Ast<'_>, (Ast<'_>, Vec<Self::Error>)>;
}

pub type Types<'src> = HashMap<Cow<'src, str>, Type<'src>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Ast<'src> {
  pub routes: Vec<Route<'src>>,
  pub types: Types<'src>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Route<'src> {
  pub name: Cow<'src, str>,
  pub endpoint: Cow<'src, str>,
  pub method: Method,
  pub description: Option<Cow<'src, str>>,
  pub parameters: Parameters<'src>,
  pub request: Option<Request<'src>>,
  pub response: Responses<'src>,
}

pub type Parameters<'src> = HashMap<Cow<'src, str>, Parameter<'src>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter<'src> {
  pub name: Cow<'src, str>,
  pub description: Option<Cow<'src, str>>,
  pub kind: ParameterKind,
  pub ty: Type<'src>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParameterKind {
  Path,
  Query(Index),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Index {
  Array,
  Key,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Request<'src> {
  pub mime_type: Option<Cow<'src, str>>,
  pub headers: Vec<Cow<'src, str>>,
}

pub type Code = usize;

pub type Responses<'src> = Vec<(Code, Response<'src>)>;

#[derive(Debug, Clone, PartialEq)]
pub struct Response<'src> {
  pub code: Code,
  pub mime_type: Option<Cow<'src, str>>,
  pub body: Option<Body<'src>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Body<'src> {
  Untyped(Cow<'src, str>),
  Typed(Type<'src>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type<'src> {
  Any,
  Number,
  String,
  Boolean,
  Enum(Vec<Cow<'src, str>>),
  Array(Box<Type<'src>>),
  Object(HashMap<Cow<'src, str>, Type<'src>>),
  Union(Vec<Type<'src>>),
  Optional(Box<Type<'src>>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Method {
  Get,
  Post,
  Put,
  Delete,
  Patch,
  Head,
  Options,
  Trace,
  Connect,
}

impl<'src> TryFrom<&'src str> for Method {
  type Error = ();
  fn try_from(value: &'src str) -> Result<Self, Self::Error> {
    use Method::*;
    Ok(match value {
      "get" => Get,
      "post" => Post,
      "put" => Put,
      "delete" => Delete,
      "patch" => Patch,
      "head" => Head,
      "options" => Options,
      "trace" => Trace,
      "connect" => Connect,
      _ => return Err(()),
    })
  }
}

impl TryFrom<String> for Method {
  type Error = ();
  fn try_from(value: String) -> Result<Self, Self::Error> {
    value.as_str().try_into()
  }
}

impl std::fmt::Display for Method {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    std::fmt::Debug::fmt(self, f)
  }
}
