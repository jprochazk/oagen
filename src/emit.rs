use std::borrow::Cow;

use crate::{ast, util::trim_in_place};

pub enum Token<'src> {
  /// |
  Or,
  /// :
  Colon,
  /// =
  Equals,
  /// ?
  Question,
  /// ,
  Comma,
  /// .
  Dot,
  /// ;
  Semicolon,
  /// {
  LeftBrace,
  /// }
  RightBrace,
  /// [
  LeftBracket,
  /// ]
  RightBracket,
  /// (
  LeftParen,
  /// )
  RightParen,
  /// <
  LowerThan,
  /// >
  GreaterThan,

  /// identifier
  Identifier(Cow<'src, str>),
  /// "string literal"
  String(Cow<'src, str>),
  /// /** doc comment */
  Doc(Cow<'src, str>),
}

impl<'src> std::fmt::Display for Token<'src> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Token::Or => write!(f, "|"),
      Token::Colon => write!(f, ":"),
      Token::Equals => write!(f, "="),
      Token::Question => write!(f, "?"),
      Token::Comma => write!(f, ","),
      Token::Dot => write!(f, "."),
      Token::Semicolon => write!(f, ";"),
      Token::LeftBrace => write!(f, "{{"),
      Token::RightBrace => write!(f, "}}"),
      Token::LeftBracket => write!(f, "["),
      Token::RightBracket => write!(f, "]"),
      Token::LeftParen => write!(f, "("),
      Token::RightParen => write!(f, ")"),
      Token::LowerThan => write!(f, "<"),
      Token::GreaterThan => write!(f, ">"),
      Token::Identifier(i) => write!(f, "{i}"),
      Token::String(s) => write!(f, "'{s}'"),
      Token::Doc(d) => write!(f, "/** {d} */"),
    }
  }
}

pub struct Buffer<'src> {
  tokens: Vec<Token<'src>>,
}

impl<'src> From<Buffer<'src>> for String {
  fn from(buffer: Buffer<'src>) -> Self {
    use std::fmt::Write;
    let mut out = String::new();
    for token in buffer.tokens {
      write!(out, "{token} ").unwrap();
    }
    out
  }
}

impl<'src> Default for Buffer<'src> {
  fn default() -> Self {
    Self::new()
  }
}

impl<'src> Buffer<'src> {
  pub fn new() -> Self {
    Self {
      tokens: Vec::with_capacity(1024),
    }
  }

  pub fn extend(&mut self, iter: impl IntoIterator<Item = Token<'src>>) {
    self.tokens.extend(iter);
  }

  pub fn or(&mut self) {
    self.tokens.push(Token::Or);
  }

  pub fn colon(&mut self) {
    self.tokens.push(Token::Colon);
  }

  pub fn equals(&mut self) {
    self.tokens.push(Token::Equals);
  }

  pub fn question(&mut self) {
    self.tokens.push(Token::Question);
  }

  pub fn comma(&mut self) {
    self.tokens.push(Token::Comma);
  }

  pub fn dot(&mut self) {
    self.tokens.push(Token::Dot);
  }

  pub fn semicolon(&mut self) {
    self.tokens.push(Token::Semicolon);
  }

  /// { ... }
  pub fn braces(&mut self, f: impl FnOnce(&mut Self)) {
    self.tokens.push(Token::LeftBrace);
    f(self);
    self.tokens.push(Token::RightBrace);
  }
  /// {}
  pub fn braces0(&mut self) {
    self.tokens.push(Token::LeftBrace);
    self.tokens.push(Token::RightBrace);
  }

  /// [ ... ]
  pub fn brackets(&mut self, f: impl FnOnce(&mut Self)) {
    self.tokens.push(Token::LeftBracket);
    f(self);
    self.tokens.push(Token::RightBracket);
  }
  /// []
  pub fn brackets0(&mut self) {
    self.tokens.push(Token::LeftBracket);
    self.tokens.push(Token::RightBracket);
  }

  /// ( ... )
  pub fn parens(&mut self, f: impl FnOnce(&mut Self)) {
    self.tokens.push(Token::LeftParen);
    f(self);
    self.tokens.push(Token::RightParen);
  }
  /// ()
  pub fn parens0(&mut self) {
    self.tokens.push(Token::LeftParen);
    self.tokens.push(Token::RightParen);
  }

