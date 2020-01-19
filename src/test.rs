use crate::{
    args::Args,
    event::{DefaultEventHandler, EventHandler, Outcome, Report},
};
use futures::{channel::oneshot, future::Future};
use futures_intrusive::sync::ManualResetEvent;
use std::{
    collections::hash_map::{Entry, HashMap},
    sync::Arc,
};

/// A set of options for a test or a benchmark.
#[derive(Copy, Clone, Debug, Default)]
pub struct TestOptions {
    ignored: bool,
}

impl TestOptions {
    /// Create a new `TestOptions`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark that the test will be ignored.
    pub fn ignored(mut self, value: bool) -> Self {
        self.ignored = value;
        self
    }
}

#[derive(Debug)]
struct TestCase {
    name: String,
    is_test: bool,
    opts: TestOptions,
    rx: Option<oneshot::Receiver<Outcome>>,
}

/// A type that represents a test suite.
#[derive(Debug)]
pub struct TestSuite {
    args: Args,
    tests: HashMap<String, TestCase>,
    started: Arc<ManualResetEvent>,
}

impl TestSuite {
    /// Create a test suite.
    pub fn from_env() -> Self {
        match Args::from_env() {
            Ok(args) => Self::new(args),
            Err(code) => {
                // The process should not be exited at here
                // in order for the resources in main function to
                // be appropriately dropped.
                std::process::exit(code);
            }
        }
    }

    /// Create a test suite, if possible.
    pub fn try_from_env() -> Option<Self> {
        Args::from_env().ok().map(Self::new)
    }

    fn new(args: Args) -> Self {
        Self {
            args,
            tests: HashMap::new(),
            started: Arc::new(ManualResetEvent::new(false)),
        }
    }

    fn is_filtered(&self, name: &str) -> bool {
        if let Some(ref filter) = self.args.filter {
            if self.args.filter_exact && name != filter {
                return true;
            }
            if !name.contains(filter) {
                return true;
            }
        }

        for skip_filter in &self.args.skip {
            if self.args.filter_exact && name != skip_filter {
                return true;
            }
            if !name.contains(skip_filter) {
                return true;
            }
        }

        false
    }

    /// Register a single test to the suite.
    ///
    /// This method will return a handle if the specified test needs
    /// to be driven.
    pub fn add_test(&mut self, name: &str, opts: TestOptions) -> Option<Test> {
        self.add_test_inner(name, opts, true) //
            .map(|tx| Test {
                started: self.started.clone(),
                tx,
            })
    }

    /// Register a single benchmark test to the suite.
    ///
    /// This method will return a handle if the specified benchmark test needs
    /// to be driven.
    pub fn add_bench(&mut self, name: &str, opts: TestOptions) -> Option<Benchmark> {
        self.add_test_inner(name, opts, false) //
            .map(|tx| Benchmark {
                started: self.started.clone(),
                tx,
            })
    }

    fn add_test_inner(
        &mut self,
        name: &str,
        opts: TestOptions,
        is_test: bool,
    ) -> Option<oneshot::Sender<Outcome>> {
        let is_target_mode = if is_test {
            self.args.run_tests
        } else {
            self.args.run_benchmarks
        };
        let ignored = opts.ignored || !is_target_mode || self.is_filtered(name);

        match self.tests.entry(name.into()) {
            Entry::Occupied(..) => panic!("the test name is duplicated"),
            Entry::Vacant(entry) => {
                let name = entry.key().clone();
                let ignored = ignored ^ self.args.run_ignored;
                if ignored {
                    entry.insert(TestCase {
                        name,
                        is_test,
                        opts,
                        rx: None,
                    });
                    None
                } else {
                    let (tx, rx) = oneshot::channel();
                    entry.insert(TestCase {
                        name,
                        is_test,
                        opts,
                        rx: Some(rx),
                    });
                    Some(tx)
                }
            }
        }
    }

    /// Run the test suite and aggregate the results.
    ///
    /// The test suite is executed as follows:
    ///
    /// 1. A startup signal is sent to the handle `Test` returned when adding a test.
    /// 2. Each test case is executed. This is usually performed by driving `progress`.
    /// 3. After `progress` is completed, a cancellation signal is sent to each test
    ///    case.
    pub async fn run_tests<F>(&mut self, progress: F) -> i32
    where
        F: Future<Output = ()>,
    {
        if self.args.list {
            // TODO: list test cases.
            return 0;
        }

        let _report = self
            .run_tests_with(progress, DefaultEventHandler::default())
            .await;
        // TODO: summary
        0
    }

    async fn run_tests_with<F, H>(&mut self, progress: F, handler: H) -> Report
    where
        F: Future<Output = ()>,
        H: EventHandler,
    {
        self.started.set();
        progress.await;

        // TODO: send cancellation signal to test handles.

        let mut has_failed = false;
        for (name, ctx) in self.tests.drain() {
            let outcome = match ctx.rx {
                Some(rx) => rx.await.unwrap_or_else(|_| Outcome::Canceled),
                None => Outcome::Ignored,
            };
            if let Outcome::Failed { .. } = outcome {
                has_failed = true;
            }
            handler.dump_result(&name, outcome);
        }

        Report { has_failed }
    }
}

/// The handle to a test.
#[derive(Debug)]
pub struct Test {
    started: Arc<ManualResetEvent>,
    tx: oneshot::Sender<Outcome>,
}

impl Test {
    /// Wrap a future to catch events from the test suite.
    pub async fn run<Fut>(self, test_case: Fut)
    where
        Fut: Future<Output = Result<(), Option<String>>>,
    {
        self.started.wait().await;
        let outcome = match test_case.await {
            Ok(()) => Outcome::Passed,
            Err(msg) => Outcome::Failed { msg },
        };
        let _ = self.tx.send(outcome);
    }
}

/// The handle to a benchmark test.
#[derive(Debug)]
pub struct Benchmark {
    started: Arc<ManualResetEvent>,
    tx: oneshot::Sender<Outcome>,
}

impl Benchmark {
    /// Wrap a future to catch events from the test suite.
    pub async fn run<Fut>(self, test_case: Fut)
    where
        Fut: Future<Output = Result<(u64, u64), Option<String>>>,
    {
        self.started.wait().await;
        let outcome = match test_case.await {
            Ok((average, variance)) => Outcome::Measured { average, variance },
            Err(msg) => Outcome::Failed { msg },
        };
        let _ = self.tx.send(outcome);
    }
}
