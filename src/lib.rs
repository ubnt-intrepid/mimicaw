//! Minimal test harness that mimics libtest for asynchronous integration tests.

use futures::{channel::oneshot, future::Future};
use futures_intrusive::sync::ManualResetEvent;
use std::{
    collections::hash_map::{Entry, HashMap},
    sync::Arc,
};
use structopt::StructOpt;

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

#[derive(Debug, StructOpt)]
#[structopt(
    template = "Usage: [FLAGS] [OPTIONS] [FILTER]\n\n{all-args}\n\n\n{after-help}",
    setting = structopt::clap::AppSettings::DisableVersion,
)]
struct Arguments {
    /// Run ignored tests
    #[structopt(long = "--ignored")]
    enable_ignored_tests: bool,

    /// List all tests
    #[structopt(long = "--list")]
    list: bool,

    /// Exactly match filters rather than by substring
    #[structopt(long = "--exact")]
    exact: bool,

    /// Skip tests whose names contain FILTER
    #[structopt(long = "--skip", value_name = "FILTER", number_of_values = 1)]
    skip: Vec<String>,

    /// The FILTER string is tested against the name of all tests, and only
    /// those tests whose names contain the filter are run.
    #[structopt(name = "FILTER")]
    filter_string: Option<String>,
}

#[derive(Debug)]
struct TestContext {
    rx: Option<oneshot::Receiver<Outcome>>,
    ignored: bool,
}

/// The runner of a test suite.
#[derive(Debug)]
pub struct TestRunner {
    args: Arguments,
    tests: HashMap<String, TestContext>,
    num_filtered_out: usize,
    started: Arc<ManualResetEvent>,
}

impl TestRunner {
    /// Create a `TestRunner` in the current environment.
    pub fn from_env() -> Self {
        let args = Arguments::from_args();
        Self {
            args,
            tests: HashMap::new(),
            num_filtered_out: 0,
            started: Arc::new(ManualResetEvent::new(false)),
        }
    }

    fn is_filtered(&self, name: &str) -> bool {
        if let Some(ref filter) = self.args.filter_string {
            if self.args.exact && name != filter {
                return true;
            }
            if !name.contains(filter) {
                return true;
            }
        }

        for skip_filter in &self.args.skip {
            if self.args.exact && name != skip_filter {
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
                if ignored && !self.args.enable_ignored_tests {
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
