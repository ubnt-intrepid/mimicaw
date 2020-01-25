/*!
A library for writing asynchronous tests.

This library provides a framework for writing the free-style,
asynchronous tests without using the default test harness
provided by `rustc`.

The concept and design are **strongly** inspired by
[`libtest-mimic`](https://github.com/LukasKalbertodt/libtest-mimic),
but also focuses on the affinity with the `async`/`.await` syntax.

# Example

```no_run
# fn main() { futures::executor::block_on(async {
use mimicaw::{Args, Test, TestDesc, Outcome};

// Parse command line arguments.
let args = Args::from_env().unwrap_or_else(|st| st.exit());

// Each test case is described using `Test` having one associated data.
//
// The data will be used by the runner described below to run tests.
let tests = vec![
    Test::test("case1", "foo"),
    Test::test("case2", "bar"),
    Test::test("case3_long_computation", "baz").ignore(true),
    Test::test("case4", "The quick brown fox jumps over the lazy dog."),
];

// A function for running the test cases.
//
// Each test result is asynchronous and a future is returned to acquire the result.
let runner = |_desc: TestDesc, data: &'static str| {
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
// The test cases are filtered according to the command line arguments, and then executed concurrently from the top.
let status = mimicaw::run_tests(&args, tests, runner).await;
status.exit()
# }) }
```
!*/

#![doc(html_root_url = "https://docs.rs/mimicaw/0.1.1")]
#![deny(missing_docs)]
#![forbid(unsafe_code, clippy::unimplemented, clippy::todo)]

mod args;
mod driver;
mod printer;
mod report;
mod test;

pub use crate::{
    args::{Args, ColorConfig, OutputFormat},
    driver::TestRunner,
    report::Report,
    test::{Outcome, Test, TestDesc},
};

use crate::driver::TestDriver;

/// Exit status code used as a result of the test process.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ExitStatus(i32);

impl ExitStatus {
    const OK: Self = Self(0);
    const FAILED: Self = Self(101);

    /// Return whether the status is successful or not.
    #[inline]
    pub fn success(self) -> bool {
        self.code() == 0
    }

    /// Return the raw exit code.
    #[inline]
    pub fn code(self) -> i32 {
        self.0
    }

    /// Terminate the test process with the exit code.
    ///
    /// This method **should not** be called before the cleanup
    /// of the test process has completed.
    #[inline]
    pub fn exit(self) -> ! {
        std::process::exit(self.code());
    }

    /// Terminate the test process if the exit code is not successful.
    ///
    /// This method **should not** be called before the cleanup
    /// of the test process has completed.
    #[inline]
    pub fn exit_if_failed(self) {
        if !self.success() {
            self.exit();
        }
    }
}

/// Run a test suite using the specified test runner.
///
/// The test suite runs as follows:
///
/// * Test cases that do not satisfy the conditions given in
///   the command line options are filtered out.
/// * Apply the test runner to each test case that passed to
///   the filter, and create futures for awaiting their outcomes.
///   these futures are executed concurrently, and their results
///   are written to the console in the order of completion.
/// * Finally, the results of all test cases are aggregated.
pub async fn run_tests<D>(
    args: &Args,
    tests: impl IntoIterator<Item = Test<D>>,
    runner: impl TestRunner<D>,
) -> ExitStatus {
    match run_tests_with_report(args, tests, runner).await {
        Ok(report) => report.status(),
        Err(status) => status,
    }
}

/// Run a test suite and report the summary.
pub async fn run_tests_with_report<D>(
    args: &Args,
    tests: impl IntoIterator<Item = Test<D>>,
    runner: impl TestRunner<D>,
) -> Result<Report, ExitStatus> {
    let driver = TestDriver::new(&args);
    driver.run_tests(tests, runner).await
}

#[test]
fn test_html_root_url() {
    version_sync::assert_html_root_url_updated!("src/lib.rs");
}
