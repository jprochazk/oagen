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
  pub requests: Requests<'src>,
  pub responses: Responses<'src>,
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

pub type Requests<'src> = Vec<(MimeType, Request<'src>)>;

#[derive(Debug, Clone, PartialEq)]
pub struct Request<'src> {
  pub headers: Vec<Cow<'src, str>>,
  pub body: Option<Body<'src>>,
}

pub type Code = usize;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Responses<'src> {
  pub default: Option<Response<'src>>,
  pub specific: Vec<(Code, Response<'src>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Response<'src> {
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

#[derive(Clone, Copy, PartialEq)]
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

impl std::fmt::Debug for Method {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use Method::*;
    write!(
      f,
      "{}",
      match self {
        Get => "get",
        Post => "post",
        Put => "put",
        Delete => "delete",
        Patch => "patch",
        Head => "head",
        Options => "options",
        Trace => "trace",
        Connect => "connect",
      }
    )
  }
}

impl std::fmt::Display for Method {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    std::fmt::Debug::fmt(self, f)
  }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq)]
pub enum MimeType {
  Multipart_FormData,
  Application_FormUrlEncoded,
  Application_Json,
  Text_Plain,
}

impl<'src> TryFrom<&'src str> for MimeType {
  type Error = ();

  fn try_from(value: &'src str) -> Result<Self, Self::Error> {
    use MimeType::*;
    Ok(match value {
      "multipart/form-data" => Multipart_FormData,
      "application/x-www-form-urlencoded" => Application_FormUrlEncoded,
      "application/json" => Application_Json,
      "text/plain" => Text_Plain,
      _ => return Err(()),
    })
  }
}

impl std::fmt::Debug for MimeType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::Multipart_FormData => "multipart/form-data",
        Self::Application_FormUrlEncoded => "application/x-www-form-urlencoded",
        Self::Application_Json => "application/json",
        Self::Text_Plain => "text/plain",
      }
    )
  }
}

impl std::fmt::Display for MimeType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    std::fmt::Debug::fmt(self, f)
  }
}
