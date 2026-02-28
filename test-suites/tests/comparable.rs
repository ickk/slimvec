#![cfg(feature = "comparable")]

//! This is an adaptation of the test suite for `Comparable`.
//! repo: https://github.com/jwiegley/comparable.git
//! commit: 49ee58a232a35133a47995ed4aee60d9c390d9fb
//! files:
//! - comparable_test/test/set.rs
//!
//! The "comparable" library is distributed under an MIT-style license. The
//! license text is replicated here:
//!
//! > Permission is hereby granted, free of charge, to any
//! > person obtaining a copy of this software and associated
//! > documentation files (the "Software"), to deal in the
//! > Software without restriction, including without
//! > limitation the rights to use, copy, modify, merge,
//! > publish, distribute, sublicense, and/or sell copies of
//! > the Software, and to permit persons to whom the Software
//! > is furnished to do so, subject to the following
//! > conditions:
//! >
//! > The above copyright notice and this permission notice
//! > shall be included in all copies or substantial portions
//! > of the Software.
//! >
//! > THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
//! > ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
//! > TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
//! > PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
//! > SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
//! > CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
//! > OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
//! > IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
//! > DEALINGS IN THE SOFTWARE.
//!

use {
  ::comparable::{
    Changed::{Changed, Unchanged},
    I32Change, VecChange, assert_changes,
  },
  ::slimvec::{SlimVec, slimvec},
};

#[test]
fn test_comparable() {
  assert_changes!(&(slimvec![] as SlimVec<i32>), &slimvec![], Unchanged);
  assert_changes!(
    &slimvec![],
    &slimvec![1 as i32, 2, 3],
    Changed(slimvec![
      VecChange::Added(0, 1),
      VecChange::Added(1, 2),
      VecChange::Added(2, 3),
    ]),
  );
  assert_changes!(
    &slimvec![1 as i32, 2, 3],
    &slimvec![],
    Changed(slimvec![
      VecChange::Removed(0, 1),
      VecChange::Removed(1, 2),
      VecChange::Removed(2, 3),
    ]),
  );
  assert_changes!(
    &slimvec![1 as i32, 2],
    &slimvec![1 as i32, 2, 3],
    Changed(slimvec![VecChange::Added(2, 3)]),
  );
  assert_changes!(
    &slimvec![1 as i32, 2, 3],
    &slimvec![1 as i32, 2],
    Changed(slimvec![VecChange::Removed(2, 3)]),
  );
  assert_changes!(
    &slimvec![1 as i32, 3],
    &slimvec![1 as i32, 2, 3],
    Changed(slimvec![
      VecChange::Changed(1, I32Change(3, 2)),
      VecChange::Added(2, 3),
    ]),
  );
  assert_changes!(
    &slimvec![1 as i32, 2, 3],
    &slimvec![1 as i32, 3],
    Changed(slimvec![
      VecChange::Changed(1, I32Change(2, 3)),
      VecChange::Removed(2, 3),
    ]),
  );
  assert_changes!(
    &slimvec![1 as i32, 2, 3],
    &slimvec![1 as i32, 4, 3],
    Changed(slimvec![VecChange::Changed(1, I32Change(2, 4))]),
  );
}
