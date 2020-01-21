//! Minimal test harness that mimics libtest for asynchronous integration tests.

mod args;
mod driver;
mod test;

pub use crate::{
    driver::TestDriver,
    test::{Handle, TestOptions},
};

const ERROR_STATUS_CODE: i32 = 101;
