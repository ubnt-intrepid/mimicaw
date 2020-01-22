use std::{borrow::Cow, sync::Arc};

#[derive(Copy, Clone, Debug)]
pub(crate) enum TestKind {
    Test,
    Bench,
}

/// Data that describes a single test.
pub struct Test<D = ()> {
    name: Arc<String>,
    kind: TestKind,
    ignored: bool,
    context: Option<D>,
}

impl Test {
    /// Create a single test.
    pub fn test(name: &str) -> Self {
        Self::new(name, TestKind::Test)
    }

    /// Create a single benchmark test.
    pub fn bench(name: &str) -> Self {
        Self::new(name, TestKind::Bench)
    }

    fn new(name: &str, kind: TestKind) -> Self {
        Self {
            name: Arc::new(name.into()),
            kind,
            ignored: false,
            context: None,
        }
    }
}

impl<D> Test<D> {
    /// Specify the context value associated with this test.
    pub fn context<T>(self, context: T) -> Test<T> {
        Test {
            name: self.name,
            kind: self.kind,
            ignored: self.ignored,
            context: Some(context),
        }
    }

    /// Mark that this test should be ignored.
    pub fn ignore(self, value: bool) -> Self {
        Self {
            ignored: value,
            ..self
        }
    }

    pub(crate) fn name(&self) -> &Arc<String> {
        &self.name
    }

    pub(crate) fn kind(&self) -> &TestKind {
        &self.kind
    }

    pub(crate) fn ignored(&self) -> bool {
        self.ignored
    }

    pub(crate) fn take_context(&mut self) -> D {
        self.context
            .take()
            .expect("the context has already been taken")
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
