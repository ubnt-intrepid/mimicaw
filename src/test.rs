use std::{borrow::Cow, sync::Arc};

#[derive(Copy, Clone, Debug)]
pub(crate) enum TestKind {
    Test,
    Bench,
}

/// Description about a test.
#[derive(Debug, Clone)]
pub struct TestDesc(Arc<TestDescInner>);

#[derive(Debug)]
struct TestDescInner {
    name: Cow<'static, str>,
    kind: TestKind,
    ignored: bool,
}

impl AsRef<Self> for TestDesc {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl TestDesc {
    pub(crate) fn kind(&self) -> &TestKind {
        &self.0.kind
    }

    /// Return the name of test.
    #[inline]
    pub fn name(&self) -> &str {
        &*self.0.name
    }

    /// Return whether the test is a benchmark or not.
    #[inline]
    pub fn is_bench(&self) -> bool {
        match self.0.kind {
            TestKind::Bench => true,
            _ => false,
        }
    }

    /// Return whether the test should be ignored or not.
    #[inline]
    pub fn ignored(&self) -> bool {
        self.0.ignored
    }
}

/// Data that describes a single test.
pub struct Test<D> {
    desc: TestDesc,
    data: D,
}

impl<D> Test<D> {
    /// Create a single test.
    pub fn test(name: impl Into<Cow<'static, str>>, data: D) -> Self {
        Self::new(name.into(), TestKind::Test, data)
    }

    /// Create a single benchmark test.
    pub fn bench(name: impl Into<Cow<'static, str>>, data: D) -> Self {
        Self::new(name.into(), TestKind::Bench, data)
    }

    fn new(name: Cow<'static, str>, kind: TestKind, data: D) -> Self {
        Self {
            desc: TestDesc(Arc::new(TestDescInner {
                name,
                kind,
                ignored: false,
            })),
            data,
        }
    }

    /// Mark that this test should be ignored.
    pub fn ignore(mut self, value: bool) -> Self {
        Arc::get_mut(&mut self.desc.0).unwrap().ignored = value;
        self
    }

    pub(crate) fn desc(&self) -> &TestDesc {
        &self.desc
    }

    pub(crate) fn deconstruct(self) -> (TestDesc, D) {
        (self.desc, self.data)
    }
}

/// The outcome of performing a test.
#[derive(Debug)]
pub struct Outcome {
    kind: OutcomeKind,
    err_msg: Option<Arc<Cow<'static, str>>>,
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
            err_msg: Some(Arc::new(err_msg.into())),
            ..self
        }
    }

    pub(crate) fn kind(&self) -> &OutcomeKind {
        &self.kind
    }

    pub(crate) fn err_msg(&self) -> Option<Arc<Cow<'static, str>>> {
        self.err_msg.clone()
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum OutcomeKind {
    Passed,
    Failed,
    Measured { average: u64, variance: u64 },
}
