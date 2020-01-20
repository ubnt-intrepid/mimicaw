use crate::args::Args;
use console::style;
use futures::{channel::oneshot, future::Future};
use futures_intrusive::sync::ManualResetEvent;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
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

#[derive(Copy, Clone, Debug)]
enum TestKind {
    Test,
    Bench,
}

#[derive(Debug)]
struct TestCase {
    name: String,
    kind: TestKind,
    opts: TestOptions,
    rx: Option<oneshot::Receiver<Outcome>>,
}

/// A type that represents a test suite.
#[derive(Debug)]
pub struct TestSuite {
    args: Args,
    tests: HashMap<String, TestCase>,
    started: Arc<ManualResetEvent>,
    multi_progress: Arc<MultiProgress>,
    progress_style: ProgressStyle,
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
            multi_progress: Arc::new(MultiProgress::new()),
            progress_style: {
                ProgressStyle::default_spinner() //
                    .template("{prefix:.bold.dim} {spinner} {wide_msg}")
            },
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
        self.add_test_inner(name, TestKind::Test, opts) //
            .map(Test)
    }

    /// Register a single benchmark test to the suite.
    ///
    /// This method will return a handle if the specified benchmark test needs
    /// to be driven.
    pub fn add_bench(&mut self, name: &str, opts: TestOptions) -> Option<Benchmark> {
        self.add_test_inner(name, TestKind::Bench, opts)
            .map(Benchmark)
    }

    fn add_test_inner(&mut self, name: &str, kind: TestKind, opts: TestOptions) -> Option<Handle> {
        let is_target_mode = match kind {
            TestKind::Test => self.args.run_tests,
            TestKind::Bench => self.args.run_benchmarks,
        };
        let filtered = opts.ignored || !is_target_mode || self.is_filtered(name);
        let filtered = filtered ^ self.args.run_ignored;

        match self.tests.entry(name.into()) {
            Entry::Occupied(..) => panic!("the test name is duplicated"),
            Entry::Vacant(entry) => {
                let (tx_opt, rx_opt) = if !filtered {
                    let (tx, rx) = oneshot::channel();
                    (Some(tx), Some(rx))
                } else {
                    (None, None)
                };

                let name = entry.key().clone();
                entry.insert(TestCase {
                    name: name.clone(),
                    kind,
                    opts,
                    rx: rx_opt,
                });

                let kind = match kind {
                    TestKind::Test => "test",
                    TestKind::Bench => "benchmark",
                };
                let progress = self.multi_progress.add(ProgressBar::new_spinner());
                progress.set_style(self.progress_style.clone());
                progress.set_prefix(&format!("[{} {}]", kind, name));
                if filtered {
                    progress.finish_with_message(&style("ignored").yellow().to_string());
                }

                tx_opt.map(|tx| Handle {
                    started: self.started.clone(),
                    progress,
                    tx,
                })
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
    pub async fn run_tests(&mut self) -> i32 {
        if self.args.list {
            let quiet = self.args.format == crate::args::OutputFormat::Terse;

            let mut num_tests = 0;
            let mut num_benches = 0;

            for test in self.tests.values() {
                let kind_str = match test.kind {
                    TestKind::Test => {
                        num_tests += 1;
                        "test"
                    }
                    TestKind::Bench => {
                        num_benches += 1;
                        "benchmark"
                    }
                };
                println!("{}: {}", test.name, kind_str);
            }

            if !quiet {
                fn plural_suffix(n: usize) -> &'static str {
                    match n {
                        1 => "",
                        _ => "s",
                    }
                }

                if num_tests != 0 || num_benches != 0 {
                    println!();
                }
                println!(
                    "{} test{}, {} benchmark{}",
                    num_tests,
                    plural_suffix(num_tests),
                    num_benches,
                    plural_suffix(num_benches)
                );
            }

            return 0;
        }

        println!("======== RUNNING TESTS ========");
        self.started.set();
        // TODO: await completion of test jobs.
        {
            let (tx, rx) = oneshot::channel();
            let multi_progress = self.multi_progress.clone();
            std::thread::spawn(move || {
                let res = multi_progress.join();
                let _ = tx.send(res);
            });
            rx.await.unwrap().unwrap();
        }

        let mut report = Report { has_failed: false };

        println!();
        println!("======== SUMMARY ========");

        for (name, test) in self.tests.drain() {
            let outcome = match test.rx {
                Some(rx) => rx.await.unwrap_or_else(|_| Outcome::Canceled),
                None => Outcome::Ignored,
            };
            if let Outcome::Failed { .. } = outcome {
                report.has_failed = true;
            }
            match outcome {
                Outcome::Passed => println!("{}: passed", name),
                Outcome::Ignored => println!("{}: ignored", name),
                Outcome::Canceled => println!("{}: canceled", name),
                Outcome::Measured { average, variance } => {
                    println!("{}: measured (avg={}, var={})", name, average, variance)
                }
                Outcome::Failed { msg } => match msg {
                    Some(msg) => println!("{}: failed:\n{}", name, msg),
                    None => println!("{}: failed", name),
                },
            }
        }

        // TODO: summary report

        if !report.has_failed {
            0
        } else {
            crate::ERROR_STATUS_CODE
        }
    }
}

#[derive(Debug)]
struct Handle {
    started: Arc<ManualResetEvent>,
    progress: ProgressBar,
    tx: oneshot::Sender<Outcome>,
}

impl Handle {
    async fn run<Fut>(self, fut: Fut)
    where
        Fut: Future<Output = Outcome>,
    {
        self.started.wait().await;
        self.progress.enable_steady_tick(100);
        self.progress.set_message("running...");

        let outcome = fut.await;
        match outcome {
            Outcome::Passed => {
                self.progress
                    .finish_with_message(&style("passed").green().to_string());
            }
            Outcome::Failed { .. } => {
                self.progress
                    .finish_with_message(&style("failed").red().bold().to_string());
            }
            Outcome::Measured { .. } => {
                self.progress
                    .finish_with_message(&style("finished").green().to_string());
            }
            _ => (),
        };

        let _ = self.tx.send(outcome);
    }
}

/// The handle to a test.
#[derive(Debug)]
pub struct Test(Handle);

impl Test {
    /// Wrap a future to catch events from the test suite.
    pub async fn run<Fut>(self, test_case: Fut)
    where
        Fut: Future<Output = Result<(), Option<String>>>,
    {
        self.0
            .run(async {
                match test_case.await {
                    Ok(()) => Outcome::Passed,
                    Err(msg) => Outcome::Failed { msg },
                }
            })
            .await
    }
}

/// The handle to a benchmark test.
#[derive(Debug)]
pub struct Benchmark(Handle);

impl Benchmark {
    /// Wrap a future to catch events from the test suite.
    pub async fn run<Fut>(self, test_case: Fut)
    where
        Fut: Future<Output = Result<(u64, u64), Option<String>>>,
    {
        self.0
            .run(async {
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

#[derive(Debug)]
#[must_use]
pub struct Report {
    pub(crate) has_failed: bool,
}