  /// < ... >
  pub fn generics(&mut self, f: impl FnOnce(&mut Self)) {
    self.tokens.push(Token::LowerThan);
    f(self);
    self.tokens.push(Token::GreaterThan);
  }

  /// <
  pub fn lower_than(&mut self) {
    self.tokens.push(Token::LowerThan);
  }

  /// >
  pub fn greater_than(&mut self) {
    self.tokens.push(Token::GreaterThan);
  }

  pub fn identifier(&mut self, v: impl Into<Cow<'src, str>>) {
    self.tokens.push(Token::Identifier(v.into()));
  }

  pub fn string(&mut self, v: impl Into<Cow<'src, str>>) {
    self.tokens.push(Token::String(v.into()));
  }

  pub fn doc(&mut self, v: impl Into<Cow<'src, str>>) {
    self.tokens.push(Token::Doc(v.into()));
  }
}

pub fn emit<'src>(input: impl Emit<'src>) -> String {
  let mut buffer = Buffer::new();
  input.emit(&mut buffer);
  trim_in_place(buffer)
}

pub trait Emit<'src> {
  fn emit(self, buffer: &mut Buffer<'src>);
}

impl<'src> Emit<'src> for ast::Ast<'src> {
  fn emit(self, buffer: &mut Buffer<'src>) {
    //self.schemes.emit(buffer);
    self.types.emit(buffer);
    //self.routes.emit(buffer);
  }
}

impl<'src> Emit<'src> for ast::Types<'src> {
  fn emit(self, buffer: &mut Buffer<'src>) {
    for (name, ty) in self.into_iter() {
      (name, ty).emit(buffer);
    }
  }
}

impl<'src> Emit<'src> for (Cow<'src, str>, ast::Type<'src>) {
  fn emit(self, buffer: &mut Buffer<'src>) {
    let (name, ty) = self;
    buffer.identifier("type");
    buffer.identifier(name);
    buffer.equals();
    ty.emit(buffer);
    buffer.semicolon();
  }
}

impl<'src> Emit<'src> for ast::TypeRef<'src> {
  fn emit(self, buffer: &mut Buffer<'src>) {
    match self {
      ast::TypeRef::Type(ty) => ty.emit(buffer),
      ast::TypeRef::Ref(name) => buffer.identifier(name),
    }
  }
}

impl<'src> Emit<'src> for ast::Type<'src> {
  fn emit(self, buffer: &mut Buffer<'src>) {
    match self {
      // any
      ast::Type::Any => {
        buffer.identifier("any");
      }
      // number
      ast::Type::Number => {
        buffer.identifier("number");
      }
      // string
      ast::Type::String => {
        buffer.identifier("string");
      }
      // boolean
      ast::Type::Boolean => {
        buffer.identifier("boolean");
      }
      // ("a" | "b" | "c" | ...)
      ast::Type::Enum(v) => buffer.parens(|buffer| {
        buffer.extend(
          v.into_iter()
            .map(Token::String)
            .intersperse_with(|| Token::Or),
        )
      }),
      // (T)[]
      ast::Type::Array(box ty) => {
        buffer.parens(|buffer| ty.emit(buffer));
        buffer.brackets0();
      }
      // ({ a: T, b: T, ... })
      ast::Type::Object(o) => buffer.parens(|buffer| {
        buffer.braces(|buffer| {
          for (key, ty) in o {
            buffer.string(key);
            if matches!(&ty, ast::TypeRef::Type(ast::Type::Optional(..))) {
              buffer.question();
            }
            buffer.colon();
            ty.emit(buffer);
            buffer.comma();
          }
        })
      }),
      // (A | B)
      ast::Type::Union(v) => buffer.parens(|buffer| {
        let mut iter = v.into_iter();
        if let Some(first) = iter.next() {
          first.emit(buffer);
        }
        for ty in iter {
          buffer.or();
          ty.emit(buffer);
        }
      }),
      // (T | undefined)
      ast::Type::Optional(box ty) => buffer.parens(|buffer| {
        ty.emit(buffer);
        buffer.or();
        buffer.identifier("undefined");
      }),
    }
  }
}

