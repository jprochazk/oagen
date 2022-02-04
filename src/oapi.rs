use crate::{ast, util};
use openapiv3 as oapi3;
use std::{borrow::Cow, collections::HashMap};
use thiserror::Error;

// TODO: error scopes instead of `at`
#[derive(Debug, Clone, Error)]
pub enum Error {
  #[error("Error in {at}: References may not appear in `{field}`")]
  UnsupportedReference {
    at: Cow<'static, str>,
    field: Cow<'static, str>,
  },
  #[error("Could not resolve reference {name}")]
  UnresolvedReference { name: Cow<'static, str> },
  #[error("Error in {at}: Field `{field}` is required")]
  RequiredField {
    at: Cow<'static, str>,
    field: Cow<'static, str>,
  },
  #[error("Error in {at}: Field `{field_a}` is required, but may be substituted with `{field_b}`")]
  RequiredFieldOr {
    at: Cow<'static, str>,
    field_a: Cow<'static, str>,
    field_b: Cow<'static, str>,
  },
  #[error("Error in {at}: Field `{field}` has unknown value `{value}`")]
  InvalidValue {
    at: Cow<'static, str>,
    field: Cow<'static, str>,
    value: String,
  },
  #[error("{0} is unsupported")]
  Unsupported(Cow<'static, str>),
  #[error("{0}")]
  Generic(Cow<'static, str>),
}

// TODO: parse components - schemas, security
// TODO: parse parameters - path, query - resolve references
// TODO: parse request + responses -

#[derive(Default)]
struct Context {
  errors: Vec<Error>,
}

impl Context {
  pub fn error(&mut self, e: Error) {
    self.errors.push(e);
  }
}

fn parse_name<'src>(
  ctx: &mut Context,
  uri: &str,
  method: ast::Method,
  op: &'src oapi3::Operation,
) -> Option<Cow<'src, str>> {
  match op.summary.as_ref().or_else(|| op.operation_id.as_ref()) {
    Some(v) => Some(util::to_camel_case(v).into()),
    None => {
      ctx.error(Error::RequiredFieldOr {
        at: format!("{} {}", method, uri).into(),
        field_a: "summary".into(),
        field_b: "operation_id".into(),
      });
      None
    }
  }
}

fn parse_desc(op: &oapi3::Operation) -> Option<Cow<'_, str>> {
  op.description
    .as_ref()
    .map(util::trim_in_place)
    .map(Cow::from)
}

fn parse_params<'src>(
  ctx: &mut Context,
  op: &'src oapi3::Operation,
) -> Option<HashMap<Cow<'src, str>, ast::Parameter<'src>>> {
  Some(HashMap::new())
}

fn parse_route<'src>(
  ctx: &mut Context,
  uri: &'src str,
  method: ast::Method,
  op: &'src oapi3::Operation,
) -> Option<ast::Route<'src>> {
  let name = parse_name(ctx, uri, method, op);
  let endpoint = uri.into();
  let description = parse_desc(op);
  let parameters = parse_params(ctx, op);
  let request = None;
  let response = vec![];

  Some(ast::Route {
    name: name?,
    endpoint,
    method,
    description,
    parameters: parameters?,
    request,
    response,
  })
}

fn one_of_to_union<'src>(
  ctx: &mut Context,
  types: &ast::Types<'src>,
  one_of: &'src [oapi3::ReferenceOr<oapi3::Schema>],
) -> Option<ast::Type<'src>> {
  let mut parts = Vec::with_capacity(one_of.len());
  for schema in one_of.iter() {
    if let Some(t) = resolve_type(ctx, types, schema) {
      parts.push(t);
    } else {
      return None;
    }
  }
  Some(ast::Type::Union(parts))
}

fn object_type<'src>(
  ctx: &mut Context,
  types: &ast::Types<'src>,
  obj: &'src oapi3::ObjectType,
) -> Option<ast::Type<'src>> {
  let mut props = HashMap::with_capacity(obj.properties.len());
  for (key, schema) in obj.properties.iter() {
    let ty = resolve_type_boxed(ctx, types, schema)?;
    props.insert(
      key.as_str().into(),
      if obj.required.contains(key) {
        ty
      } else {
        ast::Type::Optional(box ty)
      },
    );
  }
  Some(if !props.is_empty() {
    ast::Type::Object(props)
  } else {
    ast::Type::Any
  })
}

