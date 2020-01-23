//! A library for writing asynchronous tests.

#![doc(html_root_url = "https://docs.rs/mimicaw/0.0.2")]
#![deny(missing_docs)]
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
