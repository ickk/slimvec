// Copyright © ickk, 2026

use {
  crate::SlimVec,
  ::valuable::{Listable, Valuable, Value, Visit},
};

impl<T> Valuable for SlimVec<T>
where
  T: Valuable,
{
  fn as_value(&self) -> Value<'_> {
    Value::Listable(self as &dyn Listable)
  }

  fn visit(&self, visit: &mut dyn Visit) {
    T::visit_slice(self, visit);
  }
}

impl<T> Listable for SlimVec<T>
where
  T: Valuable,
{
  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    (self.len(), Some(self.len()))
  }
}
