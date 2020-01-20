//! Minimal test harness that mimics libtest for asynchronous integration tests.

mod args;
mod driver;

pub use crate::driver::{Benchmark, Test, TestDriver, TestOptions, TestSuite};

const ERROR_STATUS_CODE: i32 = 101;
