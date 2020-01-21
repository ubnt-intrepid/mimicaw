use futures::future::Future;
use pin_project_lite::pin_project;
use std::{pin::Pin, sync::Arc};

#[derive(Copy, Clone, Debug)]
pub(crate) enum TestKind {
    Test,
    Bench,
}

pin_project! {
    /// The context object that tracks the progress of a test case.
    pub struct Test {
        name: Arc<String>,
        kind: TestKind,
        pub(crate) ignored: bool,
        #[pin]
        test_case: Pin<Box<dyn Future<Output = Outcome> + 'static>>,
    }
}

impl Test {
    /// Register a single test to the suite.
    pub fn test<Fut>(name: &str, test_case: Fut) -> Self
    where
        Fut: Future<Output = Result<(), Option<String>>> + 'static,
    {
        Self::new(name, TestKind::Test, async move {
            match test_case.await {
                Ok(()) => Outcome::Passed,
                Err(msg) => Outcome::Failed { msg },
            }
        })
    }

    /// Register a single benchmark test to the suite.
    pub fn bench<Fut>(name: &str, bench_fn: Fut) -> Self
    where
        Fut: Future<Output = Result<(u64, u64), Option<String>>> + 'static,
    {
        Self::new(name, TestKind::Bench, async move {
            match bench_fn.await {
                Ok((average, variance)) => Outcome::Measured { average, variance },
                Err(msg) => Outcome::Failed { msg },
            }
        })
    }

    fn new<Fut>(name: &str, kind: TestKind, test_case: Fut) -> Self
    where
        Fut: Future<Output = Outcome> + 'static,
    {
        Self {
            name: Arc::new(name.into()),
            kind,
            ignored: false,
            test_case: Box::pin(test_case),
        }
    }

    /// Mark that the test will be ignored.
    pub fn ignored(mut self, value: bool) -> Self {
        self.ignored = value;
        self
    }

    pub(crate) fn name(&self) -> &Arc<String> {
        &self.name
    }

    pub(crate) fn kind(&self) -> &TestKind {
        &self.kind
    }

    pub(crate) fn test_case(
        self: Pin<&mut Self>,
    ) -> Pin<&mut Pin<Box<dyn Future<Output = Outcome> + 'static>>> {
        self.project().test_case
    }
}

#[derive(Debug)]
pub(crate) enum Outcome {
    Passed,
    Failed { msg: Option<String> },
    Measured { average: u64, variance: u64 },
}
