The `slimvec` package includes the following cargo-features:


### `arbitrary`

[`arbitrary`]: https://crates.io/crates/arbitrary

Enables support for constructing arbitrary values via [`arbitrary`].

- Adds implementation of `arbitrary::Arbitrary` for `SlimVec<T: Arbitrary>`.


### `borsh`

[`borsh`]: https://crates.io/crates/borsh

Enables support for serialisation & deserialisation via [`borsh`].

- Adds implementation of `borsh::BorshSerialize` for
  `SlimVec<T: BorshSerialize>`.
- Adds implementation of `borsh::BorshDeserialize` for
  `SlimVec<T: BorshDeserialize>`.


### `comparable`

[`comparable`]: https://crates.io/crates/comparable

Enables support for structural comparison via [`comparable`].

- Adds implementation of `comparable::Comparable` for `SlimVec<T: Comparable>`.


### `serde`

[`serde`]: https://crates.io/crates/serde

Enables support for serialisation & deserialisation via [`serde`].

- Adds implementation of `serde::Serialize` for `SlimVec<T: Serialize>`.
- Adds implementation of `serde::Deserialize` for `SlimVec<T: Deserialize>`.


### `std`

[Rust Standard Library]: https://doc.rust-lang.org/std

Enables support for sinking byte streams via the [Rust Standard Library].

- Adds implementation of `std::io::Write` for `SlimVec<u8>`.


### `valuable`

[`valuable`]: https://crates.io/crates/valuable

Enables dyn-compatible value inspection via [`valuable`].

- Adds implementation of `valuable::Valuable` for `SlimVec<T: Valuable>`.
- Adds implementation of `valuable::Listable` for `SlimVec<T: Valuable>`.
