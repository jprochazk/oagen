pub mod error;

use self::error::{Error, Scope};
use crate::{ast, util};
use openapiv3 as oapi3;
use std::{borrow::Cow, collections::HashMap};

// TODO: parse security

struct Context<'src> {
  scope: Scope,
  errors: Vec<Error>,
  can_insert: bool,
  types: ast::Types<'src>,
  components: Option<&'src oapi3::Components>,
}

impl<'src> Context<'src> {
  pub fn new(components: Option<&'src oapi3::Components>) -> Self {
    Self {
      scope: Scope::default(),
      errors: vec![],
      can_insert: false,
      types: ast::Types::default(),
      components,
    }
  }
  pub fn error(&mut self, e: Error) {
    self.errors.push(e);
  }

  pub fn scope(&self) -> Scope {
    self.scope.clone()
  }
}

fn parse_name<'src>(
  ctx: &mut Context<'src>,
  op: &'src oapi3::Operation,
) -> Option<Cow<'src, str>> {
  ctx.scope.push("name");
  let name = match op.summary.as_ref().or_else(|| op.operation_id.as_ref()) {
    Some(v) => Some(util::to_camel_case(v).into()),
    None => {
      ctx.error(Error::required_field_or(
        ctx.scope(),
        "summary",
        "operation_id",
      ));
      None
    }
  };
  ctx.scope.pop();
  name
}

fn parse_desc(op: &oapi3::Operation) -> Option<Cow<'_, str>> {
  op.description
    .as_ref()
    .map(util::trim_in_place)
    .map(Cow::from)
}

fn parse_params<'src>(
  ctx: &mut Context<'src>,
  op: &'src oapi3::Operation,
) -> Option<ast::Parameters<'src>> {
  ctx.scope.push("parameters");
  let mut params = ast::Parameters::<'src>::with_capacity(op.parameters.len());
  for param in op.parameters.iter() {
    if let Some(param) = param.as_item() {
      use oapi3::Parameter::*;
      let (kind, data) = match param {
        Query { parameter_data, .. } => {
          (ast::ParameterKind::Query(ast::Index::Array), parameter_data)
        }
        Path { parameter_data, .. } => {
          (ast::ParameterKind::Path, parameter_data)
        }
        Header { .. } | Cookie { .. } => {
          ctx.scope.pop();
          return None;
        }
      };
      let ty = match resolve_type(
        ctx,
        Some(data.name.as_str()),
        match data.format {
          oapi3::ParameterSchemaOrContent::Schema(ref s) => s,
          oapi3::ParameterSchemaOrContent::Content(_) => todo!(),
        },
      ) {
        Some(ty) => ty,
        None => {
          ctx.scope.pop();
          return None;
        }
      };
      let param = ast::Parameter {
        name: data.name.as_str().into(),
        description: data.description.as_ref().map(|v| v.as_str().into()),
        kind,
        ty: if data.required {
          ty
        } else {
          ast::Type::Optional(box ty)
        },
      };
      params.insert(data.name.as_str().into(), param);
    } else {
      ctx.scope.pop();
      return None;
    }
  }
  ctx.scope.pop();
  Some(params)
}

fn parse_request<'src>(
  ctx: &mut Context<'src>,
  op: &'src oapi3::Operation,
) -> Option<ast::Request<'src>> {
  ctx.scope.push("request");
  let req = match op.request_body.as_ref() {
    Some(v) => v,
    None => {
      ctx.scope.pop();
      return None;
    }
  };
  let req = match req.as_item() {
    Some(b) => b,
    None => {
      ctx.error(Error::unsupported_ref(ctx.scope(), "request_body"));
      ctx.scope.pop();
      return None;
    }
  };
  let req = match req.content.get("application/json") {
    Some(b) => b,
    None => {
      ctx.error(Error::generic(
        ctx.scope(),
        "only `application/json` mime-type is supported",
      ));
      ctx.scope.pop();
      return None;
    }
  };
  let req = req
    .schema
    .as_ref()
    .and_then(|b| resolve_type(ctx, None, b))
    .map(|ty| ast::Request {
      mime_type: Some("application/json".into()),
      headers: vec![],
      body: Some(ast::Body::Typed(ty)),
    });
  ctx.scope.pop();
  req
}

fn parse_code<'src>(
  ctx: &mut Context<'src>,
  code: &'src oapi3::StatusCode,
) -> Option<ast::Code> {
  match code {
    oapi3::StatusCode::Code(v) => Some(*v as usize),
    oapi3::StatusCode::Range(_) => {
      ctx.error(Error::unsupported(ctx.scope(), "status code range"));
      None
    }
  }
}

