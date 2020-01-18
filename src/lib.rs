//! Minimal test harness that mimics libtest for asynchronous integration tests.

mod args;
mod suite;

pub use crate::suite::{Test, TestSuite};

const ERROR_CODE: i32 = 101;

#[derive(Debug)]
pub struct Outcome(OutcomeKind);

#[derive(Debug)]
enum OutcomeKind {
    Passed,
    Failed { msg: Option<String> },
    Ignored,
    Canceled,
}

impl Outcome {
    pub fn passed() -> Self {
        Self(OutcomeKind::Passed)
    }

    pub fn failed(msg: Option<&str>) -> Self {
        Self(OutcomeKind::Failed {
            msg: msg.map(|s| s.into()),
        })
    }

    fn ignored() -> Self {
        Self(OutcomeKind::Ignored)
    }

    fn canceled() -> Self {
        Self(OutcomeKind::Canceled)
    }
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
