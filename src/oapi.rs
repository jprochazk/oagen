pub mod error;

use self::error::{Error, ErrorKind, Scope};
use crate::{ast, util};
use indexmap::IndexMap;
use openapiv3 as oapi3;
use std::borrow::Cow;

struct Context<'src> {
  scope: Scope,
  errors: Vec<Error>,
  can_insert: bool,
  types: ast::Types<'src>,
  security: ast::SecuritySchemes<'src>,
  components: Option<&'src oapi3::Components>,
}

impl<'src> Context<'src> {
  pub fn new(components: Option<&'src oapi3::Components>) -> Self {
    Self {
      scope: Scope::default(),
      errors: vec![],
      can_insert: false,
      types: ast::Types::default(),
      security: ast::SecuritySchemes::default(),
      components,
    }
  }
  pub fn error(&mut self, e: ErrorKind) {
    self.errors.push(Error::new(self.scope.clone(), e));
  }

  pub fn scope<S: Into<String>>(&self, name: S) -> error::ScopeGuard {
    self.scope.named(name)
  }

  pub fn scope_opt<S: Into<String>>(
    &self,
    name: Option<S>,
  ) -> error::ScopeGuard {
    self.scope.named_opt(name)
  }
}

fn op_parse_name<'src>(
  ctx: &mut Context<'src>,
  op: &'src oapi3::Operation,
) -> Option<Cow<'src, str>> {
  let _scope = ctx.scope("name");
  op.operation_id.as_ref().map(Cow::from).or_else(|| {
    match op.summary.as_ref() {
      Some(v) => Some(util::to_camel_case(v).into()),
      None => {
        ctx.error(Error::required_field_or("summary", "operation_id"));
        None
      }
    }
  })
}

fn op_parse_desc(op: &oapi3::Operation) -> Option<Cow<'_, str>> {
  op.description
    .as_ref()
    .map(util::trim_in_place)
    .map(Cow::from)
}

fn op_parse_params<'src>(
  ctx: &mut Context<'src>,
  op: &'src oapi3::Operation,
) -> Option<ast::Parameters<'src>> {
  let _scope = ctx.scope("parameters");
  let mut params = ast::Parameters::<'src>::with_capacity(op.parameters.len());
  for param in op.parameters.iter() {
    if let Some(param) = param.as_item() {
      use oapi3::Parameter::*;
      let (kind, data) = match param {
        Query { parameter_data, .. } => {
          (ast::ParameterKind::Query, parameter_data)
        }
        Path { parameter_data, .. } => {
          (ast::ParameterKind::Path, parameter_data)
        }
        Header { parameter_data, .. } => {
          (ast::ParameterKind::Header, parameter_data)
        }
        Cookie { .. } => {
          return None;
        }
      };
      let ty = resolve_type(
        ctx,
        Some(data.name.as_str()),
        match data.format {
          oapi3::ParameterSchemaOrContent::Schema(ref s) => s,
          oapi3::ParameterSchemaOrContent::Content(_) => {
            return None;
          }
        },
      )?;
      // TODO: validate types here
      // - cannot be a TypeRef::Ref
      // - inner can only be String, Number, Boolean, single-level Object, Array + any of those in Optional
      let param = ast::Parameter {
        name: data.name.as_str().into(),
        description: data.description.as_ref().map(|v| v.as_str().into()),
        kind,
        ty: if data.required {
          ty
        } else {
          ast::TypeRef::Type(ast::Type::Optional(Box::new(ty)))
        },
      };
      params.insert(data.name.as_str().into(), param);
    } else {
      return None;
    }
  }
  Some(params)
}

fn op_parse_request<'src>(
  ctx: &mut Context<'src>,
  op: &'src oapi3::Operation,
) -> Option<ast::RequestBody<'src>> {
  let _scope = ctx.scope("requests");
  let body = match op.request_body.as_ref() {
    Some(body) => body,
    None => {
      return None;
    }
  };
  let body = match body.as_item() {
    Some(body) => body,
    None => {
      ctx.error(Error::unsupported_ref("request_body"));

      return None;
    }
  };
  if let Some((mime, inner)) = body.content.first() {
    let mime = match mime.as_str().try_into() {
      Ok(v) => v,
      Err(..) => {
        ctx.error(Error::invalid_value("mime type", mime.to_string()));
        return None;
      }
    };
    inner
      .schema
      .as_ref()
      .and_then(|b| resolve_type(ctx, None, b))
      .map(|ty| ast::RequestBody {
        mime_type: mime,
        headers: vec![],
        ty,
      })
  } else {
    None
  }
}

