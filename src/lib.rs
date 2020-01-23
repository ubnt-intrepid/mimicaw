//! A tiny test framework for asynchronous integration tests.

#![doc(html_root_url = "https://docs.rs/mimicaw/0.0.2")]
#![deny(missing_docs)]
#![forbid(clippy::unimplemented, clippy::todo)]

mod args;
mod driver;
mod printer;
mod test;

pub use crate::{
    args::{parse_args, Args, ColorConfig, OutputFormat},
    test::{Outcome, Test, TestDesc},
};

use crate::driver::TestDriver;
use futures_core::future::Future;

const ERROR_STATUS_CODE: i32 = 101;

/// Run a set of tests.
pub async fn run_tests<D, R>(
    args: &Args,
    tests: impl IntoIterator<Item = Test<D>>,
    runner: impl FnMut(&TestDesc, D) -> R,
) -> i32
where
    R: Future<Output = Outcome> + Unpin,
{
    let driver = TestDriver::new(&args);
    driver.run_tests(tests, runner).await
}

#[test]
fn test_html_root_url() {
    version_sync::assert_html_root_url_updated!("src/lib.rs");
}
