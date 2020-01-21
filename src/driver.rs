use crate::{
    args::Args,
    test::{Handle, Outcome, TestCase, TestKind, TestOptions},
};
use futures::channel::oneshot;
use indexmap::IndexMap;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;

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

    /// Register a single test to the suite.
    ///
    /// This method will return a handle if the specified test needs
    /// to be driven.
    pub fn add_test(&mut self, name: &str, opts: TestOptions) -> Option<Handle> {
        let name = Arc::new(name.to_string());
        assert!(
            !self.pending_tests.contains_key(&*name),
            "the test name is duplicated"
        );

        let is_target_mode = match opts.kind {
            TestKind::Test => self.args.run_tests,
            TestKind::Bench => self.args.run_benchmarks,
        };
        let filtered = opts.ignored || !is_target_mode || self.is_filtered(&*name);
        let filtered = filtered ^ self.args.run_ignored;

        let (test, handle) = TestCase::new(&name, opts, filtered);
        self.pending_tests.insert(name, test);
        handle
    }

    fn print_list(&self) {
        let quiet = self.args.format == crate::args::OutputFormat::Terse;

        let mut num_tests = 0;
        let mut num_benches = 0;

        for test in self.pending_tests.values() {
            let kind_str = match test.opts().kind {
                TestKind::Test => {
                    num_tests += 1;
                    "test"
                }
                TestKind::Bench => {
                    num_benches += 1;
                    "benchmark"
                }
            };
            println!("{}: {}", test.name(), kind_str);
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
            progress.set_prefix(&*test.name());
            test.start(progress);

            if !test.filtered() {
                running_tests.push(test);
            } else {
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
        for mut test in running_tests {
            match test.take_outcome() {
                Outcome::Passed => passed_tests.push(test),
                Outcome::Failed { msg } => failed_tests.push((test, msg)),
                Outcome::Measured { average, variance } => {
                    benchmark_tests.push((test, average, variance))
                }
                _ => (),
            }
        }

        println!();
        println!("======== SUMMARY ========");
        println!("PASSED: {}", passed_tests.len());
        println!("FAILED: {}", failed_tests.len());
        println!("IGNORE: {}", filtered_tests.len());
        println!("BENCH:  {}", benchmark_tests.len());

        if failed_tests.is_empty() {
            0
        } else {
            crate::ERROR_STATUS_CODE
        }
    }
}
