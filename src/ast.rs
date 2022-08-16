use indexmap::IndexMap;
use std::borrow::Cow;

pub trait AsAst {
  type Error;
  fn as_ast(&self) -> Result<Ast<'_>, (Ast<'_>, Vec<Self::Error>)>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ast<'src> {
  pub routes: Routes<'src>,
  pub types: Types<'src>,
  pub schemes: SecuritySchemes<'src>,
  /// Default security scheme
  pub security: Option<Security<'src>>,
}

pub type Routes<'src> = Vec<Route<'src>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Route<'src> {
  pub name: Cow<'src, str>,
  pub endpoint: Cow<'src, str>,
  pub method: Method,
  pub description: Option<Cow<'src, str>>,
  pub parameters: Parameters<'src>,
  pub request_body: Option<RequestBody<'src>>,
  pub responses: Responses<'src>,
  pub security: Option<Security<'src>>,
}

pub type Parameters<'src> = IndexMap<Cow<'src, str>, Parameter<'src>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter<'src> {
  pub name: Cow<'src, str>,
  pub description: Option<Cow<'src, str>>,
  pub kind: ParameterKind,
  pub ty: TypeRef<'src>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterKind {
  Path,
  Query,
  Header,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RequestBody<'src> {
  pub mime_type: MimeType,
  pub headers: Vec<Cow<'src, str>>,
  pub ty: TypeRef<'src>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Code(u64);

impl Code {
  pub fn is_ok(&self) -> bool {
    (200..299).contains(&self.0)
  }
}

macro_rules! impl_code_from {
  ($n:ident) => {
    impl From<$n> for Code {
      fn from(v: $n) -> Self {
        Self(v as u64)
      }
    }
  };
}
impl_code_from!(u64);
impl_code_from!(u32);
impl_code_from!(u16);
impl_code_from!(u8);

impl std::fmt::Display for Code {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    std::fmt::Debug::fmt(self, f)
  }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Responses<'src> {
  pub default: Option<Response<'src>>,
  pub specific: Vec<(Code, Response<'src>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Response<'src> {
  pub body: Option<TypeRef<'src>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Security<'src> {
  pub name: Cow<'src, str>,
  pub key: Cow<'src, str>,
}

pub type SecuritySchemes<'src> = IndexMap<Cow<'src, str>, Security<'src>>;

pub type Types<'src> = IndexMap<Cow<'src, str>, Type<'src>>;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeRef<'src> {
  Type(Type<'src>),
  Ref(Cow<'src, str>),
}

impl<'src> TypeRef<'src> {
  pub fn into_type(self) -> Option<Type<'src>> {
    match self {
      TypeRef::Type(ty) => Some(ty),
      TypeRef::Ref(_) => None,
    }
  }

  pub fn into_name(self) -> Option<Cow<'src, str>> {
    match self {
      TypeRef::Type(_) => None,
      TypeRef::Ref(name) => Some(name),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type<'src> {
  Any,
  Number,
  String,
  Boolean,
  Enum(Vec<Cow<'src, str>>),
  Array(Box<TypeRef<'src>>),
  Object(IndexMap<Cow<'src, str>, TypeRef<'src>>),
  Union(Vec<TypeRef<'src>>),
  Optional(Box<TypeRef<'src>>),
}

#[derive(Clone, Copy, PartialEq, Eq)]
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

impl Method {
  pub fn as_str(&self) -> &'static str {
    use Method::*;
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
  }
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
    write!(f, "{}", self.as_str())
  }
}

impl std::fmt::Display for Method {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    std::fmt::Debug::fmt(self, f)
  }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq)]
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
