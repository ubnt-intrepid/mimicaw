use crate::{
    args::Args,
    test::{Outcome, Test, TestKind},
};
use futures::{
    channel::oneshot,
    future::Future,
    stream::StreamExt,
    task::{self, Poll},
};
use indexmap::IndexMap;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use pin_project_lite::pin_project;
use std::{pin::Pin, sync::Arc};

pin_project! {
    struct RunningTest {
        #[pin]
        test: Test,
        filtered: bool,
        progress: ProgressBar,
        outcome: Option<Outcome>,
    }
}

impl Future for RunningTest {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let me = self.project();
        if *me.filtered {
            me.progress
                .finish_with_message(&console::style("ignored").yellow().to_string());
            return Poll::Ready(());
        }

        me.progress.enable_steady_tick(100);
        me.progress.set_message("running");

        let outcome = futures::ready!(me.test.test_case().poll(cx));
        me.progress.finish_with_message(&match outcome {
            Outcome::Passed => console::style("passed").green().to_string(),
            Outcome::Failed { .. } => console::style("failed").red().to_string(),
            Outcome::Measured { .. } => console::style("measured").green().to_string(),
        });
        me.outcome.replace(outcome);

        Poll::Ready(())
    }
}

pub struct TestDriver {
    args: Args,
    pending_tests: IndexMap<Arc<String>, Test>,
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

    pub fn add_test(&mut self, test: Test) {
        let name = test.name().clone();
        assert!(
            !self.pending_tests.contains_key(&*name),
            "the test name is duplicated"
        );
        self.pending_tests.insert(name, test);
    }

    fn print_list(&self) {
        let quiet = self.args.format == crate::args::OutputFormat::Terse;

        let mut num_tests = 0;
        let mut num_benches = 0;

        for test in self.pending_tests.values() {
            let kind_str = match test.kind() {
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

        let mut running_tests = Vec::with_capacity(self.pending_tests.len());
        for (_, test) in self.pending_tests.drain(..) {
            let is_target_mode = match test.kind() {
                TestKind::Test => self.args.run_tests,
                TestKind::Bench => self.args.run_benchmarks,
            };
            let filtered = test.ignored || !is_target_mode || self.args.is_filtered(&*test.name());
            let filtered = filtered ^ self.args.run_ignored;

            let progress = multi_progress.add(ProgressBar::new_spinner());
            progress.set_style(progress_style.clone());
            progress.set_prefix(&*test.name());
            running_tests.push(RunningTest {
                test,
                filtered,
                progress,
                outcome: None,
            });
        }

        let rx = {
            let (tx, rx) = oneshot::channel();
            std::thread::spawn(move || {
                let res = multi_progress.join();
                let _ = tx.send(res);
            });
            rx
        };
        let run_tests = futures::stream::iter(running_tests.iter_mut()) //
            .for_each_concurrent(1024, std::convert::identity);
        let _ = futures::future::join(rx, run_tests).await;

        let mut passed_tests = vec![];
        let mut failed_tests = vec![];
        let mut benchmark_tests = vec![];
        let mut ignored_tests = vec![];
        for test in running_tests {
            match test.outcome {
                Some(Outcome::Passed) => passed_tests.push(test.test),
                Some(Outcome::Failed { msg }) => failed_tests.push((test.test, msg)),
                Some(Outcome::Measured { average, variance }) => {
                    benchmark_tests.push((test.test, average, variance))
                }
                None => ignored_tests.push(test.test),
            }
        }

        println!();
        println!("======== SUMMARY ========");
        println!("PASSED: {}", passed_tests.len());
        println!("FAILED: {}", failed_tests.len());
        println!("IGNORE: {}", ignored_tests.len());
        println!("BENCH:  {}", benchmark_tests.len());

        if failed_tests.is_empty() {
            0
        } else {
            crate::ERROR_STATUS_CODE
        }
    }
}
