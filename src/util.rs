#[macro_export]
macro_rules! map {
  ($($key:expr => $value:expr),*) => {
    [
      $(
        ($key.into(), $value)
      ),*
    ].into_iter().collect::<indexmap::IndexMap<_,_,_>>()
  }
}

fn lowercase(s: &str) -> impl Iterator<Item = char> + '_ {
  s.chars().map(|c| c.to_lowercase().next().unwrap_or(c))
}

fn capital(s: &str) -> impl Iterator<Item = char> + '_ {
  s.chars().enumerate().map(|(i, c)| {
    if i == 0 {
      c.to_uppercase().next()
    } else {
      c.to_lowercase().next()
    }
    .unwrap_or(c)
  })
}

pub fn to_camel_case(s: impl Into<String>) -> String {
  let input: String = s.into();
  let mut out = String::with_capacity(input.len());
  let mut split = input.split_whitespace();
  if let Some(first) = split.next() {
    out.extend(lowercase(first));
    for word in split {
      out.extend(capital(word))
    }
    out
  } else {
    input.to_lowercase()
  }
}

#[inline]
pub fn trim_in_place(s: impl Into<String>) -> String {
  let mut s: String = s.into();
  let trimmed = s.trim();
  let start = trimmed.as_ptr();
  let len = trimmed.len();
  unsafe {
    // SAFETY:
    // - `trimmed` is valid utf-8
    // - all memory reads/writes are valid, because we are copying `N` bytes of the string into itself, where `N` <= original length
    let v = s.as_mut_vec();
    std::ptr::copy(start, v.as_mut_ptr(), len);
    v.set_len(len);
  }
  s
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn lowercased() {
    let f = |s: &str| lowercase(s).collect::<String>();
    assert_eq!(&f("test"), "test");
    assert_eq!(&f("Test"), "test");
    assert_eq!(&f("tEsT"), "test");
    assert_eq!(&f("tEST"), "test");
    assert_eq!(&f("Ünicode"), "ünicode");
  }

  #[test]
  fn capitalized() {
    let f = |s: &str| capital(s).collect::<String>();
    assert_eq!(&f("test"), "Test");
    assert_eq!(&f("Test"), "Test");
    assert_eq!(&f("tEsT"), "Test");
    assert_eq!(&f("tEST"), "Test");
    assert_eq!(&f("ünicode"), "Ünicode");
  }

  #[test]
  fn camel_cased() {
    let f = |s: &str| to_camel_case(s);
    assert_eq!(
      &f("the Quick Brown Fox jumps over the Lazy Dog"),
      "theQuickBrownFoxJumpsOverTheLazyDog"
    );
    assert_eq!(
      &f("the quick ünicode Fox jumps over the Lazy Dog"),
      "theQuickÜnicodeFoxJumpsOverTheLazyDog"
    );
  }

  #[test]
  fn trimmed_in_place() {
    let s = trim_in_place("  test  ".to_string());
    assert_eq!(s, "test");

    let s = trim_in_place("  test".to_string());
    assert_eq!(s, "test");

    let s = trim_in_place("test  ".to_string());
    assert_eq!(s, "test");
  }
}