impl<'src> Emit<'src> for ast::SecuritySchemes<'src> {
  fn emit(self, buffer: &mut Buffer<'src>) {
    /*
    let _baseUrl;
    let _authHeaders;
    export function init(baseUrl: string, scheme0: string, scheme1: string, ...) {
      _baseUrl = baseUrl;
      _authHeaders = { ["key0"]: scheme0, ["key1"]: scheme1, ... }
    }
    */
    buffer.identifier("let");
    buffer.identifier("_baseUrl");
    buffer.semicolon();

    buffer.identifier("let");
    buffer.identifier("_authHeaders");
    buffer.semicolon();

    buffer.identifier("export");
    buffer.identifier("function");
    buffer.identifier("init");
    buffer.parens(|buffer| {
      buffer.identifier("baseUrl");
      buffer.colon();
      buffer.identifier("string");
      buffer.comma();

      for scheme in self.values() {
        buffer.identifier(scheme.name.clone());
        buffer.colon();
        buffer.identifier("string");
        buffer.comma()
      }
    });
    buffer.braces(|buffer| {
      buffer.identifier("_baseUrl");
      buffer.equals();
      buffer.identifier("baseUrl");
      buffer.semicolon();

      buffer.identifier("_authHeaders");
      buffer.equals();
      buffer.braces(|buffer| {
        for scheme in self.values() {
          buffer.brackets(|buffer| buffer.string(scheme.key.clone()));
          buffer.colon();
          buffer.identifier(scheme.name.clone());
          buffer.comma();
        }
      });
      buffer.semicolon();
    });
  }
}

impl<'src> Emit<'src> for ast::Route<'src> {
  fn emit(self, buffer: &mut Buffer<'src>) {
    // TODO: emit multipart/form-data
    // TODO: a way to emit response types predicated on status code
    /*
    /**
     * #description
     */
    export async function #name (
      #$each(param) '(#name : string),
      pathParam0 : string,
      #(body? '(body : #requestType , ))
    ) : Promise < unknown > {

      return await fetch ( url , {
        method : #method,
        headers : {
          ... _authHeaders ,
          #(body.json? '("Content-Type" : "application/json"))
        },
        #(body? '(JSON . stringify ( body )))
      } )
    }
    */
    if let Some(desc) = self.description {
      buffer.doc(desc.clone());
    }
    buffer.identifier("export");
    buffer.identifier("async");
    buffer.identifier("function");
    buffer.identifier(self.name.clone());
    buffer.parens(|buffer| {
      // TODO
    });
    buffer.colon();
    buffer.identifier("Promise");
    buffer.generics(|buffer| buffer.identifier("unknown"));
    buffer.braces(|buffer| {
      // TODO
    });
  }
}

