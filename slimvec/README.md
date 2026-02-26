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
- [Features](docs/Features.md)
- [Architecture](docs/Architecture.md)
- [Testing](docs/Testing.md)
- [Future-work](docs/Future-work.md)
- [Alternatives](docs/Alternatives.md)

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
[LICENSE-ZLIB](LICENSE-ZLIB),
[LICENSE-MIT](LICENSE-MIT), or
[LICENSE-APACHE2](LICENSE-APACHE2)
at your option.

-------------------------------------------------------------------------------
<footer><small>Copyright © ickk, 2026</small></footer>
