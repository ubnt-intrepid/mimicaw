//! Minimal test harness that mimics libtest for asynchronous integration tests.

mod args;
mod driver;
mod progress;
mod test;

pub use crate::{driver::TestDriver, test::Test};

const ERROR_STATUS_CODE: i32 = 101;

pub async fn run_tests<I>(tests: I) -> i32
where
    I: IntoIterator<Item = Test>,
{
    let mut driver = TestDriver::from_env();
    driver.run_tests(tests).await
}
