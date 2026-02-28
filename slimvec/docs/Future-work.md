Almost all of the API surface area of `Vec` has been replicated for `SlimVec`.
As new functionality is added to the Standard Library and stabilised, `SlimVec`
will want to mirror that same functionality to keep the APIs coherent with
one-another.

In the future `SlimVec` would like to add support for the currently unstable
Rust features `allocator-api` & `#[may_dangle]`.
