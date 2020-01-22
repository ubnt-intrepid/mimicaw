//! Minimal test harness that mimics libtest for asynchronous integration tests.

mod args;
mod driver;
mod progress;
mod test;

pub use crate::{
    driver::TestDriver,
    test::{Outcome, Test},
};

const ERROR_STATUS_CODE: i32 = 101;

/// Run a set of tests asynchronously.
///
/// See [`TestDriver::run_tests`] for details.
///
/// [`TestDriver::run_tests`]: ./struct.TestDriver.html#method.run_tests
pub async fn run_tests<D, I, F, R>(tests: I, runner: F) -> i32
where
    I: IntoIterator<Item = Test<D>>,
    F: FnMut(D) -> R,
    R: std::future::Future<Output = Outcome> + Unpin,
{
    let mut driver = TestDriver::from_env();
    driver.run_tests(tests, runner).await
}
