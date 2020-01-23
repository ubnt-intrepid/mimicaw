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

`mimicaw` is a tiny library for writing asynchronous tests.
The concept of this library is inspired by [`libtest-mimic`](https://github.com/LukasKalbertodt/libtest-mimic), but also focuses on
the compatibility with `async`/`.await` language syntax.

## Example

```rust
use mimicaw::{Args, Test, TestDesc, Outcome};

// Parse command line arguments.
let args = Args::from_env().unwrap_or_else(|st| st.exit());

// Each test case is described using `Test` having one associated data.
//
// The data will be used by the runner described below to run tests.
let tests = vec![
    Test::test("case1", "foo"),
    Test::test("case2", "bar"),
    Test::test("case3_long_computation", "baz"),
    Test::test("case4", "The quick brown fox jumps over the lazy dog."),
];

// A closure for running the test cases.
//
// Each test result is asynchronous and a future is returned
// to acquire the result.
let runner = |_desc: TestDesc, data: &str| {
    async move {
        match data {
            "foo" | "baz" => Outcome::passed(),
            "bar" => Outcome::failed().error_message("`bar' is forbidden"),
            data => Outcome::failed().error_message(format!("unknown data: {}", data)),
        }
    }
};

// Run the process of test suite.
//
// The test cases are filtered according to the command line arguments,
// and then executed concurrently from the top.
let status = mimicaw::run_tests(&args, tests, runner).await;
status.exit()
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
