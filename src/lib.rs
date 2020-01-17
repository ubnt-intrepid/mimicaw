//! Minimal test harness that mimics libtest for asynchronous integration tests.

use futures::channel::oneshot;
use std::{
    borrow::Cow,
    collections::hash_map::{Entry, HashMap},
    future::Future,
};
use structopt::StructOpt;

#[derive(Debug)]
enum Outcome {
    Passed,
    Failed { msg: Option<Cow<'static, str>> },
    Ignored,
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

#[derive(Debug)]
pub struct TestRunner {
    args: Arguments,
    tests: HashMap<String, TestContext>,
    num_filtered_out: usize,
}

impl TestRunner {
    pub fn from_env() -> Self {
        let args = Arguments::from_args();
        Self {
            args,
            tests: HashMap::new(),
            num_filtered_out: 0,
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
    /// This method will return a `Test`
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
                    Some(Test { tx: Some(tx) })
                }
            }
        }
    }

    pub async fn start<F>(&mut self, progress: F) -> Report
    where
        F: Future<Output = ()>,
    {
        progress.await;

        let mut has_failed = false;
        for (name, ctx) in self.tests.drain() {
            let outcome = match ctx.rx {
                Some(rx) => rx.await.unwrap(),
                None => Outcome::Ignored,
            };
            match outcome {
                Outcome::Passed => println!("{}: passed", name),
                Outcome::Ignored => println!("{}: ignored", name),
                Outcome::Failed { msg } => {
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
pub struct Test {
    tx: Option<oneshot::Sender<Outcome>>,
}

impl Test {
    pub async fn wait_ready(&mut self) {}

    pub fn is_ready(&self) -> bool {
        true
    }

    fn report(&mut self, outcome: Outcome) {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(outcome);
        }
    }

    pub fn passed(&mut self) {
        self.report(Outcome::Passed)
    }

    pub fn failed(&mut self, msg: Option<Cow<'static, str>>) {
        self.report(Outcome::Failed { msg })
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