fn parse_response<'src>(
  ctx: &mut Context<'src>,
  res: &'src oapi3::Response,
) -> Option<ast::Response<'src>> {
  let res = match res.content.get("application/json") {
    Some(res) => res,
    None => {
      ctx.error(Error::generic(
        ctx.scope(),
        "only `application/json` mime-type is supported",
      ));
      return None;
    }
  };
  let res = ast::Response {
    mime_type: Some("application/json".into()),
    body: res
      .schema
      .as_ref()
      .and_then(|schema| resolve_type(ctx, None, schema))
      .map(ast::Body::Typed),
  };
  Some(res)
}

fn parse_responses<'src>(
  ctx: &mut Context<'src>,
  op: &'src oapi3::Operation,
) -> ast::Responses<'src> {
  let default =
    match op.responses.default.as_ref().and_then(|res| res.as_item()) {
      Some(res) => parse_response(ctx, res),
      None => {
        ctx.error(Error::unsupported_ref(ctx.scope(), "responses.default"));
        None
      }
    };
  let mut specific = Vec::with_capacity(op.responses.responses.len());
  for (code, res) in op.responses.responses.iter() {
    match res.as_item() {
      Some(res) => {
        let code = match parse_code(ctx, code) {
          Some(c) => c,
          None => continue,
        };
        let res = match parse_response(ctx, res) {
          Some(res) => res,
          None => continue,
        };
        specific.push((code, res));
      }
      None => {
        ctx.error(Error::unsupported_ref(ctx.scope(), "responses"));
      }
    }
  }
  ast::Responses { default, specific }
}

fn parse_route<'src>(
  ctx: &mut Context<'src>,
  uri: &'src str,
  method: ast::Method,
  op: &'src oapi3::Operation,
) -> Option<ast::Route<'src>> {
  ctx.scope.push(format!("{} {}", method, uri));
  let name = parse_name(ctx, op);
  let endpoint = uri.into();
  let description = parse_desc(op);
  let parameters = parse_params(ctx, op);
  let request = parse_request(ctx, op);
  let responses = parse_responses(ctx, op);
  ctx.scope.pop();
  Some(ast::Route {
    name: name?,
    endpoint,
    method,
    description,
    parameters: parameters?,
    request,
    responses,
  })
}

fn one_of<'src>(
  ctx: &mut Context<'src>,
  one_of: &'src [oapi3::ReferenceOr<oapi3::Schema>],
) -> Option<ast::Type<'src>> {
  let mut parts = Vec::with_capacity(one_of.len());
  for schema in one_of.iter() {
    if let Some(t) = resolve_type(ctx, None, schema) {
      parts.push(t);
    } else {
      return None;
    }
  }
  Some(ast::Type::Union(parts))
}

fn string_type<'src>(
  _ctx: &mut Context<'src>,
  str: &'src oapi3::StringType,
) -> ast::Type<'src> {
  let variants = str
    .enumeration
    .iter()
    .cloned()
    .filter_map(|v| v.map(Cow::from))
    .collect::<Vec<_>>();
  if variants.is_empty() {
    ast::Type::String
  } else {
    ast::Type::Enum(variants)
  }
}

fn object_type<'src>(
  ctx: &mut Context<'src>,
  obj: &'src oapi3::ObjectType,
) -> Option<ast::Type<'src>> {
  let mut props = HashMap::with_capacity(obj.properties.len());
  for (key, schema) in obj.properties.iter() {
    let ty = resolve_type_boxed(ctx, None, schema)?;
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

fn all_of<'src>(
  ctx: &mut Context<'src>,
  all_of: &'src [oapi3::ReferenceOr<oapi3::Schema>],
) -> Option<ast::Type<'src>> {
  let mut result = HashMap::new();
  let mut duplicates = vec![];
  for schema in all_of {
    match resolve_type(ctx, None, schema) {
      Some(ast::Type::Object(fields)) => {
        for (key, value) in fields.iter() {
          if result.contains_key(key) {
            duplicates.push(key.to_string());
          } else {
            result.insert(key.clone(), value.clone());
          }
        }
      }
      _ => break,
    }
  }

  if duplicates.is_empty() {
    Some(ast::Type::Object(result))
  } else {
    ctx.error(Error::duplicate_keys(ctx.scope(), duplicates));
    None
  }
}

fn array_type<'src>(
  ctx: &mut Context<'src>,
  arr: &'src oapi3::ArrayType,
) -> Option<ast::Type<'src>> {
  Some(ast::Type::Array(box resolve_type_boxed(
    ctx,
    None,
    arr.items.as_ref()?,
  )?))
}

