use crate::{args::Args, Outcome, OutcomeKind, Report};
use futures::{channel::oneshot, future::Future};
use futures_intrusive::sync::ManualResetEvent;
use std::{
    collections::hash_map::{Entry, HashMap},
    sync::Arc,
};

/// A type that represents a test suite.
#[derive(Debug)]
pub struct TestSuite {
    args: Args,
    tests: HashMap<String, TestContext>,
    num_filtered_out: usize,
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
            num_filtered_out: 0,
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

    /// Register a single test to the runner.
    ///
    /// This method will return a handle if the specified test case needs
    /// to be driven.
    pub fn add_test(&mut self, name: &str, ignored: bool) -> Option<Test> {
        if self.is_filtered(name) {
            self.num_filtered_out += 1;
            return None;
        }

        match self.tests.entry(name.into()) {
            Entry::Occupied(..) => panic!("the test name is duplicated"),
            Entry::Vacant(entry) => {
                let ignored = ignored ^ self.args.run_ignored;
                if ignored {
                    entry.insert(TestContext {
                        rx: None,
                        ignored: true,
                    });
                    None
                } else {
                    let (tx, rx) = oneshot::channel();
                    entry.insert(TestContext {
                        rx: Some(rx),
                        ignored: false,
                    });
                    Some(Test {
                        started: self.started.clone(),
                        tx,
                    })
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
    pub async fn run_tests<F>(&mut self, progress: F) -> Report
    where
        F: Future<Output = ()>,
    {
        self.started.set();
        progress.await;

        // TODO: send cancellation signal to test handles.

        let mut has_failed = false;
        for (name, ctx) in self.tests.drain() {
            let outcome = match ctx.rx {
                Some(rx) => rx.await.unwrap_or_else(|_| Outcome::canceled()),
                None => Outcome::ignored(),
            };
            match outcome.0 {
                OutcomeKind::Passed => println!("{}: passed", name),
                OutcomeKind::Ignored => println!("{}: ignored", name),
                OutcomeKind::Canceled => println!("{}: canceled", name),
                OutcomeKind::Failed { msg } => {
                    has_failed = true;
                    match msg {
                        Some(msg) => println!("{}: failed:\n{}", name, msg),
                        None => println!("{}: failed", name),
                    }
                }
            }
        }

        Report { has_failed }
    }
}

#[derive(Debug)]
struct TestContext {
    rx: Option<oneshot::Receiver<Outcome>>,
    ignored: bool,
}

/// The handle to a test case.
#[derive(Debug)]
pub struct Test {
    started: Arc<ManualResetEvent>,
    tx: oneshot::Sender<Outcome>,
}

impl Test {
    /// Wrap a future to catch events from the test runner.
    pub async fn run<Fut>(self, test_case: Fut)
    where
        Fut: Future<Output = Outcome>,
    {
        self.started.wait().await;
        let outcome = test_case.await;
        let _ = self.tx.send(outcome);
    }
}
