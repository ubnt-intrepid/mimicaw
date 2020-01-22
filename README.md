<h1 align="center">
  <code>mimicaw</code>
</h1>
<div align="center">
  <strong>
    Minimal test harness that mimics <code>libtest</code> for asynchronous integration tests.
  </strong>
</div>

<br />

<!--
<div align="center">
  <a href="https://crates.io/crates/polyfuse">
    <img src="https://img.shields.io/crates/v/polyfuse.svg?style=flat-square"
         alt="crates.io"
    />
  </a>
  <a href="https://blog.rust-lang.org/2019/11/07/Rust-1.39.0.html">
    <img src="https://img.shields.io/badge/rust%20toolchain-1.39.0%2B-gray.svg?style=flat-square"
         alt="rust toolchain"
    />
  </a>
  <a href="https://docs.rs/polyfuse">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
         alt="docs.rs" />
  </a>
</div>
-->


<br />

This is a small test harness library for writing integration tests using the asynchronous runtime, such as `tokio` and `async-std`.
The concept of this library is inspired from [`libtest-mimic`](https://github.com/LukasKalbertodt/libtest-mimic), but also focuses on
the compatibility with `async`/`.await` language syntax.

**WARNING:** This project is currently under active development and not ready for production use.

## License

This library is licensed under either of

* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.
