use std::borrow::Cow;
use thiserror::Error;

#[derive(Debug, Clone, Default)]
pub struct Scope(pub Vec<Option<String>>);

impl Scope {
  pub fn push<S>(&mut self, name: S)
  where
    S: Into<String>,
  {
    self.0.push(Some(name.into()));
  }

  pub fn push_opt<S>(&mut self, name: Option<S>)
  where
    S: Into<String>,
  {
    self.0.push(name.map(Into::into))
  }

  pub fn pop(&mut self) {
    self.0.pop();
  }
}

impl std::fmt::Display for Scope {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      self
        .0
        .iter()
        .filter_map(|v| v.clone())
        .collect::<Vec<_>>()
        .join(".")
    )
  }
}

#[derive(Debug, Clone, Error)]
#[error("Error in {scope}: {kind}")]
pub struct Error {
  scope: Scope,
  kind: ErrorKind,
}

impl Error {
  pub fn unsupported_ref<A: Into<Cow<'static, str>>>(scope: Scope, field: A) -> Self {
    use ErrorKind::*;
    Self {
      scope,
      kind: UnsupportedReference(field.into()),
    }
  }

  pub fn unresolved_ref<A: Into<Cow<'static, str>>>(scope: Scope, name: A) -> Self {
    use ErrorKind::*;
    Self {
      scope,
      kind: UnresolvedReference(name.into()),
    }
  }

  pub fn required_field<A: Into<Cow<'static, str>>>(scope: Scope, field: A) -> Self {
    use ErrorKind::*;
    Self {
      scope,
      kind: RequiredField(field.into()),
    }
  }

  pub fn required_field_or<A: Into<Cow<'static, str>>, B: Into<Cow<'static, str>>>(
    scope: Scope,
    a: A,
    b: B,
  ) -> Self {
    use ErrorKind::*;
    Self {
      scope,
      kind: RequiredFieldOr(a.into(), b.into()),
    }
  }

  pub fn invalid_value<A: Into<Cow<'static, str>>, B: Into<Cow<'static, str>>>(
    scope: Scope,
    field: A,
    value: B,
  ) -> Self {
    use ErrorKind::*;
    Self {
      scope,
      kind: InvalidValue(field.into(), value.into()),
    }
  }

  pub fn unsupported<A: Into<Cow<'static, str>>>(scope: Scope, what: A) -> Self {
    use ErrorKind::*;
    Self {
      scope,
      kind: Unsupported(what.into()),
    }
  }

  pub fn generic<A: Into<Cow<'static, str>>>(scope: Scope, error: A) -> Self {
    use ErrorKind::*;
    Self {
      scope,
      kind: Generic(error.into()),
    }
  }
}

#[derive(Debug, Clone, Error)]
pub enum ErrorKind {
  #[error("references may not appear in `{0}`")]
  UnsupportedReference(Cow<'static, str>),
  #[error("could not resolve reference to `{0}`")]
  UnresolvedReference(Cow<'static, str>),
  #[error("field `{0}` is required")]
  RequiredField(Cow<'static, str>),
  #[error("field `{0}` is required, but may be substituted with `{1}`")]
  RequiredFieldOr(Cow<'static, str>, Cow<'static, str>),
  #[error("field `{0}` has unknown value `{1}`")]
  InvalidValue(Cow<'static, str>, Cow<'static, str>),
  #[error("{0} is unsupported")]
  Unsupported(Cow<'static, str>),
  #[error("{0}")]
  Generic(Cow<'static, str>),
}
