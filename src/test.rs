use futures::{
    channel::oneshot,
    future::Future,
    task::{self, Poll},
};
use indicatif::ProgressBar;
use pin_project_lite::pin_project;
use std::{pin::Pin, sync::Arc};

/// A set of options for a test or a benchmark.
#[derive(Copy, Clone, Debug)]
pub struct TestOptions {
    pub(crate) kind: TestKind,
    pub(crate) ignored: bool,
}

impl TestOptions {
    /// Create a new `TestOptions`.
    pub fn test() -> Self {
        Self {
            kind: TestKind::Test,
            ignored: false,
        }
    }

    pub fn bench() -> Self {
        Self {
            kind: TestKind::Bench,
            ignored: false,
        }
    }

    /// Mark that the test will be ignored.
    pub fn ignored(mut self, value: bool) -> Self {
        self.ignored = value;
        self
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum TestKind {
    Test,
    Bench,
}

pin_project! {
    /// The context object that tracks the progress of a test case.
    #[derive(Debug)]
    pub(crate) struct TestCase {
        name: Arc<String>,
        opts: TestOptions,
        filtered: bool,
        tx_progress: Option<oneshot::Sender<ProgressBar>>,
        #[pin]
        rx_outcome: Option<oneshot::Receiver<Outcome>>,
        outcome: Option<Outcome>,
    }
}

impl TestCase {
    pub(crate) fn new(
        name: &Arc<String>,
        opts: TestOptions,
        filtered: bool,
    ) -> (Self, Option<Handle>) {
        if !filtered {
            let (tx_progress, rx_progress) = oneshot::channel();
            let (tx_outcome, rx_outcome) = oneshot::channel();

            let test = Self {
                name: name.clone(),
                opts,
                filtered,
                tx_progress: Some(tx_progress),
                rx_outcome: Some(rx_outcome),
                outcome: None,
            };

            let handle = Handle {
                progress: rx_progress,
                outcome: tx_outcome,
            };

            (test, Some(handle))
        } else {
            let test = Self {
                name: name.clone(),
                opts,
                filtered,
                tx_progress: None,
                rx_outcome: None,
                outcome: None,
            };
            (test, None)
        }
    }

    pub(crate) fn name(&self) -> &Arc<String> {
        &self.name
    }

    pub(crate) fn filtered(&self) -> bool {
        self.filtered
    }

    pub(crate) fn opts(&self) -> &TestOptions {
        &self.opts
    }

    pub(crate) fn start(&mut self, progress: ProgressBar) {
        if let Some(tx) = self.tx_progress.take() {
            progress.enable_steady_tick(100);
            progress.set_message("running");
            let _ = tx.send(progress);
        } else {
            progress.finish_with_message(&console::style("ignored").yellow().to_string());
        }
    }

    pub(crate) fn take_outcome(&mut self) -> Outcome {
        self.outcome.take().unwrap_or_else(|| Outcome::Canceled)
    }
}

impl Future for TestCase {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let me = self.project();
        let outcome = match me.rx_outcome.as_pin_mut() {
            Some(rx_outcome) => {
                futures::ready!(rx_outcome.poll(cx)).unwrap_or_else(|_| Outcome::Canceled)
            }
            None => Outcome::Ignored,
        };
        me.outcome.replace(outcome);
        Poll::Ready(())
    }
}

/// The handle to a test.
#[derive(Debug)]
pub struct Handle {
    pub(crate) progress: oneshot::Receiver<ProgressBar>,
    pub(crate) outcome: oneshot::Sender<Outcome>,
}

impl Handle {
    async fn run<Fut>(self, fut: Fut)
    where
        Fut: Future<Output = Outcome>,
    {
        let progress = self.progress.await.unwrap();

        let outcome = fut.await;
        match outcome {
            Outcome::Passed => {
                progress.finish_with_message(&console::style("passed").green().to_string());
            }
            Outcome::Failed { .. } => {
                progress.finish_with_message(&console::style("failed").red().bold().to_string());
            }
            Outcome::Measured { average, variance } => {
                progress.finish_with_message(
                    &console::style(&format!(
                        "finished: average={}, variance={}",
                        average, variance
                    ))
                    .green()
                    .to_string(),
                );
            }
            _ => (),
        };

        let _ = self.outcome.send(outcome);
    }

    /// Wrap a future to catch events from the test suite.
    pub async fn run_test<Fut>(self, test_case: Fut)
    where
        Fut: Future<Output = Result<(), Option<String>>>,
    {
        self.run(async {
            match test_case.await {
                Ok(()) => Outcome::Passed,
                Err(msg) => Outcome::Failed { msg },
            }
        })
        .await
    }

    /// Wrap a future to catch events from the test suite.
    pub async fn run_bench<Fut>(self, test_case: Fut)
    where
        Fut: Future<Output = Result<(u64, u64), Option<String>>>,
    {
        self.run(async {
            match test_case.await {
                Ok((average, variance)) => Outcome::Measured { average, variance },
                Err(msg) => Outcome::Failed { msg },
            }
        })
        .await
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Outcome {
    Passed,
    Failed {
        msg: Option<String>,
    },
    Ignored,
    Measured {
        average: u64,
        variance: u64,
    },

    #[doc(hidden)]
    Canceled,
}
