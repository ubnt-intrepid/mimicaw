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

pub async fn run_tests<D, I, F, R>(tests: I, runner: F) -> i32
where
    I: IntoIterator<Item = Test<D>>,
    F: FnMut(D) -> R,
    R: std::future::Future<Output = Outcome> + Unpin,
{
    let mut driver = TestDriver::from_env();
    driver.run_tests(tests, runner).await
}
