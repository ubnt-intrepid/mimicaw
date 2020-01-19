//! Minimal test harness that mimics libtest for asynchronous integration tests.

mod args;
mod event;
mod test;

pub use crate::test::{Benchmark, Test, TestOptions, TestSuite};

const ERROR_STATUS_CODE: i32 = 101;