// TODO: emit application/x-www-form-urlencoded in URL
// NOTE: assumes that `#name` is defined
impl<'src> Emit<'src> for (Cow<'src, str>, &'src ast::Parameters<'src>) {
  fn emit(self, buffer: &mut Buffer<'src>) {
    let (endpoint, params) = self;
    /*
    const url = new URL ( _baseUrl ) ;
    url . pathname = endpoint
      #$each(path param) '(. replace ( "{#name}" , #name ))
      ;
    */
    buffer.identifier("const");
    buffer.identifier("url");
    buffer.equals();
    buffer.identifier("new");
    buffer.identifier("URL");
    buffer.parens(|buffer| buffer.identifier("_baseUrl"));
    buffer.semicolon();

    buffer.identifier("url");
    buffer.dot();
    buffer.identifier("pathname");
    buffer.equals();
    buffer.string(endpoint.clone());
    for param in params.values() {
      buffer.dot();
      buffer.identifier("replace");
      buffer.parens(|buffer| {
        let name = param.name.clone();
        buffer.string(format!("{{{name}}}"));
        buffer.comma();
        buffer.identifier(name);
      })
    }
    buffer.semicolon();
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;

  macro_rules! type_emit_test {
    ($name:ident, $ty:expr, $expected:literal) => {
      #[test]
      fn $name() {
        let mut buffer = crate::emit::Buffer::new();
        let ty = crate::ast::TypeRef::Type($ty);
        ty.emit(&mut buffer);
        assert_eq!(String::from(buffer).trim(), $expected);
      }
    };
  }

  macro_rules! ty {
    ($inner:expr) => {
      crate::ast::TypeRef::Type($inner)
    };
  }

  macro_rules! name {
    ($name:literal) => {
      crate::ast::TypeRef::Ref($name.into())
    };
  }

  use ast::Type;

  type_emit_test!(any_type, Type::Any, "any");
  type_emit_test!(number_type, Type::Number, "number");
  type_emit_test!(string_type, Type::String, "string");
  type_emit_test!(boolean_type, Type::Boolean, "boolean");
  type_emit_test!(
    enum_abc_type,
    Type::Enum(vec!["a".into(), "b".into(), "c".into()]),
    "( 'a' | 'b' | 'c' )"
  );
  type_emit_test!(
    array_simple_type,
    Type::Array(box ty!(Type::Any)),
    "( any ) [ ]"
  );
  type_emit_test!(
    array_nested_type,
    Type::Array(box ty!(Type::Array(box ty!(Type::Any)))),
    "( ( any ) [ ] ) [ ]"
  );
  type_emit_test!(
    object_type,
    Type::Object(map! {
      "a" => ty!(Type::Any),
      "b" => ty!(Type::String),
      "c" => ty!(Type::Number),
      "d" => name!("Test")
    }),
    "( { 'a' : any , 'b' : string , 'c' : number , 'd' : Test , } )"
  );
  type_emit_test!(
    array_object_type,
    Type::Array(box ty!(Type::Object(map! {
      "a" => ty!(Type::Any),
      "b" => ty!(Type::String),
      "c" => ty!(Type::Number),
      "d" => name!("Test")
    }))),
    "( ( { 'a' : any , 'b' : string , 'c' : number , 'd' : Test , } ) ) [ ]"
  );
  type_emit_test!(
    union_type,
    Type::Union(vec![
      ty!(Type::Number),
      ty!(Type::String),
      ty!(Type::Boolean),
      name!("Test")
    ]),
    "( number | string | boolean | Test )"
  );
  type_emit_test!(
    optional_type,
    Type::Optional(box ty!(Type::String)),
    "( string | undefined )"
  );
  type_emit_test!(
    optional_ref,
    Type::Optional(box name!("Test")),
    "( Test | undefined )"
  );

  #[test]
  fn emit_type_decl() {
    let mut buffer = Buffer::new();
    ("Test".into(), ast::Type::Any).emit(&mut buffer);
    assert_eq!(String::from(buffer).trim(), "type Test = any ;");
  }

  #[test]
  fn emit_security_schemes_init() {
    let mut buffer = Buffer::new();
    macro_rules! scheme {
      ($name:literal, $key: literal) => {
        crate::ast::Security {
          name: $name.into(),
          key: $key.into(),
        }
      };
    }
    map! {
      "name0" => scheme!("name0", "header-key-0"),
      "name1" => scheme!("name1", "header-key-1")
    }
    .emit(&mut buffer);
    assert_eq!(
      String::from(buffer).trim(),
      [
        "let _baseUrl ;",
        "let _authHeaders ;",
        "export function init ( baseUrl : string , name0 : string , name1 : string , ) {",
        "_baseUrl = baseUrl ;",
        "_authHeaders = {",
        "[ 'header-key-0' ] : name0 ,",
        "[ 'header-key-1' ] : name1 ,",
        "} ;",
        "}"
      ]
      .join(" ")
    );
  }

  #[test]
  fn emit_url() {
    let mut buffer = Buffer::new();
    let params = map! {
      "a" => ast::Parameter {
        name: "a".into(),
        description: None,
        kind: ast::ParameterKind::Path,
        ty: ast::TypeRef::Ref(Default::default()),
      },
      "b" => ast::Parameter {
        name: "b".into(),
        description: None,
        kind: ast::ParameterKind::Path,
        ty: ast::TypeRef::Ref(Default::default()),
      }
    };
    ("/endpoint/{a}/test/{b}".into(), &params).emit(&mut buffer);
    assert_eq!(
      String::from(buffer).trim(),
      /*
      const url = new URL ( _baseUrl ) ;
      url . pathname = endpoint
        #$each(path param) '(. replace ( "{#name}" , #name ))
        ;
      */
      [
        "const url = new URL ( _baseUrl ) ;",
        "url . pathname = '/endpoint/{a}/test/{b}'",
        ". replace ( '{a}' , a )",
        ". replace ( '{b}' , b )",
        ";"
      ]
      .join(" ")
    );
  }
}
