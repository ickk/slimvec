// Copyright © ickk, 2026

use {
  crate::SlimVec,
  ::comparable::{Changed, Comparable, VecChange},
  ::core::iter,
};

impl<T: PartialEq + Comparable> Comparable for SlimVec<T> {
  type Desc = SlimVec<T::Desc>;
  type Change = SlimVec<VecChange<T::Desc, T::Change>>;

  fn describe(&self) -> SlimVec<T::Desc> {
    self.iter().map(|x| x.describe()).collect()
  }

  fn comparison(
    &self,
    other: &Self,
  ) -> Changed<SlimVec<VecChange<T::Desc, T::Change>>> {
    let mut iter_a = self.iter().enumerate();
    let mut iter_b = other.iter().enumerate();
    let mut changes = SlimVec::new();

    // Note: `zip(..).take(count)` is required because, while `iter::zip`
    // terminates when either iterator ends, it still unfortunately eats one of
    // the trailing values. :(
    let count = usize::min(iter_a.len(), iter_b.len());
    for ((i, a), (_, b)) in
      iter::zip(iter_a.by_ref(), iter_b.by_ref()).take(count)
    {
      if let Changed::Changed(diff) = a.comparison(b) {
        changes.push(VecChange::Changed(i, diff));
      }
    }

    for (i, a) in iter_a {
      changes.push(VecChange::Removed(i, a.describe()));
    }

    for (i, b) in iter_b {
      changes.push(VecChange::Added(i, b.describe()));
    }

    if changes.is_empty() {
      Changed::Unchanged
    } else {
      Changed::Changed(changes)
    }
  }
}
