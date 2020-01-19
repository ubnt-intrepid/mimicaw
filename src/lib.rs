//! Minimal test harness that mimics libtest for asynchronous integration tests.

mod args;
mod event;
mod test;

pub use crate::event::{DefaultEventHandler, EventHandler, Outcome, Report};
pub use crate::test::{Benchmark, Test, TestOptions, TestSuite};

const ERROR_CODE: i32 = 101;
