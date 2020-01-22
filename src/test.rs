use std::sync::Arc;

#[derive(Copy, Clone, Debug)]
pub(crate) enum TestKind {
    Test,
    Bench,
}

/// The context object that tracks the progress of a test case.
pub struct Test<D = ()> {
    name: Arc<String>,
    kind: TestKind,
    ignored: bool,
    context: Option<D>,
}

impl Test {
    /// Register a single test to the suite.
    pub fn test(name: &str) -> Self {
        Self::new(name, TestKind::Test)
    }

    /// Register a single benchmark test to the suite.
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
    pub fn context<T>(self, context: T) -> Test<T> {
        Test {
            name: self.name,
            kind: self.kind,
            ignored: self.ignored,
            context: Some(context),
        }
    }

    /// Mark that the test will be ignored.
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

#[derive(Debug)]
#[non_exhaustive]
pub enum Outcome {
    Passed,
    Failed { msg: Option<String> },
    Measured { average: u64, variance: u64 },
}