fn parse_code<'src>(
  ctx: &mut Context<'src>,
  code: &'src oapi3::StatusCode,
) -> Option<ast::Code> {
  match code {
    oapi3::StatusCode::Code(v) => Some((*v).into()),
    oapi3::StatusCode::Range(_) => {
      ctx.error(Error::unsupported("status code range"));
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
      if !res.content.is_empty() {
        ctx.error(Error::generic(
          "only `application/json` mime-type is supported",
        ));
      }
      return None;
    }
  };
  Some(ast::Response {
    body: res
      .schema
      .as_ref()
      .and_then(|schema| resolve_type(ctx, None, schema)),
  })
}

fn op_parse_responses<'src>(
  ctx: &mut Context<'src>,
  op: &'src oapi3::Operation,
) -> ast::Responses<'src> {
  let _scope = ctx.scope("responses");
  let default =
    op.responses
      .default
      .as_ref()
      .and_then(|res| match res.as_item() {
        Some(res) => parse_response(ctx, res),
        None => {
          ctx.error(Error::unsupported_ref("default"));
          None
        }
      });
  let mut specific = Vec::with_capacity(op.responses.responses.len());
  for (code, res) in op.responses.responses.iter() {
    match res.as_item() {
      Some(res) => {
        let code = match parse_code(ctx, code) {
          Some(c) => c,
          None => continue,
        };
        let _scope = ctx.scope(format!("{}", code));
        let res = match parse_response(ctx, res) {
          Some(res) => res,
          None => continue,
        };
        specific.push((code, res));
      }
      None => {
        ctx.error(Error::unsupported_ref("by_code"));
      }
    }
  }
  ast::Responses { default, specific }
}

fn parse_security<'src>(
  ctx: &mut Context<'src>,
  security: &[IndexMap<String, Vec<String>>],
) -> Option<ast::Security<'src>> {
  for supported_schemes in security.iter() {
    for (name, _) in supported_schemes.iter() {
      if let Some(scheme) = ctx.security.get(name.as_str()) {
        return Some(scheme.clone());
      } else {
        ctx.error(Error::generic(format!("unknown security schema {name}")));
      }
    }
  }
  None
}

fn op_parse_security<'src>(
  ctx: &mut Context<'src>,
  op: &'src oapi3::Operation,
) -> Option<ast::Security<'src>> {
  let _scope = ctx.scope("security");
  if let Some(security) = &op.security {
    parse_security(ctx, security)
  } else {
    None
  }
}

fn parse_route<'src>(
  ctx: &mut Context<'src>,
  uri: &'src str,
  method: ast::Method,
  op: &'src oapi3::Operation,
) -> Option<ast::Route<'src>> {
  let _scope = ctx.scope(format!("{} {}", method, uri));
  let name = op_parse_name(ctx, op);
  let endpoint = uri.into();
  let description = op_parse_desc(op);
  let parameters = op_parse_params(ctx, op);
  let request = op_parse_request(ctx, op);
  let responses = op_parse_responses(ctx, op);
  let security = op_parse_security(ctx, op);

  Some(ast::Route {
    name: name?,
    endpoint,
    method,
    description,
    parameters: parameters?,
    request_body: request,
    responses,
    security,
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
  let mut props = IndexMap::with_capacity(obj.properties.len());
  for (key, schema) in obj.properties.iter() {
    let ty = resolve_type_boxed(ctx, None, schema)?;
    props.insert(
      key.as_str().into(),
      if obj.required.contains(key) {
        ty
      } else {
        ast::TypeRef::Type(ast::Type::Optional(Box::new(ty)))
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
  let mut result = IndexMap::new();
  let mut duplicates = vec![];
  for schema in all_of {
    match resolve_type(ctx, None, schema).and_then(|r| r.into_type()) {
      Some(ast::Type::Object(fields)) => {
        for (key, value) in fields.iter() {
          if result.contains_key(key) {
            duplicates.push(key.to_string());
          } else {
            result.insert(key.clone(), value.clone());
          }
        }
      }
      // TODO: some kind of error here
      _ => break,
    }
  }

  if duplicates.is_empty() {
    Some(ast::Type::Object(result))
  } else {
    ctx.error(Error::duplicate_keys(duplicates));
    None
  }
}

fn array_type<'src>(
  ctx: &mut Context<'src>,
  arr: &'src oapi3::ArrayType,
) -> Option<ast::Type<'src>> {
  Some(ast::Type::Array(Box::new(resolve_type_boxed(
    ctx,
    None,
    arr.items.as_ref()?,
  )?)))
}

fn schema_to_type<'src>(
  ctx: &mut Context<'src>,
  name: Option<&'src str>,
  schema: &'src oapi3::Schema,
) -> Option<ast::Type<'src>> {
  let _scope = ctx.scope_opt(name);
  match &schema.schema_kind {
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
      ctx.error(Error::unsupported("any_of"));
      Some(ast::Type::Any)
    }
    oapi3::SchemaKind::Not { .. } => {
      ctx.error(Error::unsupported("not"));
      Some(ast::Type::Any)
    }
    oapi3::SchemaKind::Any(..) => Some(ast::Type::Any),
  }
}

