use crate::args::Args;
use console::style;
use futures::{
    channel::oneshot,
    future::Future,
    task::{self, Poll},
};
use indexmap::IndexMap;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use pin_project_lite::pin_project;
use std::{pin::Pin, sync::Arc};

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

pin_project! {
    #[derive(Debug)]
    struct TestCase {
        name: Arc<String>,
        kind: TestKind,
        opts: TestOptions,
        tx_progress: Option<oneshot::Sender<ProgressBar>>,
        #[pin]
        rx_outcome: Option<oneshot::Receiver<Outcome>>,
        outcome: Option<Outcome>,
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

#[derive(Debug)]
pub struct TestDriver {
    args: Args,
    pending_tests: IndexMap<Arc<String>, TestCase>,
}

impl TestDriver {
    /// Create a test suite.
    pub fn from_env() -> Self {
        match Args::from_env() {
            Ok(args) => Self {
                args,
                pending_tests: IndexMap::new(),
            },
            Err(code) => {
                // The process should not be exited at here
                // in order for the resources in main function to
                // be appropriately dropped.
                std::process::exit(code);
            }
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

    fn add_test_inner(&mut self, name: &str, kind: TestKind, opts: TestOptions) -> Option<Handle> {
        let name = Arc::new(name.to_string());
        assert!(
            !self.pending_tests.contains_key(&*name),
            "the test name is duplicated"
        );

        let is_target_mode = match kind {
            TestKind::Test => self.args.run_tests,
            TestKind::Bench => self.args.run_benchmarks,
        };
        let filtered = opts.ignored || !is_target_mode || self.is_filtered(&*name);
        let filtered = filtered ^ self.args.run_ignored;

        let (tx_progress, rx_outcome, handle) = if !filtered {
            let (tx_progress, rx_progress) = oneshot::channel();
            let (tx_outcome, rx_outcome) = oneshot::channel();
            (
                Some(tx_progress),
                Some(rx_outcome),
                Some(Handle {
                    progress: rx_progress,
                    outcome: tx_outcome,
                }),
            )
        } else {
            (None, None, None)
        };

        self.pending_tests.insert(
            name.clone(),
            TestCase {
                name,
                kind,
                opts,
                tx_progress,
                rx_outcome,
                outcome: None,
            },
        );

        handle
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

    fn print_list(&self) {
        let quiet = self.args.format == crate::args::OutputFormat::Terse;

        let mut num_tests = 0;
        let mut num_benches = 0;

        for test in self.pending_tests.values() {
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
    }

    /// Run the test suite and aggregate the results.
    pub async fn run_tests(&mut self) -> i32 {
        if self.args.list {
            self.print_list();
            return 0;
        }

        println!("======== RUNNING TESTS ========");

        let name_max_length = self
            .pending_tests
            .keys()
            .map(|key| key.len())
            .max()
            .unwrap_or(0);

        let multi_progress = MultiProgress::new();
        let progress_style = ProgressStyle::default_spinner() //
            .template(&format!(
                "{{prefix:{}.bold.dim}} {{spinner}} {{wide_msg}}",
                name_max_length
            ));

        let mut running_tests = vec![];
        let mut filtered_tests = vec![];

        for (_name, mut test) in self.pending_tests.drain(..) {
            let progress = multi_progress.add(ProgressBar::new_spinner());
            progress.set_style(progress_style.clone());
            progress.set_prefix(&*test.name);

            if let Some(tx) = test.tx_progress.take() {
                let _ = tx.send(progress);
                running_tests.push(test);
            } else {
                progress.finish_with_message(&style("ignored").yellow().to_string());
                filtered_tests.push(test);
            }
        }

        // TODO: await completion of test jobs.

        {
            let (tx, rx) = oneshot::channel();
            std::thread::spawn(move || {
                let res = multi_progress.join();
                let _ = tx.send(res);
            });
            rx.await.unwrap().unwrap();
        }

        futures::future::join_all(running_tests.iter_mut()).await;

        let mut passed_tests = vec![];
        let mut failed_tests = vec![];
        let mut benchmark_tests = vec![];
        let mut ignored_len = 0;
        for mut test in running_tests {
            let outcome = test.outcome.take().unwrap_or_else(|| Outcome::Canceled);
            match outcome {
                Outcome::Passed => passed_tests.push(test),
                Outcome::Failed { msg } => failed_tests.push((test, msg)),
                Outcome::Measured { average, variance } => {
                    benchmark_tests.push((test, average, variance))
                }
                Outcome::Ignored => ignored_len += 1,
                _ => (),
            }
        }

        println!();
        println!("======== SUMMARY ========");
        println!("PASSED: {}", passed_tests.len());
        println!("FAILED: {}", failed_tests.len());
        println!("IGNORE: {}", ignored_len);
        println!("BENCH:  {}", benchmark_tests.len());

        if failed_tests.is_empty() {
            0
        } else {
            crate::ERROR_STATUS_CODE
        }
    }
}

#[derive(Debug)]
struct Handle {
    progress: oneshot::Receiver<ProgressBar>,
    outcome: oneshot::Sender<Outcome>,
}

impl Handle {
    async fn run<Fut>(self, fut: Fut)
    where
        Fut: Future<Output = Outcome>,
    {
        let progress = self.progress.await.unwrap();
        progress.enable_steady_tick(100);
        progress.set_message("running");

        let outcome = fut.await;
        match outcome {
            Outcome::Passed => {
                progress.finish_with_message(&style("passed").green().to_string());
            }
            Outcome::Failed { .. } => {
                progress.finish_with_message(&style("failed").red().bold().to_string());
            }
            Outcome::Measured { .. } => {
                progress.finish_with_message(&style("finished").green().to_string());
            }
            _ => (),
        };

        let _ = self.outcome.send(outcome);
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
