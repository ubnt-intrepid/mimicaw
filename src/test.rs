use std::{borrow::Cow, sync::Arc};

#[derive(Copy, Clone, Debug)]
pub(crate) enum TestKind {
    Test,
    Bench,
}

/// Description about a test.
#[derive(Debug, Clone)]
pub struct TestDesc {
    name: Arc<String>,
    kind: TestKind,
    ignored: bool,
}

impl TestDesc {
    pub(crate) fn name_arc(&self) -> &Arc<String> {
        &self.name
    }

    pub(crate) fn kind(&self) -> &TestKind {
        &self.kind
    }

    /// Return the name of test.
    #[inline]
    pub fn name(&self) -> &str {
        &*self.name
    }

    /// Return whether the test is a benchmark or not.
    #[inline]
    pub fn is_bench(&self) -> bool {
        match self.kind {
            TestKind::Bench => true,
            _ => false,
        }
    }

    /// Return whether the test should be ignored or not.
    #[inline]
    pub fn ignored(&self) -> bool {
        self.ignored
    }
}

/// Data that describes a single test.
pub struct Test<D> {
    desc: TestDesc,
    data: D,
}

impl<D> Test<D> {
    /// Create a single test.
    pub fn test(name: &str, data: D) -> Self {
        Self::new(name, TestKind::Test, data)
    }

    /// Create a single benchmark test.
    pub fn bench(name: &str, data: D) -> Self {
        Self::new(name, TestKind::Bench, data)
    }

    fn new(name: &str, kind: TestKind, data: D) -> Self {
        Self {
            desc: TestDesc {
                name: Arc::new(name.into()),
                kind,
                ignored: false,
            },
            data,
        }
    }

    /// Mark that this test should be ignored.
    pub fn ignore(mut self, value: bool) -> Self {
        self.desc.ignored = value;
        self
    }

    pub(crate) fn deconstruct(self) -> (TestDesc, D) {
        (self.desc, self.data)
    }
}

/// The outcome of performing a test.
#[derive(Debug)]
pub struct Outcome {
    kind: OutcomeKind,
    err_msg: Option<Cow<'static, str>>,
}

impl Outcome {
    #[inline]
    fn new(kind: OutcomeKind) -> Self {
        Self {
            kind,
            err_msg: None,
        }
    }

    /// Create an `Outcome` representing that the test passed.
    #[inline]
    pub fn passed() -> Self {
        Self::new(OutcomeKind::Passed)
    }

    /// Create an `Outcome` representing that the test or benchmark failed.
    pub fn failed() -> Self {
        Self::new(OutcomeKind::Failed)
    }

    /// Create an `Outcome` representing that the benchmark test was successfully run.
    pub fn measured(average: u64, variance: u64) -> Self {
        Self::new(OutcomeKind::Measured { average, variance })
    }

    /// Specify the error message.
    pub fn error_message(self, err_msg: impl Into<Cow<'static, str>>) -> Self {
        Self {
            err_msg: Some(err_msg.into()),
            ..self
        }
    }

    pub(crate) fn kind(&self) -> &OutcomeKind {
        &self.kind
    }

    pub(crate) fn err_msg(&self) -> Option<Cow<'static, str>> {
        self.err_msg.clone()
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum OutcomeKind {
    Passed,
    Failed,
    Measured { average: u64, variance: u64 },
}