fn schema_to_type<'src>(
  ctx: &mut Context<'src>,
  name: Option<&'src str>,
  schema: &'src oapi3::Schema,
) -> Option<ast::Type<'src>> {
  ctx.scope.push_opt(name);
  let ty = match &schema.schema_kind {
    oapi3::SchemaKind::Type(ty) => match ty {
      oapi3::Type::String(str) => Some(string_type(ctx, str)),
      oapi3::Type::Number(_) => Some(ast::Type::Number),
      oapi3::Type::Integer(_) => Some(ast::Type::Number),
      oapi3::Type::Boolean {} => Some(ast::Type::Boolean),
      oapi3::Type::Object(obj) => object_type(ctx, obj),
      oapi3::Type::Array(arr) => array_type(ctx, arr),
    },
    oapi3::SchemaKind::OneOf { one_of: values } => one_of(ctx, &values[..]),
    oapi3::SchemaKind::AllOf { all_of: values } => all_of(ctx, &values[..]),
    oapi3::SchemaKind::AnyOf { .. } => {
      ctx.error(Error::unsupported(ctx.scope(), "any_of"));
      Some(ast::Type::Any)
    }
    oapi3::SchemaKind::Not { .. } => {
      ctx.error(Error::unsupported(ctx.scope(), "not"));
      Some(ast::Type::Any)
    }
    oapi3::SchemaKind::Any(..) => Some(ast::Type::Any),
  };
  ctx.scope.pop();
  ty
}

fn insert_unique<'src>(
  types: &mut ast::Types<'src>,
  name: Option<&'src str>,
  ty: Option<&ast::Type<'src>>,
) -> Option<()> {
  let name = name?;
  let ty = ty?;
  if !types.contains_key(name) {
    types.insert(name.into(), ty.clone());
  }
  Some(())
}

fn resolve_item<'src>(
  ctx: &mut Context<'src>,
  name: Option<&'src str>,
  schema: &'src oapi3::Schema,
) -> Option<ast::Type<'src>> {
  let ty = schema_to_type(ctx, name, schema);
  if ctx.can_insert {
    insert_unique(&mut ctx.types, name, ty.as_ref());
  }
  ty
}

fn resolve_reference<'src>(
  ctx: &mut Context<'src>,
  name: &'src str,
) -> Option<ast::Type<'src>> {
  let name = name.split('/').last().unwrap_or(name);
  match ctx.types.get(name).cloned() {
    Some(v) => Some(v),
    None => {
      if ctx.can_insert {
        if let Some(components) = ctx.components {
          if let Some(schema) = components.schemas.get(name) {
            return resolve_type(ctx, Some(name), schema);
          }
        }
      }

      ctx.error(Error::unresolved_ref(ctx.scope(), name.to_string()));
      None
    }
  }
}

// TODO: detect cycles
fn resolve_type<'src>(
  ctx: &mut Context<'src>,
  name: Option<&'src str>,
  schema: &'src oapi3::ReferenceOr<oapi3::Schema>,
) -> Option<ast::Type<'src>> {
  use oapi3::ReferenceOr::*;
  match schema {
    Item(schema) => resolve_item(ctx, name, schema),
    Reference { reference } => resolve_reference(ctx, reference.as_str()),
  }
}

fn resolve_type_boxed<'src>(
  ctx: &mut Context<'src>,
  name: Option<&'src str>,
  schema: &'src oapi3::ReferenceOr<Box<oapi3::Schema>>,
) -> Option<ast::Type<'src>> {
  use oapi3::ReferenceOr::*;
  match schema {
    Item(box schema) => resolve_item(ctx, name, schema),
    Reference { reference } => resolve_reference(ctx, reference.as_str()),
  }
}

fn parse_types(ctx: &mut Context<'_>) {
  if let Some(components) = ctx.components {
    ctx.scope.push("components");
    for (name, schema) in components.schemas.iter() {
      resolve_type(ctx, Some(name.as_str()), schema);
    }
    ctx.scope.pop();
  }
}

impl ast::AsAst for oapi3::OpenAPI {
  type Error = Error;
  fn as_ast(&self) -> Result<ast::Ast<'_>, (ast::Ast<'_>, Vec<self::Error>)> {
    let mut ctx = Context::new(self.components.as_ref());
    ctx.can_insert = true;
    parse_types(&mut ctx);
    ctx.can_insert = false;

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

    let ast = ast::Ast {
      routes,
      types: ctx.types,
    };
    if ctx.errors.is_empty() {
      Ok(ast)
    } else {
      Err((ast, ctx.errors))
    }
  }
}

#[cfg(test)]
mod tests;