fn array_type<'src>(
  ctx: &mut Context,
  types: &ast::Types<'src>,
  arr: &'src oapi3::ArrayType,
) -> Option<ast::Type<'src>> {
  Some(ast::Type::Array(box resolve_type_boxed(
    ctx,
    types,
    arr.items.as_ref()?,
  )?))
}

fn schema_to_type<'src>(
  ctx: &mut Context,
  types: &ast::Types<'src>,
  schema: &'src oapi3::Schema,
) -> Option<ast::Type<'src>> {
  let name = schema.schema_data.title.as_deref().unwrap_or("[Unknown]");
  match &schema.schema_kind {
    oapi3::SchemaKind::Type(ty) => Some(match ty {
      oapi3::Type::String(_) => ast::Type::String,
      oapi3::Type::Number(_) => ast::Type::Number,
      oapi3::Type::Integer(_) => ast::Type::Number,
      oapi3::Type::Boolean {} => ast::Type::Boolean,
      oapi3::Type::Object(obj) => object_type(ctx, types, obj)?,
      oapi3::Type::Array(arr) => array_type(ctx, types, arr)?,
    }),
    oapi3::SchemaKind::OneOf { one_of } => one_of_to_union(ctx, types, &one_of[..]),
    oapi3::SchemaKind::AllOf { .. } => {
      ctx.error(Error::Unsupported("all_of".into()));
      Some(ast::Type::Any)
    }
    oapi3::SchemaKind::AnyOf { .. } => {
      ctx.error(Error::Unsupported("any_of".into()));
      Some(ast::Type::Any)
    }
    oapi3::SchemaKind::Not { .. } => {
      ctx.error(Error::Unsupported("not".into()));
      Some(ast::Type::Any)
    }
    oapi3::SchemaKind::Any(..) => Some(ast::Type::Any),
  }
}

fn get_type<'src>(
  ctx: &mut Context,
  types: &ast::Types<'src>,
  name: &str,
) -> Option<ast::Type<'src>> {
  match types.get(name).cloned() {
    Some(v) => Some(v),
    None => {
      ctx.error(Error::UnresolvedReference {
        name: name.to_string().into(),
      });
      None
    }
  }
}

// TODO: detect cycles
fn resolve_type<'src>(
  ctx: &mut Context,
  types: &ast::Types<'src>,
  schema: &'src oapi3::ReferenceOr<oapi3::Schema>,
) -> Option<ast::Type<'src>> {
  use oapi3::ReferenceOr::*;
  match schema {
    Item(schema) => schema_to_type(ctx, types, schema),
    Reference { reference } => get_type(ctx, types, reference.as_str()),
  }
}

fn resolve_type_boxed<'src>(
  ctx: &mut Context,
  types: &ast::Types<'src>,
  schema: &'src oapi3::ReferenceOr<Box<oapi3::Schema>>,
) -> Option<ast::Type<'src>> {
  use oapi3::ReferenceOr::*;
  match schema {
    Item(box schema) => schema_to_type(ctx, types, schema),
    Reference { reference } => get_type(ctx, types, reference.as_str()),
  }
}

fn parse_types<'src>(
  ctx: &mut Context,
  components: Option<&'src oapi3::Components>,
) -> HashMap<Cow<'src, str>, ast::Type<'src>> {
  let no_types = HashMap::new();
  if let Some(components) = components {
    components
      .schemas
      .iter()
      .filter_map(|(name, item)| Some((name.as_str().into(), resolve_type(ctx, &no_types, item)?)))
      .collect()
  } else {
    no_types
  }
}

impl ast::AsAst for oapi3::OpenAPI {
  type Error = Error;
  fn as_ast(&self) -> Result<ast::Ast<'_>, (ast::Ast<'_>, Vec<self::Error>)> {
    let mut ctx = Context::default();
    let types = parse_types(&mut ctx, self.components.as_ref());
    let mut routes = vec![];
    for (uri, info) in self
      .paths
      .iter()
      .filter_map(|(uri, item)| Some((uri, item.as_item()?)))
    {
      routes.extend(info.iter().filter_map(|(m, op)| {
        parse_route(&mut ctx, uri, m.try_into().expect("Invalid method"), op)
      }));
    }

    if ctx.errors.is_empty() {
      Ok(ast::Ast { routes, types })
    } else {
      Err((ast::Ast { routes, types }, ctx.errors))
    }
  }
}

#[cfg(test)]
mod tests;
