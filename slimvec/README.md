`slimvec`
=========

[crates.io] | [docs.rs] | [github]

[crates.io]: https://crates.io/crates/slimvec
[docs.rs]: https://docs.rs/slimvec
[github]: https://github.com/ickk/slimvec

`SlimVec` is analogous to the Standard Library's `Vec` collection type, however
it has a smaller inline-size; the inline data is a thin pointer like thin-vec.

`SlimVec` implements as much of the API surface of `Vec` as possible.

- [Overview](docs/Overview.md)
- [Architecture](docs/Architecture.md)
- [Testing](docs/Testing.md)
- [Future-work](docs/Future-work.md)

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
[Zlib-License](LICENSE.md#zlib-license),
[MIT-License](LICENSE.md#mit-license), or
[Apache2-License](LICENSE.md#apache2-license)
at your option.

See [LICENSE.md](LICENSE.md) for details.

-------------------------------------------------------------------------------
<footer><small>Copyright © ickk, 2026</small></footer>
