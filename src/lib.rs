//! A tiny test framework for asynchronous integration tests.

mod args;
mod driver;
mod test;

pub use crate::test::{Outcome, Test, TestDesc};

use crate::driver::TestDriver;
use futures_core::future::Future;

const ERROR_STATUS_CODE: i32 = 101;

/// Run a set of tests.
pub async fn run_tests<D, R>(
    tests: impl IntoIterator<Item = Test<D>>,
    runner: impl FnMut(&TestDesc, D) -> R,
) -> i32
where
    R: Future<Output = Outcome> + Unpin,
{
    match TestDriver::from_env() {
        Ok(mut driver) => driver.run_tests(tests, runner).await,
        Err(code) => code,
    }
}
