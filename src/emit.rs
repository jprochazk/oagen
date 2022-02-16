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

  /// identifier
  Identifier(Cow<'src, str>),
  /// "string literal"
  String(Cow<'src, str>),
}

impl<'src> std::fmt::Display for Token<'src> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Token::Or => write!(f, "|"),
      Token::Colon => write!(f, ":"),
      Token::Equals => write!(f, "="),
      Token::Question => write!(f, "?"),
      Token::Comma => write!(f, ","),
      Token::Semicolon => write!(f, ";"),
      Token::LeftBrace => write!(f, "{{"),
      Token::RightBrace => write!(f, "}}"),
      Token::LeftBracket => write!(f, "["),
      Token::RightBracket => write!(f, "]"),
      Token::LeftParen => write!(f, "("),
      Token::RightParen => write!(f, ")"),
      Token::Identifier(i) => write!(f, "{i}"),
      Token::String(s) => write!(f, "'{s}'"),
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

  pub fn semicolon(&mut self) {
    self.tokens.push(Token::Semicolon);
  }

  pub fn braces(&mut self, f: impl FnOnce(&mut Self)) {
    self.tokens.push(Token::LeftBrace);
    f(self);
    self.tokens.push(Token::RightBrace);
  }
  pub fn braces0(&mut self) {
    self.tokens.push(Token::LeftBrace);
    self.tokens.push(Token::RightBrace);
  }

  pub fn brackets(&mut self, f: impl FnOnce(&mut Self)) {
    self.tokens.push(Token::LeftBracket);
    f(self);
    self.tokens.push(Token::RightBracket);
  }
  pub fn brackets0(&mut self) {
    self.tokens.push(Token::LeftBracket);
    self.tokens.push(Token::RightBracket);
  }

  pub fn parens(&mut self, f: impl FnOnce(&mut Self)) {
    self.tokens.push(Token::LeftParen);
    f(self);
    self.tokens.push(Token::RightParen);
  }
  pub fn parens0(&mut self) {
    self.tokens.push(Token::LeftParen);
    self.tokens.push(Token::RightParen);
  }

  pub fn identifier(&mut self, v: impl Into<Cow<'src, str>>) {
    self.tokens.push(Token::Identifier(v.into()));
  }

  pub fn string(&mut self, v: impl Into<Cow<'src, str>>) {
    self.tokens.push(Token::String(v.into()));
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
            if matches!(&ty, ast::Type::Optional(..)) {
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
    todo!()
  }
}

impl<'src> Emit<'src> for ast::Routes<'src> {
  fn emit(self, buffer: &mut Buffer<'src>) {
    todo!()
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
        let ty = $ty;
        ty.emit(&mut buffer);
        assert_eq!(String::from(buffer).trim(), $expected);
      }
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
  type_emit_test!(array_simple_type, Type::Array(box Type::Any), "( any ) [ ]");
  type_emit_test!(
    array_nested_type,
    Type::Array(box Type::Array(box Type::Any)),
    "( ( any ) [ ] ) [ ]"
  );
  type_emit_test!(
    object_type,
    Type::Object(map! {
      "a" => Type::Any,
      "b" => Type::String,
      "c" => Type::Number
    }),
    "( { 'a' : any , 'b' : string , 'c' : number , } )"
  );
  type_emit_test!(
    array_object_type,
    Type::Array(box Type::Object(map! {
      "a" => Type::Any,
      "b" => Type::String,
      "c" => Type::Number
    })),
    "( ( { 'a' : any , 'b' : string , 'c' : number , } ) ) [ ]"
  );
  type_emit_test!(
    union_type,
    Type::Union(vec![Type::Number, Type::String, Type::Boolean]),
    "( number | string | boolean )"
  );
  type_emit_test!(
    optional_type,
    Type::Optional(box Type::String),
    "( string | undefined )"
  );

  #[test]
  fn emit_type_decl() {
    let mut buffer = Buffer::new();
    ("Test".into(), ast::Type::Any).emit(&mut buffer);
    assert_eq!(String::from(buffer).trim(), "type Test = any ;");
  }
}
