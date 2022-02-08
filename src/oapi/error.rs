use std::{borrow::Cow, cell::RefCell, rc::Rc};
use thiserror::Error;

#[derive(Clone, Default)]
pub struct Scope {
  inner: Rc<RefCell<Vec<Option<String>>>>,
}

impl Scope {
  pub fn named<S>(&self, name: S) -> ScopeGuard
  where
    S: Into<String>,
  {
    self.inner.borrow_mut().push(Some(name.into()));
    ScopeGuard(self.clone())
  }

  pub fn named_opt<S>(&self, name: Option<S>) -> ScopeGuard
  where
    S: Into<String>,
  {
    self.inner.borrow_mut().push(name.map(Into::into));
    ScopeGuard(self.clone())
  }
}

impl std::fmt::Debug for Scope {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let inner = self.inner.borrow();
    if !inner.is_empty() {
      write!(
        f,
        "{}",
        inner
          .iter()
          .filter_map(|v| v.as_deref())
          .collect::<Vec<_>>()
          .join(".")
      )
    } else {
      write!(f, "(unscoped)")
    }
  }
}

pub struct ScopeGuard(Scope);

impl Drop for ScopeGuard {
  fn drop(&mut self) {
    self.0.inner.borrow_mut().pop();
  }
}

#[derive(Debug, Clone, Error)]
#[error("Error in {scope:?}: {kind}")]
pub struct Error {
  scope: Scope,
  kind: ErrorKind,
}

impl Error {
  pub fn new(scope: Scope, kind: ErrorKind) -> Self {
    Self { scope, kind }
  }
  pub fn unsupported_ref<A: Into<Cow<'static, str>>>(field: A) -> ErrorKind {
    ErrorKind::UnsupportedReference(field.into())
  }

  pub fn unresolved_ref<A: Into<Cow<'static, str>>>(name: A) -> ErrorKind {
    ErrorKind::UnresolvedReference(name.into())
  }

  pub fn required_field<A: Into<Cow<'static, str>>>(field: A) -> ErrorKind {
    ErrorKind::RequiredField(field.into())
  }

  pub fn required_field_or<
    A: Into<Cow<'static, str>>,
    B: Into<Cow<'static, str>>,
  >(
    a: A,
    b: B,
  ) -> ErrorKind {
    ErrorKind::RequiredFieldOr(a.into(), b.into())
  }

  pub fn invalid_value<
    A: Into<Cow<'static, str>>,
    B: Into<Cow<'static, str>>,
  >(
    field: A,
    value: B,
  ) -> ErrorKind {
    ErrorKind::InvalidValue(field.into(), value.into())
  }

  pub fn duplicate_keys<A: Into<Cow<'static, str>>>(keys: Vec<A>) -> ErrorKind {
    ErrorKind::DuplicateKeys(keys.into_iter().map(|v| v.into()).collect())
  }

  pub fn unsupported<A: Into<Cow<'static, str>>>(what: A) -> ErrorKind {
    ErrorKind::Unsupported(what.into())
  }

  pub fn generic<A: Into<Cow<'static, str>>>(error: A) -> ErrorKind {
    ErrorKind::Generic(error.into())
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
  #[error("field `{0}` has invalid value `{1}`")]
  InvalidValue(Cow<'static, str>, Cow<'static, str>),
  #[error("found duplicate keys: `{0:?}`")]
  DuplicateKeys(Vec<Cow<'static, str>>),
  #[error("{0} is unsupported")]
  Unsupported(Cow<'static, str>),
  #[error("{0}")]
  Generic(Cow<'static, str>),
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn scoping() {
    let scope = Scope::default();
    {
      let _scope = scope.named("a");
      {
        let _scope = scope.named("b");
        assert_eq!(
          format!("{}", Error::new(scope.clone(), Error::generic("test"))),
          "Error in a.b: test"
        );
      }
      assert_eq!(
        format!("{}", Error::new(scope.clone(), Error::generic("test"))),
        "Error in a: test"
      );
    }
    assert_eq!(
      format!("{}", Error::new(scope, Error::generic("test"))),
      "Error in (unscoped): test"
    );
  }
}
