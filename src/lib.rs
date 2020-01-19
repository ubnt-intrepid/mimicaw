//! Minimal test harness that mimics libtest for asynchronous integration tests.

mod args;
mod suite;

pub use crate::suite::{Benchmark, Measurement, Test, TestSuite};

const ERROR_CODE: i32 = 101;

#[derive(Debug)]
enum Outcome {
    Passed,
    Failed { msg: Option<String> },
    Ignored,
    Measured { measurement: Measurement },
    Canceled,
}

#[derive(Debug)]
#[must_use]
pub struct Report {
    has_failed: bool,
}

impl Report {
    pub fn has_failed(&self) -> bool {
        self.has_failed
    }

    pub fn exit(self) -> ! {
        if self.has_failed {
            std::process::exit(101);
        }
        std::process::exit(0);
    }
}