fn resolve_item<'src>(
  ctx: &mut Context<'src>,
  name: Option<&'src str>,
  schema: &'src oapi3::Schema,
) -> Option<ast::TypeRef<'src>> {
  let ty = schema_to_type(ctx, name, schema)?;
  if ctx.can_insert {
    if let Some(name) = name {
      if !ctx.types.contains_key(name) {
        ctx.types.insert(name.into(), ty.clone());
      }
    }
  }
  Some(ast::TypeRef::Type(ty))
}

fn resolve_reference<'src>(
  ctx: &mut Context<'src>,
  name: &'src str,
) -> Option<ast::TypeRef<'src>> {
  let name = name.split('/').last().unwrap_or(name);
  if !ctx.types.contains_key(name) {
    if ctx.can_insert {
      if let Some(components) = ctx.components {
        if let Some(schema) = components.schemas.get(name) {
          return resolve_type(ctx, Some(name), schema);
        }
      }
    }
    ctx.error(Error::unresolved_ref(name.to_string()));
    None
  } else {
    Some(ast::TypeRef::Ref(name.into()))
  }
}

// TODO: detect cycles
fn resolve_type<'src>(
  ctx: &mut Context<'src>,
  name: Option<&'src str>,
  schema: &'src oapi3::ReferenceOr<oapi3::Schema>,
) -> Option<ast::TypeRef<'src>> {
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
) -> Option<ast::TypeRef<'src>> {
  use oapi3::ReferenceOr::*;
  match schema {
    Item(schema) => resolve_item(ctx, name, &**schema),
    Reference { reference } => resolve_reference(ctx, reference.as_str()),
  }
}

fn parse_types(ctx: &mut Context<'_>) {
  if let Some(components) = ctx.components {
    let _scope = ctx.scope("components");
    for (name, schema) in components.schemas.iter() {
      resolve_type(ctx, Some(name.as_str()), schema);
    }
  }
}

fn parse_security_schemes(ctx: &mut Context<'_>) {
  if let Some(components) = ctx.components {
    let _scope = ctx.scope("components");
    for (name, scheme) in components.security_schemes.iter() {
      match scheme {
        oapi3::ReferenceOr::Reference { .. } => {
          ctx.error(Error::unsupported_ref("securitySchemes"));
        }
        oapi3::ReferenceOr::Item(scheme) => match scheme {
          oapi3::SecurityScheme::APIKey {
            location,
            name: key,
            ..
          } => match location {
            oapi3::APIKeyLocation::Header => {
              ctx.security.insert(
                name.clone().into(),
                ast::Security {
                  name: name.clone().into(),
                  key: key.clone().into(),
                },
              );
            }
            oapi3::APIKeyLocation::Query => {
              ctx.error(Error::unsupported("Query API key authentication"));
            }
            oapi3::APIKeyLocation::Cookie => {
              ctx.error(Error::unsupported("Cookie API key authentication"));
            }
          },
          oapi3::SecurityScheme::HTTP { .. } => {
            ctx.error(Error::unsupported("HTTP authentication"));
          }
          oapi3::SecurityScheme::OAuth2 { .. } => {
            ctx.error(Error::unsupported("OAuth2 authentication"));
          }
          oapi3::SecurityScheme::OpenIDConnect { .. } => {
            ctx.error(Error::unsupported("OpenID authentication"));
          }
        },
      }
    }
  }
}

impl ast::AsAst for oapi3::OpenAPI {
  type Error = Error;
  fn as_ast(&self) -> Result<ast::Ast<'_>, (ast::Ast<'_>, Vec<self::Error>)> {
    let mut ctx = Context::new(self.components.as_ref());

    parse_security_schemes(&mut ctx);
    let security = self
      .security
      .as_ref()
      .and_then(|v| parse_security(&mut ctx, v));

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
      schemes: ctx.security,
      security,
    };
    if ctx.errors.is_empty() {
      Ok(ast)
    } else {
      Err((ast, ctx.errors))
    }
  }
}
