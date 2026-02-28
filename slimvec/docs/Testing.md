`slimvec` has the following test suites:

- `tests/smoke_tests.rs`

  A series of tests written specifically for slimvec that ensures functionality
  behaves as expected.


- `tests/impls.rs`

  A series of tests that ensure `SlimVec`, `IntoIter`, `Drain`, & `ExtractIf`
  implement the same sets of traits as `Vec` &c. from the Standard Library.
  This helps keep the interfaces for `SlimVec` & `Vec` coherent with
  one-another, so that `SlimVec` can function as a drop-in replacement for
  `Vec` in many cases.


- `tests/alloctests.rs`

  `alloctests.rs` is a port of the Standard Library's test-suite for `Vec`.
  This set of tests ensure that `SlimVec` behaves the same as `Vec` in
  practice. These tests include some trickier edge-cases and regression tests
  for logic bugs and UB found in the Standard Library over the years.

- `tests/comparable.rs`

  `comparable.rs` is a port of the `comparable` library's tests for `Vec`.
