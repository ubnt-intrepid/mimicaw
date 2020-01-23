//! A library for writing asynchronous tests.

#![doc(html_root_url = "https://docs.rs/mimicaw/0.1.0")]
#![deny(missing_docs, unsafe_code)]
#![forbid(clippy::unimplemented, clippy::todo)]

mod args;
mod driver;
mod printer;
mod test;

pub use crate::{
    args::{Args, ColorConfig, OutputFormat},
    driver::TestRunner,
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
    let driver = TestDriver::new(&args);
    driver.run_tests(tests, runner).await
}

#[test]
fn test_html_root_url() {
    version_sync::assert_html_root_url_updated!("src/lib.rs");
}

#[test]
fn test_readme_deps() {
    version_sync::assert_markdown_deps_updated!("README.md");
}
