// Copyright © ickk, 2026

use {
  crate::SlimVec,
  ::arbitrary::{Arbitrary, Result, Unstructured},
};

impl<'a, T> Arbitrary<'a> for SlimVec<T>
where
  T: Arbitrary<'a>,
{
  fn arbitrary(u: &mut Unstructured<'a>) -> Result<SlimVec<T>> {
    u.arbitrary_iter()?.collect()
  }

  fn arbitrary_take_rest(u: Unstructured<'a>) -> Result<SlimVec<T>> {
    u.arbitrary_take_rest_iter()?.collect()
  }

  #[inline]
  fn size_hint(_: usize) -> (usize, Option<usize>) {
    (0, None)
  }
}
