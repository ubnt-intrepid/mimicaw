<h1 align="center">
  <code>mimicaw</code>
</h1>
<div align="center">
  <strong>
    A library for writing asynchronous tests.
  </strong>
</div>

<br />

<div align="center">
  <a href="https://crates.io/crates/mimicaw">
    <img src="https://img.shields.io/crates/v/mimicaw.svg?style=flat-square"
         alt="crates.io"
    />
  </a>
  <a href="https://blog.rust-lang.org/2019/12/19/Rust-1.40.0.html">
    <img src="https://img.shields.io/badge/rust-1.40.0-gray?style=flat-square"
         alt="rust toolchain"
    />
  </a>
  <a href="https://docs.rs/mimicaw">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
         alt="docs.rs" />
  </a>
</div>

<br />

`mimicaw` is a small library that provides a framework for writing the free-style, asynchronous tests without using the default test harness provided by `rustc`.
The concept and design are **strongly** inspired by [`libtest-mimic`](https://github.com/LukasKalbertodt/libtest-mimic), but also focuses on
the affinity with the `async`/`.await` syntax.

## Installation

First, add `mimicaw` as a development dependency of your package.
If you are the user of [`cargo-edit`](https://github.com/killercup/cargo-edit):

```shell-session
$ cargo add --dev mimcaw
```

The test binaries must explicitly set the `harness` key to make the default test harness provided by `rustc` disabled:

```toml
[[test]]
name = "mytest"
path = "tests/mytest.rs"
harness = false
```

## Resources

* [Examples](./examples)
* [API documentation (docs.rs)](https://docs.rs/mimicaw)
* [API documentation (master)](https://ubnt-intrepid.github.io/mimicaw/mimicaw/index.html)

## License

This library is licensed under either of

* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.
