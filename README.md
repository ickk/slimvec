<!--
Note: This is a copy of slimvec/README.md with fixed-up links. It is necessary
due to the limitations of cargo; packages are unable to include files in parent
directories.
-->

`slimvec`
=========

[crates.io] | [docs.rs] | [github]

[crates.io]: https://crates.io/crates/slimvec
[docs.rs]: https://docs.rs/slimvec
[github]: https://github.com/ickk/slimvec

`SlimVec` is analogous to the Standard Library's `Vec` collection type, however
it has a smaller inline-size; the inline data is a thin pointer like thin-vec.

`SlimVec` implements as much of the API surface of `Vec` as possible.
Additionally includes optional features for `arbitrary`, `borsh`, `comparable`,
`serde`, & `valuable`.

- [Overview](slimvec/docs/Overview.md)
- [Features](slimvec/docs/Features.md)
- [Architecture](slimvec/docs/Architecture.md)
- [Testing](slimvec/docs/Testing.md)
- [Differences to `Vec`](slimvec/docs/Differences-to-Vec.md)
- [Alternatives](slimvec/docs/Alternatives.md)

```text
  SlimVec<T>            Allocation
+-------------+       +-------------+
| ptr: 0xabcd | ----> | length:   2 |
+-------------+       |-------------|
                      | capacity: 4 |
                      |=============|
                      | 0:        A |
                      |-------------|
                      | 1:        B |
                      |-------------|
                      | 2: <uninit> |
                      |-------------|
                      | 3: <uninit> |
                      +-------------+
```

Licensing
---------

This library may be licensed under any of the following
[LICENSE-ZLIB](slimvec/LICENSE-ZLIB),
[LICENSE-MIT](slimvec/LICENSE-MIT), or
[LICENSE-APACHE2](slimvec/LICENSE-APACHE2)
at your option.

-------------------------------------------------------------------------------
<footer><small>Copyright © ickk, 2026</small></footer>
